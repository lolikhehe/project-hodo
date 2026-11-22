//! Page-side DOM bindings: opaque `u32` handles into a thread-local table of
//! `(Rc<RefCell<Dom>>, NodeId)`, every access gated by `Dom::validate`.
//!
//! `Rc<RefCell<Dom>>` is correct here: a page JS context owns exactly one
//! document and boa contexts are single-threaded, so no `Send` is required.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use boa_engine::error::JsNativeError;
use boa_engine::native_function::NativeFunction;
use boa_engine::property::PropertyKey;
use boa_engine::{JsArgs, JsString, JsValue};
use hodo_css::{matches_selector, Selector};
use hodo_dom::{Dom, NodeId, NodeKind, SHADOW_ROOT_TAG};
use serde_json::json;

use super::{error_message, js_error, register, string_arg, with_state, RuntimeState};

type DomHandleEntry = (Rc<RefCell<Dom>>, NodeId);

thread_local! {
    /// JS-visible handles are sequential integers; the real arena handle
    /// (with its generation bits) stays on this side.
    static DOM_HANDLES: RefCell<HashMap<u32, DomHandleEntry>> = RefCell::new(HashMap::new());
}
static NEXT_DOM_HANDLE: AtomicU32 = AtomicU32::new(1);

pub(crate) struct PageListener {
    pub(crate) node: NodeId,
    pub(crate) kind: String,
    pub(crate) callback_id: u64,
}

pub(crate) struct PageState {
    pub(crate) dom: Rc<RefCell<Dom>>,
    pub(crate) listeners: RefCell<Vec<PageListener>>,
    /// What `window.location.href` reports; threaded in by the host.
    pub(crate) url: String,
    handles: RefCell<Vec<u32>>,
    /// One stable JS handle per live `NodeId`. Repeated wraps of the same node
    /// (querySelector, children, parentNode, ...) therefore share an identity,
    /// which is what custom-element instances key their upgrade registry on.
    keys: RefCell<HashMap<NodeId, u32>>,
}

impl PageState {
    pub(crate) fn new(dom: Rc<RefCell<Dom>>, url: String) -> Self {
        Self {
            dom,
            listeners: RefCell::new(Vec::new()),
            url,
            handles: RefCell::new(Vec::new()),
            keys: RefCell::new(HashMap::new()),
        }
    }
}

/// Drop every handle owned by a vanished page so recycled handles cannot alias
/// a different document.
pub(crate) fn drop_page(state: &RuntimeState) {
    let Some(page) = state.page.as_ref() else {
        return;
    };
    let handles = std::mem::take(&mut *page.handles.borrow_mut());
    if handles.is_empty() {
        return;
    }
    DOM_HANDLES.with(|table| {
        let mut table = table.borrow_mut();
        for handle in handles {
            table.remove(&handle);
        }
    });
}

fn page_of(state: &RuntimeState) -> Result<&PageState, String> {
    state
        .page
        .as_ref()
        .ok_or_else(|| "DOM is not available in this context".to_string())
}

fn alloc_handle(state: &RuntimeState, node: NodeId) -> Result<u32, String> {
    let page = page_of(state)?;
    {
        let keys = page.keys.borrow();
        if let Some(handle) = keys.get(&node) {
            return Ok(*handle);
        }
    }
    let handle = NEXT_DOM_HANDLE.fetch_add(1, Ordering::Relaxed).max(1);
    DOM_HANDLES.with(|table| {
        table
            .borrow_mut()
            .insert(handle, (Rc::clone(&page.dom), node));
    });
    page.handles.borrow_mut().push(handle);
    page.keys.borrow_mut().insert(node, handle);
    Ok(handle)
}

/// Reject stale generations rather than touching a recycled arena slot.
fn resolve(handle: u32) -> Result<(Rc<RefCell<Dom>>, NodeId), String> {
    let entry = DOM_HANDLES.with(|table| table.borrow().get(&handle).cloned());
    let (dom, node) = entry.ok_or_else(|| "unknown DOM handle".to_string())?;
    if !dom.borrow().validate(node) {
        return Err("stale DOM handle".to_string());
    }
    Ok((dom, node))
}

fn resolve_for(state: &RuntimeState, handle: u32) -> Result<(Rc<RefCell<Dom>>, NodeId), String> {
    let (dom, node) = resolve(handle)?;
    if !Rc::ptr_eq(&dom, &page_of(state)?.dom) {
        return Err("DOM handle belongs to another page".to_string());
    }
    Ok((dom, node))
}

fn handle_arg(args: &[JsValue], index: usize) -> Result<u32, String> {
    let number = args
        .get_or_undefined(index)
        .as_number()
        .ok_or_else(|| "expected an opaque DOM handle".to_string())?;
    if !number.is_finite() || number < 1.0 || number > u32::MAX as f64 {
        return Err("invalid DOM handle".to_string());
    }
    Ok(number as u32)
}

/// `insertBefore`'s ref node is nullable: null/undefined means "append".
fn nullable_handle_arg(args: &[JsValue], index: usize) -> Result<Option<u32>, String> {
    let value = args.get_or_undefined(index);
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    handle_arg(args, index).map(Some)
}

fn preorder(dom: &Dom, start: NodeId, mut visit: impl FnMut(NodeId)) {
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        visit(id);
        let children = dom.children(id);
        for index in (0..children.len()).rev() {
            stack.push(children[index]);
        }
    }
}

fn find_by_id(dom: &Dom, target: &str) -> Option<NodeId> {
    let mut found = None;
    preorder(dom, dom.root(), |id| {
        if found.is_none() && dom.attr(id, "id") == Some(target) {
            found = Some(id);
        }
    });
    found
}

fn collect_text(dom: &Dom, start: NodeId) -> String {
    let mut out = String::new();
    preorder(dom, start, |id| {
        if let Some(text) = dom.text(id) {
            out.push_str(text);
        }
    });
    out
}

fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .enumerate()
            .all(|(i, c)| c.is_ascii_alphanumeric() || c == '-' || c == '_' || (i > 0 && c == ':'))
}

/// Parse a `querySelector` selector list by reusing hodo-css's full parser and
/// matcher. A synthetic one-rule stylesheet `{selector}{}` yields one `Selector`
/// per comma group, with attribute/compound/chain semantics owned by hodo-css
/// (no grammar fork here). Raw block delimiters would let a selector inject
/// extra rules into the sheet, so they are rejected outright.
fn query_selectors(selector: &str) -> Vec<Selector> {
    if selector.contains(['{', '}']) {
        return Vec::new();
    }
    hodo_css::parse_stylesheet(&format!("{selector}{{}}"))
        .rules
        .into_iter()
        .map(|rule| rule.selector)
        .collect()
}

/// Query from `start`, not always the document root: shadow roots and detached
/// DOMParser documents are subtrees of the same arena and must not leak into
/// each other's `querySelector` results. A document query hides `#shadow-root`
/// subtrees (light-DOM view); a query rooted *at* a shadow root sees them.
fn query(dom: &Dom, start: NodeId, selector: &str) -> Vec<NodeId> {
    let selectors = query_selectors(selector);
    if selectors.is_empty() {
        return Vec::new();
    }
    let include_shadow = dom.tag(start) == Some(SHADOW_ROOT_TAG);
    let mut found = Vec::new();
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        if dom.tag(id) == Some(SHADOW_ROOT_TAG) && !include_shadow {
            continue;
        }
        // querySelector* searches descendants, never the node itself.
        if id != start && selectors.iter().any(|sel| matches_selector(sel, dom, id)) {
            found.push(id);
        }
        for &child in dom.children(id).iter().rev() {
            stack.push(child);
        }
    }
    found
}

/// Whether `value` is already an absolute URL (has a `scheme:` prefix). Used so
/// `src`/`href` reflection never mangles `https:`, `data:` or `javascript:` URLs.
fn is_absolute_url(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Collapse `.`/`..` segments of a resolved path, keeping any query/fragment.
fn normalize_path(prefix: &str, path: &str) -> String {
    let (path_only, suffix) = match path.find(['?', '#']) {
        Some(index) => (&path[..index], &path[index..]),
        None => (path, ""),
    };
    let mut segments: Vec<&str> = Vec::new();
    for segment in path_only.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }
    format!("{prefix}/{}{}", segments.join("/"), suffix)
}

/// Resolve a relative `src`/`href` against the page URL. Absolute and
/// scheme-relative values need no base; without a usable base the raw value is
/// returned. TODO(unify with hodo-net's URL resolver once hodo-js may depend on it).
fn resolve_url(base: &str, value: &str) -> String {
    let value = value.trim();
    if value.is_empty() || is_absolute_url(value) {
        return value.to_string();
    }
    let Some((scheme, rest)) = base.split_once("://") else {
        return value.to_string();
    };
    if let Some(stripped) = value.strip_prefix("//") {
        return format!("{scheme}:{stripped}");
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let prefix = format!("{scheme}://{authority}");
    if value.starts_with('/') {
        return format!("{prefix}{value}");
    }
    if value.starts_with(['?', '#']) {
        let before_query = base.split(['?', '#']).next().unwrap_or(base);
        return format!("{before_query}{value}");
    }
    let tail = &rest[authority_end..];
    let path_part = tail.split(['?', '#']).next().unwrap_or(tail);
    let base_dir = &path_part[..path_part.rfind('/').map(|p| p + 1).unwrap_or(0)];
    normalize_path(&prefix, &format!("{base_dir}{value}"))
}

fn op_dom_root(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let page = page_of(&state).map_err(js_error)?;
        let root = page.dom.borrow().root();
        let handle = alloc_handle(&state, root).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

fn op_dom_get_element_by_id(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, _) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let target = string_arg(args, 1, ctx).map_err(js_error)?;
        let found = {
            let dom = dom.borrow();
            find_by_id(&dom, &target)
        };
        match found {
            Some(node) => Ok(JsValue::from(
                alloc_handle(&state, node).map_err(js_error)? as f64
            )),
            None => Ok(JsValue::null()),
        }
    })
}

fn op_dom_query(id: u64, all: bool) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let selector = string_arg(args, 1, ctx).map_err(js_error)?;
        let found = {
            let dom = dom.borrow();
            query(&dom, node, &selector)
        };
        let mut handles = Vec::new();
        for node in found {
            handles.push(alloc_handle(&state, node).map_err(js_error)?);
        }
        if all {
            JsValue::from_json(&json!(handles), ctx).map_err(|err| js_error(err.to_string()))
        } else {
            Ok(handles
                .first()
                .map_or(JsValue::null(), |handle| JsValue::from(*handle as f64)))
        }
    })
}

fn op_dom_create_element(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let tag = string_arg(args, 0, ctx).map_err(js_error)?;
        if !is_name(&tag) || tag.len() > 256 {
            return Err(js_error(
                "createElement expects a valid tag name".to_string(),
            ));
        }
        let page = page_of(&state).map_err(js_error)?;
        let node = page.dom.borrow_mut().create(NodeKind::Element {
            tag,
            attrs: Vec::new(),
        });
        let handle = alloc_handle(&state, node).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

/// Page-only: an unparented `#document-fragment` node, the same kind hodo-html
/// uses for `<template>` contents. It stays valid (unlike `detach_subtree`
/// trash) and is invisible to `Dom::walk`, so fragments never paint.
fn op_dom_create_fragment(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let page = page_of(&state).map_err(js_error)?;
        let node = page.dom.borrow_mut().create(NodeKind::Element {
            tag: "#document-fragment".to_string(),
            attrs: Vec::new(),
        });
        let handle = alloc_handle(&state, node).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

/// Page-only: replace a node's children with parsed HTML. Reuses hodo-html's
/// fragment path (no html/body wrapping, template side-map, depth cap). Old
/// children are trashed, so their JS handles go stale — the browser replaces
/// them too. `Element.innerHTML` and `template.innerHTML` (which passes the
/// content fragment handle here) both land on this one native.
fn op_dom_set_inner_html(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let html = string_arg(args, 1, ctx).map_err(js_error)?;
        let mut dom = dom.borrow_mut();
        for child in dom.children(node).to_vec() {
            dom.detach_subtree(child);
        }
        hodo_html::parse_fragment(&mut dom, node, &html)
            .map_err(|err| js_error(err.to_string()))?;
        Ok(JsValue::undefined())
    })
}

/// Page-only: parse an HTML string into an *unparented* `#document` node in the
/// page arena. Nothing parsed here is reachable from the live root, so it never
/// paints; the handle backs DOMParser / createHTMLDocument wrappers.
/// shortcut: literal `<html>`/`<body>` tags may be dropped by html5ever's
/// in-body fragment mode; the wrappers fall back to the parsed root/body tag.
fn op_dom_parse_detached(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let page = page_of(&state).map_err(js_error)?;
        let html = string_arg(args, 0, ctx).map_err(js_error)?;
        let root = {
            let mut dom = page.dom.borrow_mut();
            let root = dom.create(NodeKind::Element {
                tag: "#document".to_string(),
                attrs: Vec::new(),
            });
            if root == dom.root() {
                return Err(js_error("DOMParser: DOM node limit reached".to_string()));
            }
            root
        };
        {
            let mut dom = page.dom.borrow_mut();
            hodo_html::parse_fragment(&mut dom, root, &html)
                .map_err(|err| js_error(err.to_string()))?;
        }
        let handle = alloc_handle(&state, root).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

/// Page-only: a real `Text` node, unparented until `appendChild`/`insertBefore`
/// attaches it (fragments included). Modeled on `op_dom_create_element`.
fn op_dom_create_text_node(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let data = string_arg(args, 0, ctx).map_err(js_error)?;
        let page = page_of(&state).map_err(js_error)?;
        let node = page.dom.borrow_mut().create(NodeKind::Text(data));
        if node == page.dom.borrow().root() {
            return Err(js_error(
                "createTextNode: DOM node limit reached".to_string(),
            ));
        }
        let handle = alloc_handle(&state, node).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

/// Page-only: comments are `#comment` elements with their data in a `data`
/// attribute. They have no text children, so layout/render emit nothing.
fn op_dom_create_comment(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let data = string_arg(args, 0, ctx).map_err(js_error)?;
        let page = page_of(&state).map_err(js_error)?;
        let node = page.dom.borrow_mut().create(NodeKind::Element {
            tag: "#comment".to_string(),
            attrs: vec![("data".to_string(), data)],
        });
        if node == page.dom.borrow().root() {
            return Err(js_error(
                "createComment: DOM node limit reached".to_string(),
            ));
        }
        let handle = alloc_handle(&state, node).map_err(js_error)?;
        Ok(JsValue::from(handle as f64))
    })
}

/// Page-only node kind for wrapper fidelity: 1 element, 3 text, 8 comment,
/// 9 document, 11 fragment. `hodo-dom` has no comment kind, so `#comment` is
/// the marker; text is the only kind with `Dom::text` data.
fn op_dom_node_type(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let dom = dom.borrow();
        let kind = match dom.tag(node) {
            Some("#document") => 9,
            Some("#document-fragment") | Some(SHADOW_ROOT_TAG) => 11,
            Some("#comment") => 8,
            Some(_) => 1,
            None if dom.text(node).is_some() => 3,
            None => 9,
        };
        Ok(JsValue::from(kind as f64))
    })
}

fn op_dom_append_child(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let parent_handle = handle_arg(args, 0).map_err(js_error)?;
        let child_handle = handle_arg(args, 1).map_err(js_error)?;
        let (dom, parent) = resolve_for(&state, parent_handle).map_err(js_error)?;
        let (_, child) = resolve_for(&state, child_handle).map_err(js_error)?;
        if dom.borrow().tag(child) == Some("#document-fragment") {
            // Spec append: move the fragment's children, leave the fragment empty
            // and reusable. Snapshot first — `append` reparents out of the source.
            let moved = dom.borrow().children(child).to_vec();
            for node in moved {
                dom.borrow_mut().append(parent, node);
            }
        } else {
            dom.borrow_mut().append(parent, child);
        }
        Ok(JsValue::undefined())
    })
}

fn op_dom_remove_child(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let parent_handle = handle_arg(args, 0).map_err(js_error)?;
        let child_handle = handle_arg(args, 1).map_err(js_error)?;
        let (dom, parent) = resolve_for(&state, parent_handle).map_err(js_error)?;
        let (_, child) = resolve_for(&state, child_handle).map_err(js_error)?;
        if !dom.borrow().children(parent).contains(&child) {
            return Err(js_error("removeChild: node is not a child".to_string()));
        }
        dom.borrow_mut().detach_subtree(child);
        Ok(JsValue::undefined())
    })
}

/// Page-only `insertBefore`. `Dom::append` moves (never trashes) a node and
/// always appends, so the tail is rotated after the insert to keep order and
/// stable handles for every sibling. A null ref means append.
fn op_dom_insert_before(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let parent_handle = handle_arg(args, 0).map_err(js_error)?;
        let new_handle = handle_arg(args, 1).map_err(js_error)?;
        let ref_handle = nullable_handle_arg(args, 2).map_err(js_error)?;
        let (dom, parent) = resolve_for(&state, parent_handle).map_err(js_error)?;
        let (_, new_node) = resolve_for(&state, new_handle).map_err(js_error)?;
        let ref_node = match ref_handle {
            Some(handle) => {
                let (_, node) = resolve_for(&state, handle).map_err(js_error)?;
                if !dom.borrow().children(parent).contains(&node) {
                    return Err(JsNativeError::typ()
                        .with_message("insertBefore: reference node is not a child")
                        .into());
                }
                Some(node)
            }
            None => None,
        };
        let mut dom = dom.borrow_mut();
        // Fragment children are moved, the fragment itself stays empty/reusable.
        let moved: Vec<NodeId> = if dom.tag(new_node) == Some("#document-fragment") {
            dom.children(new_node).to_vec()
        } else {
            vec![new_node]
        };
        let trailing: Vec<NodeId> = ref_node.map_or_else(Vec::new, |reference| {
            let children = dom.children(parent);
            let index = children
                .iter()
                .position(|&child| child == reference)
                .unwrap_or(children.len());
            children[index..].to_vec()
        });
        for node in moved {
            dom.append(parent, node);
        }
        for node in trailing {
            dom.append(parent, node);
        }
        Ok(JsValue::undefined())
    })
}

fn op_dom_text(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let text = {
            let dom = dom.borrow();
            if dom.tag(node) == Some("#comment") {
                dom.attr(node, "data").unwrap_or("").to_string()
            } else {
                collect_text(&dom, node)
            }
        };
        Ok(JsValue::from(JsString::from(text)))
    })
}

fn op_dom_set_text(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        let text = string_arg(args, 1, ctx).map_err(js_error)?;
        let (dom, node) = resolve_for(&state, handle).map_err(js_error)?;
        if dom.borrow().tag(node) == Some("#comment") {
            dom.borrow_mut().set_attr(node, "data", text);
            return Ok(JsValue::undefined());
        }
        if dom.borrow().text(node).is_some() {
            // `Dom` has no in-place text setter. Replace the arena node but keep
            // the JS handle (and wrapper identity) pointed at the new slot.
            let (parent, index) = {
                let dom = dom.borrow();
                let parent = dom.parent(node);
                let index = parent.and_then(|p| dom.children(p).iter().position(|&c| c == node));
                (parent, index)
            };
            let dom_rc = Rc::clone(&dom);
            let mut dom = dom.borrow_mut();
            let new_node = dom.create(NodeKind::Text(text));
            if new_node == dom.root() {
                return Err(js_error("set text: DOM node limit reached".to_string()));
            }
            let trailing: Vec<NodeId> = parent
                .zip(index)
                .map(|(p, i)| dom.children(p)[i + 1..].to_vec())
                .unwrap_or_default();
            if let Some(parent) = parent {
                dom.append(parent, new_node);
                dom.detach_subtree(node);
                for child in trailing {
                    dom.append(parent, child);
                }
            }
            drop(dom);
            DOM_HANDLES.with(|table| {
                if let Some(entry) = table.borrow_mut().get_mut(&handle) {
                    *entry = (Rc::clone(&dom_rc), new_node);
                }
            });
            if let Some(page) = state.page.as_ref() {
                let mut keys = page.keys.borrow_mut();
                keys.remove(&node);
                keys.insert(new_node, handle);
            }
            return Ok(JsValue::undefined());
        }
        let mut dom = dom.borrow_mut();
        if dom.tag(node).is_none() {
            return Err(js_error(
                "textContent can only be set on an element".to_string(),
            ));
        }
        let old: Vec<NodeId> = dom.children(node).to_vec();
        for child in old {
            dom.detach_subtree(child);
        }
        let text_node = dom.create(NodeKind::Text(text));
        dom.append(node, text_node);
        Ok(JsValue::undefined())
    })
}

/// Page-only: the tag name of a wrapped node, or `null` for the document root.
/// Backs `nodeType`/`tagName`/`nodeName` on wrapper objects.
fn op_dom_tag(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let tag = {
            let dom = dom.borrow();
            dom.tag(node).map(str::to_string)
        };
        Ok(tag.map_or(JsValue::null(), |tag| JsValue::from(JsString::from(tag))))
    })
}

/// Page-only: element children as opaque handles, backing `children`.
/// Text nodes are filtered out here so the JS getter needs no shape knowledge.
fn op_dom_children(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let mut handles = Vec::new();
        {
            let dom = dom.borrow();
            for &child in dom.children(node) {
                // Light-DOM view: the shadow root is a real arena child (so
                // walk/upgrade still reach it) but JS must not see it here.
                if dom.tag(child).is_some_and(|tag| tag != SHADOW_ROOT_TAG) {
                    handles.push(alloc_handle(&state, child).map_err(js_error)?);
                }
            }
        }
        JsValue::from_json(&json!(handles), ctx).map_err(|err| js_error(err.to_string()))
    })
}

/// Page-only: every child handle (text nodes included), backing `childNodes`.
fn op_dom_child_nodes(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let mut handles = Vec::new();
        {
            let dom = dom.borrow();
            for &child in dom.children(node) {
                if dom.tag(child) != Some(SHADOW_ROOT_TAG) {
                    handles.push(alloc_handle(&state, child).map_err(js_error)?);
                }
            }
        }
        JsValue::from_json(&json!(handles), ctx).map_err(|err| js_error(err.to_string()))
    })
}

/// Page-only: the real parent handle, or null at the document root. Backs both
/// `parentNode` and `parentElement` (the JS side filters non-element parents).
fn op_dom_parent(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let parent = dom.borrow().parent(node);
        match parent {
            Some(parent) => Ok(JsValue::from(
                alloc_handle(&state, parent).map_err(js_error)? as f64,
            )),
            None => Ok(JsValue::null()),
        }
    })
}

/// Page-only: first descendant with `tag`, backing `document.head`/`body`.
fn op_dom_find_tag(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let tag = string_arg(args, 1, ctx).map_err(js_error)?;
        let mut found = None;
        {
            let dom = dom.borrow();
            preorder(&dom, node, |id| {
                if found.is_none() && dom.tag(id).is_some_and(|t| t.eq_ignore_ascii_case(&tag)) {
                    found = Some(id);
                }
            });
        }
        match found {
            Some(node) => Ok(JsValue::from(
                alloc_handle(&state, node).map_err(js_error)? as f64
            )),
            None => Ok(JsValue::null()),
        }
    })
}

/// Page-only: every descendant with `tag` as opaque handles, in document
/// order. Backs the custom-elements upgrade walk on `define`.
fn op_dom_find_all_tag(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let tag = string_arg(args, 1, ctx).map_err(js_error)?;
        let mut found = Vec::new();
        {
            let dom = dom.borrow();
            preorder(&dom, node, |id| {
                if dom.tag(id).is_some_and(|t| t.eq_ignore_ascii_case(&tag)) {
                    found.push(id);
                }
            });
        }
        let mut handles = Vec::new();
        for node in found {
            handles.push(alloc_handle(&state, node).map_err(js_error)?);
        }
        JsValue::from_json(&json!(handles), ctx).map_err(|err| js_error(err.to_string()))
    })
}

/// Page-only: Zone A's `Dom::attach_shadow`. The root is parented under its
/// host, so `walk` (and therefore `__hodo_dom_find_all_tag`) still upgrades
/// shadow-nested custom elements; JS views hide it via `SHADOW_ROOT_TAG`.
fn op_dom_attach_shadow(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, host) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let attached = dom.borrow_mut().attach_shadow(host);
        match attached {
            Some(root) => Ok(JsValue::from(
                alloc_handle(&state, root).map_err(js_error)? as f64
            )),
            None => Ok(JsValue::null()),
        }
    })
}

fn op_dom_get_attribute(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        let name = string_arg(args, 1, ctx).map_err(js_error)?;
        let (dom, node) = resolve_for(&state, handle).map_err(js_error)?;
        let value = dom.borrow().attr(node, &name).map(str::to_string);
        Ok(value.map_or(JsValue::null(), |value| {
            JsValue::from(JsString::from(value))
        }))
    })
}

/// Page-only: `src`/`href` reflection. Returns the attribute resolved against
/// the page URL (or the raw value when no page URL is threaded in); absent
/// attributes yield `null` like the browser getters do.
fn op_dom_resolved_url(id: u64, attr: &'static str) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let raw = { dom.borrow().attr(node, attr).map(str::to_string) };
        let value = match raw {
            None => return Ok(JsValue::null()),
            Some(raw) => match state.page.as_ref() {
                Some(page) if !page.url.is_empty() => resolve_url(&page.url, &raw),
                _ => raw,
            },
        };
        Ok(JsValue::from(JsString::from(value)))
    })
}

fn op_dom_set_attribute(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        let name = string_arg(args, 1, ctx).map_err(js_error)?;
        let value = string_arg(args, 2, ctx).map_err(js_error)?;
        let (dom, node) = resolve_for(&state, handle).map_err(js_error)?;
        dom.borrow_mut().set_attr(node, &name, value);
        Ok(JsValue::undefined())
    })
}

fn op_dom_class_list(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        let operation = string_arg(args, 1, ctx).map_err(js_error)?;
        let class = string_arg(args, 2, ctx).map_err(js_error)?;
        if class.is_empty() || class.split_whitespace().count() != 1 {
            return Err(js_error(
                "classList expects a single class name".to_string(),
            ));
        }
        let (dom, node) = resolve_for(&state, handle).map_err(js_error)?;
        let mut dom = dom.borrow_mut();
        let current = dom.attr(node, "class").unwrap_or("").to_string();
        let mut classes: Vec<String> = current.split_whitespace().map(str::to_string).collect();
        let mut result = JsValue::undefined();
        match operation.as_str() {
            "add" => {
                if !classes.iter().any(|item| item == &class) {
                    classes.push(class);
                }
            }
            "remove" => classes.retain(|item| item != &class),
            "contains" => result = JsValue::from(classes.iter().any(|item| item == &class)),
            "toggle" => {
                if let Some(position) = classes.iter().position(|item| item == &class) {
                    classes.remove(position);
                    result = JsValue::from(false);
                } else {
                    classes.push(class);
                    result = JsValue::from(true);
                }
            }
            _ => return Err(js_error("unknown classList operation".to_string())),
        }
        if operation != "contains" {
            dom.set_attr(node, "class", classes.join(" "));
        }
        Ok(result)
    })
}

fn op_dom_add_listener(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        let kind = string_arg(args, 1, ctx).map_err(js_error)?;
        let callback = args
            .get_or_undefined(2)
            .as_callable()
            .ok_or_else(|| js_error("addEventListener expects a function".to_string()))?;
        let (_, node) = resolve_for(&state, handle).map_err(js_error)?;
        let callback_id = state.fresh_id();
        let key = PropertyKey::from(JsString::from(format!("__hodoListener_{callback_id}")));
        ctx.global_object()
            .set(key, JsValue::from(callback), false, ctx)
            .map_err(|err| js_error(err.to_string()))?;
        let page = page_of(&state).map_err(js_error)?;
        page.listeners.borrow_mut().push(PageListener {
            node,
            kind,
            callback_id,
        });
        Ok(JsValue::undefined())
    })
}

/// Page-only: drop one listener by identity. The callback lives on a global
/// `__hodoListener_*` slot, so comparing that slot with the passed function is
/// both exact and cheap; the matching task is removed from the page registry.
fn op_dom_remove_listener(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (_, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let kind = string_arg(args, 1, ctx).map_err(js_error)?;
        let callback = args.get_or_undefined(2).clone();
        if callback.as_callable().is_none() {
            return Err(js_error(
                "removeEventListener expects a function".to_string(),
            ));
        }
        let page = page_of(&state).map_err(js_error)?;
        let mut removed = Vec::new();
        {
            let listeners = page.listeners.borrow();
            for listener in listeners.iter() {
                if listener.node != node || listener.kind != kind {
                    continue;
                }
                let key = PropertyKey::from(JsString::from(format!(
                    "__hodoListener_{}",
                    listener.callback_id
                )));
                if ctx
                    .global_object()
                    .get(key, ctx)
                    .is_ok_and(|stored| stored.strict_equals(&callback))
                {
                    removed.push(listener.callback_id);
                }
            }
        }
        for callback_id in &removed {
            let key = PropertyKey::from(JsString::from(format!("__hodoListener_{callback_id}")));
            let _ = ctx
                .global_object()
                .set(key, JsValue::undefined(), false, ctx);
        }
        if !removed.is_empty() {
            page.listeners
                .borrow_mut()
                .retain(|listener| !removed.contains(&listener.callback_id));
        }
        Ok(JsValue::undefined())
    })
}

/// Page-only: synchronously deliver a JS `Event` object to listeners registered
/// for its type on the target node, then (when bubbling) to listeners on each
/// ancestor up to the document root. Reuses the exact same listener registry
/// and `__hodoListener_*` globals as the host-driven `dispatch_page_event` path.
/// Unlike that host path, this loop sees the event object and so honors
/// `stopPropagation()`; the shell's `dispatch_page_event` still ignores it.
fn op_dom_dispatch_event(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let handle = handle_arg(args, 0).map_err(js_error)?;
        // Keep the event as a real JS object: listeners read `e.type`/`e.target`
        // and may call `e.preventDefault()`/`e.stopPropagation()` on it.
        let event = args.get_or_undefined(1).clone();
        let bubbles = args.get_or_undefined(2).as_boolean().unwrap_or(false);
        let (dom, node) = resolve_for(&state, handle).map_err(js_error)?;

        let event_object = event
            .as_object()
            .ok_or_else(|| js_error("dispatchEvent expects an event object".to_string()))?;
        let kind = event_object
            .get(JsString::from("type"), ctx)
            .and_then(|value| value.to_string(ctx))
            .map(|text| text.to_std_string_escaped())
            .map_err(|err| js_error(err.to_string()))?;

        // Target first, then ancestors up to the document root. Document-level
        // listeners are registered on the root, so bubbling reaches them here.
        let mut chain = vec![node];
        if bubbles {
            let dom = dom.borrow();
            let mut current = node;
            while let Some(parent) = dom.parent(current) {
                chain.push(parent);
                current = parent;
            }
        }

        let page = page_of(&state).map_err(js_error)?;
        'nodes: for current in chain {
            let callbacks: Vec<u64> = page
                .listeners
                .borrow()
                .iter()
                .filter(|listener| listener.node == current && listener.kind == kind)
                .map(|listener| listener.callback_id)
                .collect();
            for callback_id in callbacks {
                let key =
                    PropertyKey::from(JsString::from(format!("__hodoListener_{callback_id}")));
                let callback = ctx
                    .global_object()
                    .get(key, ctx)
                    .map_err(|err| js_error(err.to_string()))?;
                if let Some(function) = callback.as_callable() {
                    if let Err(err) =
                        function.call(&JsValue::undefined(), std::slice::from_ref(&event), ctx)
                    {
                        state.log(format!("dispatchEvent listener: {}", error_message(&err)));
                    }
                }
            }
            let stopped = event_object
                .get(JsString::from("__hodoStopPropagation"), ctx)
                .map(|value| value.as_boolean() == Some(true))
                .unwrap_or(false);
            if stopped {
                break 'nodes;
            }
        }
        Ok(JsValue::from(true))
    })
}

/// Page-only: the URL the host threaded into `new_page_with_url` (or the
/// `<base>`/empty fallback). No network or navigation is performed here.
fn op_page_url(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let url = state
            .page
            .as_ref()
            .map_or_else(String::new, |page| page.url.clone());
        Ok(JsValue::from(JsString::from(url)))
    })
}

/// Page-only: getter/setter behind `document.title`. The value lives in
/// `RuntimeState` so the host can read it without evaluating JS.
fn op_page_title(id: u64, set: bool) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        if set {
            let title = string_arg(args, 0, ctx).map_err(js_error)?;
            *state.title.borrow_mut() = title;
            Ok(JsValue::undefined())
        } else {
            Ok(JsValue::from(JsString::from(state.title.borrow().clone())))
        }
    })
}

/// Buffer one `document.write` call. Multiple arguments concatenate, matching
/// the web overload; the shell parses the drained text once a script completes.
fn op_doc_write(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        if !state.doc_write_open.get() {
            state.warn_doc_write();
            return Ok(JsValue::undefined());
        }
        let mut text = String::new();
        for (index, _) in args.iter().enumerate() {
            text.push_str(&string_arg(args, index, ctx).unwrap_or_default());
        }
        state.doc_write_buffer.borrow_mut().push_str(&text);
        Ok(JsValue::undefined())
    })
}

/// `document.close()` ends buffering; the shell still drains anything already
/// written when the current script completes.
fn op_doc_close(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, _args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        state.doc_write_open.set(false);
        Ok(JsValue::undefined())
    })
}

pub(crate) fn register_natives(context: &mut boa_engine::Context, id: u64) {
    register(context, "__hodo_page_url", 0, op_page_url(id));
    register(context, "__hodo_doc_write", 1, op_doc_write(id));
    register(context, "__hodo_doc_close", 0, op_doc_close(id));
    register(context, "__hodo_get_title", 0, op_page_title(id, false));
    register(context, "__hodo_set_title", 1, op_page_title(id, true));
    register(context, "__hodo_dom_root", 0, op_dom_root(id));
    register(
        context,
        "__hodo_dom_get_element_by_id",
        1,
        op_dom_get_element_by_id(id),
    );
    register(context, "__hodo_dom_query", 2, op_dom_query(id, false));
    register(context, "__hodo_dom_query_all", 2, op_dom_query(id, true));
    register(context, "__hodo_dom_src", 1, op_dom_resolved_url(id, "src"));
    register(
        context,
        "__hodo_dom_href",
        1,
        op_dom_resolved_url(id, "href"),
    );
    register(
        context,
        "__hodo_dom_create_element",
        1,
        op_dom_create_element(id),
    );
    register(
        context,
        "__hodo_dom_create_fragment",
        0,
        op_dom_create_fragment(id),
    );
    register(
        context,
        "__hodo_dom_set_inner_html",
        2,
        op_dom_set_inner_html(id),
    );
    register(
        context,
        "__hodo_dom_parse_detached",
        1,
        op_dom_parse_detached(id),
    );
    register(
        context,
        "__hodo_dom_attach_shadow",
        1,
        op_dom_attach_shadow(id),
    );
    register(
        context,
        "__hodo_dom_create_text_node",
        1,
        op_dom_create_text_node(id),
    );
    register(
        context,
        "__hodo_dom_create_comment",
        1,
        op_dom_create_comment(id),
    );
    register(context, "__hodo_dom_node_type", 1, op_dom_node_type(id));
    register(
        context,
        "__hodo_dom_append_child",
        2,
        op_dom_append_child(id),
    );
    register(
        context,
        "__hodo_dom_remove_child",
        2,
        op_dom_remove_child(id),
    );
    register(
        context,
        "__hodo_dom_insert_before",
        3,
        op_dom_insert_before(id),
    );
    register(context, "__hodo_dom_text", 1, op_dom_text(id));
    register(context, "__hodo_dom_set_text", 2, op_dom_set_text(id));
    register(context, "__hodo_dom_tag", 1, op_dom_tag(id));
    register(context, "__hodo_dom_children", 1, op_dom_children(id));
    register(context, "__hodo_dom_child_nodes", 1, op_dom_child_nodes(id));
    register(context, "__hodo_dom_parent", 1, op_dom_parent(id));
    register(context, "__hodo_dom_find_tag", 2, op_dom_find_tag(id));
    register(
        context,
        "__hodo_dom_find_all_tag",
        2,
        op_dom_find_all_tag(id),
    );
    register(
        context,
        "__hodo_dom_get_attribute",
        2,
        op_dom_get_attribute(id),
    );
    register(
        context,
        "__hodo_dom_set_attribute",
        3,
        op_dom_set_attribute(id),
    );
    register(context, "__hodo_dom_class_list", 3, op_dom_class_list(id));
    register(
        context,
        "__hodo_dom_add_listener",
        3,
        op_dom_add_listener(id),
    );
    register(
        context,
        "__hodo_dom_remove_listener",
        3,
        op_dom_remove_listener(id),
    );
    register(
        context,
        "__hodo_dom_dispatch_event",
        3,
        op_dom_dispatch_event(id),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExtScriptRuntime;
    use serde_json::json;

    fn sample_dom() -> (Rc<RefCell<Dom>>, NodeId, NodeId) {
        let mut dom = Dom::new();
        let host = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("id".into(), "host".into()), ("class".into(), "x y".into())],
        });
        let kid = dom.create(NodeKind::Element {
            tag: "span".into(),
            attrs: vec![("id".into(), "kid".into()), ("class".into(), "x".into())],
        });
        dom.append(dom.root(), host);
        dom.append(host, kid);
        (Rc::new(RefCell::new(dom)), host, kid)
    }

    #[test]
    fn page_prelude_installs_no_extension_globals() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([typeof chrome, typeof browser, typeof fetch, \
                 typeof i18n, typeof action, typeof document, typeof console])",
            )
            .unwrap();
        // Page fetch is the shell's GET-only loader, not an extension API.
        assert_eq!(
            probe,
            r#"["undefined","undefined","function","undefined","undefined","object","object"]"#
        );
    }

    #[test]
    fn console_logs_are_collected_by_take_logs() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval("console.log('hi'); console.warn('w'); console.error('e');")
            .unwrap();
        assert_eq!(runtime.take_logs(), ["hi", "w", "e"]);
    }

    #[test]
    fn get_element_by_id_and_query_selectors() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        assert_eq!(
            runtime
                .eval("document.getElementById('host').getAttribute('id')")
                .unwrap(),
            "host"
        );
        assert_eq!(
            runtime
                .eval("document.querySelectorAll('.x').length")
                .unwrap(),
            "2"
        );
        assert_eq!(
            runtime
                .eval("document.querySelector('span').getAttribute('id')")
                .unwrap(),
            "kid"
        );
        // hodo-css also provides combinators: `div > span` now matches for real.
        assert_eq!(
            runtime
                .eval("document.querySelector('div > span').getAttribute('id')")
                .unwrap(),
            "kid"
        );
        // Genuinely unsupported syntax still yields null.
        assert_eq!(
            runtime
                .eval("String(document.querySelector(':nope'))")
                .unwrap(),
            "null"
        );
        assert_eq!(
            runtime
                .eval("String(document.getElementById('missing'))")
                .unwrap(),
            "null"
        );
    }

    #[test]
    fn create_element_and_append_child_mutate_the_arena() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval("var d = document.createElement('em'); document.getElementById('host').appendChild(d);")
            .unwrap();
        runtime.tick();
        assert_eq!(dom.borrow().children(host).len(), 2);
        assert_eq!(
            runtime
                .eval("document.querySelector('em').getAttribute('class')")
                .unwrap(),
            "null"
        );
    }

    #[test]
    fn class_list_add_remove_toggle_contains() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let result = runtime
            .eval(
                "var el = document.getElementById('host'); \
                 el.classList.add('z'); var a = el.classList.contains('z'); \
                 el.classList.remove('x'); var b = el.classList.contains('x'); \
                 var c = el.classList.toggle('q'); var d = el.classList.toggle('q'); \
                 JSON.stringify([a, b, c, d, el.getAttribute('class')]);",
            )
            .unwrap();
        assert_eq!(result, r#"[true,false,true,false,"y z"]"#);
    }

    #[test]
    fn text_content_get_and_set_replace_children() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        assert_eq!(
            runtime
                .eval("document.getElementById('host').textContent")
                .unwrap(),
            ""
        );
        runtime
            .eval("document.getElementById('host').textContent = 'hello';")
            .unwrap();
        runtime.tick();
        assert_eq!(
            runtime
                .eval("document.getElementById('host').innerText")
                .unwrap(),
            "hello"
        );
        let dom = dom.borrow();
        let children = dom.children(host);
        assert_eq!(children.len(), 1);
        assert_eq!(dom.text(children[0]), Some("hello"));
    }

    #[test]
    fn stale_handle_after_detach_and_slot_reuse_is_a_clean_error() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let result = runtime
            .eval(
                "var host = document.getElementById('host'); \
                 var kid = document.getElementById('kid'); \
                 host.removeChild(kid); \
                 var fresh = document.createElement('fresh'); \
                 var caught = 'none'; \
                 try { kid.textContent; } catch (e) { caught = 'stale'; } \
                 caught;",
            )
            .unwrap();
        assert_eq!(result, "stale");
    }

    #[test]
    fn listener_registration_and_page_event_dispatch() {
        let (dom, _, _) = sample_dom();
        let root = dom.borrow().root();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "document.addEventListener('click', function (e) { console.log('clicked:' + e); }); \
                 document.addEventListener('DOMContentLoaded', function () { console.log('ready'); });",
            )
            .unwrap();
        runtime.dispatch_page_event(root, "click", json!("ok"));
        runtime.dispatch_page_event(root, "DOMContentLoaded", json!({}));
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["clicked:ok", "ready"]);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn page_contexts_have_independent_dom_handles() {
        let (first, _, _) = sample_dom();
        let (second, _, _) = sample_dom();
        let mut a = ExtScriptRuntime::new_page(first);
        let mut b = ExtScriptRuntime::new_page(second);
        assert_eq!(
            a.eval("document.getElementById('host').getAttribute('id')")
                .unwrap(),
            "host"
        );
        assert_eq!(
            b.eval("document.getElementById('host').getAttribute('id')")
                .unwrap(),
            "host"
        );
        let _ = &mut a;
        let _ = &mut b;
    }

    #[test]
    fn remove_event_listener_drops_only_the_matching_callback() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var a = function () { console.log('a'); }; \
                 var b = function () { console.log('b'); }; \
                 window.addEventListener('click', a); \
                 window.addEventListener('click', b); \
                 window.removeEventListener('click', a); \
                 window.dispatchEvent({ type: 'click', bubbles: false }); \
                 JSON.stringify(true)",
            )
            .unwrap();
        assert_eq!(probe, "true");
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["b"]);
    }

    #[test]
    fn window_add_event_listener_fires_on_document_event() {
        let (dom, _, _) = sample_dom();
        let root = dom.borrow().root();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "window.addEventListener('DOMContentLoaded', function (e) { \
                 console.log('ready:' + e); });",
            )
            .unwrap();
        runtime.dispatch_page_event(root, "DOMContentLoaded", json!("go"));
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["ready:go"]);
    }

    #[test]
    fn window_set_timeout_schedules_on_the_existing_timer_registry() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval("window.setTimeout(function () { console.log('fired'); }, 1);")
            .unwrap();
        assert_eq!(runtime.timer_count(), 1);
        std::thread::sleep(std::time::Duration::from_millis(5));
        runtime.tick();
        assert_eq!(runtime.timer_count(), 0);
        assert_eq!(runtime.take_logs(), ["fired"]);
    }

    #[test]
    fn window_location_href_is_the_threaded_url() {
        let (dom, _, _) = sample_dom();
        let mut runtime =
            ExtScriptRuntime::new_page_with_url(dom, "https://archive.org/details/foo?a=1");
        assert_eq!(
            runtime.eval("window.location.href").unwrap(),
            "https://archive.org/details/foo?a=1"
        );
        // Wave-2 navigation surface: href is writable and re-parsed in JS; no
        // network or popstate reaction happens.
        assert_eq!(
            runtime
                .eval("window.location.href = 'https://elsewhere.test/'; window.location.href")
                .unwrap(),
            "https://elsewhere.test/"
        );
    }

    #[test]
    fn window_location_falls_back_to_base_href() {
        let mut dom = Dom::new();
        let base = dom.create(NodeKind::Element {
            tag: "base".into(),
            attrs: vec![("href".into(), "https://example.org/base/".into())],
        });
        dom.append(dom.root(), base);
        let mut runtime = ExtScriptRuntime::new_page(Rc::new(RefCell::new(dom)));
        assert_eq!(
            runtime.eval("window.location.href").unwrap(),
            "https://example.org/base/"
        );
    }

    fn nested_dom() -> (Rc<RefCell<Dom>>, NodeId) {
        let mut dom = Dom::new();
        let head = dom.create(NodeKind::Element {
            tag: "head".into(),
            attrs: vec![("data-x".into(), "1".into())],
        });
        let body = dom.create(NodeKind::Element {
            tag: "body".into(),
            attrs: vec![("id".into(), "b".into()), ("class".into(), "c d".into())],
        });
        let para = dom.create(NodeKind::Element {
            tag: "p".into(),
            attrs: vec![("id".into(), "kid".into())],
        });
        let text = dom.create(NodeKind::Text("hello".into()));
        let span = dom.create(NodeKind::Element {
            tag: "span".into(),
            attrs: Vec::new(),
        });
        dom.append(dom.root(), head);
        dom.append(dom.root(), body);
        dom.append(body, text);
        dom.append(body, para);
        dom.append(body, span);
        (Rc::new(RefCell::new(dom)), body)
    }

    #[test]
    fn children_filters_text_nodes_and_returns_wrapped_elements() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var b = document.body; \
                 JSON.stringify([b.children.length, b.children[0].tagName, \
                 b.children[1].tagName, b.children[0] instanceof HTMLElement, \
                 b.children[0].getAttribute('id')]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[2,"P","SPAN",true,"kid"]"#);
    }

    #[test]
    fn attributes_view_reports_real_names_and_values() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var a = document.body.attributes; \
                 JSON.stringify([a.length, a.getNamedItem('id').value, \
                 a.getNamedItem('class').value, a[0].name, \
                 a.getNamedItem('missing')]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[2,"b","c d","id",null]"#);
    }

    #[test]
    fn parent_node_and_parent_element_resolve_the_real_parent() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([document.body.parentNode.nodeName, \
                 document.body.parentElement, \
                 document.body.children[0].parentElement.getAttribute('id'), \
                 document.parentNode]);",
            )
            .unwrap();
        assert_eq!(probe, r##"["#document",null,"b",null]"##);
    }

    #[test]
    fn document_head_and_body_are_real_handles() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([document.head.tagName, document.body.tagName, \
                 document.head.getAttribute('data-x'), document.body.getAttribute('id'), \
                 String(document.currentScript)]);",
            )
            .unwrap();
        assert_eq!(probe, r#"["HEAD","BODY","1","b","null"]"#);
    }

    #[test]
    fn wrapped_elements_stringify_without_parent_cycles() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime.eval("JSON.stringify(document.body)").unwrap();
        assert!(
            !probe.contains("children")
                && !probe.contains("parentNode")
                && !probe.contains("attributes"),
            "unexpected: {probe}"
        );
    }

    #[test]
    fn navigator_is_frozen_and_supports_feature_detection() {
        let (dom, _) = nested_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([Object.isFrozen(navigator), \
                 navigator.userAgent.indexOf('hodo') !== -1, navigator.language, \
                 navigator.languages[0], navigator.platform, navigator.vendor, \
                 navigator.foo === undefined, Boolean(navigator.mediaDevices), \
                 navigator === window.navigator, self === window]);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,"en-US","en-US","Win32","",true,false,true,true]"#
        );
    }

    #[test]
    fn document_title_roundtrips_into_runtime_state() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        assert_eq!(runtime.eval("document.title").unwrap(), "");
        runtime.eval("document.title = 'Archive Page';").unwrap();
        assert_eq!(runtime.eval("document.title").unwrap(), "Archive Page");
        assert_eq!(
            runtime.eval("window.document.title").unwrap(),
            "Archive Page"
        );
        assert_eq!(runtime.document_title(), "Archive Page");
    }

    #[test]
    fn dom_constructor_globals_are_page_only() {
        let (dom, _, _) = sample_dom();
        let mut page = ExtScriptRuntime::new_page(dom);
        assert_eq!(
            page.eval(
                "JSON.stringify([typeof Node, typeof Element, typeof HTMLElement, typeof Document])"
            )
            .unwrap(),
            r#"["function","function","function","function"]"#
        );
        let mut ext = ExtScriptRuntime::new();
        assert_eq!(
            ext.eval(
                "JSON.stringify([typeof Node, typeof Element, typeof HTMLElement, typeof Document])"
            )
            .unwrap(),
            r#"["undefined","undefined","undefined","undefined"]"#
        );
    }

    #[test]
    fn wrapped_elements_and_document_have_the_dom_prototype_chain() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var e = document.createElement('div'); \
                 var g = document.getElementById('host'); \
                 var q = document.querySelector('span'); \
                 var qa = document.querySelectorAll('span')[0]; \
                 JSON.stringify([\
                   e instanceof HTMLElement, e instanceof Element, e instanceof Node, \
                   g instanceof HTMLElement, q instanceof Element, qa instanceof Node, \
                   document instanceof Document, document instanceof Node, \
                   document instanceof Element])",
            )
            .unwrap();
        assert_eq!(probe, "[true,true,true,true,true,true,true,true,false]");
    }

    #[test]
    fn character_data_family_has_real_chains_and_illegal_direct_constructors() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var t = document.createTextNode('hi'); \
                 var c = document.createComment('note'); \
                 var illegal = []; \
                 ['CharacterData', 'Text', 'Comment'].forEach(function (n) { \
                   var C = window[n]; \
                   try { new C(); illegal.push('no-throw'); } \
                   catch (e) { illegal.push(e instanceof TypeError); } \
                 }); \
                 function MyText() { Text.call(this); } \
                 MyText.prototype = Object.create(Text.prototype); \
                 MyText.prototype.constructor = MyText; \
                 var mt = new MyText(); \
                 JSON.stringify([\
                   t instanceof Text, t instanceof CharacterData, t instanceof Node, \
                   c instanceof Comment, c instanceof CharacterData, c instanceof Node, \
                   t.nodeType, t.nodeName, c.nodeType, c.nodeName, \
                   illegal[0], illegal[1], illegal[2], \
                   mt instanceof Text, mt instanceof CharacterData, mt instanceof Node])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[true,true,true,true,true,true,3,"#text",8,"#comment",true,true,true,true,true,true]"##
        );
    }

    #[test]
    fn create_text_node_appends_a_real_arena_text_node() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval(
                "var t = document.createTextNode('HELLO'); \
                 document.getElementById('host').appendChild(t);",
            )
            .unwrap();
        runtime.tick();
        {
            let dom = dom.borrow();
            let children = dom.children(host);
            assert_eq!(children.len(), 2);
            assert_eq!(dom.text(children[0]), None);
            assert_eq!(dom.text(children[1]), Some("HELLO"));
        }
        assert_eq!(
            runtime
                .eval("document.getElementById('host').textContent")
                .unwrap(),
            "HELLO"
        );
    }

    #[test]
    fn create_comment_never_contributes_text_or_children() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var c = document.createComment('SECRET'); h.appendChild(c); \
                 JSON.stringify([c.nodeType, c.nodeName, c.data, c.nodeValue, \
                                 c.textContent, h.textContent, c.childNodes.length, \
                                 h.childNodes.length])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[8,"#comment","SECRET","SECRET","SECRET","",0,2]"##
        );
        {
            let dom = dom.borrow();
            let children = dom.children(host);
            assert_eq!(dom.tag(children[1]), Some("#comment"));
            assert_eq!(dom.attr(children[1], "data"), Some("SECRET"));
            assert!(dom.children(children[1]).is_empty());
        }
    }

    #[test]
    fn character_data_writes_mutate_the_arena_and_keep_sibling_order() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var a = document.createTextNode('A'); \
                 var b = document.createElement('b'); \
                 var c = document.createTextNode('C'); \
                 h.appendChild(a); h.appendChild(b); h.appendChild(c); \
                 a.data = 'AA'; \
                 b.appendChild(document.createTextNode('B')); \
                 c.nodeValue = 'CC';",
            )
            .unwrap();
        runtime.tick();
        {
            let dom = dom.borrow();
            let children = dom.children(host);
            let texts: Vec<_> = children
                .iter()
                .map(|&id| dom.text(id).unwrap_or("").to_string())
                .collect();
            assert_eq!(texts, ["", "AA", "", "CC"]);
            assert_eq!(dom.tag(children[0]), Some("span"));
            assert_eq!(dom.tag(children[2]), Some("b"));
        }
        assert_eq!(
            runtime
                .eval("document.getElementById('host').textContent")
                .unwrap(),
            "AABCC"
        );
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 JSON.stringify([h.childNodes[1].data, h.childNodes[1].length, \
                                 h.childNodes[3].data, h.firstChild.nodeName])",
            )
            .unwrap();
        assert_eq!(probe, r#"["AA",2,"CC","SPAN"]"#);
    }

    #[test]
    fn child_nodes_hands_out_typed_text_and_comment_wrappers() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var t = document.createTextNode('T'); \
                 var c = document.createComment('C'); \
                 h.appendChild(t); h.appendChild(c); \
                 var n = h.childNodes, ch = h.children; \
                 JSON.stringify([n.length, n[0].nodeType, n[1].nodeType, n[2].nodeType, \
                   n[1] instanceof Text, n[1] instanceof CharacterData, n[1] instanceof Node, \
                   n[2] instanceof Comment, n[2] instanceof CharacterData, n[2] instanceof Node, \
                   n[1].data, n[2].data, n[1].nodeName, n[2].nodeName, \
                   ch[1] instanceof Comment, ch[1].nodeType, t.length, n[1].length])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[3,1,3,8,true,true,true,true,true,true,"T","C","#text","#comment",true,8,1,1]"##
        );
    }

    #[test]
    fn insert_before_handles_order_fragments_null_ref_and_bad_ref() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var a = document.createElement('a'), b = document.createElement('b'); \
                 h.appendChild(a); h.appendChild(b); \
                 var x = document.createElement('x'); h.insertBefore(x, a); \
                 var y = document.createElement('y'); h.insertBefore(y, null); \
                 var f = document.createDocumentFragment(); \
                 var p = document.createElement('p'), q = document.createElement('q'); \
                 f.appendChild(p); f.appendChild(q); \
                 h.insertBefore(f, b); \
                 var order = [], kids = h.children; \
                 for (var i = 0; i < kids.length; i++) order.push(kids[i].tagName.toLowerCase()); \
                 var stray = document.createElement('stray'); var bad = false; \
                 try { h.insertBefore(stray, stray); } catch (e) { bad = e instanceof TypeError; } \
                 JSON.stringify([order.join(','), bad, f.childNodes.length])",
            )
            .unwrap();
        assert_eq!(probe, r#"["span,x,a,p,q,b,y",true,0]"#);
    }

    #[test]
    fn sibling_getters_walk_a_mixed_tree() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var kid = document.getElementById('kid'); \
                 var t = document.createTextNode('T'), c = document.createComment('C'); \
                 h.appendChild(t); h.appendChild(c); \
                 JSON.stringify([h.firstChild.nodeType, h.lastChild.nodeType, \
                   kid.previousSibling === null, kid.nextSibling.nodeType, \
                   t.previousSibling.nodeType, t.nextSibling.nodeType, \
                   c.nextSibling === null, c.previousSibling.nodeType, \
                   t.parentNode.nodeType, t instanceof CharacterData, \
                   c instanceof CharacterData])",
            )
            .unwrap();
        assert_eq!(probe, "[1,8,true,3,1,8,true,3,1,true,true]");
    }

    #[test]
    fn node_constants_match_the_dom_spec() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([Node.ELEMENT_NODE, Node.ATTRIBUTE_NODE, Node.TEXT_NODE, \
                 Node.CDATA_SECTION_NODE, Node.ENTITY_REFERENCE_NODE, Node.ENTITY_NODE, \
                 Node.PROCESSING_INSTRUCTION_NODE, Node.COMMENT_NODE, Node.DOCUMENT_NODE, \
                 Node.DOCUMENT_TYPE_NODE, Node.DOCUMENT_FRAGMENT_NODE, Node.NOTATION_NODE])",
            )
            .unwrap();
        assert_eq!(probe, "[1,2,3,4,5,6,7,8,9,10,11,12]");
    }

    #[test]
    fn node_type_and_tag_name_come_from_the_real_node() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var host = document.getElementById('host'); \
                 var kid = document.querySelector('span'); \
                 var made = document.createElement('em'); \
                 JSON.stringify([host.nodeType, host.tagName, host.nodeName, \
                 kid.tagName, made.tagName, document.nodeType, document.nodeName, \
                 host.parentNode.nodeName, String(host.parentElement)])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[1,"DIV","DIV","SPAN","EM",9,"#document","#document","null"]"##
        );
    }

    #[test]
    fn direct_dom_constructor_invocation_throws() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 ['Node', 'Element', 'HTMLElement', 'Document'].forEach(function (n) { \
                   var C = window[n]; \
                   try { new C(); out.push('no-throw'); } \
                   catch (e) { out.push(e.name + ': ' + e.message); } \
                 }); \
                 JSON.stringify(out)",
            )
            .unwrap();
        assert!(!probe.contains("no-throw"), "unexpected: {probe}");
        assert_eq!(
            probe,
            r#"["TypeError: Illegal constructor: Node","TypeError: Illegal constructor: Element","TypeError: Illegal constructor: HTMLElement","TypeError: Illegal constructor: Document"]"#
        );
    }

    #[test]
    fn class_extends_htmlellement_parses_and_constructs() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "class Foo extends HTMLElement {} \
                 var f = new Foo(); \
                 JSON.stringify([f instanceof Foo, f instanceof HTMLElement, \
                 f instanceof Element, f instanceof Node])",
            )
            .unwrap();
        assert_eq!(probe, "[true,true,true,true]");
    }

    #[test]
    fn document_fragment_constructs_and_is_a_node() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var a = new DocumentFragment(); \
                 var b = document.createDocumentFragment(); \
                 class Frag extends DocumentFragment {} \
                 var c = new Frag(); \
                 JSON.stringify([a instanceof DocumentFragment, a instanceof Node, \
                 b instanceof DocumentFragment, b instanceof Node, \
                 c instanceof Frag, c instanceof DocumentFragment, c instanceof Node, \
                 a.nodeType, a.nodeName, a.tagName, b.nodeType, b.childNodes.length])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[true,true,true,true,true,true,true,11,"#document-fragment",null,11,0]"##
        );
    }

    #[test]
    fn document_fragment_children_flatten_into_live_dom() {
        let (dom, host, kid) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval(
                "var f = document.createDocumentFragment(); \
                 f.appendChild(document.createElement('b')); \
                 f.appendChild(document.createElement('i')); \
                 window.__before = JSON.stringify([f.children.length, f.childNodes.length, \
                 f.children[0].tagName, f.children[1].tagName]); \
                 document.getElementById('host').appendChild(f); \
                 window.__after = JSON.stringify([f.children.length, f.childNodes.length, \
                 document.getElementById('host').children.length, \
                 document.getElementById('host').children[1].tagName, \
                 document.getElementById('host').children[2].tagName]);",
            )
            .unwrap();
        assert_eq!(runtime.eval("window.__before").unwrap(), r#"[2,2,"B","I"]"#);
        assert_eq!(
            runtime.eval("window.__after").unwrap(),
            r#"[0,0,3,"B","I"]"#
        );
        let kids = dom.borrow().children(host).to_vec();
        let tags: Vec<_> = kids
            .iter()
            .map(|id| dom.borrow().tag(*id).unwrap().to_string())
            .collect();
        assert_eq!(kids[0], kid);
        assert_eq!(tags, ["span", "b", "i"]);
    }

    #[test]
    fn document_fragment_is_reusable_after_emptying() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var f = document.createDocumentFragment(); \
                 var host = document.getElementById('host'); \
                 f.appendChild(document.createElement('b')); host.appendChild(f); \
                 f.appendChild(document.createElement('i')); host.appendChild(f); \
                 JSON.stringify([f.children.length, host.children.length, \
                 host.children[1].tagName, host.children[2].tagName, \
                 f instanceof DocumentFragment])",
            )
            .unwrap();
        assert_eq!(probe, r#"[0,3,"B","I",true]"#);
        assert_eq!(dom.borrow().children(host).len(), 3);
    }

    #[test]
    fn fragments_never_enter_the_rendered_tree() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval(
                "var f = document.createDocumentFragment(); \
                 f.appendChild(document.createElement('b')); \
                 f.appendChild(document.createElement('i'));",
            )
            .unwrap();
        let mut tags = Vec::new();
        dom.borrow().walk(|id, _| {
            if let Some(tag) = dom.borrow().tag(id) {
                tags.push(tag.to_string());
            }
        });
        assert_eq!(tags, ["div", "span"]);
    }

    #[test]
    fn range_is_unsupported_and_import_node_clones() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 function probe(fn) { \
                   try { fn(); out.push('no-throw'); } \
                   catch (e) { out.push(e.name + ':' + /not supported/.test(e.message)); } \
                 } \
                 probe(function () { document.createRange(); }); \
                 probe(function () { Range; }); \
                 var original = document.createElement('b'); \
                 original.setAttribute('id', 'copied'); \
                 var copy = document.importNode(original, true); \
                 out.push(copy !== original && copy.getAttribute('id') === 'copied' \
                   ? 'clone' : 'bad'); \
                 JSON.stringify(out)",
            )
            .unwrap();
        assert_eq!(probe, r#"["TypeError:true","TypeError:true","clone"]"#);
    }

    #[test]
    fn clone_node_shallow_is_detached_and_copies_attributes() {
        let (dom, host, kid) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var c = h.cloneNode(); \
                 JSON.stringify([c !== h, c.tagName, c.getAttribute('id'), \
                   c.getAttribute('class'), c.children.length, c.childNodes.length, \
                   c.parentNode, c instanceof HTMLElement, c instanceof Node])",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,"DIV","host","x y",0,0,null,true,true]"#);
        // The live tree is untouched and the source keeps its child.
        assert_eq!(dom.borrow().children(host), [kid]);
    }

    #[test]
    fn clone_node_deep_is_recursive_detached_and_roundtrips_text_and_comments() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        runtime
            .eval(
                "var h = document.getElementById('host'); \
                 h.appendChild(document.createTextNode('T')); \
                 h.appendChild(document.createComment('C')); \
                 window.__copy = h.cloneNode(true);",
            )
            .unwrap();
        // Deep copy has the same kinds in order, but lives outside the document.
        assert_eq!(
            runtime
                .eval(
                    "var c = window.__copy, n = c.childNodes, out = []; \
                     for (var i = 0; i < n.length; i++) out.push([n[i].nodeType, n[i].data]); \
                     JSON.stringify([c.getAttribute('id'), out, c.parentNode, \
                       c.children[0].tagName, c.children[0].getAttribute('id')])"
                )
                .unwrap(),
            r##"["host",[[1,null],[3,"T"],[8,"C"]],null,"SPAN","kid"]"##
        );
        // Mutating the clone does not touch the original.
        runtime
            .eval(
                "window.__copy.setAttribute('id', 'copy'); \
                 window.__copy.children[0].setAttribute('id', 'copy-kid');",
            )
            .unwrap();
        assert_eq!(
            runtime
                .eval(
                    "JSON.stringify([document.getElementById('host').getAttribute('id'), \
                       document.getElementById('kid').getAttribute('id'), \
                       window.__copy.getAttribute('id')])"
                )
                .unwrap(),
            r#"["host","kid","copy"]"#
        );
        assert_eq!(dom.borrow().children(host).len(), 3);
    }

    #[test]
    fn clone_node_deep_on_a_fragment_keeps_its_children_detached() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var f = document.createDocumentFragment(); \
                 f.appendChild(document.createElement('b')); \
                 f.appendChild(document.createTextNode('txt')); \
                 var c = f.cloneNode(true); \
                 JSON.stringify([c instanceof DocumentFragment, c !== f, \
                   c.childNodes.length, c.children[0].tagName, c.childNodes[1].data, \
                   c.parentNode, f.childNodes.length, \
                   document.createDocumentFragment().cloneNode() instanceof DocumentFragment])",
            )
            .unwrap();
        assert_eq!(probe, r##"[true,true,2,"B","txt",null,2,true]"##);
    }

    #[test]
    fn window_location_reports_every_field_from_the_threaded_url() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page_with_url(
            dom,
            "https://archive.org:8443/details/foo?a=1&b=2#frag",
        );
        let probe = runtime
            .eval(
                "var l = window.location; \
                 JSON.stringify([l.href, l.protocol, l.host, l.hostname, l.port, l.origin, \
                   l.pathname, l.search, l.search.slice(1), l.hash, \
                   Object.isFrozen(l)])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"["https://archive.org:8443/details/foo?a=1&b=2#frag","https:","archive.org:8443","archive.org","8443","https://archive.org:8443","/details/foo","?a=1&b=2","a=1&b=2","#frag",false]"##
        );
    }

    #[test]
    fn window_location_on_a_bare_url_has_empty_search_and_hash() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page_with_url(dom, "https://archive.org");
        let probe = runtime
            .eval(
                "var l = location; \
                 JSON.stringify([l.href, l.protocol, l.host, l.hostname, l.port, l.origin, \
                   l.pathname, l.search, l.search.slice(1), l.hash])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["https://archive.org","https:","archive.org","archive.org","","https://archive.org","/","","",""]"#
        );
    }

    #[test]
    fn window_and_custom_elements_have_their_class_surfaces() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 ['CustomElementRegistry', 'Window'].forEach(function (n) { \
                   var C = window[n]; \
                   try { new C(); out.push('no-throw'); } \
                   catch (e) { out.push(e instanceof TypeError); } \
                 }); \
                 JSON.stringify(out.concat([\
                   window.customElements instanceof CustomElementRegistry, \
                   window instanceof Window, \
                   typeof CustomElementRegistry.prototype.define !== 'undefined', \
                   typeof Window.prototype !== 'undefined']))",
            )
            .unwrap();
        assert_eq!(probe, "[true,true,true,true,false,true]");
    }

    /// Document-order fixture: scripts, img, anchor, div with assorted attrs.
    fn url_dom() -> Rc<RefCell<Dom>> {
        let mut dom = Dom::new();
        let script = |src: &str, id: &str, class: &str| NodeKind::Element {
            tag: "script".into(),
            attrs: vec![
                ("src".into(), src.into()),
                ("id".into(), id.into()),
                ("class".into(), class.into()),
            ],
        };
        let nodes = vec![
            dom.create(script(
                "/_static/js/webcomponents-loader.js",
                "s1",
                "loader",
            )),
            dom.create(script(
                "https://cdn.example/webcomponents-loader.js",
                "s2",
                "loader extra",
            )),
            dom.create(NodeKind::Element {
                tag: "img".into(),
                attrs: vec![
                    ("src".into(), "pic.png".into()),
                    ("alt".into(), "photo".into()),
                ],
            }),
            dom.create(NodeKind::Element {
                tag: "a".into(),
                attrs: vec![
                    ("href".into(), "bar?x=1".into()),
                    ("class".into(), "link".into()),
                ],
            }),
            dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![
                    ("data-role".into(), "card".into()),
                    ("id".into(), "main".into()),
                ],
            }),
            dom.create(NodeKind::Element {
                tag: "link".into(),
                attrs: vec![("href".into(), "../up.css".into())],
            }),
        ];
        for node in nodes {
            dom.append(dom.root(), node);
        }
        Rc::new(RefCell::new(dom))
    }

    #[test]
    fn attribute_selectors_operators_quotes_compounds_and_lists() {
        let mut runtime = ExtScriptRuntime::new_page(url_dom());
        let mut ids = |js: &str| runtime.eval(js).unwrap();
        // The archive.org blocker form: contains operator with a quoted value.
        assert_eq!(
            ids("document.querySelector('script[src*=\"webcomponents-loader\"]').getAttribute('id')"),
            "s1"
        );
        assert_eq!(
            ids("var n='webcomponents-loader'; document.querySelector('script[src*=\"'+n+'\"]').getAttribute('id')"),
            "s1"
        );
        assert_eq!(
            ids("document.querySelectorAll('script[src*=\"webcomponents-loader\"]').length"),
            "2"
        );
        // Bare and quoted equality, contains, prefix, suffix, exists.
        assert_eq!(
            ids("document.querySelector('[data-role=card]').getAttribute('id')"),
            "main"
        );
        assert_eq!(
            ids("document.querySelector('[data-role=\"card\"]').getAttribute('id')"),
            "main"
        );
        assert_eq!(ids("document.querySelectorAll('[src]').length"), "3");
        assert_eq!(
            ids("document.querySelector('script[src^=\"/_static\"]').getAttribute('id')"),
            "s1"
        );
        assert_eq!(
            ids("document.querySelectorAll('script[src$=\".js\"]').length"),
            "2"
        );
        // HTML attribute names are case-insensitive; values are not.
        assert_eq!(
            ids("document.querySelectorAll('[SRC*=\"webcomponents\"]').length"),
            "2"
        );
        assert_eq!(
            ids("document.querySelectorAll('[src*=\"WEBCOMPONENTS\"]').length"),
            "0"
        );
        // Compounds and comma lists, in document order.
        assert_eq!(
            ids("document.querySelector('script#s1[src*=\"loader\"]').getAttribute('id')"),
            "s1"
        );
        assert_eq!(ids("document.querySelectorAll('.loader[src]').length"), "2");
        assert_eq!(ids("document.querySelectorAll('img, a.link').length"), "2");
        assert_eq!(
            ids(
                "JSON.stringify([].map.call(document.querySelectorAll('[src], a'), function(e){return e.tagName;}))"
            ),
            r#"["SCRIPT","SCRIPT","IMG","A"]"#
        );
        // Absent attribute: querySelector null, querySelectorAll empty.
        assert_eq!(
            ids("String(document.querySelector('[data-missing]'))"),
            "null"
        );
        assert_eq!(
            ids("document.querySelectorAll('[data-missing]').length"),
            "0"
        );
        // Unsupported syntax stays an honest empty result.
        assert_eq!(ids("String(document.querySelector(':nope'))"), "null");
    }

    #[test]
    fn src_and_href_reflect_resolved_urls_and_raw_fallback() {
        let mut runtime =
            ExtScriptRuntime::new_page_with_url(url_dom(), "https://archive.org/details/foo?x=1");
        assert_eq!(
            runtime.eval("document.getElementById('s1').src").unwrap(),
            "https://archive.org/_static/js/webcomponents-loader.js"
        );
        assert_eq!(
            runtime.eval("document.getElementById('s2').src").unwrap(),
            "https://cdn.example/webcomponents-loader.js"
        );
        assert_eq!(
            runtime.eval("document.querySelector('img').src").unwrap(),
            "https://archive.org/details/pic.png"
        );
        assert_eq!(
            runtime.eval("document.querySelector('a').href").unwrap(),
            "https://archive.org/details/bar?x=1"
        );
        assert_eq!(
            runtime.eval("document.querySelector('link').href").unwrap(),
            "https://archive.org/up.css"
        );
        // No src attribute -> null; direct assignment writes through to the DOM.
        assert_eq!(
            runtime.eval("document.getElementById('main').src").unwrap(),
            "null"
        );
        runtime
            .eval("document.getElementById('main').src = 'https://evil.test/x';")
            .unwrap();
        assert_eq!(
            runtime.eval("document.getElementById('main').src").unwrap(),
            "https://evil.test/x"
        );
        runtime
            .eval("document.getElementById('main').setAttribute('src','next.png');")
            .unwrap();
        assert_eq!(
            runtime.eval("document.getElementById('main').src").unwrap(),
            "https://archive.org/details/next.png"
        );
        // Without a threaded page URL the raw attribute value is reported.
        let mut raw = ExtScriptRuntime::new_page(url_dom());
        assert_eq!(
            raw.eval("document.querySelector('img').src").unwrap(),
            "pic.png"
        );
    }
    /// Fixture with a `<body>` holding two `<x-card>` elements already in the
    /// document, so `define` exercises the upgrade walk.
    fn custom_dom() -> (Rc<RefCell<Dom>>, NodeId, NodeId) {
        let mut dom = Dom::new();
        let body = dom.create(NodeKind::Element {
            tag: "body".into(),
            attrs: Vec::new(),
        });
        let first = dom.create(NodeKind::Element {
            tag: "x-card".into(),
            attrs: Vec::new(),
        });
        let second = dom.create(NodeKind::Element {
            tag: "x-card".into(),
            attrs: Vec::new(),
        });
        dom.append(dom.root(), body);
        dom.append(body, first);
        dom.append(body, second);
        (Rc::new(RefCell::new(dom)), body, first)
    }

    #[test]
    fn custom_elements_define_and_get_round_trip() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var C = class XCard extends HTMLElement {}; \
                 customElements.define('x-card', C); \
                 typeof customElements.define + '|' + typeof customElements.get + '|' + \
                 typeof customElements.whenDefined + '|' + \
                 (customElements.get('x-card') === C) + '|' + \
                 (customElements.get('x-nope') === null);",
            )
            .unwrap();
        assert_eq!(probe, "function|function|function|true|true");
    }

    #[test]
    fn custom_elements_duplicate_define_throws() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var C = class XCard extends HTMLElement {}; \
                 customElements.define('x-card', C); \
                 var out = 'no-throw'; \
                 try { customElements.define('x-card', C); } catch (e) { out = e.name; } \
                 out;",
            )
            .unwrap();
        assert_eq!(probe, "Error");
    }

    #[test]
    fn custom_elements_name_must_contain_a_hyphen() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 ['noHyphen', 'x-', '-x', 'ok-el'].forEach(function (n) { \
                   try { customElements.define(n, class extends HTMLElement {}); out.push('ok'); } \
                   catch (e) { out.push(e.name); } \
                 }); \
                 out.join(',');",
            )
            .unwrap();
        assert_eq!(probe, "SyntaxError,SyntaxError,SyntaxError,ok");
    }

    #[test]
    fn custom_elements_upgrade_existing_tags_with_working_methods() {
        let (dom, _, first) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var all = []; \
                 var C = class XCard extends HTMLElement { \
                   constructor() { super(); all.push(this); } \
                   setLabel(v) { this.textContent = v; } \
                 }; \
                 customElements.define('x-card', C); \
                 all[0].setLabel('hello'); \
                 JSON.stringify([all.length, all[0] instanceof C, \
                   document.body.children.length, \
                   document.body.children[0] === all[0], \
                   document.body.children[1] === all[1], \
                   document.body.children[0].textContent]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[2,true,2,true,true,"hello"]"#);
        runtime.tick();
        let dom = dom.borrow();
        let children = dom.children(first);
        assert_eq!(children.len(), 1);
        assert_eq!(dom.text(children[0]), Some("hello"));
    }

    #[test]
    fn custom_elements_create_element_constructs_the_class() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var C = class XItem extends HTMLElement {}; \
                 customElements.define('x-item', C); \
                 var el = document.createElement('x-item'); \
                 var plain = document.createElement('div'); \
                 JSON.stringify([el instanceof C, el instanceof HTMLElement, \
                   el.tagName, plain instanceof C]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,true,"X-ITEM",false]"#);
    }

    #[test]
    fn custom_elements_connected_callback_on_upgrade_and_append() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var seen = []; \
                 var C = class XThing extends HTMLElement { \
                   connectedCallback() { seen.push(this.tagName); } \
                 }; \
                 customElements.define('x-card', C); \
                 var upgraded = seen.length; \
                 var el = document.createElement('x-card'); \
                 var afterCreate = seen.length; \
                 document.body.appendChild(el); \
                 document.body.appendChild(el); \
                 JSON.stringify([upgraded, afterCreate, seen]);",
            )
            .unwrap();
        // Two in-DOM <x-card> tags upgrade and connect; createElement alone
        // does not connect; a repeated append must not fire twice.
        assert_eq!(probe, r#"[2,2,["X-CARD","X-CARD","X-CARD"]]"#);
    }

    #[test]
    fn fragment_children_connect_only_when_moved_into_the_live_tree() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var seen = []; \
                 customElements.define('x-card', class XThing extends HTMLElement { \
                   connectedCallback() { seen.push(this.tagName); } \
                 }); \
                 var afterDefine = seen.length; \
                 var f = document.createDocumentFragment(); \
                 f.appendChild(document.createElement('x-card')); \
                 var inFragment = seen.length; \
                 document.body.appendChild(f); \
                 JSON.stringify([afterDefine, inFragment, seen.length])",
            )
            .unwrap();
        assert_eq!(probe, "[2,2,3]");
    }

    #[test]
    fn custom_elements_when_defined_resolves_on_define() {
        let (dom, _, _) = custom_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "var a = false, b = false; \
                 customElements.whenDefined('x-later').then(function (ctor) { \
                   a = ctor === customElements.get('x-later'); \
                 }); \
                 customElements.define('x-later', class CL extends HTMLElement {}); \
                 customElements.whenDefined('x-later').then(function (ctor) { \
                   b = ctor === customElements.get('x-later'); \
                 });",
            )
            .unwrap();
        // eval drains microtasks, but the stringify above would run first; read
        // the settled flags on a second eval so both handlers have executed.
        let probe = runtime.eval("JSON.stringify([a, b])").unwrap();
        assert_eq!(probe, "[true,true]");
    }

    #[test]
    fn document_write_buffers_until_the_shell_drains_it() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        assert!(runtime.doc_write_is_open());
        runtime
            .eval("document.write('<div id=w>', 'Written', '</div>');")
            .unwrap();
        assert_eq!(
            runtime.take_doc_write(),
            "<div id=w>Written</div>",
            "multiple arguments concatenate"
        );
        assert_eq!(
            runtime.take_doc_write(),
            "",
            "the buffer drains exactly once"
        );
    }

    #[test]
    fn document_close_stops_buffering_and_later_writes_warn_once() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval("document.write('early'); document.close();")
            .unwrap();
        assert_eq!(runtime.take_doc_write(), "early");
        assert!(!runtime.doc_write_is_open());

        runtime
            .eval("document.write('late-1'); document.write('late-2');")
            .unwrap();
        assert_eq!(
            runtime.take_doc_write(),
            "",
            "post-close writes are ignored"
        );
        assert!(runtime.doc_write_warned());
        let logs = runtime.take_logs();
        assert_eq!(logs.len(), 1, "one warning even for two writes");
        assert!(logs[0].contains("document.write after page load is ignored"));
    }

    #[test]
    fn create_event_init_event_dispatch_fires_listener_with_type_and_target() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var host = document.getElementById('host'); \
                 var seen = null; \
                 host.addEventListener('ping', function (e) { \
                   seen = [e.type, e.target === host, e.bubbles, e.cancelable]; \
                 }); \
                 var evt = document.createEvent('HTMLEvents'); \
                 evt.initEvent('ping', false, true); \
                 var ok = host.dispatchEvent(evt); \
                 JSON.stringify([ok, seen]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,["ping",true,false,true]]"#);
    }

    #[test]
    fn dispatch_event_bubbles_to_document_only_when_bubbles_is_true() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var host = document.getElementById('host'); \
                 var docSeen = []; \
                 document.addEventListener('ping', function (e) { docSeen.push(e.target.tagName); }); \
                 var flat = document.createEvent('Event'); \
                 flat.initEvent('ping', false, false); \
                 host.dispatchEvent(flat); \
                 var afterFlat = docSeen.length; \
                 host.dispatchEvent(new Event('ping', { bubbles: true })); \
                 JSON.stringify([afterFlat, docSeen]);",
            )
            .unwrap();
        assert_eq!(probe, r#"[0,["DIV"]]"#);
    }

    #[test]
    fn stop_propagation_halts_bubbling_to_ancestors() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var host = document.getElementById('host'); \
                 var seen = []; \
                 document.addEventListener('ping', function () { seen.push('doc'); }); \
                 host.addEventListener('ping', function (e) { \
                   seen.push('host'); e.stopPropagation(); \
                 }); \
                 host.dispatchEvent(new Event('ping', { bubbles: true })); \
                 seen.join(',');",
            )
            .unwrap();
        assert_eq!(probe, "host");
    }

    #[test]
    fn prevent_default_sets_default_prevented_only_when_cancelable() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var c = new Event('x', { cancelable: true }); c.preventDefault(); \
                 var n = new Event('x'); n.preventDefault(); \
                 JSON.stringify([c.defaultPrevented, n.defaultPrevented]);",
            )
            .unwrap();
        assert_eq!(probe, "[true,false]");
    }

    #[test]
    fn create_event_rejects_unsupported_dom_types() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 ['MouseEvent', 'UIEvent', 'CustomEvent'].forEach(function (t) { \
                   try { document.createEvent(t); out.push('no-throw'); } \
                   catch (e) { out.push(e.name + ':' + /unsupported/.test(e.message)); } \
                 }); \
                 JSON.stringify(out);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["TypeError:true","TypeError:true","TypeError:true"]"#
        );
    }

    #[test]
    fn event_constructor_shape_and_subclassing() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var e = new Event('click', { bubbles: true, cancelable: true }); \
                 class MyEvent extends Event {} \
                 var m = new MyEvent('boom'); \
                 JSON.stringify([e.type, e.bubbles, e.cancelable, e.target, e.defaultPrevented, \
                   typeof e.preventDefault, typeof e.stopPropagation, typeof e.initEvent, \
                   m instanceof MyEvent, m instanceof Event, m.type]);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["click",true,true,null,false,"function","function","function",true,true,"boom"]"#
        );
    }

    #[test]
    fn custom_event_carries_detail_and_is_an_event() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var e = new CustomEvent('thing', { detail: { n: 7 } }); \
                 var d = new CustomEvent('thing'); \
                 JSON.stringify([e.type, e.detail.n, e instanceof CustomEvent, \
                   e instanceof Event, d.detail]);",
            )
            .unwrap();
        assert_eq!(probe, r#"["thing",7,true,true,null]"#);
    }

    #[test]
    fn event_family_instanceof_chain_and_honest_defaults() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var m = new MouseEvent('click'); \
                 var u = new UIEvent('u'); \
                 var k = new KeyboardEvent('keydown'); \
                 var f = new FocusEvent('focus'); \
                 var i = new InputEvent('input'); \
                 JSON.stringify([ \
                   m instanceof MouseEvent, m instanceof UIEvent, m instanceof Event, \
                   u instanceof UIEvent, u instanceof Event, \
                   k instanceof KeyboardEvent, k instanceof UIEvent, k instanceof Event, \
                   f instanceof FocusEvent, f instanceof UIEvent, \
                   i instanceof InputEvent, i instanceof UIEvent, \
                   [m.clientX, m.clientY, m.screenX, m.screenY, m.pageX, m.pageY, \
                    m.button, m.buttons, m.relatedTarget, \
                    m.altKey, m.ctrlKey, m.shiftKey, m.metaKey, \
                    m.view, m.detail, m.bubbles, m.cancelable, m.type], \
                   [k.key, k.code, k.keyCode, k.which, k.ctrlKey], \
                   [f.relatedTarget], [i.inputType, i.data], \
                   typeof m.initMouseEvent \
                 ]);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,true,true,true,true,true,true,true,true,true,true,[0,0,0,0,0,0,0,0,null,false,false,false,false,null,0,false,false,"click"],["","",0,0,false],[null],["",null],"function"]"#
        );
    }

    #[test]
    fn create_event_returns_event_family_instances() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var me = document.createEvent('MouseEvents'); \
                 var ue = document.createEvent('UIEvents'); \
                 var ke = document.createEvent('KeyboardEvent'); \
                 var ee = document.createEvent('Event'); \
                 var he = document.createEvent('HTMLEvents'); \
                 JSON.stringify([ \
                   me instanceof MouseEvent, me instanceof UIEvent, me instanceof Event, \
                   ue instanceof UIEvent, ue instanceof Event, \
                   ke instanceof KeyboardEvent, ke instanceof UIEvent, \
                   ee instanceof Event, he instanceof Event \
                 ]);",
            )
            .unwrap();
        assert_eq!(probe, "[true,true,true,true,true,true,true,true,true]");
    }

    #[test]
    fn polyfill_prototype_assignment_pattern_does_not_throw() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var out = []; \
                 ['MouseEvent', 'UIEvent', 'KeyboardEvent', 'FocusEvent', 'InputEvent'].forEach(function (n) { \
                   var Ctor = window[n]; \
                   var shim = function () {}; \
                   try { window[n].prototype = shim.prototype; \
                         out.push(typeof Ctor + ':' + (window[n].prototype === shim.prototype)); } \
                   catch (e) { out.push('THREW:' + e.name); } \
                 }); \
                 JSON.stringify(out);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["function:true","function:true","function:true","function:true","function:true"]"#
        );
    }

    #[test]
    fn event_family_subclassable_and_honors_init_options() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "class MyMouse extends MouseEvent {} \
                 var m = new MyMouse('click', { clientX: 7, clientY: 9, button: 2, ctrlKey: true }); \
                 var k = new KeyboardEvent('keydown', { key: 'a', code: 'KeyA', keyCode: 65 }); \
                 var i = new InputEvent('input', { inputType: 'insertText', data: 'x' }); \
                 JSON.stringify([ \
                   m instanceof MyMouse, m instanceof MouseEvent, m instanceof UIEvent, m instanceof Event, \
                   m.type, m.clientX, m.clientY, m.button, m.ctrlKey, \
                   k.key, k.code, k.keyCode, i.inputType, i.data \
                 ]);",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,true,true,"click",7,9,2,true,"a","KeyA",65,"insertText","x"]"#
        );
    }

    #[test]
    fn dispatch_event_returns_true_with_no_listeners() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval("document.getElementById('host').dispatchEvent(new Event('nobody'));")
            .unwrap();
        assert_eq!(probe, "true");
    }

    #[test]
    fn svg_element_family_chains_and_illegal_direct_constructors() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var illegal = []; \
                 ['SVGElement', 'SVGSVGElement'].forEach(function (n) { \
                   var C = window[n]; \
                   try { new C(); illegal.push('no-throw'); } \
                   catch (e) { illegal.push(e instanceof TypeError); } \
                 }); \
                 class MySvg extends SVGSVGElement {} \
                 var ms = new MySvg(); \
                 JSON.stringify([\
                   typeof window.SVGElement, typeof window.SVGSVGElement, \
                   illegal[0], illegal[1], \
                   ms instanceof MySvg, ms instanceof SVGSVGElement, \
                   ms instanceof SVGElement, ms instanceof Element, ms instanceof Node, \
                   SVGElement.prototype instanceof Element, \
                   SVGSVGElement.prototype instanceof SVGElement])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["function","function",true,true,true,true,true,true,true,true,true]"#
        );
    }

    #[test]
    fn parsed_svg_wrapper_aligns_to_svg_prototype_with_null_svg_accessors() {
        let mut dom = Dom::new();
        let svg = dom.create(NodeKind::Element {
            tag: "svg".into(),
            attrs: vec![
                ("id".into(), "chart".into()),
                ("width".into(), "100%".into()),
                ("height".into(), "200".into()),
            ],
        });
        dom.append(dom.root(), svg);
        let mut runtime = ExtScriptRuntime::new_page(Rc::new(RefCell::new(dom)));
        let probe = runtime
            .eval(
                "var s = document.getElementById('chart'); \
                 var c = document.querySelector('svg'); \
                 var all = document.querySelectorAll('svg')[0]; \
                 var created = document.createElement('svg'); \
                 var kid = document.createElement('span'); \
                 s.appendChild(kid); \
                 JSON.stringify([\
                   s instanceof SVGSVGElement, s instanceof SVGElement, \
                   s instanceof Element, s instanceof HTMLElement, \
                   c instanceof SVGSVGElement, all instanceof SVGSVGElement, \
                   created instanceof SVGSVGElement, \
                   s.nodeType, s.tagName, s.ownerSVGElement, s.viewportElement, \
                   typeof kid.ownerSVGElement, kid instanceof SVGSVGElement, \
                   kid instanceof HTMLElement])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,true,false,true,true,true,1,"SVG",null,null,"undefined",false,true]"#
        );
    }

    #[test]
    fn svg_width_height_reflect_real_attributes_honestly() {
        let mut dom = Dom::new();
        let svg = dom.create(NodeKind::Element {
            tag: "svg".into(),
            attrs: vec![
                ("id".into(), "chart".into()),
                ("width".into(), "100%".into()),
                ("height".into(), "200".into()),
            ],
        });
        dom.append(dom.root(), svg);
        let mut runtime = ExtScriptRuntime::new_page(Rc::new(RefCell::new(dom)));
        let probe = runtime
            .eval(
                "var s = document.getElementById('chart'); \
                 var before = [s.width, s.height]; \
                 s.setAttribute('width', '50'); \
                 var after = [s.width, s.height]; \
                 var absent = [document.createElement('svg').width, \
                               document.createElement('svg').height]; \
                 JSON.stringify([before, after, absent])",
            )
            .unwrap();
        assert_eq!(probe, r#"[["100%","200"],["50","200"],[null,null]]"#);
    }

    // ---- Zone B: shadow DOM / templates / detached document surfaces -----

    #[test]
    fn inner_html_roundtrip_on_simple_markup() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 h.innerHTML = '<b class=\"x\">hi</b><i>there</i>'; \
                 JSON.stringify([h.innerHTML, h.children.length, h.children[0].tagName, \
                   h.children[0].getAttribute('class'), h.children[0].textContent, \
                   h.children[1].tagName])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["<b class=\"x\">hi</b><i>there</i>",2,"B","x","hi","I"]"#
        );
        assert_eq!(dom.borrow().children(host).len(), 2);
    }

    #[test]
    fn template_content_is_a_document_fragment_side_tree() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var t = document.createElement('template'); \
                 t.innerHTML = '<b>hi</b>'; \
                 JSON.stringify([t instanceof HTMLTemplateElement, \
                   t.content instanceof DocumentFragment, t.content.childNodes.length, \
                   t.content.querySelector('b').textContent, t.children.length, \
                   t.innerHTML, t.content.firstChild instanceof Element])",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,true,1,"hi",0,"<b>hi</b>",true]"#);
    }

    #[test]
    fn template_capability_probe_skips_the_bundle_polyfill() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        // Same shape webcomponents-sd-ce-pf.js uses to decide `c`/`M`: with
        // native template content + deep clone, both stay false.
        let probe = runtime
            .eval(
                "var t = document.createElement('template'); \
                 var inner = document.createElement('template'); \
                 inner.content.appendChild(document.createElement('div')); \
                 t.content.appendChild(inner); \
                 var clone = t.cloneNode(true); \
                 JSON.stringify([typeof HTMLTemplateElement, clone.content.childNodes.length, \
                   clone.content.firstChild.content.childNodes.length, \
                   document.createDocumentFragment().cloneNode() instanceof DocumentFragment])",
            )
            .unwrap();
        assert_eq!(probe, r#"["function",1,1,true]"#);
    }

    #[test]
    fn dom_parser_parses_into_a_detached_document_wrapper() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        let probe = runtime
            .eval(
                "var doc = new DOMParser().parseFromString( \
                   '<html><body><p class=\"x\">hello</p></body></html>', 'text/html'); \
                 JSON.stringify([doc instanceof Document, doc.querySelector('p').textContent, \
                   doc.querySelectorAll('p').length, doc.body !== null, \
                   doc.documentElement !== null, doc.querySelector('span') === null, \
                   document.querySelector('p') === null])",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,"hello",1,true,true,true,true]"#);
    }

    #[test]
    fn dom_parser_rejects_non_html_mime_types() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var caught = 'none'; \
                 try { new DOMParser().parseFromString('<p>x</p>', 'application/xml'); } \
                 catch (e) { caught = e.name; } caught",
            )
            .unwrap();
        assert_eq!(probe, "TypeError");
    }

    #[test]
    fn create_html_document_returns_a_usable_detached_document() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var d = document.implementation.createHTMLDocument('t'); \
                 d.body.innerHTML = '<b>x</b>'; \
                 JSON.stringify([d.title, d.body.querySelector('b').textContent, \
                   d instanceof Document, d.createElement('i').tagName])",
            )
            .unwrap();
        assert_eq!(probe, r#"["t","x",true,"I"]"#);
    }

    #[test]
    fn get_root_node_reaches_the_document_and_the_shadow_root() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var kid = document.getElementById('kid'); \
                 var r = h.attachShadow({mode:'open'}); \
                 r.innerHTML = '<b>deep</b>'; \
                 var deep = r.querySelector('b'); \
                 JSON.stringify([h.getRootNode() === document, kid.getRootNode() === document, \
                   document.getRootNode() === document, r.getRootNode() === r, \
                   deep.getRootNode() === r, deep.parentNode === r])",
            )
            .unwrap();
        assert_eq!(probe, "[true,true,true,true,true,true]");
    }

    #[test]
    fn attach_shadow_open_and_closed_shadow_root_nullness() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var r = h.attachShadow({mode:'open'}); \
                 r.innerHTML = '<b>hi</b>'; \
                 var c = document.createElement('div'); \
                 var cr = c.attachShadow({mode:'closed'}); \
                 JSON.stringify([h.shadowRoot === r, r.host === h, r.mode, \
                   r instanceof ShadowRoot, h.children.length, h.childNodes.length, \
                   r.querySelector('b').textContent, r.childNodes.length, \
                   document.querySelectorAll('b').length, \
                   c.shadowRoot === null, cr.mode, cr.host === c])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,"open",true,1,1,"hi",1,0,true,"closed",true]"#
        );
    }

    #[test]
    fn custom_element_upgrade_walks_into_shadow_roots() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var r = h.attachShadow({mode:'open'}); \
                 r.innerHTML = '<x-deep id=\"deep\"></x-deep>'; \
                 function XDeep() { HTMLElement.call(this); } \
                 XDeep.prototype = Object.create(HTMLElement.prototype); \
                 XDeep.prototype.constructor = XDeep; \
                 XDeep.prototype.connectedCallback = function () { console.log('upgraded'); }; \
                 customElements.define('x-deep', XDeep); \
                 JSON.stringify([r.querySelector('x-deep').getAttribute('id'), \
                   r.querySelector('x-deep') instanceof XDeep])",
            )
            .unwrap();
        assert_eq!(probe, r#"["deep",true]"#);
        assert_eq!(runtime.take_logs(), ["upgraded"]);
    }

    #[test]
    fn css_style_sheet_constructor_and_replace_sync_store_text() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var s = new CSSStyleSheet({media: 'screen'}); \
                 s.replaceSync('p{color:red}'); \
                 JSON.stringify([s.cssText, s.cssRules.length, s instanceof CSSStyleSheet, \
                   typeof window.CSSStyleSheet, s.__hodoOptions.media])",
            )
            .unwrap();
        assert_eq!(probe, r#"["p{color:red}",0,true,"function","screen"]"#);
    }

    #[test]
    fn css_style_sheet_replace_returns_a_resolved_promise() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var s = new CSSStyleSheet(); \
                 var p = s.replace('b{color:blue}'); \
                 p.then(function (v) { \
                   console.log((v === s) + ':' + (s.cssText === 'b{color:blue}')); \
                 }); \
                 typeof p.then",
            )
            .unwrap();
        assert_eq!(probe, "function");
        assert_eq!(runtime.take_logs(), ["true:true"]);
    }

    #[test]
    fn css_style_sheet_rule_mutation_is_honestly_unsupported() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var s = new CSSStyleSheet(); var out = []; \
                 try { s.insertRule('p{}', 0); } catch (e) { out.push(e.name); } \
                 try { s.deleteRule(0); } catch (e) { out.push(e.name); } \
                 JSON.stringify(out)",
            )
            .unwrap();
        assert_eq!(probe, r#"["TypeError","TypeError"]"#);
    }

    #[test]
    fn adopted_style_sheets_set_and_get_on_shadow_root_and_document() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 var r = h.attachShadow({mode:'open'}); \
                 var s = new CSSStyleSheet(); \
                 r.adoptedStyleSheets = [s]; \
                 document.adoptedStyleSheets = [s]; \
                 JSON.stringify([r.adoptedStyleSheets.length, \
                   r.adoptedStyleSheets[0] === s, \
                   document.adoptedStyleSheets[0] === s, \
                   'adoptedStyleSheets' in Document.prototype, \
                   'replace' in CSSStyleSheet.prototype])",
            )
            .unwrap();
        assert_eq!(probe, "[1,true,true,true,true]");
    }

    #[test]
    fn lit_mimic_custom_element_stamps_shadow_content_with_adopted_sheet() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "class XLit extends HTMLElement { \
                   connectedCallback() { \
                     const s = this.attachShadow({mode:'open'}); \
                     s.innerHTML = '<style>p{color:red}</style><p>t</p>'; \
                     this.shadowRoot.adoptedStyleSheets = [new CSSStyleSheet()]; \
                   } \
                 } \
                 customElements.define('x-lit', XLit); \
                 var el = document.createElement('x-lit'); \
                 document.getElementById('host').appendChild(el); \
                 var sr = el.shadowRoot; \
                 JSON.stringify([sr !== null, sr.querySelector('p').textContent, \
                   sr.querySelector('style').textContent, \
                   sr.adoptedStyleSheets.length, \
                   sr.adoptedStyleSheets[0] instanceof CSSStyleSheet])",
            )
            .unwrap();
        assert_eq!(probe, r#"[true,"t","p{color:red}",1,true]"#);
        assert!(
            runtime.take_logs().is_empty(),
            "upgrade must not report an error"
        );
    }

    #[test]
    fn window_svg_prototype_patch_pattern_does_not_throw() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var shim = { sniff: true }; var out = []; \
                 try { window.SVGElement.prototype = shim; \
                       out.push(window.SVGElement.prototype === shim); } \
                 catch (e) { out.push('THREW:' + e.name); } \
                 try { window.SVGSVGElement.prototype = shim; \
                       out.push(window.SVGSVGElement.prototype === shim); } \
                 catch (e) { out.push('THREW:' + e.name); } \
                 JSON.stringify(out)",
            )
            .unwrap();
        assert_eq!(probe, "[true,true]");
    }

    // ---- ESM graph loader / dynamic import (wave 6) ----

    #[test]
    fn module_registry_evaluates_each_url_only_once() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var calls = 0; \
                 function f(__mods) { calls++; return { v: calls }; } \
                 var a = __hodoRunModule('https://x/a.js', [], f); \
                 var b = __hodoRunModule('https://x/a.js', [], f); \
                 JSON.stringify([calls, a.v, b.v, a === b]);",
            )
            .unwrap();
        assert_eq!(probe, "[1,1,1,true]");
    }

    #[test]
    fn module_factory_receives_dependency_exports_in_first_mention_order() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "__hodoRunModule('https://x/a.js', [], function () { return { name: 'a' }; }); \
                 __hodoRunModule('https://x/b.js', [], function () { return { name: 'b' }; }); \
                 var seen = []; \
                 __hodoRunModule('https://x/c.js', ['https://x/b.js', 'https://x/a.js'], \
                   function (mods) { seen = mods.map(function (m) { return m.name; }); return {}; }); \
                 seen.join(',');",
            )
            .unwrap();
        assert_eq!(probe, "b,a");
    }

    #[test]
    fn module_missing_dependency_throws_with_both_urls() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let message = runtime
            .eval(
                "var msg = ''; \
                 try { __hodoRunModule('https://x/importer.js', ['https://x/absent.js'], \
                   function () { return {}; }); } \
                 catch (e) { msg = e.message; } msg;",
            )
            .unwrap();
        assert!(
            message.contains("https://x/importer.js") && message.contains("https://x/absent.js"),
            "honest error must name importer and missing dep: {message}"
        );
        let not_registered = runtime
            .eval("typeof __hodoModules['https://x/importer.js']")
            .unwrap();
        assert_eq!(not_registered, "undefined");
    }

    #[test]
    fn module_exports_are_snapshotted() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var raw = { a: 1, b: 2 }; \
                 var ns = __hodoRunModule('https://x/snap.js', [], function () { return raw; }); \
                 raw.a = 99; raw.c = 3; \
                 JSON.stringify([Object.keys(ns).join(','), ns.a, ns.b, \
                   ns.c === undefined, Object.getPrototypeOf(ns) === null, \
                   ns === __hodoModules['https://x/snap.js']]);",
            )
            .unwrap();
        assert_eq!(probe, r#"["a,b",1,2,true,true,true]"#);
    }

    #[test]
    fn module_factory_keeps_strict_semantics() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        // No directive in the emitted factory itself (the shell strips chunk
        // directives): the strict graph-eval path must supply it.
        let probe = runtime
            .eval_module_graph(
                "__hodoRunModule('https://x/strict.js', [], (function (__mods) { \
                   var threw = false; \
                   try { __hodoUndeclared = 1; } catch (e) { threw = true; } \
                   return { thisUndef: this === undefined, threw: threw }; \
                 })); \
                 JSON.stringify(__hodoModules['https://x/strict.js']);",
            )
            .unwrap();
        assert_eq!(probe, r#"{"thisUndef":true,"threw":true}"#);
    }

    #[test]
    fn dynamic_import_resolves_with_shell_supplied_exports() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let seen = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&seen);
        runtime.set_message_reply_hook(Some(Rc::new(move |message, _| {
            sink.borrow_mut().push(message.clone());
        })));
        runtime
            .eval(
                "__hodoModuleBaseUrl = 'https://x/pages/index.html'; \
                 __dyn = __hodoDynImport('./child.js'); \
                 __dyn.then(function (m) { console.log('resolved:' + m.v); }); \
                 JSON.stringify(true);",
            )
            .unwrap();
        // With a reply hook installed the shell owns the request channel; the
        // internal queue must stay empty so it cannot grow without bound.
        assert!(runtime.take_dynamic_module_requests().is_empty());
        let message = seen.borrow()[0].clone();
        assert_eq!(message["__hodoDynImport"], 1);
        assert_eq!(message["spec"], "https://x/pages/child.js");
        assert_eq!(message["base"], "https://x/pages/index.html");
        runtime.resolve_dynamic_import(
            message["id"].as_u64().expect("numeric promise id"),
            &json!({ "v": 42 }),
        );
        assert_eq!(runtime.take_logs(), ["resolved:42"]);
    }

    #[test]
    fn dynamic_import_rejects_with_shell_message() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "__hodoModuleBaseUrl = 'https://x/index.html'; \
                 __hodoDynImport('./gone.js').catch(function (e) { \
                   console.log('rejected:' + e.message); \
                 }); JSON.stringify(true);",
            )
            .unwrap();
        let requests = runtime.take_dynamic_module_requests();
        assert_eq!(requests.len(), 1);
        runtime.reject_dynamic_import(requests[0].promise_id, "load failed");
        assert_eq!(runtime.take_logs(), ["rejected:load failed"]);
    }

    #[test]
    fn late_or_unknown_dynamic_import_callback_is_a_no_op() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "__hodoDynImport('./child.js').then(function (m) { \
                   console.log('only:' + m.v); \
                 }); JSON.stringify(true);",
            )
            .unwrap();
        let requests = runtime.take_dynamic_module_requests();
        let id = requests[0].promise_id;
        runtime.resolve_dynamic_import(id, &json!({ "v": 1 }));
        // Both a duplicate settle of a known id and a never-issued id must be
        // silent no-ops (no second log, no thrown error).
        runtime
            .eval(&format!(
                "__hodoDynResolved({id}, {{ v: 2 }}); \
                 __hodoDynRejected({id}, 'too late'); \
                 __hodoDynResolved(99999999, {{}}); JSON.stringify(true);"
            ))
            .unwrap();
        assert_eq!(runtime.take_logs(), ["only:1"]);
    }

    #[test]
    fn host_page_event_honors_stop_propagation_flag() {
        let (dom, _, _) = sample_dom();
        let root = dom.borrow().root();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                "document.addEventListener('click', function (e) { \
                   console.log('one:' + e.type); e.stopPropagation(); \
                 }); \
                 document.addEventListener('click', function () { \
                   console.log('two'); \
                 });",
            )
            .unwrap();
        runtime.dispatch_page_event(root, "click", json!({}));
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["one:click"]);
    }

    #[test]
    fn tree_walker_next_node_and_node_remove_work() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var root = document.getElementById('host');
                var walker = document.createTreeWalker(root, 129);
                var seen = [];
                var node;
                while ((node = walker.nextNode()) !== null) {
                  seen.push(node.nodeName);
                }
                var kid = document.getElementById('kid');
                kid.remove();
                JSON.stringify([seen, document.getElementById('kid') === null]);
                "#,
            )
            .unwrap();
        assert_eq!(probe, r#"[["SPAN"],true]"#);
    }

    // Lit's EventPart registers itself as an EventListener object with
    // handleEvent (DOM-legal). The native registry needs a callable, so the
    // prelude adapts once and must preserve removeEventListener identity.
    #[test]
    fn listener_object_with_handle_event_is_supported() {
        let (dom, host, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                r#"
                window.__listener = {
                  count: 0,
                  handleEvent: function (e) {
                    this.count++;
                    console.log('handle:' + this.count + ':' + e.type);
                  }
                };
                document.getElementById('host').addEventListener('click', window.__listener);
                "#,
            )
            .unwrap();
        runtime.dispatch_page_event(host, "click", json!({}));
        runtime.tick();
        runtime
            .eval(
                "document.getElementById('host').removeEventListener('click', window.__listener);",
            )
            .unwrap();
        runtime.dispatch_page_event(host, "click", json!({}));
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["handle:1:click"]);
    }

    // Lit's two TreeWalker consumers, modelled end to end in-process. The
    // archive.org `<app-root>` shadow tree is whitespace + markers + nested
    // parts: (1) the template part-finder must FIND the `<!--?lit$…$-->`
    // comments past non-showing whitespace, and (2) ChildPart materialisation
    // must re-walk the imported clone at the very same node indices so each
    // binding's start marker is the right comment. A walk that stops at the
    // first non-showing sibling finds zero markers; a walk that desends into
    // REJECTED subtrees misaligns the clone walk and collapses the tree.
    #[test]
    fn lit_marker_find_and_child_part_materialization_work() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var t = document.createElement('template');
                t.innerHTML = '<main> <a href="/x">link</a> <header>h</header> ' +
                  '<!--?lit$TEST$--> <b>x</b> <!--?lit$TEST$--> y <!--?lit$TEST$--> </main>';
                var MARK = '?lit$TEST$';
                var w = document.createTreeWalker(t.content, 129);
                var parts = [], a = 0, n;
                w.currentNode = t.content;
                while ((n = w.nextNode()) !== null) {
                  if (n.nodeType === 8 && n.data === MARK) parts.push(a);
                  a++;
                }
                var clone = document.importNode(t.content, true);
                w.currentNode = clone;
                var i = w.nextNode(), idx = 0, starts = [];
                for (var p = 0; p < parts.length; p++) {
                  while (idx < parts[p]) { i = w.nextNode(); idx++; }
                  var end = i.nextSibling;
                  var txt = document.createTextNode('V' + p);
                  i.parentNode.insertBefore(txt, end);
                  starts.push(i.nodeType + ':' + i.data);
                }
                JSON.stringify([parts, starts, /V0.*V1.*V2/.test(clone.textContent)]);
                "#,
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[[3,5,6],["8:?lit$TEST$","8:?lit$TEST$","8:?lit$TEST$"],true]"#
        );
    }

    // The real archive.org <app-root> template (index-NFRVgqV2.js). The live
    // stamped tree has 12 children: a/header/main/whitespace + three markers.
    // Lit's Je constructor and Ye.u clone walk must agree on marker positions.
    #[test]
    fn lit_archive_template_marker_find_and_clone_walk() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var t = document.createElement('template');
                t.innerHTML = '<a id="skip" @click$lit$="lit$MK$" >' +
                  '<!--?lit$MK$--></a>\n      \n      <header>\n        ' +
                  '<!--?lit$MK$-->\n        <!--?lit$MK$-->\n      </header>\n    ';
                var MARK = '?lit$MK$';
                var w = document.createTreeWalker(t.content, 129);
                var parts = [], a = 0, n;
                while ((n = w.nextNode()) !== null) {
                  if (n.nodeType === 8 && n.data === MARK) parts.push(a);
                  a++;
                }
                var clone = document.importNode(t.content, true);
                w.currentNode = clone;
                var i = w.nextNode(), idx = 0, filled = [];
                for (var p = 0; p < parts.length; p++) {
                  while (idx < parts[p]) { i = w.nextNode(); idx++; }
                  var txt = document.createTextNode('V' + p);
                  i.parentNode.insertBefore(txt, i.nextSibling);
                  filled.push(i.nodeType + ':' + i.data);
                }
                var kids = []; for (var k = 0; k < clone.childNodes.length; k++) kids.push(clone.childNodes[k].nodeName);
                JSON.stringify([parts, filled, kids, clone.textContent.replace(/\s+/g, '')]);
                "#,
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[[1,3,4],["8:?lit$MK$","8:?lit$MK$","8:?lit$MK$"],["A","#text","HEADER","#text"],"V0V1V2"]"##
        );
    }

    // The regression guard: FILTER_SKIP drops only the node and still walks
    // its children; FILTER_REJECT prunes the whole subtree. A walker that
    // conflates the two either invents or loses nodes, which shifts Lit's
    // part indices and collapses ChildPart materialisation.
    #[test]
    fn tree_walker_filter_reject_prunes_but_skip_descends() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var t = document.createElement('template');
                t.innerHTML = '<r><s><t></t></s><u></u></r>';
                var root = t.content.firstChild;
                function walk(filter) {
                  var w = document.createTreeWalker(root, 129, filter);
                  var names = [], n;
                  while ((n = w.nextNode()) !== null) names.push(n.nodeName);
                  return names.join(',');
                }
                JSON.stringify([
                  walk({ acceptNode: function (n) {
                    return n.nodeName === 'S' ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT;
                  }}),
                  walk({ acceptNode: function (n) {
                    return n.nodeName === 'S' ? NodeFilter.FILTER_SKIP : NodeFilter.FILTER_ACCEPT;
                  }}),
                  walk(null)
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(probe, r#"["U","T,U","S,T,U"]"#);
    }

    #[test]
    fn page_text_encoder_emits_utf8_bytes() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval("Array.from(new TextEncoder().encode('Aé')).join(',')")
            .unwrap();
        assert_eq!(probe, "65,195,169");
    }

    #[test]
    fn page_attribute_surface_sees_custom_names_from_inner_html() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var host = document.getElementById('host');
                host.innerHTML = '<span id="kid" data-x="1" class="x">t</span>';
                var kid = host.firstElementChild;
                var names = kid.getAttributeNames();
                var clone = kid.cloneNode(false);
                kid.toggleAttribute('hidden');
                var toggled = kid.hasAttribute('hidden');
                kid.toggleAttribute('hidden', false);
                JSON.stringify([
                  names.indexOf('data-x') >= 0, kid.hasAttribute('data-x'),
                  kid.hasAttributes(), kid.getAttribute('data-x'),
                  clone.hasAttribute('data-x'), toggled,
                  typeof kid.removeAttribute
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(probe, r#"[true,true,true,"1",true,true,"function"]"#);
    }

    #[test]
    fn page_get_elements_by_tag_name_returns_wrapped_matches() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "JSON.stringify([\
                   document.getElementsByTagName('span').length,\
                   document.getElementsByTagName('span')[0].getAttribute('id'),\
                   document.getElementsByTagName('missing').length])",
            )
            .unwrap();
        assert_eq!(probe, r#"[1,"kid",0]"#);
    }

    #[test]
    fn page_history_state_roundtrip_and_replace_state_updates_location() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page_with_url(dom, "https://archive.org/");
        let probe = runtime
            .eval(
                r#"
                var initial = history.state;
                history.pushState({ a: 1 }, '', '/one');
                var pushed = [history.state.a, location.pathname, location.href];
                history.replaceState({ b: 2 }, '', '/two?x=1#f');
                var replaced = [
                  history.state.b, location.pathname, location.search,
                  location.hash, location.href
                ];
                history.scrollRestoration = 'manual';
                var own = ['pushState', 'replaceState', 'forward', 'go', 'back'].map(
                  function (k) { return Object.prototype.hasOwnProperty.call(history, k); });
                history.back();
                JSON.stringify([
                  initial, pushed, replaced, history.length,
                  history.scrollRestoration, own
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(
            probe,
            r##"[null,[1,"/one","https://archive.org/one"],[2,"/two","?x=1","#f","https://archive.org/two?x=1#f"],1,"manual",[true,true,true,true,true]]"##
        );
        assert_eq!(
            runtime.take_logs(),
            ["hodo: history.back is a no-op in page scripts"]
        );
    }

    #[test]
    fn page_request_animation_frame_fires_on_the_house_timer() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                r#"
                var frames = [];
                var id = requestAnimationFrame(function (t) { frames.push(typeof t); });
                if (!(id > 0)) throw new Error('rAF id');
                "#,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        runtime.tick();
        assert_eq!(runtime.eval("frames.join(',')").unwrap(), "number");
    }

    #[test]
    fn page_cancel_animation_frame_suppresses_the_callback() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                r#"
                var frames = 0;
                var id = requestAnimationFrame(function () { frames++; });
                cancelAnimationFrame(id);
                "#,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        runtime.tick();
        assert_eq!(runtime.eval("frames").unwrap(), "0");
    }

    #[test]
    fn page_queue_microtask_runs_before_the_next_timer() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                r#"
                queueMicrotask(function () { console.log('micro'); });
                setTimeout(function () { console.log('timer'); }, 0);
                "#,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        runtime.tick();
        assert_eq!(runtime.take_logs(), ["micro", "timer"]);
    }

    #[test]
    fn page_match_media_is_a_false_capability_shape() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var m = matchMedia('(max-width: 889px)');
                JSON.stringify([
                  m.matches, m.media, typeof m.addEventListener,
                  typeof m.removeEventListener, typeof m.addListener,
                  m.dispatchEvent()
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[false,"(max-width: 889px)","function","function","function",true]"#
        );
    }

    #[test]
    fn page_observers_are_capability_only_with_no_callbacks() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var fired = 0;
                var mo = new MutationObserver(function () { fired++; });
                var io = new IntersectionObserver(function () { fired++; });
                var ro = new ResizeObserver(function () { fired++; });
                mo.observe(document, { childList: true });
                io.observe(document.documentElement);
                ro.observe(document.documentElement);
                JSON.stringify([
                  mo instanceof MutationObserver, io instanceof IntersectionObserver,
                  ro instanceof ResizeObserver, typeof mo.disconnect,
                  typeof io.unobserve, typeof ro.takeRecords,
                  ro.takeRecords().length, fired
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[true,true,true,"function","function","function",0,0]"#
        );
    }

    #[test]
    fn page_computed_style_performance_and_beacon_shape() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var s = getComputedStyle(document.documentElement);
                JSON.stringify([
                  s.fontSize, typeof s.getPropertyValue, typeof performance.now,
                  performance.getEntriesByType('navigation').length,
                  typeof navigator.sendBeacon, navigator.sendBeacon('x')
                ]);
                "#,
            )
            .unwrap();
        assert_eq!(probe, r#"["","function","function",0,"function",true]"#);
    }

    #[test]
    fn raf_scheduler_stamps_shadow_dom_content_after_a_wait() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        runtime
            .eval(
                r#"
                function XLit() { HTMLElement.call(this); }
                XLit.prototype = Object.create(HTMLElement.prototype);
                XLit.prototype.constructor = XLit;
                XLit.prototype.connectedCallback = function () {
                  var self = this;
                  requestAnimationFrame(function () {
                    var root = self.attachShadow({ mode: 'open' });
                    root.innerHTML = '<b>stamped</b>';
                  });
                };
                customElements.define('x-lit', XLit);
                document.getElementById('host').appendChild(
                  document.createElement('x-lit'));
                "#,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        runtime.tick();
        let logs = runtime.take_logs();
        assert!(logs.is_empty(), "unexpected page logs: {logs:?}");
        assert_eq!(
            runtime
                .eval(
                    "document.getElementById('host')
                       .querySelector('x-lit').shadowRoot
                       .querySelector('b').textContent"
                )
                .unwrap(),
            "stamped"
        );
    }

    #[test]
    fn inner_html_serializes_custom_and_unknown_elements() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 h.innerHTML = '<app-root id=\"a\"><x-lit>hi</x-lit></app-root>'; \
                 JSON.stringify([h.innerHTML, h.childNodes.length, h.firstChild.nodeName, \
                   h.firstElementChild.getAttribute('id')])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"["<app-root id=\"a\"><x-lit>hi</x-lit></app-root>",1,"APP-ROOT","a"]"#
        );
    }

    #[test]
    fn inner_html_roundtrips_lit_marker_comments() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                "var h = document.getElementById('host'); \
                 h.innerHTML = '<!--<?lit$834982315>-->x<!--/*lit*/-->'; \
                 JSON.stringify([h.childNodes.length, h.childNodes[0].nodeType, \
                   h.childNodes[0].data, h.childNodes[2].data, h.innerHTML])",
            )
            .unwrap();
        assert_eq!(
            probe,
            r#"[3,8,"<?lit$834982315>","/*lit*/","<!--<?lit$834982315>-->x<!--/*lit*/-->"]"#
        );
    }

    #[test]
    fn page_html_element_classes_are_subclassable_and_match_instances() {
        let (dom, _, _) = sample_dom();
        let mut runtime = ExtScriptRuntime::new_page(dom);
        let probe = runtime
            .eval(
                r#"
                var pairs = [
                  ['button','HTMLButtonElement'], ['a','HTMLAnchorElement'],
                  ['input','HTMLInputElement'], ['img','HTMLImageElement'],
                  ['select','HTMLSelectElement'], ['textarea','HTMLTextAreaElement'],
                  ['form','HTMLFormElement'], ['video','HTMLMediaElement']
                ];
                var ok = pairs.map(function (p) {
                  var ctor = window[p[1]];
                  var directThrows = false;
                  try { new ctor(); } catch (e) { directThrows = /Illegal constructor/.test(String(e)); }
                  return typeof ctor === 'function' && directThrows
                    && document.createElement(p[0]) instanceof ctor;
                });
                class Btn extends HTMLButtonElement {}
                JSON.stringify([ok, new Btn() instanceof HTMLButtonElement]);
                "#,
            )
            .unwrap();
        assert_eq!(probe, r#"[[true,true,true,true,true,true,true,true],true]"#);
    }
}

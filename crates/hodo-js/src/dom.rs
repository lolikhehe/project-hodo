//! Page-side DOM bindings: opaque `u32` handles into a thread-local table of
//! `(Rc<RefCell<Dom>>, NodeId)`, every access gated by `Dom::validate`.
//!
//! `Rc<RefCell<Dom>>` is correct here: a page JS context owns exactly one
//! document and boa contexts are single-threaded, so no `Send` is required.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use boa_engine::native_function::NativeFunction;
use boa_engine::property::PropertyKey;
use boa_engine::{JsArgs, JsString, JsValue};
use hodo_css::{matches_selector, Selector};
use hodo_dom::{Dom, NodeId, NodeKind};
use serde_json::json;

use super::{js_error, register, string_arg, with_state, RuntimeState};

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

fn query(dom: &Dom, selector: &str) -> Vec<NodeId> {
    let selectors = query_selectors(selector);
    if selectors.is_empty() {
        return Vec::new();
    }
    let mut found = Vec::new();
    preorder(dom, dom.root(), |id| {
        if selectors.iter().any(|sel| matches_selector(sel, dom, id)) {
            found.push(id);
        }
    });
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
        let (dom, _) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let selector = string_arg(args, 1, ctx).map_err(js_error)?;
        let found = {
            let dom = dom.borrow();
            query(&dom, &selector)
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

fn op_dom_append_child(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let parent_handle = handle_arg(args, 0).map_err(js_error)?;
        let child_handle = handle_arg(args, 1).map_err(js_error)?;
        let (dom, parent) = resolve_for(&state, parent_handle).map_err(js_error)?;
        let (_, child) = resolve_for(&state, child_handle).map_err(js_error)?;
        dom.borrow_mut().append(parent, child);
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

fn op_dom_text(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let (dom, node) =
            resolve_for(&state, handle_arg(args, 0).map_err(js_error)?).map_err(js_error)?;
        let text = {
            let dom = dom.borrow();
            collect_text(&dom, node)
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
                if dom.tag(child).is_some() {
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

pub(crate) fn register_natives(context: &mut boa_engine::Context, id: u64) {
    register(context, "__hodo_page_url", 0, op_page_url(id));
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
    register(context, "__hodo_dom_text", 1, op_dom_text(id));
    register(context, "__hodo_dom_set_text", 2, op_dom_set_text(id));
    register(context, "__hodo_dom_tag", 1, op_dom_tag(id));
    register(context, "__hodo_dom_children", 1, op_dom_children(id));
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
        assert_eq!(
            probe,
            r#"["undefined","undefined","undefined","undefined","undefined","object","object"]"#
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
        // Both the holder and the property are read-only.
        assert_eq!(
            runtime
                .eval("window.location.href = 'https://elsewhere.test/'; window.location.href")
                .unwrap(),
            "https://archive.org/details/foo?a=1"
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
}

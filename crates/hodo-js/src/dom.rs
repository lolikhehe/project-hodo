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
    handles: RefCell<Vec<u32>>,
}

impl PageState {
    pub(crate) fn new(dom: Rc<RefCell<Dom>>) -> Self {
        Self {
            dom,
            listeners: RefCell::new(Vec::new()),
            handles: RefCell::new(Vec::new()),
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
    let handle = NEXT_DOM_HANDLE.fetch_add(1, Ordering::Relaxed).max(1);
    DOM_HANDLES.with(|table| {
        table
            .borrow_mut()
            .insert(handle, (Rc::clone(&page.dom), node));
    });
    page.handles.borrow_mut().push(handle);
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

/// Only `tag`, `#id` and `.class` are supported; anything else yields an empty
/// result, matching `querySelector` returning `null`.
fn simple_selector(text: &str) -> Option<Selector> {
    let text = text.trim();
    let plain = |tag, id, classes| Selector {
        tag,
        id,
        classes,
        universal: false,
    };
    if let Some(id) = text.strip_prefix('#') {
        is_name(id).then(|| plain(None, Some(id.to_string()), Vec::new()))
    } else if let Some(class) = text.strip_prefix('.') {
        is_name(class).then(|| plain(None, None, vec![class.to_string()]))
    } else if is_name(text) {
        Some(plain(Some(text.to_string()), None, Vec::new()))
    } else {
        None
    }
}

fn query(dom: &Dom, selector: &str) -> Vec<NodeId> {
    let Some(selector) = simple_selector(selector) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    preorder(dom, dom.root(), |id| {
        if matches_selector(&selector, dom, id) {
            found.push(id);
        }
    });
    found
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

pub(crate) fn register_natives(context: &mut boa_engine::Context, id: u64) {
    register(context, "__hodo_dom_root", 0, op_dom_root(id));
    register(
        context,
        "__hodo_dom_get_element_by_id",
        1,
        op_dom_get_element_by_id(id),
    );
    register(context, "__hodo_dom_query", 2, op_dom_query(id, false));
    register(context, "__hodo_dom_query_all", 2, op_dom_query(id, true));
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
        assert_eq!(
            runtime
                .eval("String(document.querySelector('div > span'))")
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
}

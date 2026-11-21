//! JavaScript execution for WebExtensions.
//!
//! Injects the cross-compat `chrome.*` / `browser.*` globals into a pure-Rust
//! [`boa_engine`] context, dispatching every call into the shared
//! [`hodo_ext_api::ApiRegistry`]. `chrome` and `browser` are the *same* object,
//! which is what makes the two dialects interchangeable.
//!
//! A runtime built with [`ExtScriptRuntime::with_allowed`] only installs the
//! namespaces the caller declares, so a host can gate APIs before any script
//! runs. All native functions still write to one shared registry.

use std::cell::RefCell;
use std::rc::Rc;

use hodo_ext_api::ApiRegistry;
#[cfg(feature = "js")]
use hodo_ext_api::{ApiCall, ApiResult};

#[cfg(feature = "js")]
use serde_json::Value;

#[cfg(feature = "js")]
use std::borrow::Cow;
#[cfg(feature = "js")]
use std::collections::HashMap;
#[cfg(feature = "js")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "js")]
use boa_engine::{
    error::JsNativeError, native_function::NativeFunction, Context, JsArgs, JsError, JsString,
    JsValue, Source,
};

// Boa's `from_closure` is unsafe (its GC cannot inspect captured variables), and
// `Rc<RefCell<ApiRegistry>>` has no `Trace` impl for `from_copy_closure_with_captures`.
// A `Copy` runtime id plus this handle table keeps the bridge 100% safe Rust while
// still sharing one registry between Rust and every native function.
#[cfg(feature = "js")]
thread_local! {
    static HANDLES: RefCell<HashMap<u64, Rc<RefCell<ApiRegistry>>>> = RefCell::new(HashMap::new());
}

#[cfg(feature = "js")]
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A JavaScript context wired to one extension API registry.
pub struct ExtScriptRuntime {
    registry: Rc<RefCell<ApiRegistry>>,
    #[cfg(feature = "js")]
    context: Context,
    #[cfg(feature = "js")]
    id: u64,
}

impl ExtScriptRuntime {
    /// Creates a runtime with a fresh registry and every WebExtension global installed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a runtime with a fresh registry that injects only `allowed` namespaces.
    ///
    /// `runtime` is always installed, mirroring the host's permission gate.
    #[must_use]
    pub fn with_allowed(allowed: &[&str]) -> Self {
        Self::with_allowed_and_registry(allowed, Rc::new(RefCell::new(ApiRegistry::new())))
    }

    /// Creates a filtered runtime whose `chrome.*` / `browser.*` calls dispatch into
    /// `registry`, letting a host share its own registry with the JS engine.
    #[must_use]
    pub fn with_allowed_and_registry(allowed: &[&str], registry: Rc<RefCell<ApiRegistry>>) -> Self {
        #[cfg(feature = "js")]
        {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            HANDLES.with(|handles| {
                handles.borrow_mut().insert(id, Rc::clone(&registry));
            });
            let mut context = Context::default();
            install_globals(&mut context, id, allowed);
            Self {
                registry,
                context,
                id,
            }
        }
        #[cfg(not(feature = "js"))]
        {
            let _ = allowed;
            Self { registry }
        }
    }

    /// The single registry every `chrome.*` / `browser.*` call dispatches into.
    #[must_use]
    pub fn registry(&self) -> &Rc<RefCell<ApiRegistry>> {
        &self.registry
    }

    /// Evaluates `code`, returning the last expression's value as a string.
    ///
    /// Pending microtasks are drained first so `async` background scripts finish
    /// before this returns.
    ///
    /// # Errors
    /// Returns a short message on a syntax error, a runtime error, or when the
    /// `js` feature is disabled.
    #[cfg(feature = "js")]
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        let value = self
            .context
            .eval(Source::from_bytes(code))
            .map_err(|err| error_message(&err))?;
        // Promise callbacks from `async` background scripts run as microtasks.
        let _ = self.context.run_jobs();
        value
            .to_string(&mut self.context)
            .map(|s| s.to_std_string_escaped())
            .map_err(|err| error_message(&err))
    }

    /// Stub used when the `js` feature is disabled.
    ///
    /// # Errors
    /// Always returns an error; see the `js` feature.
    #[cfg(not(feature = "js"))]
    pub fn eval(&mut self, _code: &str) -> Result<String, String> {
        Err("js engine disabled".to_string())
    }
}

impl Default for ExtScriptRuntime {
    fn default() -> Self {
        Self::with_allowed_and_registry(
            &["storage", "tabs", "runtime", "notifications"],
            Rc::new(RefCell::new(ApiRegistry::new())),
        )
    }
}

#[cfg(feature = "js")]
impl Drop for ExtScriptRuntime {
    fn drop(&mut self) {
        HANDLES.with(|handles| {
            handles.borrow_mut().remove(&self.id);
        });
    }
}

// Boa's minimal build has no `console`, and background scripts call it freely.
#[cfg(feature = "js")]
const PRELUDE_HELPERS: &str = r#"
var console = {
  log: function () {}, info: function () {}, warn: function () {},
  error: function () {}, debug: function () {}
};
// WebExtension setters take either (key, value) or an object of keys; getters
// take either a key or an array of keys. Normalize both to JSON for the bridge.
function __hodoEntries(k, v) {
  if (typeof k === 'string') { var o = {}; o[k] = v === undefined ? null : v; return o; }
  return (k && typeof k === 'object') ? k : {};
}
"#;

#[cfg(feature = "js")]
const NS_STORAGE: &str = r#"  storage: {
    local: {
      set: function (k, v) { __hodo_storage_set(JSON.stringify(__hodoEntries(k, v))); },
      get: function (k) { return JSON.parse(__hodo_storage_get(JSON.stringify(k))); }
    },
    sync: {
      set: function (k, v) { __hodo_storage_sync_set(JSON.stringify(__hodoEntries(k, v))); },
      get: function (k) { return JSON.parse(__hodo_storage_sync_get(JSON.stringify(k))); }
    }
  }"#;

#[cfg(feature = "js")]
const NS_TABS: &str = r#"  tabs: {
    create: function (url) { return JSON.parse(__hodo_tabs_create(String(url))); },
    list: function () { return JSON.parse(__hodo_tabs_list()); },
    query: function () { return JSON.parse(__hodo_tabs_list()); }
  }"#;

#[cfg(feature = "js")]
const NS_RUNTIME: &str = r#"  runtime: {
    sendMessage: function (msg) { return JSON.parse(__hodo_send_message(JSON.stringify(msg))); }
  }"#;

#[cfg(feature = "js")]
const NS_NOTIFICATIONS: &str = r#"  notifications: {
    create: function (title, message) { return JSON.parse(__hodo_notify(String(title), String(message))); }
  }"#;

#[cfg(feature = "js")]
fn build_prelude(allowed: &[&str]) -> String {
    // `runtime` is permitted unconditionally by the host, so expose it always.
    let wants = |ns: &str| ns == "runtime" || allowed.contains(&ns);
    let mut namespaces = Vec::new();
    if wants("storage") {
        namespaces.push(NS_STORAGE);
    }
    if wants("tabs") {
        namespaces.push(NS_TABS);
    }
    if wants("runtime") {
        namespaces.push(NS_RUNTIME);
    }
    if wants("notifications") {
        namespaces.push(NS_NOTIFICATIONS);
    }
    let mut prelude = String::from(PRELUDE_HELPERS);
    prelude.push_str("var __hodoApi = {\n");
    prelude.push_str(&namespaces.join(",\n"));
    prelude.push_str("\n};\nvar chrome = __hodoApi;\nvar browser = __hodoApi;\n");
    prelude
}

#[cfg(feature = "js")]
fn install_globals(context: &mut Context, id: u64, allowed: &[&str]) {
    register(context, "__hodo_storage_set", 1, op_storage_set, id);
    register(context, "__hodo_storage_get", 1, op_storage_get, id);
    register(
        context,
        "__hodo_storage_sync_set",
        1,
        op_storage_sync_set,
        id,
    );
    register(
        context,
        "__hodo_storage_sync_get",
        1,
        op_storage_sync_get,
        id,
    );
    register(context, "__hodo_tabs_create", 1, op_tabs_create, id);
    register(context, "__hodo_tabs_list", 0, op_tabs_list, id);
    register(context, "__hodo_send_message", 1, op_send_message, id);
    register(context, "__hodo_notify", 2, op_notify, id);
    let prelude = build_prelude(allowed);
    context
        .eval(Source::from_bytes(prelude.as_str()))
        .expect("the WebExtension prelude is valid JavaScript");
}

#[cfg(feature = "js")]
type Op = fn(&Rc<RefCell<ApiRegistry>>, &[JsValue], &mut Context) -> Result<Value, String>;

#[cfg(feature = "js")]
fn register(context: &mut Context, name: &str, length: usize, op: Op, id: u64) {
    context
        .register_global_callable(JsString::from(name), length, make_native(id, op))
        .expect("registering a fresh global callable should not fail");
}

#[cfg(feature = "js")]
fn with_registry<R>(id: u64, f: impl FnOnce(&Rc<RefCell<ApiRegistry>>) -> R) -> Option<R> {
    let registry = HANDLES.with(|handles| handles.borrow().get(&id).cloned());
    registry.as_ref().map(f)
}

#[cfg(feature = "js")]
fn make_native(id: u64, op: Op) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let result = with_registry(id, |registry| op(registry, args, ctx))
            .ok_or_else(|| js_error("hodo runtime was dropped".to_string()))?
            .map_err(js_error)?;
        let json = serde_json::to_string(&result).map_err(|err| js_error(err.to_string()))?;
        Ok(JsValue::new(JsString::from(json)))
    })
}

#[cfg(feature = "js")]
fn js_error(message: String) -> JsError {
    JsNativeError::error()
        .with_message(Cow::Owned(message))
        .into()
}

#[cfg(feature = "js")]
fn error_message(err: &JsError) -> String {
    let message = err.to_string();
    message
        .strip_prefix("Uncaught ")
        .unwrap_or(&message)
        .to_string()
}

#[cfg(feature = "js")]
fn dispatch(registry: &Rc<RefCell<ApiRegistry>>, call: ApiCall) -> Result<Value, String> {
    match registry.borrow_mut().dispatch(&call) {
        ApiResult::Ok(value) => Ok(value),
        ApiResult::Err(message) => Err(message),
    }
}

#[cfg(feature = "js")]
fn arg_string(args: &[JsValue], index: usize, ctx: &mut Context) -> Result<String, String> {
    args.get_or_undefined(index)
        .to_string(ctx)
        .map(|s| s.to_std_string_escaped())
        .map_err(|err| err.to_string())
}

#[cfg(feature = "js")]
fn parse_json(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|err| err.to_string())
}

#[cfg(feature = "js")]
fn op_storage_set(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let entries = parse_json(&arg_string(args, 0, ctx)?)?;
    let object = entries
        .as_object()
        .ok_or_else(|| "storage.set expects an object".to_string())?;
    for (key, value) in object {
        dispatch(
            registry,
            ApiCall::StorageSet {
                key: key.clone(),
                value: value.clone(),
            },
        )?;
    }
    Ok(Value::Null)
}

#[cfg(feature = "js")]
fn op_storage_get(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let keys = parse_json(&arg_string(args, 0, ctx)?)?;
    match keys {
        Value::String(key) => dispatch(registry, ApiCall::StorageGet { key }),
        Value::Array(list) => {
            let mut result = serde_json::Map::new();
            for key in list {
                let key = key
                    .as_str()
                    .ok_or_else(|| "storage.get keys must be strings".to_string())?
                    .to_string();
                result.insert(
                    key.clone(),
                    dispatch(registry, ApiCall::StorageGet { key })?,
                );
            }
            Ok(Value::Object(result))
        }
        _ => Err("storage.get expects a key or an array of keys".into()),
    }
}

#[cfg(feature = "js")]
fn op_storage_sync_set(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let entries = parse_json(&arg_string(args, 0, ctx)?)?;
    let object = entries
        .as_object()
        .ok_or_else(|| "storage.sync.set expects an object".to_string())?;
    for (key, value) in object {
        dispatch(
            registry,
            ApiCall::StorageSyncSet {
                key: key.clone(),
                value: value.clone(),
            },
        )?;
    }
    Ok(Value::Null)
}

#[cfg(feature = "js")]
fn op_storage_sync_get(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let keys = parse_json(&arg_string(args, 0, ctx)?)?;
    match keys {
        Value::String(key) => dispatch(registry, ApiCall::StorageSyncGet { key }),
        Value::Array(list) => {
            let mut result = serde_json::Map::new();
            for key in list {
                let key = key
                    .as_str()
                    .ok_or_else(|| "storage.sync.get keys must be strings".to_string())?
                    .to_string();
                result.insert(
                    key.clone(),
                    dispatch(registry, ApiCall::StorageSyncGet { key })?,
                );
            }
            Ok(Value::Object(result))
        }
        _ => Err("storage.sync.get expects a key or an array of keys".into()),
    }
}

#[cfg(feature = "js")]
fn op_tabs_create(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let url = arg_string(args, 0, ctx)?;
    dispatch(registry, ApiCall::TabsCreate { url })
}

#[cfg(feature = "js")]
fn op_tabs_list(
    registry: &Rc<RefCell<ApiRegistry>>,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> Result<Value, String> {
    dispatch(registry, ApiCall::TabsList)
}

#[cfg(feature = "js")]
fn op_send_message(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let message = parse_json(&arg_string(args, 0, ctx)?)?;
    dispatch(registry, ApiCall::SendMessage { message })
}

#[cfg(feature = "js")]
fn op_notify(
    registry: &Rc<RefCell<ApiRegistry>>,
    args: &[JsValue],
    ctx: &mut Context,
) -> Result<Value, String> {
    let title = arg_string(args, 0, ctx)?;
    let message = arg_string(args, 1, ctx)?;
    dispatch(registry, ApiCall::NotificationsCreate { title, message })
}

#[cfg(test)]
#[cfg(feature = "js")]
mod tests {
    use super::*;
    use serde_json::json;

    fn get(registry: &Rc<RefCell<ApiRegistry>>, key: &str) -> Value {
        match registry
            .borrow_mut()
            .dispatch(&ApiCall::StorageGet { key: key.into() })
        {
            ApiResult::Ok(value) => value,
            ApiResult::Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn eval_arithmetic() {
        let mut runtime = ExtScriptRuntime::new();
        assert_eq!(runtime.eval("1+1").unwrap(), "2");
    }

    #[test]
    fn eval_syntax_error_is_err() {
        let mut runtime = ExtScriptRuntime::new();
        assert!(runtime.eval("1 +").is_err());
    }

    #[test]
    fn chrome_and_browser_share_one_dispatch_table() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval("chrome.storage.local.set('greeting', 'hello'); browser.storage.local.get('greeting')")
            .unwrap();
        assert_eq!(result, "hello");
        assert_eq!(get(runtime.registry(), "greeting"), json!("hello"));
    }

    #[test]
    fn object_style_storage_matches_real_extensions() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval(
                "browser.storage.local.set({ greeting: 'Hello Gecko', count: 2 }); \
                 JSON.stringify(browser.storage.local.get(['greeting', 'count']))",
            )
            .unwrap();
        assert_eq!(result, r#"{"count":2,"greeting":"Hello Gecko"}"#);
        assert_eq!(get(runtime.registry(), "greeting"), json!("Hello Gecko"));
    }

    #[test]
    fn sync_and_local_storage_use_separate_maps() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval(
                "browser.storage.sync.set('k', 42); \
                 JSON.stringify([browser.storage.sync.get('k'), chrome.storage.local.get('k')])",
            )
            .unwrap();
        assert_eq!(result, "[42,null]");
    }

    #[test]
    fn tabs_create_via_browser_returns_tab_id() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval("browser.tabs.create('https://example.com').tabId")
            .unwrap();
        assert_eq!(result, "1");
        let next = runtime
            .registry()
            .borrow_mut()
            .dispatch(&ApiCall::TabsCreate {
                url: "probe".into(),
            });
        match next {
            ApiResult::Ok(value) => assert_eq!(value["tabId"], 2),
            ApiResult::Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn tabs_create_then_list_shows_the_tab() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval(
                "chrome.tabs.create('https://example.com'); \
                 JSON.stringify(chrome.tabs.list())",
            )
            .unwrap();
        assert_eq!(result, r#"[{"tabId":1,"url":"https://example.com"}]"#);
    }

    #[test]
    fn notifications_create_returns_id_json() {
        let mut runtime = ExtScriptRuntime::new();
        let result = runtime
            .eval("JSON.stringify(browser.notifications.create('Title', 'Body'))")
            .unwrap();
        assert_eq!(result, r#"{"notificationId":1}"#);
    }

    #[test]
    fn with_allowed_omits_disallowed_namespaces() {
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        assert_eq!(runtime.eval("typeof chrome.storage").unwrap(), "object");
        assert_eq!(runtime.eval("typeof browser.tabs").unwrap(), "undefined");
        assert_eq!(
            runtime.eval("typeof chrome.notifications").unwrap(),
            "undefined"
        );
        assert_eq!(runtime.eval("typeof chrome.runtime").unwrap(), "object");
        assert!(runtime.eval("chrome.tabs.list()").is_err());
    }

    #[test]
    fn async_background_body_is_drained() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("async function main() { await null; chrome.storage.local.set('done', 1); } main();")
            .unwrap();
        assert_eq!(get(runtime.registry(), "done"), json!(1));
    }
}

#[cfg(test)]
#[cfg(not(feature = "js"))]
mod tests {
    use super::*;

    #[test]
    fn eval_without_feature_is_disabled() {
        let mut runtime = ExtScriptRuntime::new();
        assert_eq!(runtime.eval("1+1"), Err("js engine disabled".to_string()));
    }

    #[test]
    fn with_allowed_without_feature_is_disabled() {
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        assert_eq!(runtime.eval("1+1"), Err("js engine disabled".to_string()));
    }
}

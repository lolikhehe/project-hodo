//! JavaScript execution for WebExtensions: one dispatch table, two API styles.
//!
//! `chrome.*` and `browser.*` are thin frozen wrappers over the same native
//! task queue and [`hodo_ext_api::ApiRegistry`]. Every API call, timer, message
//! and event is a task; boa microtasks are drained after each task, giving
//! browser-like "microtask before next timer" ordering.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use hodo_ext_api::{
    validate_api_value, ApiCall, ApiRegistry, ApiResult, Clock, ExtEvent, RuntimeConfig,
    SystemClock,
};
use serde_json::{json, Map, Value};

mod executor;
mod prelude;

#[cfg(feature = "js")]
use executor::{ExtensionHooks, HodoJobExecutor};
#[cfg(feature = "js")]
use std::borrow::Cow;
#[cfg(feature = "js")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "js")]
use boa_engine::{
    error::JsNativeError, native_function::NativeFunction, property::PropertyKey, Context, JsArgs,
    JsError, JsString, JsValue, Source,
};

// Boa's `from_closure` is unsafe (its GC cannot inspect captured variables), and
// our shared state has no `Trace` impl for `from_copy_closure_with_captures`.
// A `Copy` runtime id plus this handle table keeps the bridge 100% safe Rust.
#[cfg(feature = "js")]
thread_local! {
    static HANDLES: RefCell<HashMap<u64, Rc<RuntimeState>>> = RefCell::new(HashMap::new());
}

#[cfg(feature = "js")]
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Safety limits local to this wave; the ones with an exact match in the
/// central table alias it here, the rest stay local until calibrated.
const MAX_TIMERS: usize = hodo_types::limits::MAX_ACTIVE_TIMERS;
const MAX_TASKS_PER_TICK: usize = 4096;
const LOOP_ITERATION_LIMIT: u64 = 1_000_000;
const RECURSION_LIMIT: usize = 256;
/// Boa 0.22 parses deeply nested literals recursively; guard the parser before
/// handing it extension-controlled source (runtime limits apply only after parse).
const MAX_SOURCE_BRACKET_DEPTH: usize = 24;

#[cfg(feature = "js")]
type FetchHook = Rc<dyn Fn(&ApiCall) -> ApiResult>;

#[cfg(feature = "js")]
#[derive(Clone)]
struct Timer {
    id: u64,
    deadline: u64,
    interval_ms: Option<u64>,
    args: Value,
}

#[cfg(feature = "js")]
enum Task {
    Failure {
        id: u64,
        message: String,
    },
    Api {
        id: u64,
        calls: Vec<ApiCall>,
        merge_keys: Option<Vec<String>>,
    },
    Timer {
        id: u64,
        args: Value,
        repeating: bool,
    },
    Event {
        name: String,
        args: Vec<Value>,
    },
}

#[cfg(feature = "js")]
pub(crate) struct RuntimeState {
    registry: Rc<RefCell<ApiRegistry>>,
    clock: Rc<dyn Clock>,
    config: RuntimeConfig,
    queue: RefCell<VecDeque<Task>>,
    timers: RefCell<Vec<Timer>>,
    logs: RefCell<Vec<String>>,
    /// Host-side permission-gated implementation of `fetch` (absent in tests).
    fetch_hook: RefCell<Option<FetchHook>>,
    next_id: Cell<u64>,
}

#[cfg(feature = "js")]
impl RuntimeState {
    fn new(
        registry: Rc<RefCell<ApiRegistry>>,
        clock: Rc<dyn Clock>,
        config: RuntimeConfig,
    ) -> Self {
        Self {
            registry,
            clock,
            config,
            queue: RefCell::new(VecDeque::new()),
            timers: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
            fetch_hook: RefCell::new(None),
            next_id: Cell::new(1),
        }
    }

    fn fresh_id(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id.wrapping_add(1).max(1));
        id
    }

    fn log(&self, message: String) {
        let name = &self.config.extension_name;
        let line = if name.is_empty() || message.contains(name) {
            message
        } else {
            format!("{name}: {message}")
        };
        self.logs.borrow_mut().push(line);
    }

    fn schedule_timer(
        &self,
        id: u64,
        delay_ms: u64,
        interval: Option<u64>,
        args: Value,
    ) -> Result<(), String> {
        let mut timers = self.timers.borrow_mut();
        if timers.len() >= MAX_TIMERS {
            return Err(format!("timer limit of {MAX_TIMERS} exceeded"));
        }
        let now = self.clock.now_ms();
        timers.push(Timer {
            id,
            deadline: now.saturating_add(delay_ms.max(1)),
            interval_ms: interval,
            args,
        });
        Ok(())
    }
}

/// A JavaScript context wired to one extension API registry.
pub struct ExtScriptRuntime {
    registry: Rc<RefCell<ApiRegistry>>,
    #[cfg(feature = "js")]
    context: Context,
    #[cfg(feature = "js")]
    id: u64,
    #[cfg(feature = "js")]
    state: Rc<RuntimeState>,
    #[cfg(feature = "js")]
    executor: Rc<HodoJobExecutor>,
    #[cfg(feature = "js")]
    hooks: Rc<ExtensionHooks>,
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
        let config = RuntimeConfig {
            allowed_namespaces: allowed.iter().map(|s| (*s).to_string()).collect(),
            ..RuntimeConfig::default()
        };
        Self::with_config_and_registry(config, registry, Rc::new(SystemClock))
    }

    /// Full constructor: dialect, extension identity, permission-derived
    /// namespaces, clock and registry all come from the host.
    #[must_use]
    pub fn with_config_and_registry(
        config: RuntimeConfig,
        registry: Rc<RefCell<ApiRegistry>>,
        clock: Rc<dyn Clock>,
    ) -> Self {
        #[cfg(feature = "js")]
        {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let state = Rc::new(RuntimeState::new(
                Rc::clone(&registry),
                clock,
                config.clone(),
            ));
            HANDLES.with(|handles| {
                handles.borrow_mut().insert(id, Rc::clone(&state));
            });

            let executor = Rc::new(HodoJobExecutor::default());
            let hooks = Rc::new(ExtensionHooks::new(config.extension_name.clone()));
            let mut context = Context::builder()
                .job_executor(Rc::clone(&executor))
                .host_hooks(Rc::clone(&hooks))
                .build()
                .expect("building a fresh boa context should not fail");
            let mut limits = context.runtime_limits();
            limits.set_loop_iteration_limit(LOOP_ITERATION_LIMIT);
            limits.set_recursion_limit(RECURSION_LIMIT);
            context.set_runtime_limits(limits);

            register_natives(&mut context, id);
            let prelude = prelude::build_prelude(&config);
            context
                .eval(Source::from_bytes(prelude.as_str()))
                .expect("the WebExtension prelude is valid JavaScript");
            // Host-driven `eval` is trusted; script-driven eval/Function is not.
            hooks.set_block_compile(true);

            Self {
                registry,
                context,
                id,
                state,
                executor,
                hooks,
            }
        }
        #[cfg(not(feature = "js"))]
        {
            let _ = (config, clock);
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
    /// Pending microtasks and due tasks are drained before this returns.
    ///
    /// # Errors
    /// Returns a short message on a syntax error, a runtime error, or when the
    /// `js` feature is disabled.
    #[cfg(feature = "js")]
    pub fn eval(&mut self, code: &str) -> Result<String, String> {
        if !source_bracket_depth_ok(code) {
            return Err("source nesting too deep".to_string());
        }
        // `Context::eval` compiles the source directly and does not go through
        // the host hook, so script-level eval/Function stay blocked.
        let evaluated = self.context.eval(Source::from_bytes(code));
        let value = evaluated.map_err(|err| error_message(&err))?;
        self.tick();
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

    /// Runs due tasks, timers, events and their microtasks to quiescence.
    /// Returns the number of tasks and jobs executed.
    #[cfg(feature = "js")]
    pub fn tick(&mut self) -> usize {
        let mut count = 0;
        while count < MAX_TASKS_PER_TICK {
            count += self.executor.drain(&mut self.context);
            self.deliver_events();
            let Some(task) = self.next_task() else { break };
            self.run_task(task);
            count += 1;
            count += self.executor.drain(&mut self.context);
            self.deliver_events();
        }
        for line in self.executor.take_errors() {
            self.state.log(format!("internal job error: {line}"));
        }
        for line in self.hooks.take_unhandled() {
            self.state.log(line);
        }
        count
    }

    /// Non-JS builds have no event loop.
    #[cfg(not(feature = "js"))]
    pub fn tick(&mut self) -> usize {
        0
    }

    /// Install the host's permission-gated `fetch` implementation. Calls with
    /// no hook return a clean API error instead of reaching the network.
    #[cfg(feature = "js")]
    pub fn set_fetch_hook(&mut self, hook: FetchHook) {
        *self.state.fetch_hook.borrow_mut() = Some(hook);
    }

    /// Non-JS builds have no fetch hook.
    #[cfg(not(feature = "js"))]
    pub fn set_fetch_hook(&mut self, _hook: Rc<dyn Fn(&ApiCall) -> ApiResult>) {}

    /// Enqueue an externally-originated event (e.g. an extension message or an
    /// alarm fired by the host) for delivery during the next tick.
    #[cfg(feature = "js")]
    pub fn fire_event(&mut self, name: &str, payload: Value) {
        self.state.queue.borrow_mut().push_back(Task::Event {
            name: "__hodoFire".to_string(),
            args: vec![json!(name), payload],
        });
    }

    /// Non-JS builds have no event loop.
    #[cfg(not(feature = "js"))]
    pub fn fire_event(&mut self, _name: &str, _payload: Value) {}

    /// Number of active timers (used by tests and idle-unload accounting).
    #[must_use]
    pub fn timer_count(&self) -> usize {
        #[cfg(feature = "js")]
        {
            self.state.timers.borrow().len()
        }
        #[cfg(not(feature = "js"))]
        {
            0
        }
    }

    /// Number of queued tasks (used by tests).
    #[must_use]
    pub fn pending_task_count(&self) -> usize {
        #[cfg(feature = "js")]
        {
            self.state.queue.borrow().len()
        }
        #[cfg(not(feature = "js"))]
        {
            0
        }
    }

    /// Drain and return diagnostic log lines (unhandled rejections, callback
    /// errors, CSP violations).
    #[cfg(feature = "js")]
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.state.logs.borrow_mut())
    }

    /// Non-JS builds have no logs.
    #[cfg(not(feature = "js"))]
    pub fn take_logs(&mut self) -> Vec<String> {
        Vec::new()
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
        // All timers and queued callbacks live only in this context; clearing
        // them here prevents any late delivery after the context is gone.
        self.state.timers.borrow_mut().clear();
        self.state.queue.borrow_mut().clear();
    }
}

#[cfg(feature = "js")]
impl ExtScriptRuntime {
    fn next_task(&self) -> Option<Task> {
        let now = self.state.clock.now_ms();
        let mut timers = self.state.timers.borrow_mut();
        let mut due: Vec<u64> = timers
            .iter()
            .filter(|timer| timer.deadline <= now)
            .map(|timer| timer.id)
            .collect();
        if !due.is_empty() {
            due.sort_unstable();
            let id = due[0];
            let timer = timers.iter().find(|t| t.id == id).expect("just found");
            let args = timer.args.clone();
            let next_deadline = timer
                .interval_ms
                .map(|interval| timer.deadline.saturating_add(interval.max(1)));
            if let Some(deadline) = next_deadline {
                if let Some(timer) = timers.iter_mut().find(|t| t.id == id) {
                    timer.deadline = deadline;
                }
            } else {
                timers.retain(|t| t.id != id);
            }
            return Some(Task::Timer {
                id,
                args,
                repeating: next_deadline.is_some(),
            });
        }
        drop(timers);
        self.state.queue.borrow_mut().pop_front()
    }

    fn run_task(&mut self, task: Task) {
        match task {
            Task::Failure { id, message } => self.settle(id, Err(message)),
            Task::Api {
                id,
                calls,
                merge_keys,
            } => {
                let mut last = Value::Null;
                for call in calls {
                    let hooked = matches!(call, ApiCall::Fetch { .. })
                        && self.state.fetch_hook.borrow().is_some();
                    let result = if hooked {
                        let hook = self.state.fetch_hook.borrow();
                        hook.as_ref().expect("just checked")(&call)
                    } else if matches!(call, ApiCall::Fetch { .. }) {
                        ApiResult::Err("fetch: no host hook installed".to_string())
                    } else {
                        self.state.registry.borrow_mut().dispatch(&call)
                    };
                    last = match result {
                        ApiResult::Ok(value) => value,
                        ApiResult::Err(err) => {
                            self.settle(id, Err(err));
                            return;
                        }
                    };
                }
                if let Some(keys) = merge_keys {
                    let mut map = Map::new();
                    for key in keys {
                        map.insert(
                            key.clone(),
                            self.state
                                .registry
                                .borrow_mut()
                                .dispatch(&ApiCall::StorageGet { key: key.clone() })
                                .into_ok_or_null(),
                        );
                    }
                    last = Value::Object(map);
                }
                self.settle(id, Ok(last));
            }
            Task::Timer {
                id,
                args,
                repeating,
            } => self.fire_timer(id, &args, repeating),
            Task::Event { name, args } => {
                let values: Vec<JsValue> = args
                    .iter()
                    .filter_map(|arg| JsValue::from_json(arg, &mut self.context).ok())
                    .collect();
                if let Err(err) = call_global(&mut self.context, &name, &values) {
                    self.state.log(err);
                }
            }
        }
    }

    fn settle(&mut self, id: u64, result: Result<Value, String>) {
        let (ok, value, error) = match result {
            Ok(value) => (true, value, String::new()),
            Err(err) => (false, Value::Null, err),
        };
        let value = match JsValue::from_json(&value, &mut self.context) {
            Ok(value) => value,
            Err(err) => {
                self.state
                    .log(format!("api result conversion failed: {err}"));
                JsValue::null()
            }
        };
        let args = [
            JsValue::from(id as f64),
            JsValue::from(ok),
            value,
            JsValue::from(JsString::from(error)),
        ];
        if let Err(err) = call_global(&mut self.context, "__hodoSettle", &args) {
            self.state.log(err);
        }
    }

    fn fire_timer(&mut self, id: u64, timer_args: &Value, repeating: bool) {
        let key = PropertyKey::from(JsString::from(format!("__hodoTimer_{id}")));
        let callback = {
            match self
                .context
                .global_object()
                .get(key.clone(), &mut self.context)
            {
                Ok(value) => value,
                Err(err) => {
                    self.state.log(format!("timer lookup failed: {err}"));
                    return;
                }
            }
        };
        let args: Vec<JsValue> = match timer_args.as_array() {
            Some(values) => values
                .iter()
                .filter_map(|value| JsValue::from_json(value, &mut self.context).ok())
                .collect(),
            None => Vec::new(),
        };
        if let Some(function) = callback.as_callable() {
            if let Err(err) = function.call(&JsValue::undefined(), &args, &mut self.context) {
                self.state
                    .log(format!("timer callback: {}", error_message(&err)));
            }
        }
        if !repeating {
            let _ = self.context.global_object().set(
                key,
                JsValue::undefined(),
                false,
                &mut self.context,
            );
        }
    }

    fn deliver_events(&mut self) {
        let events = self.state.registry.borrow_mut().drain_events();
        for event in events {
            match event {
                ExtEvent::StorageChanged {
                    area,
                    key,
                    old_value,
                    new_value,
                } => {
                    let changes = json!({ key: { "oldValue": old_value, "newValue": new_value } });
                    self.fire_internal(
                        "__hodoFire",
                        &[json!("storage.onChanged"), changes, json!(area)],
                    );
                }
                ExtEvent::AlarmFired { name } => {
                    self.fire_internal(
                        "__hodoFire",
                        &[json!("alarms.onAlarm"), json!({ "name": name })],
                    );
                }
                ExtEvent::MessageReceived { message } => {
                    self.fire_internal("__hodoFireMessage", &[message]);
                }
                ExtEvent::TabActivated { .. } | ExtEvent::NotificationShown { .. } => {}
            }
        }
    }

    fn fire_internal(&mut self, function: &str, args: &[Value]) {
        let values: Vec<JsValue> = args
            .iter()
            .filter_map(|arg| JsValue::from_json(arg, &mut self.context).ok())
            .collect();
        if let Err(err) = call_global(&mut self.context, function, &values) {
            self.state.log(err);
        }
    }
}

#[cfg(feature = "js")]
trait IntoOkOrNull {
    fn into_ok_or_null(self) -> Value;
}

#[cfg(feature = "js")]
impl IntoOkOrNull for ApiResult {
    fn into_ok_or_null(self) -> Value {
        match self {
            ApiResult::Ok(value) => value,
            ApiResult::Err(_) => Value::Null,
        }
    }
}

#[cfg(feature = "js")]
fn source_bracket_depth_ok(code: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = '\0';
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        let next = bytes.get(i + 1).copied().unwrap_or(0) as char;
        if line_comment {
            if ch == '\n' {
                line_comment = false;
            }
        } else if block_comment {
            if ch == '*' && next == '/' {
                block_comment = false;
                i += 1;
            }
        } else if quote != '\0' {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                quote = '\0';
            }
        } else if ch == '/' && next == '/' {
            line_comment = true;
            i += 1;
        } else if ch == '/' && next == '*' {
            block_comment = true;
            i += 1;
        } else if ch == '\'' || ch == '"' || ch == '`' {
            quote = ch;
        } else if matches!(ch, '(' | '[' | '{') {
            depth += 1;
            if depth > MAX_SOURCE_BRACKET_DEPTH {
                return false;
            }
        } else if matches!(ch, ')' | ']' | '}') {
            depth = depth.saturating_sub(1);
        }
        i += 1;
    }
    true
}

#[cfg(feature = "js")]
fn call_global(context: &mut Context, name: &str, args: &[JsValue]) -> Result<(), String> {
    let global = context.global_object();
    let function = global
        .get(JsString::from(name), context)
        .map_err(|err| format!("{name}: {err}"))?;
    let Some(function) = function.as_callable() else {
        return Ok(());
    };
    function
        .call(&JsValue::undefined(), args, context)
        .map(|_| ())
        .map_err(|err| format!("{name}: {}", error_message(&err)))
}

#[cfg(feature = "js")]
fn with_state<R>(id: u64, f: impl FnOnce(&Rc<RuntimeState>) -> R) -> Result<R, String> {
    let state = HANDLES.with(|handles| handles.borrow().get(&id).cloned());
    state
        .as_ref()
        .map(f)
        .ok_or_else(|| "hodo runtime was dropped".to_string())
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
fn json_arg(args: &[JsValue], index: usize, context: &mut Context) -> Result<Value, String> {
    let value = args.get_or_undefined(index);
    value
        .to_json(context)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "only JSON-like values are accepted by extension APIs".to_string())
}

#[cfg(feature = "js")]
fn string_arg(args: &[JsValue], index: usize, context: &mut Context) -> Result<String, String> {
    args.get_or_undefined(index)
        .to_string(context)
        .map(|s| s.to_std_string_escaped())
        .map_err(|err| err.to_string())
}

#[cfg(feature = "js")]
fn map_entries(value: &Value) -> Option<Vec<(String, Value)>> {
    value.as_object().map(|object| {
        object
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    })
}

#[cfg(feature = "js")]
fn make_calls(
    name: &str,
    args: &[Value],
    state: &RuntimeState,
) -> Result<(Vec<ApiCall>, Option<Vec<String>>), String> {
    let mut calls = Vec::new();
    let mut merge_keys = None;
    match name {
        "storage.local.set"
        | "storage.sync.set"
        | "storage.session.set"
        | "storage.local.remove"
        | "storage.sync.remove"
        | "storage.session.remove" => {
            let area = name.split('.').nth(1).unwrap_or("local");
            let removing = name.ends_with(".remove");
            let entries = if args.len() >= 2 && args[0].is_string() {
                vec![(string_value(&args[0]), args[1].clone())]
            } else {
                args.first()
                    .and_then(map_entries)
                    .ok_or_else(|| format!("{name} expects an object"))?
            };
            for (key, value) in entries {
                let value = if removing { Value::Null } else { value };
                validate_api_value(&value)?;
                calls.push(match area {
                    "sync" => ApiCall::StorageSyncSet { key, value },
                    "session" => ApiCall::StorageSessionSet { key, value },
                    _ => ApiCall::StorageSet { key, value },
                });
            }
        }
        "storage.local.get" | "storage.sync.get" | "storage.session.get" => {
            let area = name.split('.').nth(1).unwrap_or("local");
            let keys = match args.first() {
                Some(Value::String(key)) => vec![key.clone()],
                Some(Value::Array(items)) => items
                    .iter()
                    .map(|item| {
                        item.as_str()
                            .map(str::to_string)
                            .ok_or_else(|| "storage keys must be strings".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                Some(Value::Null) | None => Vec::new(),
                _ => return Err("storage.get expects a key or an array of keys".to_string()),
            };
            if keys.len() == 1 {
                let key = keys[0].clone();
                calls.push(match area {
                    "sync" => ApiCall::StorageSyncGet { key },
                    "session" => ApiCall::StorageSessionGet { key },
                    _ => ApiCall::StorageGet { key },
                });
            } else {
                merge_keys = Some(keys);
            }
        }
        "tabs.create" => {
            let url = match args.first() {
                Some(Value::String(url)) => url.clone(),
                Some(Value::Object(map)) => map
                    .get("url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                _ => return Err("tabs.create expects a url".to_string()),
            };
            calls.push(ApiCall::TabsCreate { url });
        }
        "tabs.list" | "tabs.query" => calls.push(ApiCall::TabsList),
        "runtime.sendMessage" | "runtime.portMessage" => {
            let message = args.first().cloned().unwrap_or(Value::Null);
            validate_api_value(&message)?;
            calls.push(ApiCall::SendMessage { message });
        }
        "notifications.create" => {
            let (title, message) = match args.first() {
                Some(Value::Object(map)) => (
                    map.get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    map.get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                ),
                Some(Value::String(title)) => (
                    title.clone(),
                    args.get(1)
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                ),
                _ => return Err("notifications.create expects a title and message".to_string()),
            };
            calls.push(ApiCall::NotificationsCreate { title, message });
        }
        "alarms.create" => {
            let name = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "alarms.create expects a name".to_string())?;
            let info = args.get(1).cloned().unwrap_or_else(|| json!({}));
            let number = |key: &str| -> u64 {
                info.get(key)
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
                    .max(0.0) as u64
            };
            calls.push(ApiCall::AlarmCreate {
                name: name.to_string(),
                delay_ms: number("delayMs").max(number("delay")),
                period_ms: info.get("periodMs").and_then(Value::as_u64),
                now_ms: state.clock.now_ms(),
            });
        }
        "alarms.clear" => {
            let name = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "alarms.clear expects a name".to_string())?;
            calls.push(ApiCall::AlarmClear {
                name: name.to_string(),
            });
        }
        "fetch" => {
            let url = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "fetch expects a url".to_string())?;
            let method = args
                .get(1)
                .and_then(Value::as_str)
                .unwrap_or("GET")
                .to_string();
            let body = args
                .get(2)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            validate_api_value(&json!({ "url": url, "body": body }))?;
            calls.push(ApiCall::Fetch {
                url: url.to_string(),
                method,
                body,
            });
        }
        other => return Err(format!("unknown api: {other}")),
    }
    Ok((calls, merge_keys))
}

#[cfg(feature = "js")]
fn string_value(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

#[cfg(feature = "js")]
fn op_post_task(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        // Conversion failures become a clean task-level error, so a cyclic or
        // oversized value rejects the returned Promise instead of throwing out
        // of the wrapper and killing the surrounding background script.
        let prepared = string_arg(args, 0, ctx).and_then(|name| {
            let values = json_arg(args, 1, ctx)?;
            let list = values
                .as_array()
                .cloned()
                .ok_or_else(|| "api args must be an array".to_string())?;
            make_calls(&name, &list, &state)
        });
        let call_id = state.fresh_id();
        let task = match prepared {
            Ok((calls, merge_keys)) => Task::Api {
                id: call_id,
                calls,
                merge_keys,
            },
            Err(message) => Task::Failure {
                id: call_id,
                message,
            },
        };
        state.queue.borrow_mut().push_back(task);
        Ok(JsValue::from(call_id as f64))
    })
}

#[cfg(feature = "js")]
fn op_timer(id: u64, interval: bool) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let callback = args
            .get_or_undefined(0)
            .as_callable()
            .ok_or_else(|| js_error("timer expects a function".into()))?
            .clone();
        let delay = args.get_or_undefined(1).as_number().unwrap_or(1.0);
        let delay_ms = if delay.is_finite() && delay >= 1.0 {
            delay as u64
        } else {
            1
        };
        let values = json_arg(args, 2, ctx).unwrap_or_else(|_| json!([]));
        let timer_id = state.fresh_id();
        let key = PropertyKey::from(JsString::from(format!("__hodoTimer_{timer_id}")));
        ctx.global_object()
            .set(key, JsValue::from(callback), false, ctx)
            .map_err(|err| js_error(err.to_string()))?;
        state
            .schedule_timer(timer_id, delay_ms, interval.then_some(delay_ms), values)
            .map_err(js_error)?;
        Ok(JsValue::from(timer_id as f64))
    })
}

#[cfg(feature = "js")]
fn op_clear_timer(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, _ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let timer_id = args.get_or_undefined(0).as_number().unwrap_or(-1.0) as i64;
        if timer_id >= 0 {
            state
                .timers
                .borrow_mut()
                .retain(|timer| timer.id != timer_id as u64);
        }
        Ok(JsValue::undefined())
    })
}

#[cfg(feature = "js")]
fn op_log(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        with_state(id, |state| {
            if let Ok(message) = string_arg(args, 0, ctx) {
                state.log(message);
            }
        })
        .map_err(js_error)?;
        Ok(JsValue::undefined())
    })
}

#[cfg(feature = "js")]
fn register(context: &mut Context, name: &str, length: usize, function: NativeFunction) {
    context
        .register_global_callable(JsString::from(name), length, function)
        .expect("registering a fresh global callable should not fail");
}

#[cfg(feature = "js")]
fn register_natives(context: &mut Context, id: u64) {
    register(context, "__hodo_post_task", 2, op_post_task(id));
    register(context, "__hodo_set_timeout", 3, op_timer(id, false));
    register(context, "__hodo_set_interval", 3, op_timer(id, true));
    register(context, "__hodo_clear_timer", 1, op_clear_timer(id));
    register(context, "__hodo_log", 1, op_log(id));
}

#[cfg(test)]
#[cfg(feature = "js")]
mod tests {
    use super::*;
    use hodo_ext_api::{ApiDialect, FakeClock};
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

    fn trace(runtime: &mut ExtScriptRuntime) -> Vec<String> {
        runtime.take_logs()
    }

    fn runtime_with(clock: Rc<dyn Clock>, allowed: &[&str]) -> ExtScriptRuntime {
        let config = RuntimeConfig {
            allowed_namespaces: allowed.iter().map(|s| (*s).to_string()).collect(),
            ..RuntimeConfig::default()
        };
        ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            clock,
        )
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
        runtime
            .eval("chrome.storage.local.set({ greeting: 'hello' });")
            .unwrap();
        runtime.tick();
        runtime
            .eval("browser.storage.local.get('greeting').then(function (v) { __hodo_log(v); });")
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["hello".to_string()]);
        assert_eq!(get(runtime.registry(), "greeting"), json!("hello"));
    }

    #[test]
    fn object_style_storage_matches_real_extensions() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("browser.storage.local.set({ greeting: 'Hello Gecko', count: 2 });")
            .unwrap();
        runtime.tick();
        runtime
            .eval("browser.storage.local.get(['greeting', 'count']).then(function (r) { __hodo_log(JSON.stringify(r)); });")
            .unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec![r#"{"count":2,"greeting":"Hello Gecko"}"#.to_string()]
        );
        assert_eq!(get(runtime.registry(), "greeting"), json!("Hello Gecko"));
    }

    #[test]
    fn sync_and_local_storage_use_separate_maps() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("browser.storage.sync.set('k', 42); browser.storage.local.set('k', null);")
            .unwrap();
        runtime.tick();
        runtime
            .eval("Promise.all([browser.storage.sync.get('k'), browser.storage.local.get('k')]).then(function (r) { __hodo_log(JSON.stringify(r)); });")
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["[42,null]".to_string()]);
    }

    #[test]
    fn tabs_create_via_browser_returns_tab_id() {
        let mut runtime = ExtScriptRuntime::new();
        runtime.eval("browser.tabs.create('https://example.com').then(function (t) { __hodo_log('id=' + t.tabId); });").unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["id=1".to_string()]);
    }

    #[test]
    fn notifications_create_returns_id_json() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("browser.notifications.create('Title', 'Body').then(function (r) { __hodo_log(JSON.stringify(r)); });")
            .unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec![r#"{"notificationId":1}"#.to_string()]
        );
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
            .eval("async function main() { await browser.storage.local.set('done', 1); } main();")
            .unwrap();
        runtime.tick();
        assert_eq!(get(runtime.registry(), "done"), json!(1));
    }

    #[test]
    fn microtask_runs_before_next_timer() {
        let clock = Rc::new(FakeClock::new(0));
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            RuntimeConfig::default(),
            Rc::new(RefCell::new(ApiRegistry::new())),
            clock.clone(),
        );
        runtime
            .eval(
                "setTimeout(function () { __hodo_log('timer1'); Promise.resolve().then(function () { __hodo_log('micro'); }); }, 0); \
                 setTimeout(function () { __hodo_log('timer2'); }, 0); \
                 __hodo_log('sync');",
            )
            .unwrap();
        assert_eq!(trace(&mut runtime), vec!["sync".to_string()]);
        assert_eq!(runtime.timer_count(), 2);
        clock.set_ms(1);
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec![
                "timer1".to_string(),
                "micro".to_string(),
                "timer2".to_string()
            ]
        );
        assert_eq!(runtime.timer_count(), 0);
    }

    #[test]
    fn interval_fires_on_fake_clock_and_can_be_cleared() {
        let clock = Rc::new(FakeClock::new(0));
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            RuntimeConfig::default(),
            Rc::new(RefCell::new(ApiRegistry::new())),
            clock.clone(),
        );
        runtime
            .eval("var id = setInterval(function () { __hodo_log('tick'); }, 5); setTimeout(function () { clearInterval(id); }, 12);")
            .unwrap();
        clock.set_ms(5);
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["tick".to_string()]);
        clock.set_ms(10);
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["tick".to_string()]);
        clock.set_ms(12);
        runtime.tick();
        assert!(trace(&mut runtime).is_empty());
        assert_eq!(runtime.timer_count(), 0);
    }

    #[test]
    fn timer_cap_rejects_excess_timers() {
        let mut runtime = ExtScriptRuntime::new();
        let mut script = String::new();
        for _ in 0..MAX_TIMERS {
            script.push_str("setTimeout(function(){}, 1000);");
        }
        script.push_str("try { setTimeout(function(){}, 1000); __hodo_log('accepted'); } catch (e) { __hodo_log('rejected'); }");
        runtime.eval(&script).unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["rejected".to_string()]);
    }

    #[test]
    fn chrome_mv3_supports_both_styles_and_last_error() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval(
                "chrome.storage.local.set({k: 1}, function () { __hodo_log('cb:' + chrome.runtime.lastError); }); \
                 chrome.storage.local.get('missing', function (v) { __hodo_log('err:' + (chrome.runtime.lastError !== null)); });",
            )
            .unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec!["cb:null".to_string(), "err:false".to_string()]
        );
    }

    #[test]
    fn mv2_chrome_is_callback_only_and_browser_is_promise() {
        let config = RuntimeConfig {
            dialect: ApiDialect::ChromeMv2,
            allowed_namespaces: vec!["storage".to_string()],
            ..RuntimeConfig::default()
        };
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            Rc::new(SystemClock),
        );
        runtime
            .eval("__hodo_log(String(chrome.storage.local.set({k: 1})));")
            .unwrap();
        runtime.tick();
        assert_eq!(get(runtime.registry(), "k"), json!(1));
        assert!(trace(&mut runtime).iter().any(|line| line == "undefined"));
        runtime
            .eval("browser.storage.local.get('k').then(function (v) { __hodo_log('p:' + v); });")
            .unwrap();
        runtime.tick();
        assert!(trace(&mut runtime).iter().any(|line| line == "p:1"));
    }

    #[test]
    fn api_objects_are_frozen() {
        let mut runtime = ExtScriptRuntime::new();
        assert_eq!(runtime.eval("Object.isFrozen(chrome) && Object.isFrozen(browser) && Object.isFrozen(chrome.storage.local)").unwrap(), "true");
        runtime.eval("chrome.storage.local.set = null;").unwrap();
        assert_eq!(
            runtime.eval("typeof chrome.storage.local.set").unwrap(),
            "function"
        );
    }

    #[test]
    fn unhandled_rejection_is_logged_with_extension_name() {
        let config = RuntimeConfig {
            extension_name: "noisy-ext".to_string(),
            ..RuntimeConfig::default()
        };
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            Rc::new(SystemClock),
        );
        runtime.eval("Promise.reject(new Error('boom'));").unwrap();
        runtime.tick();
        assert!(
            trace(&mut runtime)
                .iter()
                .any(|line| line.contains("noisy-ext")
                    && line.contains("unhandled promise rejection"))
        );
    }

    #[test]
    fn eval_and_function_are_blocked_by_csp() {
        let mut runtime = ExtScriptRuntime::new();
        assert!(runtime.eval("eval('1+1')").is_err());
        assert!(runtime.eval("new Function('return 1')()").is_err());
    }

    #[test]
    fn message_response_timeout_uses_injected_clock() {
        let clock = Rc::new(FakeClock::new(0));
        let mut runtime = runtime_with(clock.clone(), &[]);
        runtime
            .eval(
                "chrome.runtime.onMessage.addListener(function () { return true; }); \
                 chrome.runtime.sendMessage('ping').then(function (r) { __hodo_log('r:' + r); });",
            )
            .unwrap();
        runtime.tick();
        assert!(trace(&mut runtime).is_empty());
        clock.set_ms(1000);
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["r:null".to_string()]);
    }

    #[test]
    fn fetch_response_helpers_expose_text_json_and_bytes() {
        let config = RuntimeConfig {
            allowed_namespaces: vec!["storage".to_string()],
            ..RuntimeConfig::default()
        };
        let clock: Rc<dyn Clock> = Rc::new(SystemClock);
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            clock,
        );
        runtime.set_fetch_hook(Rc::new(|call| match call {
            ApiCall::Fetch { .. } => ApiResult::Ok(json!({
                "ok": true,
                "status": 200,
                "body": "{\"answer\":42}"
            })),
            _ => ApiResult::Err("unexpected".into()),
        }));
        runtime
            .eval("fetch('https://example.test/a.json').then(function (r) { return r.json(); }).then(function (j) { __hodo_log('json:' + j.answer); });")
            .unwrap();
        runtime.tick();
        runtime
            .eval("fetch('https://example.test/a.json').then(function (r) { return r.arrayBuffer(); }).then(function (b) { __hodo_log('bytes:' + b.byteLength); });")
            .unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec!["json:42".to_string(), "bytes:13".to_string()]
        );
    }

    #[test]
    fn storage_on_changed_reports_old_and_new_values() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("chrome.storage.onChanged.addListener(function (changes, area) { __hodo_log(area + ':' + changes.k.oldValue + '->' + changes.k.newValue); });")
            .unwrap();
        runtime.tick();
        runtime.eval("chrome.storage.local.set({k: 7});").unwrap();
        runtime.tick();
        runtime.eval("chrome.storage.local.set({k: 8});").unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec!["local:null->7".to_string(), "local:7->8".to_string()]
        );
    }

    #[test]
    fn on_message_response_via_return_value_first_wins() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval(
                "chrome.runtime.onMessage.addListener(function (msg) { return 'reply'; }); \
                 chrome.runtime.onMessage.addListener(function (msg) { return 'second'; }); \
                 chrome.runtime.sendMessage('ping').then(function (r) { __hodo_log('r:' + r); });",
            )
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["r:reply".to_string()]);
    }

    #[test]
    fn on_message_response_via_promise() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval(
                "chrome.runtime.onMessage.addListener(function (msg) { return Promise.resolve('async-reply'); }); \
                 chrome.runtime.sendMessage('ping').then(function (r) { __hodo_log('r:' + r); });",
            )
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["r:async-reply".to_string()]);
    }

    #[test]
    fn on_message_response_via_send_response_async() {
        let clock = Rc::new(FakeClock::new(0));
        let mut runtime = runtime_with(clock.clone(), &[]);
        runtime
            .eval(
                "chrome.runtime.onMessage.addListener(function (msg, sender, sendResponse) { \
                   setTimeout(function () { sendResponse('late'); }, 2); return true; }); \
                 chrome.runtime.sendMessage('ping').then(function (r) { __hodo_log('r:' + r); });",
            )
            .unwrap();
        runtime.tick();
        assert!(trace(&mut runtime).is_empty());
        clock.set_ms(2);
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["r:late".to_string()]);
    }

    #[test]
    fn connect_creates_a_port_and_notifies_on_connect() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("chrome.runtime.onConnect.addListener(function (port) { __hodo_log('connected:' + port.name); }); var p = chrome.runtime.connect('popup'); p.onMessage.addListener(function (m) { __hodo_log('port:' + m); });")
            .unwrap();
        runtime.tick();
        runtime.fire_event("__portMessage:popup", json!("hello"));
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec!["connected:popup".to_string(), "port:hello".to_string()]
        );
    }

    #[test]
    fn alarms_fire_on_injected_clock() {
        let clock = Rc::new(FakeClock::new(0));
        let config = RuntimeConfig {
            allowed_namespaces: vec!["alarms".to_string()],
            ..RuntimeConfig::default()
        };
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            clock.clone(),
        );
        runtime
            .eval("chrome.alarms.onAlarm.addListener(function (alarm) { __hodo_log('alarm:' + alarm.name); }); chrome.alarms.create('wake', {delayMs: 3});")
            .unwrap();
        runtime.tick();
        clock.set_ms(3);
        runtime
            .state
            .registry
            .borrow_mut()
            .advance_clock(clock.now_ms());
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["alarm:wake".to_string()]);
    }

    #[test]
    fn session_storage_is_ram_backed() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("browser.storage.session.set({s: 1});")
            .unwrap();
        runtime.tick();
        assert!(get(runtime.registry(), "s").is_null());
        runtime
            .eval("browser.storage.session.get('s').then(function (v) { __hodo_log('s:' + v); });")
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["s:1".to_string()]);
    }

    #[test]
    fn huge_and_nested_values_are_rejected_cleanly() {
        let mut runtime = ExtScriptRuntime::new();
        let huge = "x".repeat(8 * 1024);
        let script = format!(
            "chrome.storage.local.set({{huge: '{}'}}); __hodo_log('done');",
            huge
        );
        runtime.eval(&script).unwrap();
        runtime.tick();
        assert!(trace(&mut runtime).iter().any(|line| line == "done"));
        let nested = Value::Array(vec![Value::Array(vec![json!({"k": 1})])]);
        runtime.fire_event("probe", nested);
        runtime.tick();
    }

    #[test]
    fn deeply_nested_source_is_rejected_before_boa_parses_it() {
        let mut runtime = ExtScriptRuntime::new();
        let mut source = String::from("var x = ");
        for _ in 0..(MAX_SOURCE_BRACKET_DEPTH + 20) {
            source.push('[');
        }
        source.push('1');
        for _ in 0..(MAX_SOURCE_BRACKET_DEPTH + 20) {
            source.push(']');
        }
        assert!(runtime.eval(&source).is_err());
        assert_eq!(runtime.eval("1+1").unwrap(), "2");
    }

    #[test]
    fn cyclic_message_is_rejected_without_panic() {
        let mut runtime = ExtScriptRuntime::new();
        runtime
            .eval("var cyclic = {}; cyclic.self = cyclic; browser.runtime.sendMessage(cyclic).catch(function () { __hodo_log('rejected'); });")
            .unwrap();
        runtime.tick();
        assert!(trace(&mut runtime).iter().any(|line| line == "rejected"));
    }
}

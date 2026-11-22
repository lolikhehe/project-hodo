//! JavaScript execution for WebExtensions: one dispatch table, two API styles.
//!
//! `chrome.*` and `browser.*` are thin frozen wrappers over the same native
//! task queue and [`hodo_ext_api::ApiRegistry`]. Every API call, timer, message
//! and event is a task; boa microtasks are drained after each task, giving
//! browser-like "microtask before next timer" ordering.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[cfg(feature = "js")]
use hodo_dom::NodeId;
use hodo_ext_api::{
    validate_api_value, ApiCall, ApiRegistry, ApiResult, Clock, ExtEvent, RuntimeConfig,
    SystemClock,
};
use serde_json::{json, Map, Value};

#[cfg(feature = "js")]
mod dom;
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

/// Read-only methods eligible for the per-tick memo.
///
/// Audited against `hodo-ext-api`'s `ApiCall` dispatch table (read-only): only
/// the three storage getters reach a native registry call today. The other four
/// names are frozen here for the ext-api integrator — `i18n.getMessage` is
/// still a pure JS stub and `runtime.getURL`/`getManifest`/`id` are not exposed
/// yet. Extending this list needs the matching dispatch-table/prelude entry.
/// Never add a write, event, DOM native or hook-backed method (in particular
/// `fetch`). Hook/fetch *implementations* are never cached: on a memo hit we
/// never reach them, while `fetch` is deliberately non-PURE so it always
/// reaches its side-effecting custom seam.
const PURE_METHODS: &[&str] = &[
    "storage.local.get",
    "storage.sync.get",
    "storage.session.get",
    "i18n.getMessage",
    "runtime.getURL",
    "runtime.getManifest",
    "runtime.id",
];
/// Hard bound on cached results per runtime (FIFO eviction).
const MEMO_CAP: usize = 128;

#[cfg(feature = "js")]
type FetchHook = Rc<dyn Fn(&ApiCall) -> ApiResult>;

/// Worker/broker seam: when installed, every dispatch goes through this hook
/// instead of the local registry. `None` keeps today's in-process behaviour.
#[cfg(feature = "js")]
type DispatchHook = Rc<dyn Fn(&ApiCall) -> ApiResult>;

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
    /// Page DOM event from the shell (`click`, `input`, `keydown`,
    /// `DOMContentLoaded`, ...), delivered during the next tick.
    PageEvent {
        node_id: NodeId,
        kind: String,
        payload: Value,
    },
}

/// Generic host-facing seam for message replies. The host decides how a reply
/// is correlated to a request; hodo-js only produces `(message, response)`.
#[cfg(feature = "js")]
type ReplyHook = Rc<dyn Fn(&Value, &Value)>;

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
    /// Isolated-worker forwarder; `None` means the registry handles calls.
    dispatch_hook: RefCell<Option<DispatchHook>>,
    /// `Some` only for page (popup) contexts.
    pub(crate) page: Option<dom::PageState>,
    /// Replies produced by `runtime.onMessage` listeners' `sendResponse`.
    message_replies: RefCell<Vec<(Value, Value)>>,
    reply_hook: RefCell<Option<ReplyHook>>,
    next_id: Cell<u64>,
    /// Incremented at every tick boundary and on every write post. The
    /// prelude's PURE-read memo compares it and drops stale entries lazily.
    memo_epoch: Cell<u64>,
    /// Writes posted but not yet dispatched: a synchronous PURE read would see
    /// stale state, so the prelude falls back to the normal task path.
    pending_writes: Cell<u32>,
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
            dispatch_hook: RefCell::new(None),
            page: None,
            message_replies: RefCell::new(Vec::new()),
            reply_hook: RefCell::new(None),
            next_id: Cell::new(1),
            memo_epoch: Cell::new(1),
            pending_writes: Cell::new(0),
        }
    }

    /// Bump the memo epoch visible to the prelude. Called at tick boundaries
    /// and when a write is posted, so every cached PURE read is dropped on the
    /// next lookup (one global write, no per-entry work in Rust).
    fn bump_memo_epoch(&self, context: &mut Context) {
        let epoch = self.memo_epoch.get().wrapping_add(1).max(1);
        self.memo_epoch.set(epoch);
        let _ = context.global_object().set(
            JsString::from("__hodoMemoEpoch"),
            JsValue::from(epoch as f64),
            false,
            context,
        );
    }

    /// Single dispatch path: hook first, then the specialized fetch hook, then
    /// the in-process registry. Identical to the old behaviour when both hooks
    /// are absent.
    fn dispatch_call(&self, call: &ApiCall) -> ApiResult {
        if let Some(hook) = self.dispatch_hook.borrow().as_ref() {
            return hook(call);
        }
        if matches!(call, ApiCall::Fetch { .. }) {
            if let Some(hook) = self.fetch_hook.borrow().as_ref() {
                return hook(call);
            }
            return ApiResult::Err("fetch: no host hook installed".to_string());
        }
        self.registry.borrow_mut().dispatch(call)
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

    /// Create a *page* (popup) context over `dom`. This is the only constructor
    /// that installs the page prelude; no extension API global is ever created.
    #[cfg(feature = "js")]
    #[must_use]
    pub fn new_page(dom: Rc<RefCell<hodo_dom::Dom>>) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let config = RuntimeConfig::default();
        let registry = Rc::new(RefCell::new(ApiRegistry::new()));
        let mut state_inner =
            RuntimeState::new(Rc::clone(&registry), Rc::new(SystemClock), config.clone());
        state_inner.page = Some(dom::PageState::new(dom));
        let state = Rc::new(state_inner);
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
        dom::register_natives(&mut context, id);
        let prelude = prelude::build_page_prelude();
        context
            .eval(Source::from_bytes(prelude.as_str()))
            .expect("the page prelude is valid JavaScript");
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
        // One drain = one tick. Invalidate after the drain so nothing cached
        // during this tick leaks into the next one, and clear the write guard
        // now that every posted task has been dispatched.
        self.state.bump_memo_epoch(&mut self.context);
        self.state.pending_writes.set(0);
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

    /// Install (or clear with `None`) the isolated-worker dispatch hook. While
    /// set, every `chrome.*` / `browser.*` call is forwarded to it instead of
    /// the local registry. `None` restores the in-process path.
    #[cfg(feature = "js")]
    pub fn set_dispatch_hook(&mut self, hook: Option<DispatchHook>) {
        *self.state.dispatch_hook.borrow_mut() = hook;
    }

    /// Non-JS builds have no dispatch hook.
    #[cfg(not(feature = "js"))]
    pub fn set_dispatch_hook(&mut self, _hook: Option<Rc<dyn Fn(&ApiCall) -> ApiResult>>) {}

    /// Enqueue an externally-originated event (e.g. an extension message or an
    /// alarm fired by the host) for delivery during the next tick.
    #[cfg(feature = "js")]
    pub fn fire_event(&mut self, name: &str, payload: Value) {
        // `runtime.onMessage` is the one event whose listeners receive a
        // `sendResponse` callback; route it through the listener-aware,
        // reply-capable dispatcher instead of the generic event fan-out.
        let task = if name == "runtime.onMessage" {
            Task::Event {
                name: "__hodoFireMessage".to_string(),
                args: vec![payload],
            }
        } else {
            Task::Event {
                name: "__hodoFire".to_string(),
                args: vec![json!(name), payload],
            }
        };
        self.state.queue.borrow_mut().push_back(task);
    }

    /// Non-JS builds have no event loop.
    #[cfg(not(feature = "js"))]
    pub fn fire_event(&mut self, _name: &str, _payload: Value) {}

    /// Deliver a DOM event from the shell to listeners registered on `node_id`.
    /// The event is queued and invoked on the next [`Self::tick`].
    #[cfg(feature = "js")]
    pub fn dispatch_page_event(&mut self, node_id: NodeId, kind: &str, payload: Value) {
        self.state.queue.borrow_mut().push_back(Task::PageEvent {
            node_id,
            kind: kind.to_string(),
            payload,
        });
    }

    /// Install a host callback for message replies. Keeping this generic lets
    /// the host own request/reply correlation without exposing its queue here.
    #[cfg(feature = "js")]
    pub fn set_message_reply_hook(&mut self, hook: Option<ReplyHook>) {
        *self.state.reply_hook.borrow_mut() = hook;
    }

    /// Non-JS builds have no message replies.
    #[cfg(not(feature = "js"))]
    pub fn set_message_reply_hook(&mut self, _hook: Option<Rc<dyn Fn(&Value, &Value)>>) {}

    /// Drain the `(message, response)` pairs produced by `sendResponse`.
    #[cfg(feature = "js")]
    pub fn take_message_replies(&mut self) -> Vec<(Value, Value)> {
        std::mem::take(&mut self.state.message_replies.borrow_mut())
    }

    /// Non-JS builds have no message replies.
    #[cfg(not(feature = "js"))]
    pub fn take_message_replies(&mut self) -> Vec<(Value, Value)> {
        Vec::new()
    }

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
        dom::drop_page(&self.state);
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
                    let result = self.state.dispatch_call(&call);
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
                                .dispatch_call(&ApiCall::StorageGet { key: key.clone() })
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
            Task::PageEvent {
                node_id,
                kind,
                payload,
            } => self.fire_page_event(node_id, &kind, &payload),
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

    fn fire_page_event(&mut self, node_id: NodeId, kind: &str, payload: &Value) {
        let Some(page) = self.state.page.as_ref() else {
            return;
        };
        let callbacks: Vec<u64> = page
            .listeners
            .borrow()
            .iter()
            .filter(|listener| listener.node == node_id && listener.kind == kind)
            .map(|listener| listener.callback_id)
            .collect();
        if callbacks.is_empty() {
            return;
        }
        let value = match JsValue::from_json(payload, &mut self.context) {
            Ok(value) => value,
            Err(err) => {
                self.state
                    .log(format!("page event payload conversion failed: {err}"));
                return;
            }
        };
        for callback_id in callbacks {
            let key = PropertyKey::from(JsString::from(format!("__hodoListener_{callback_id}")));
            let callback = match self.context.global_object().get(key, &mut self.context) {
                Ok(callback) => callback,
                Err(err) => {
                    self.state
                        .log(format!("page listener lookup failed: {err}"));
                    continue;
                }
            };
            if let Some(function) = callback.as_callable() {
                if let Err(err) = function.call(
                    &JsValue::undefined(),
                    std::slice::from_ref(&value),
                    &mut self.context,
                ) {
                    self.state
                        .log(format!("page listener: {}", error_message(&err)));
                }
            }
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
fn pure_method(name: &str) -> Option<&'static str> {
    PURE_METHODS.iter().copied().find(|method| *method == name)
}

#[cfg(feature = "js")]
fn make_calls(
    name: &str,
    args: &[Value],
    now_ms: u64,
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
                now_ms,
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
        // PURE reads for the ext-api integrator: the prelude's __hodoApiCall
        // memoizes every name in PURE_METHODS automatically once it reaches
        // this table. None of these dispatch to a write or a host hook.
        "i18n.getMessage" => {
            let key = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "i18n.getMessage expects a key".to_string())?
                .to_string();
            let substitutions = args
                .get(1)
                .and_then(Value::as_array)
                .map(|items| items.iter().map(string_value).collect())
                .unwrap_or_default();
            calls.push(ApiCall::I18nGetMessage { key, substitutions });
        }
        "runtime.getURL" => {
            let path = args
                .first()
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            calls.push(ApiCall::RuntimeGetUrl { path });
        }
        "runtime.getManifest" => calls.push(ApiCall::RuntimeGetManifest),
        "runtime.id" => calls.push(ApiCall::RuntimeId),
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

/// Translate one wire `ApiCall` (`method` + `params`) through the same
/// `make_calls` table the JS wrappers use. `params` may be a positional array
/// or a single value. Multi-key `storage.*.get` has no single-call wire form
/// and therefore yields no calls.
///
/// # Errors
/// Returns the same message as an unknown or malformed JS API call.
#[cfg(feature = "js")]
pub fn decode_api_call(method: &str, params: Value) -> Result<Vec<ApiCall>, String> {
    let args: Vec<Value> = match params {
        Value::Array(items) => items,
        other => vec![other],
    };
    let (calls, _merge_keys) = make_calls(method, &args, SystemClock.now_ms())?;
    Ok(calls)
}

/// Non-JS builds have no dispatch table.
#[cfg(not(feature = "js"))]
pub fn decode_api_call(_method: &str, _params: Value) -> Result<Vec<ApiCall>, String> {
    Err("js engine disabled".to_string())
}

#[cfg(feature = "js")]
fn string_value(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

/// Synchronous result for a PURE read, used by the prelude to fill its memo
/// before the next JS statement runs. Returns `undefined` when a write is
/// pending or the call cannot be dispatched; the prelude then falls back to
/// the ordinary task queue, keeping error and ordering behaviour unchanged.
#[cfg(feature = "js")]
fn op_pure_read(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        // A host reply hook means the host owns call/message routing (the
        // isolated worker). A local sync read would be stale and would bypass
        // the host's wire entirely, so fall through to __hodoPost; the memo is
        // still filled from the eventual settle result.
        if state.reply_hook.borrow().is_some() || state.pending_writes.get() > 0 {
            return Ok(JsValue::undefined());
        }
        let name = string_arg(args, 0, ctx).unwrap_or_default();
        if pure_method(&name).is_none() {
            return Ok(JsValue::undefined());
        }
        let values = json_arg(args, 1, ctx).unwrap_or(Value::Null);
        let Some(list) = values.as_array() else {
            return Ok(JsValue::undefined());
        };
        let Ok((calls, merge_keys)) = make_calls(&name, list, state.clock.now_ms()) else {
            return Ok(JsValue::undefined());
        };
        let mut last = Value::Null;
        for call in &calls {
            match state.dispatch_call(call) {
                ApiResult::Ok(value) => last = value,
                ApiResult::Err(_) => return Ok(JsValue::undefined()),
            }
        }
        if let Some(keys) = &merge_keys {
            let mut map = Map::new();
            for key in keys {
                map.insert(
                    key.clone(),
                    state
                        .dispatch_call(&ApiCall::StorageGet { key: key.clone() })
                        .into_ok_or_null(),
                );
            }
            last = Value::Object(map);
        }
        JsValue::from_json(&last, ctx).map_err(|err| js_error(err.to_string()))
    })
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
            make_calls(&name, &list, state.clock.now_ms())
                .map(|(calls, merge_keys)| (name, calls, merge_keys))
        });
        let call_id = state.fresh_id();
        // Any non-PURE post is a write or a side-effecting/host-hook path.
        // Invalidate the whole PURE memo immediately, before a later read in
        // the same JS turn can observe a stale value.
        let pure = match &prepared {
            Ok((name, _, _)) => pure_method(name).is_some(),
            Err(_) => false,
        };
        if !pure {
            state.bump_memo_epoch(ctx);
            state
                .pending_writes
                .set(state.pending_writes.get().saturating_add(1));
        }
        let task = match prepared {
            Ok((_, calls, merge_keys)) => Task::Api {
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
fn op_message_reply(id: u64) -> NativeFunction {
    NativeFunction::from_copy_closure(move |_this, args, ctx| {
        let state = with_state(id, Rc::clone).map_err(js_error)?;
        let message = json_arg(args, 0, ctx).map_err(js_error)?;
        let response = json_arg(args, 1, ctx).map_err(js_error)?;
        validate_api_value(&response).map_err(js_error)?;
        state
            .message_replies
            .borrow_mut()
            .push((message.clone(), response.clone()));
        if let Some(hook) = state.reply_hook.borrow().as_ref() {
            hook(&message, &response);
        }
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
    register(context, "__hodo_read_pure", 2, op_pure_read(id));
    register(context, "__hodo_set_timeout", 3, op_timer(id, false));
    register(context, "__hodo_set_interval", 3, op_timer(id, true));
    register(context, "__hodo_clear_timer", 1, op_clear_timer(id));
    register(context, "__hodo_log", 1, op_log(id));
    register(context, "__hodo_message_reply", 2, op_message_reply(id));
}

#[cfg(test)]
#[cfg(feature = "js")]
mod tests {
    use super::*;
    use hodo_ext_api::{ApiDialect, FakeClock};
    use serde_json::json;
    use std::time::Instant;

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

    /// A host-owned (isolated worker) context has no authoritative local state:
    /// a PURE read must travel through `__hodoPost` so the wire shim can emit
    /// the broker call, not short-circuit on the shadow registry's local null.
    #[test]
    fn worker_style_context_routes_pure_reads_through_post() {
        let config = RuntimeConfig {
            dialect: hodo_ext_api::ApiDialect::ChromeMv3,
            allowed_namespaces: vec!["storage".into(), "tabs".into()],
            ..RuntimeConfig::default()
        };
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::new(RefCell::new(ApiRegistry::new())),
            Rc::new(SystemClock),
        );
        let wires = Rc::new(RefCell::new(Vec::new()));
        let seen = Rc::clone(&wires);
        runtime.set_message_reply_hook(Some(Rc::new(move |message, _| {
            seen.borrow_mut().push(message.clone());
        })));
        runtime
            .eval(
                "var __hodo_native_post = __hodoPost; \
                 __hodoPost = function (name, args) { \
                   __hodo_message_reply(JSON.stringify({ name: name, args: args }), null); \
                   return 1; \
                 };",
            )
            .unwrap();
        runtime
            .eval("chrome.storage.local.get('k').then(function (v) { __hodo_log('got:' + v.k); });")
            .unwrap();
        let wires = wires.borrow();
        assert_eq!(wires.len(), 1, "no wire registration for the PURE read");
        let registration: Value =
            serde_json::from_str(wires[0].as_str().expect("wire text")).expect("wire JSON");
        assert_eq!(registration["name"], json!("storage.local.get"));
        assert_eq!(registration["args"], json!(["k"]));
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
    fn pure_runtime_and_i18n_reads_use_dispatch_table() {
        use hodo_ext_api::RuntimeInfo;
        let registry = Rc::new(RefCell::new(ApiRegistry::new()));
        registry.borrow_mut().set_runtime_info(Rc::new(RuntimeInfo {
            id: "ext-123".into(),
            url_base: "chrome-extension://ext-123".into(),
            manifest: json!({ "name": "Demo", "version": "1.0" }),
            messages: None,
        }));
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            RuntimeConfig {
                dialect: ApiDialect::ChromeMv3,
                ..RuntimeConfig::default()
            },
            registry,
            Rc::new(SystemClock),
        );
        runtime
            .eval(
                "Promise.all([chrome.runtime.id, chrome.runtime.getURL('popup.html'), \
                 chrome.runtime.getManifest(), chrome.i18n.getMessage('missing')]) \
                 .then(function (r) { __hodo_log(JSON.stringify(r)); });",
            )
            .unwrap();
        runtime.tick();
        assert_eq!(
            trace(&mut runtime),
            vec![
                r#"["ext-123","chrome-extension://ext-123/popup.html",{"name":"Demo","version":"1.0"},""]"#
                    .to_string()
            ]
        );
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

    #[test]
    fn dispatch_hook_forwards_calls_and_can_be_cleared() {
        let registry = Rc::new(RefCell::new(ApiRegistry::new()));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let seen_hook = Rc::clone(&seen);
        let mut runtime =
            ExtScriptRuntime::with_allowed_and_registry(&["storage"], Rc::clone(&registry));
        runtime.set_dispatch_hook(Some(Rc::new(move |call| {
            seen_hook.borrow_mut().push(format!("{call:?}"));
            ApiResult::Ok(json!("hooked"))
        })));
        runtime
            .eval("browser.storage.local.set('k', 1).then(function (v) { __hodo_log(v); });")
            .unwrap();
        runtime.tick();
        assert_eq!(trace(&mut runtime), vec!["hooked".to_string()]);
        assert_eq!(seen.borrow().len(), 1);
        assert!(get(&registry, "k").is_null());

        runtime.set_dispatch_hook(None);
        runtime.eval("browser.storage.local.set('k', 2);").unwrap();
        runtime.tick();
        assert_eq!(get(&registry, "k"), json!(2));
    }

    #[test]
    fn decode_api_call_uses_existing_dispatch_table() {
        let calls = decode_api_call("storage.local.set", json!({"k": 1})).unwrap();
        assert!(matches!(
            &calls[0],
            ApiCall::StorageSet { key, value } if key == "k" && *value == json!(1)
        ));
        assert!(decode_api_call("nope", json!([])).is_err());
    }

    #[test]
    fn fire_message_reply_roundtrip_with_mock_responder() {
        let mut runtime = ExtScriptRuntime::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let seen_hook = Rc::clone(&seen);
        runtime.set_message_reply_hook(Some(Rc::new(move |message, response| {
            seen_hook
                .borrow_mut()
                .push((message.clone(), response.clone()));
        })));
        runtime
            .eval("chrome.runtime.onMessage.addListener(function (m, sender, reply) { reply('pong:' + m); });")
            .unwrap();
        runtime.tick();
        runtime.fire_event("runtime.onMessage", json!("ping"));
        runtime.tick();
        let expected = vec![(json!("ping"), json!("pong:ping"))];
        assert_eq!(*seen.borrow(), expected);
        assert_eq!(runtime.take_message_replies(), expected);
        assert!(runtime.take_message_replies().is_empty());
    }

    #[test]
    fn fire_message_without_listener_produces_no_reply() {
        let mut runtime = ExtScriptRuntime::new();
        runtime.fire_event("runtime.onMessage", json!("ping"));
        runtime.tick();
        assert!(runtime.take_message_replies().is_empty());
    }

    // ---- per-tick PURE memo -------------------------------------------------

    fn counting_storage_hook(
        calls: Rc<RefCell<u32>>,
        store: Rc<RefCell<HashMap<String, Value>>>,
    ) -> DispatchHook {
        Rc::new(move |call| {
            *calls.borrow_mut() += 1;
            match call {
                ApiCall::StorageGet { key } => {
                    ApiResult::Ok(store.borrow().get(key).cloned().unwrap_or(Value::Null))
                }
                ApiCall::StorageSet { key, value } => {
                    store.borrow_mut().insert(key.clone(), value.clone());
                    ApiResult::Ok(Value::Null)
                }
                _ => ApiResult::Ok(Value::Null),
            }
        })
    }

    #[test]
    fn memo_same_key_repeats_only_one_underlying_dispatch() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        store.borrow_mut().insert("k".to_string(), json!("v"));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        runtime
            .eval("var out=[]; for (var i=0;i<10;i++) browser.storage.local.get('k').then(function(v){ out.push(v); });")
            .unwrap();
        assert_eq!(*calls.borrow(), 1);
        runtime.eval("__hodo_log(out.join('|'));").unwrap();
        assert_eq!(trace(&mut runtime), vec!["v|v|v|v|v|v|v|v|v|v".to_string()]);
    }

    #[test]
    fn memo_write_invalidates_before_later_reads_in_the_same_tick() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        runtime
            .eval(
                "browser.storage.local.get('k').then(function(v){__hodo_log('a:'+v);}); \
                   browser.storage.local.set('k',2); \
                   browser.storage.local.get('k').then(function(v){__hodo_log('b:'+v);});",
            )
            .unwrap();
        assert_eq!(*calls.borrow(), 3);
        assert_eq!(
            trace(&mut runtime),
            vec!["a:null".to_string(), "b:2".to_string()]
        );
    }

    #[test]
    fn memo_is_invalidated_at_each_tick_boundary() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        store.borrow_mut().insert("k".to_string(), json!(1));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        for _ in 0..3 {
            runtime.eval("browser.storage.local.get('k');").unwrap();
        }
        assert_eq!(*calls.borrow(), 3);
    }

    #[test]
    fn memo_distinct_params_are_distinct_entries() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        runtime
            .eval(
                "browser.storage.local.get('a'); browser.storage.local.get('b'); \
                   browser.storage.local.get('a'); browser.storage.local.get('b');",
            )
            .unwrap();
        assert_eq!(*calls.borrow(), 2);
    }

    #[test]
    fn memo_is_bounded_and_evicts_oldest_entries() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        let mut script = String::new();
        for i in 0..=MEMO_CAP {
            script.push_str(&format!("browser.storage.local.get('k{i}');"));
        }
        // k0 is evicted once the 129th distinct key arrives; k128 survives.
        script.push_str("browser.storage.local.get('k0'); browser.storage.local.get('k128');");
        runtime.eval(&script).unwrap();
        assert_eq!(*calls.borrow() as usize, MEMO_CAP + 2);
    }

    #[test]
    fn page_runtimes_have_independent_memos_and_dom_reads_stay_uncached() {
        let mut a = ExtScriptRuntime::new_page(Rc::new(RefCell::new(hodo_dom::Dom::new())));
        let mut b = ExtScriptRuntime::new_page(Rc::new(RefCell::new(hodo_dom::Dom::new())));
        assert_eq!(
            a.eval("__hodoMemoPut('x','A'); __hodoMemoGet('x').value")
                .unwrap(),
            "A"
        );
        assert_eq!(
            b.eval("__hodoMemoGet('x') === __hodoMemoMiss").unwrap(),
            "true"
        );
        assert_eq!(
            a.eval(
                "__hodoMemoPut('x','A'); __hodoMemoPut('y','B'); \
                 Object.keys(__hodoMemo.entries).length"
            )
            .unwrap(),
            "2"
        );
        assert_eq!(
            b.eval("Object.keys(__hodoMemo.entries).length").unwrap(),
            "0"
        );

        // DOM natives are separate ops, never routed through the memo: a
        // write is visible to the very next read in the same page context.
        let dom = Rc::new(RefCell::new(hodo_dom::Dom::new()));
        let mut page = ExtScriptRuntime::new_page(Rc::clone(&dom));
        page.eval("var el = document.createElement('div'); el.setAttribute('x','1');")
            .unwrap();
        assert_eq!(page.eval("el.getAttribute('x')").unwrap(), "1");
        page.eval("el.setAttribute('x','2');").unwrap();
        assert_eq!(page.eval("el.getAttribute('x')").unwrap(), "2");
    }

    #[test]
    fn fetch_hook_path_bypasses_the_memo() {
        let calls = Rc::new(RefCell::new(0));
        let counter = Rc::clone(&calls);
        let mut runtime = ExtScriptRuntime::new();
        runtime.set_fetch_hook(Rc::new(move |_call| {
            *counter.borrow_mut() += 1;
            ApiResult::Ok(json!({ "ok": true, "status": 200, "body": "x" }))
        }));
        runtime.eval("fetch('/x'); fetch('/x');").unwrap();
        assert_eq!(*calls.borrow(), 2);
    }

    #[test]
    fn memo_perf_ab_1000_reads() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        runtime
            .eval("browser.storage.local.set('bench', 7);")
            .unwrap();
        runtime.tick();
        *calls.borrow_mut() = 0;
        let script = "var sum=0; for (var i=0;i<1000;i++) { \
                      browser.storage.local.get('bench').then(function(v){ sum+=v; }); }";
        runtime
            .eval("var __hodoPureSaved = __hodoPure; __hodoPure = {};")
            .unwrap();
        let start = Instant::now();
        runtime.context.eval(Source::from_bytes(script)).unwrap();
        let off_js = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let start = Instant::now();
        runtime.tick();
        let off_tick = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let off = off_js + off_tick;
        let off_calls = *calls.borrow();
        *calls.borrow_mut() = 0;
        runtime.eval("__hodoPure = __hodoPureSaved;").unwrap();
        let start = Instant::now();
        runtime.context.eval(Source::from_bytes(script)).unwrap();
        let on_js = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let start = Instant::now();
        runtime.tick();
        let on_tick = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let on = on_js + on_tick;
        let on_calls = *calls.borrow();
        eprintln!(
            "memo A/B (1000 storage.get): off={off:.3} us/call (js={off_js:.3}, tick={off_tick:.3}, \
             {off_calls} dispatches), on={on:.3} us/call (js={on_js:.3}, tick={on_tick:.3}, \
             {on_calls} dispatches), speedup={:.2}x",
            off / on
        );
        assert!(on < off, "memo path must be faster than full dispatch");
    }

    #[test]
    fn memo_hit_preserves_callback_style_invocation() {
        let calls = Rc::new(RefCell::new(0));
        let store = Rc::new(RefCell::new(HashMap::new()));
        store.borrow_mut().insert("k".to_string(), json!("v"));
        let mut runtime = ExtScriptRuntime::with_allowed(&["storage"]);
        runtime.set_dispatch_hook(Some(counting_storage_hook(
            Rc::clone(&calls),
            Rc::clone(&store),
        )));
        runtime
            .eval("var n=0; chrome.storage.local.get('k', function (v) { n++; __hodo_log('a:'+v); }); \
                   chrome.storage.local.get('k', function (v) { n++; __hodo_log('b:'+v); });")
            .unwrap();
        assert_eq!(*calls.borrow(), 1);
        runtime.eval("__hodo_log('n:'+n);").unwrap();
        assert_eq!(
            trace(&mut runtime),
            vec!["a:v".to_string(), "b:v".to_string(), "n:2".to_string()]
        );
    }

    #[test]
    fn memo_no_op_tick_invalidation_cost() {
        let mut runtime = ExtScriptRuntime::new();
        for _ in 0..10 {
            runtime.tick();
        }
        let start = Instant::now();
        for _ in 0..1000 {
            runtime.tick();
        }
        let tick = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let start = Instant::now();
        for _ in 0..1000 {
            runtime.state.bump_memo_epoch(&mut runtime.context);
        }
        let epoch = start.elapsed().as_secs_f64() * 1e6 / 1000.0;
        let populate = format!("for (var i=0;i<{MEMO_CAP};i++) __hodoMemoPut('k'+i, i);");
        runtime.context.eval(Source::from_bytes(&populate)).unwrap();
        let start = Instant::now();
        runtime
            .context
            .eval(Source::from_bytes("__hodoMemoSync();"))
            .unwrap();
        let lazy_clear = start.elapsed().as_secs_f64() * 1e6;
        eprintln!(
            "no-op tick={tick:.3} us, epoch bump={epoch:.3} us, \
             lazy clear of {MEMO_CAP} entries={lazy_clear:.3} us"
        );
    }
}

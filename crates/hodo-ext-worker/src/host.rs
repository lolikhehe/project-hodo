//! Isolated worker JS context: scripts run locally, every API call travels to
//! the broker, and the matching `ApiResponse` settles the JS Promise that
//! originated it. The shadow registry only answers broker-initiated calls and
//! mirrors committed state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Stdout, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

use hodo_ext_api::{ApiCall, ApiRegistry, ApiResult, RuntimeConfig, SystemClock};
use hodo_ext_ipc::{encode, Message, MAX_OUTSTANDING_CALLS};
use hodo_js::{decode_api_call, ExtScriptRuntime};
use serde::Deserialize;
use serde_json::{json, Value};

const CALL_TIMEOUT: Duration = Duration::from_secs(5);
const WIRE_MARKER: u64 = 1;

/// JS → Rust call registration emitted by the `__hodoPost` shim below.
#[derive(Debug, Deserialize)]
struct WireRegistration {
    #[serde(rename = "__hodoWire")]
    wire: u64,
    id: u64,
    method: String,
    params: Value,
}

/// Rebind the prelude's post helper so the JS-side promise id *is* the wire id.
/// `__hodoState.pending[id]` is then keyed exactly like the broker's replies,
/// so settling is a single `__hodoSettle(id, ...)` eval — no hodo-js change.
const WIRE_SHIM: &str = r#"
var __hodo_native_post = __hodoPost;
var __hodo_wire_next = 1;
__hodoPost = function (name, args) {
  var id = __hodo_wire_next++;
  try {
    __hodo_message_reply(JSON.stringify({ __hodoWire: 1, id: id, method: name, params: args }), null);
    return id;
  } catch (e) {
    return __hodo_native_post(name, args);
  }
};
"#;

struct PendingCall {
    calls: Vec<ApiCall>,
    deadline: Instant,
}

#[derive(Default)]
struct PendingState {
    calls: HashMap<u64, PendingCall>,
}

impl PendingState {
    /// Reject a duplicate id or a call beyond the protocol's 256-call cap.
    fn insert(&mut self, id: u64, calls: Vec<ApiCall>) -> bool {
        if id == 0 || self.calls.contains_key(&id) || self.calls.len() >= MAX_OUTSTANDING_CALLS {
            return false;
        }
        self.calls.insert(
            id,
            PendingCall {
                calls,
                deadline: Instant::now() + CALL_TIMEOUT,
            },
        );
        true
    }
}

pub struct WorkerHost {
    runtime: ExtScriptRuntime,
    shadow: Rc<RefCell<ApiRegistry>>,
    out: Rc<RefCell<Stdout>>,
    pending: Rc<RefCell<PendingState>>,
}

impl WorkerHost {
    #[must_use]
    pub fn new(config: RuntimeConfig, out: Rc<RefCell<Stdout>>) -> Self {
        let shadow = Rc::new(RefCell::new(ApiRegistry::new()));
        let mut runtime = ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::clone(&shadow),
            Rc::new(SystemClock),
        );

        let pending = Rc::new(RefCell::new(PendingState::default()));
        let hook_out = Rc::clone(&out);
        let hook_pending = Rc::clone(&pending);
        runtime.set_message_reply_hook(Some(Rc::new(move |message, _| {
            let Some(text) = message.as_str() else { return };
            let Ok(reg) = serde_json::from_str::<WireRegistration>(text) else {
                return;
            };
            if reg.wire != WIRE_MARKER || reg.id == 0 {
                return; // a genuine sendResponse reply, not a call registration
            }
            let calls = decode_api_call(&reg.method, reg.params.clone()).unwrap_or_default();
            if !hook_pending.borrow_mut().insert(reg.id, calls) {
                return;
            }
            write_frame(
                &hook_out,
                &Message::ApiCall {
                    id: reg.id,
                    method: reg.method,
                    params: reg.params,
                },
            );
        })));
        if let Err(err) = runtime.eval(WIRE_SHIM) {
            eprintln!("hodo-ext-worker: wire shim: {err}");
        }
        Self {
            runtime,
            shadow,
            out,
            pending,
        }
    }

    /// Mirror the in-process host: join script sources with `\n` and eval once.
    /// A throw is fatal — the broker restarts (then disables) a crashing worker.
    pub fn eval_scripts(&mut self, sources: &[String]) -> Result<(), String> {
        if sources.is_empty() {
            return Ok(());
        }
        let result = self.runtime.eval(&sources.join("\n")).map(|_| ());
        self.drain_logs();
        result
    }

    /// Drive timers/microtasks and reject calls that outlived `CALL_TIMEOUT`.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let expired: Vec<u64> = self
            .pending
            .borrow()
            .calls
            .iter()
            .filter(|(_, pending)| pending.deadline <= now)
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            self.pending.borrow_mut().calls.remove(&id);
            let message = format!(
                "api call {id} timed out after {}ms",
                CALL_TIMEOUT.as_millis()
            );
            self.settle_js(id, false, &Value::Null, &message);
            self.log(message);
        }
        self.runtime.tick();
        self.drain_logs();
    }

    /// Deliver a broker event into the JS runtime (settled on the next tick).
    pub fn fire_event(&mut self, name: &str, payload: Value) {
        self.runtime.fire_event(name, payload);
    }

    /// Send a diagnostic line through the broker's log channel.
    pub fn log(&self, line: String) {
        write_frame(
            &self.out,
            &Message::Event {
                name: "__hodo_log".to_string(),
                payload: json!({ "message": line }),
            },
        );
    }

    /// Settle the JS Promise whose id matches a broker `ApiResponse`.
    pub fn on_api_response(&mut self, id: u64, result: Option<Value>, error: Option<String>) {
        let removed = self.pending.borrow_mut().calls.remove(&id);
        let Some(pending) = removed else {
            self.log(format!("unsolicited api response {id}"));
            return;
        };
        match result {
            Some(value) => {
                // Keep the local mirror in sync so broker-initiated calls see
                // the state the script just committed through the broker.
                {
                    let mut shadow = self.shadow.borrow_mut();
                    for call in &pending.calls {
                        let _ = shadow.dispatch(call);
                    }
                }
                self.settle_js(id, true, &value, "");
            }
            None => {
                let err = error.unwrap_or_else(|| "api error".to_string());
                self.settle_js(id, false, &Value::Null, &err);
            }
        }
        self.tick();
    }

    /// Answer a broker-initiated `ApiCall` through the real dispatch table.
    pub fn answer_call(&self, id: u64, method: &str, params: Value) {
        let reply = match decode_api_call(method, params) {
            Ok(calls) => {
                let mut shadow = self.shadow.borrow_mut();
                let mut last = Value::Null;
                let mut error = None;
                for call in calls {
                    match shadow.dispatch(&call) {
                        ApiResult::Ok(value) => last = value,
                        ApiResult::Err(err) => {
                            error = Some(err);
                            break;
                        }
                    }
                }
                match error {
                    Some(err) => Message::ApiResponse {
                        id,
                        result: None,
                        error: Some(err),
                    },
                    None => Message::ApiResponse {
                        id,
                        result: Some(last),
                        error: None,
                    },
                }
            }
            Err(err) => Message::ApiResponse {
                id,
                result: None,
                error: Some(err),
            },
        };
        write_frame(&self.out, &reply);
    }

    fn settle_js(&mut self, id: u64, ok: bool, value: &Value, error: &str) {
        let value = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
        let error = serde_json::to_string(error).unwrap_or_else(|_| "\"api error\"".to_string());
        let code = format!("__hodoSettle({id}, {ok}, {value}, {error});");
        if let Err(err) = self.runtime.eval(&code) {
            self.log(format!("settle {id}: {err}"));
        }
    }

    fn drain_logs(&mut self) {
        for line in self.runtime.take_logs() {
            self.log(line);
        }
    }
}

fn write_frame(out: &Rc<RefCell<Stdout>>, msg: &Message) {
    let mut out = out.borrow_mut();
    let _ = out.write_all(&encode(msg));
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_calls_are_capped_at_the_protocol_limit() {
        let mut state = PendingState::default();
        for id in 1..=MAX_OUTSTANDING_CALLS as u64 {
            assert!(state.insert(id, Vec::new()));
        }
        assert_eq!(state.calls.len(), MAX_OUTSTANDING_CALLS);
        assert!(!state.insert(0, Vec::new()));
        assert!(!state.insert(1, Vec::new()));
        assert!(!state.insert(MAX_OUTSTANDING_CALLS as u64 + 1, Vec::new()));
    }

    #[test]
    fn smoke_new() {
        let out = Rc::new(RefCell::new(std::io::stdout()));
        let _host = WorkerHost::new(RuntimeConfig::default(), out);
    }

    #[test]
    fn registration_shape_decodes() {
        let text = r#"{"__hodoWire":1,"id":9,"method":"storage.local.set","params":[{"k":1}]}"#;
        let reg: WireRegistration = serde_json::from_str(text).unwrap();
        assert_eq!(reg.id, 9);
        assert_eq!(reg.method, "storage.local.set");
        assert_eq!(decode_api_call(&reg.method, reg.params).unwrap().len(), 1);
    }
}

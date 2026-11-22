//! Isolated extension worker: inherited stdio, no argv secrets, no JS before
//! the broker's `__hodo_ready` event.

mod host;
mod sandbox;

use std::cell::RefCell;
use std::env;
use std::io::{self, Write};
use std::process::exit;
use std::rc::Rc;

use hodo_ext_api::{ApiDialect, RuntimeConfig};
use hodo_ext_ipc::{encode, FrameReader, Message};
use host::WorkerHost;
use serde::Deserialize;
use serde_json::{json, Map, Value};

type Out = Rc<RefCell<io::Stdout>>;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ReadyConfig {
    extension_name: String,
    idle_unload_ms: Option<u64>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReadyDialect {
    ChromeMv2,
    #[default]
    ChromeMv3,
    Gecko,
}

impl From<ReadyDialect> for ApiDialect {
    fn from(value: ReadyDialect) -> Self {
        match value {
            ReadyDialect::ChromeMv2 => Self::ChromeMv2,
            ReadyDialect::ChromeMv3 => Self::ChromeMv3,
            ReadyDialect::Gecko => Self::Gecko,
        }
    }
}

/// Payload of the broker's `Event { name: "__hodo_ready" }`.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ReadyPayload {
    config: ReadyConfig,
    allowed_namespaces: Vec<String>,
    host_permissions: Vec<String>,
    dialect: ReadyDialect,
    /// Source of the background scripts, in the host's eval order.
    background_scripts: Vec<String>,
}

fn main() {
    // Boa's parser/context build recurses deeper than a spawned process's
    // default main-thread stack on Windows. All work (including the sandbox)
    // stays on this thread; the starter thread only joins.
    let worker = std::thread::Builder::new()
        .name("hodo-ext-worker-js".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(run)
        .expect("spawn worker js thread");
    if worker.join().is_err() {
        exit(1);
    }
}

fn run() {
    let token = match env::var("HODO_EXT_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("hodo-ext-worker: HODO_EXT_TOKEN is missing");
            exit(2);
        }
    };
    let manifest_digest = match manifest_digest_arg() {
        Some(digest) => digest,
        None => {
            eprintln!("hodo-ext-worker: --manifest-digest <hex> is required");
            exit(2);
        }
    };

    let out: Out = Rc::new(RefCell::new(io::stdout()));
    let log = |line: String| write_event(&out, "__hodo_log", json!({ "message": line }));

    if let Err(err) = sandbox::apply(log) {
        eprintln!("hodo-ext-worker: {err}");
        exit(3);
    }

    write_frame(
        &out,
        &Message::Hello {
            token,
            manifest_digest,
        },
    );

    let mut reader = FrameReader::new(io::stdin());
    let payload = wait_for_ready(&mut reader, &log);
    let ready: ReadyPayload = serde_json::from_value(payload).unwrap_or_default();
    let config = RuntimeConfig {
        dialect: ready.dialect.into(),
        extension_name: ready.config.extension_name,
        allowed_namespaces: ready.allowed_namespaces,
        host_permissions: ready.host_permissions,
        idle_unload_ms: ready.config.idle_unload_ms,
    };
    let mut host = WorkerHost::new(config, Rc::clone(&out));
    if let Err(err) = host.eval_scripts(&ready.background_scripts) {
        eprintln!("hodo-ext-worker: {err}");
        let _ = io::stdout().flush();
        exit(1);
    }

    loop {
        match reader.read_frame() {
            Ok(Some(Message::ApiCall { id, method, params })) => {
                host.answer_call(id, &method, params);
            }
            Ok(Some(Message::ApiResponse { id, result, error })) => {
                host.on_api_response(id, result, error);
            }
            Ok(Some(Message::Event { name, payload })) => match name.as_str() {
                "__hodo_log" => {
                    if let Some(message) = payload.get("message").and_then(Value::as_str) {
                        eprintln!("hodo-ext-worker: {message}");
                    }
                }
                "__hodo_ping" | "ping" => {
                    let mut reply = Map::new();
                    for key in ["at", "id"] {
                        if let Some(value) = payload.get(key) {
                            reply.insert(key.to_string(), value.clone());
                        }
                    }
                    write_event(&out, "__hodo_pong", Value::Object(reply));
                }
                _ => host.fire_event(&name, payload),
            },
            Ok(Some(Message::Shutdown)) => {
                host.tick();
                break;
            }
            Ok(Some(Message::Hello { .. })) => host.log("unexpected hello from broker".into()),
            Ok(None) => break,
            Err(err) => {
                eprintln!("hodo-ext-worker: {err}");
                break;
            }
        }
        host.tick();
    }
    let _ = io::stdout().flush();
    exit(0);
}

/// Nothing is evaluated until the ready event arrives.
fn wait_for_ready(reader: &mut FrameReader<io::Stdin>, log: &impl Fn(String)) -> Value {
    loop {
        match reader.read_frame() {
            Ok(Some(Message::Event { name, payload })) if name == "__hodo_ready" => {
                return payload;
            }
            Ok(Some(Message::Shutdown)) => exit(0),
            Ok(Some(other)) => log(format!("frame before ready: {other:?}")),
            Ok(None) => {
                eprintln!("hodo-ext-worker: broker closed the pipe before ready");
                exit(0);
            }
            Err(err) => {
                eprintln!("hodo-ext-worker: {err}");
                exit(0);
            }
        }
    }
}

fn manifest_digest_arg() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--manifest-digest" {
            return args.next();
        }
        if let Some(value) = arg.strip_prefix("--manifest-digest=") {
            return Some(value.to_string());
        }
    }
    None
}

fn write_frame(out: &Out, msg: &Message) {
    let mut out = out.borrow_mut();
    let _ = out.write_all(&encode(msg));
    let _ = out.flush();
}

fn write_event(out: &Out, name: &str, payload: Value) {
    write_frame(
        out,
        &Message::Event {
            name: name.to_string(),
            payload,
        },
    );
}

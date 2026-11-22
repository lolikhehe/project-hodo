//! Spawn end-to-end tests for the *real* `hodo-ext-worker` binary.
//!
//! Run with: `cargo test -p hodo-ext-worker --test isolation_e2e -- --nocapture`
//! (Cargo builds `CARGO_BIN_EXE_hodo-ext-worker` automatically). `.isolated(true)`
//! is the programmatic equivalent of `HODO_ISOLATION=1`.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hodo_ext_api::{ApiCall, ApiResult};
use hodo_ext_host::broker::{Broker, BrokerHost};
use hodo_ext_host::worker::{spawn, SpawnSpec, WorkerProc};
use hodo_ext_host::HostBuilder;
use hodo_ext_ipc::Message;
use serde_json::{json, Value};

const WORKER: &str = env!("CARGO_BIN_EXE_hodo-ext-worker");

fn repo_ext(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../extensions")
        .join(name)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Same digest algorithm as `Host::load_dir`, so the broker's Hello check binds.
fn manifest_digest(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn ready_payload(scripts: &[&str]) -> Value {
    json!({
        "config": { "extension_name": "Hello Chrome", "idle_unload_ms": null },
        "allowed_namespaces": ["storage", "tabs"],
        "host_permissions": [],
        "dialect": "chrome_mv3",
        "background_scripts": scripts,
    })
}

fn spawn_direct() -> WorkerProc {
    spawn(&SpawnSpec {
        worker_path: PathBuf::from(WORKER),
        token: "e2e-token".to_string(),
        digest: "e2e-digest".to_string(),
        extra_env: Vec::new(),
    })
    .expect("spawn real worker")
}

fn next_frame(proc: &mut WorkerProc, timeout: Duration) -> Message {
    proc.frames
        .recv_timeout(timeout)
        .expect("worker produced no frame in time")
        .expect("worker emitted a bad frame")
}

fn handshake(proc: &mut WorkerProc, ready: Value) {
    // The worker logs sandbox setup before emitting Hello.
    let hello = loop {
        let frame = next_frame(proc, Duration::from_secs(5));
        if !matches!(frame, Message::Event { .. }) {
            break frame;
        }
    };
    assert!(
        matches!(hello, Message::Hello { .. }),
        "first frame: {hello:?}"
    );
    proc.send(&Message::Event {
        name: "__hodo_ready".to_string(),
        payload: ready,
    })
    .expect("send ready");
}

/// Gate 1: the real worker evaluates hello-chrome's bg.js and both its storage
/// calls travel to the broker's real permission-gated `Host::dispatch`.
#[test]
fn hello_chrome_eval_after_ready_and_storage_roundtrips_through_broker() {
    let ext = repo_ext("hello-chrome");
    let manifest = fs::read(ext.join("manifest.json")).expect("manifest");
    let digest = manifest_digest(&manifest);
    let bg = fs::read_to_string(ext.join("bg.js")).expect("bg.js");

    let mut host = HostBuilder::new().isolated(true).build();
    host.load_dir(&ext).expect("load hello-chrome");

    let mut broker = Broker::default();
    broker.set_worker_path(PathBuf::from(WORKER));
    broker
        .start(0, &digest, ready_payload(&[&bg]), now_ms(), &mut host)
        .expect("start worker");
    assert!(broker.is_live(0));

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut round_tripped = false;
    while Instant::now() < deadline {
        broker.poll(now_ms(), &mut host);
        if let ApiResult::Ok(value) = host.dispatch(
            0,
            &ApiCall::StorageGet {
                key: "greeting".to_string(),
            },
        ) {
            if value == json!("Hello from a Chrome-style MV3 extension") {
                round_tripped = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        round_tripped,
        "bg.js never wrote through the broker: {:?}",
        host.take_diagnostics()
    );

    // Kill mid-flight: the broker observes the dead channel and stops routing.
    broker.terminate(0);
    assert!(!broker.is_live(0));
    assert!(!broker.send_event(0, "probe", json!({})));
}

/// Gate 2: `__hodo_ping` is answered with `__hodo_pong` inside the 1s cadence,
/// carrying the ping id so a broker can correlate it if it wants to.
#[test]
fn ping_is_answered_with_pong() {
    let mut proc = spawn_direct();
    handshake(&mut proc, ready_payload(&[]));
    proc.send(&Message::Event {
        name: "__hodo_ping".to_string(),
        payload: json!({ "at": 1234, "id": 77 }),
    })
    .expect("send ping");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut pong = None;
    while Instant::now() < deadline {
        match next_frame(&mut proc, Duration::from_millis(250)) {
            Message::Event { name, payload } if name == "__hodo_pong" => {
                pong = Some(payload);
                break;
            }
            _ => {}
        }
    }
    let payload = pong.expect("no pong inside the ping cadence");
    assert_eq!(payload.get("id"), Some(&json!(77)));
    assert_eq!(payload.get("at"), Some(&json!(1234)));
}

/// Gate 3: a worker Promise resolves from the *broker's* ApiResponse, not from
/// a local Null ack. The only way the script can log `got:7` is via correlation.
#[test]
fn worker_promise_resolves_from_broker_api_response() {
    let script = "chrome.storage.local.get('k').then(function (r) { __hodo_log('got:' + r.k); });";
    let mut proc = spawn_direct();
    handshake(&mut proc, ready_payload(&[script]));

    let (id, method) = loop {
        match next_frame(&mut proc, Duration::from_secs(5)) {
            Message::ApiCall { id, method, .. } => break (id, method),
            Message::Event { .. } => {}
            other => panic!("unexpected frame before call: {other:?}"),
        }
    };
    assert_eq!(method, "storage.local.get");
    proc.send(&Message::ApiResponse {
        id,
        result: Some(json!({ "k": 7 })),
        error: None,
    })
    .expect("send api response");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut resolved = false;
    while Instant::now() < deadline {
        if let Message::Event { name, payload } = next_frame(&mut proc, Duration::from_millis(500))
        {
            if name == "__hodo_log"
                && payload
                    .get("message")
                    .and_then(Value::as_str)
                    .is_some_and(|line| line.ends_with("got:7"))
            {
                resolved = true;
                break;
            }
        }
    }
    assert!(
        resolved,
        "JS promise did not settle from the broker response"
    );
}

/// The 5s cap rejects instead of hanging: pings drive the worker's tick loop.
#[test]
fn unanswered_call_rejects_after_five_seconds() {
    let script = "chrome.storage.local.get('k').then(\
                   function () { __hodo_log('resolved'); },\
                   function (e) { __hodo_log('rejected:' + e.message); });";
    let mut proc = spawn_direct();
    handshake(&mut proc, ready_payload(&[script]));

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut rejected = false;
    while Instant::now() < deadline {
        proc.send(&Message::Event {
            name: "__hodo_ping".to_string(),
            payload: json!({}),
        })
        .expect("send ping");
        match next_frame(&mut proc, Duration::from_millis(200)) {
            Message::Event { name, payload }
                if name == "__hodo_log"
                    && payload
                        .get("message")
                        .and_then(Value::as_str)
                        .is_some_and(|line| {
                            line.ends_with("rejected:api call 1 timed out after 5000ms")
                        }) =>
            {
                rejected = true;
                break;
            }
            _ => {}
        }
    }
    assert!(rejected, "outstanding call never rejected");
}

/// Test-only host that keeps handing the broker the crashing payload on every
/// restart, so the broker's real backoff→disable path can be observed.
struct CrashHost {
    inner: hodo_ext_host::Host,
    digest: String,
    ready: Value,
    disabled: bool,
}

impl BrokerHost for CrashHost {
    fn dispatch_wire_call(&mut self, ext: usize, method: &str, params: Value) -> ApiResult {
        self.inner.dispatch_wire_call(ext, method, params)
    }
    fn route_worker_event(&mut self, ext: usize, name: &str, payload: Value) {
        self.inner.route_worker_event(ext, name, payload);
    }
    fn diagnostic(&mut self, ext: usize, line: String) {
        self.inner.diagnostic(ext, line);
    }
    fn disable(&mut self, ext: usize, reason: String) {
        self.disabled = true;
        self.inner.disable(ext, reason);
    }
    fn ready_payload(&self, _ext: usize) -> Option<(String, Value)> {
        Some((self.digest.clone(), self.ready.clone()))
    }
}

struct TempExt(PathBuf);

impl TempExt {
    fn new(manifest: &str, script: &str) -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("hodo-ext-worker-e2e-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp ext dir");
        fs::write(root.join("manifest.json"), manifest).expect("manifest");
        fs::write(root.join("bg.js"), script).expect("bg.js");
        Self(root)
    }
}

impl Drop for TempExt {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// P2: a worker-originated message must reach an *isolated* target through
/// `broker.send_event`, not into the in-process queue that a live worker never
/// drains. The listener worker proves delivery by committing to storage.
#[test]
fn isolated_worker_message_is_delivered_through_the_broker() {
    let listener = TempExt::new(
        r#"{
            "manifest_version": 3,
            "name": "Message Listener",
            "version": "1.0.0",
            "permissions": ["storage"],
            "background": { "service_worker": "bg.js" }
        }"#,
        "__hodo_log('listener ready'); chrome.runtime.onMessage.addListener(function (m) { chrome.storage.local.set({ received: m.text }); });",
    );
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(PathBuf::from(WORKER))
        .build();
    host.load_dir(&repo_ext("hello-chrome"))
        .expect("source ext");
    host.load_dir(&listener.0).expect("listener ext");
    assert_eq!(host.run_background_scripts(0).unwrap(), "worker started");
    assert_eq!(host.run_background_scripts(1).unwrap(), "worker started");

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut ready = false;
    while Instant::now() < deadline && !ready {
        host.tick();
        ready = host
            .take_diagnostics()
            .iter()
            .any(|line| line.contains("listener ready"));
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        ready,
        "listener worker never evaluated: {:?}",
        host.take_diagnostics()
    );

    // Simulate the source worker's `runtime.onMessage` frame reaching the host.
    host.route_worker_event(0, "runtime.onMessage", json!({ "text": "ping" }));

    let mut delivered = false;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !delivered {
        host.tick();
        delivered = matches!(
            host.dispatch(
                1,
                &ApiCall::StorageGet {
                    key: "received".into()
                }
            ),
            ApiResult::Ok(value) if value == json!("ping")
        );
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        delivered,
        "isolated target never received the message: {:?}",
        host.take_diagnostics()
    );
    host.terminate_worker(0);
    host.terminate_worker(1);
}

/// Gate 4: a throwing script exits nonzero; the broker restarts with backoff
/// and disables the extension once the budget is exhausted.
#[test]
fn panicking_script_exits_nonzero_and_broker_disables_extension() {
    let ext = repo_ext("hello-chrome");
    let digest = manifest_digest(&fs::read(ext.join("manifest.json")).expect("manifest"));
    let ready = ready_payload(&["throw new Error('worker boom');"]);
    let mut host = CrashHost {
        inner: HostBuilder::new().isolated(true).build(),
        digest: digest.clone(),
        ready: ready.clone(),
        disabled: false,
    };
    host.inner.load_dir(&ext).expect("load extension");

    let mut broker = Broker::default();
    broker.set_worker_path(PathBuf::from(WORKER));
    broker
        .start(0, &digest, ready, now_ms(), &mut host)
        .expect("start worker");

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline && !host.disabled {
        broker.poll(now_ms(), &mut host);
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        host.disabled,
        "broker never disabled the crashing extension"
    );
    assert!(!broker.is_live(0));
}

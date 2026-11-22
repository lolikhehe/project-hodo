//! Spawn end-to-end tests. Ignored by default: they need the worker binaries
//! built first (`cargo build -p hodo-ext-worker -p hodo-ext-host`), and they
//! really create OS processes. Run with `cargo test -p hodo-ext-host --test
//! broker_e2e -- --ignored --nocapture`.

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use hodo_ext_api::{ApiCall, FakeClock};
use hodo_ext_host::HostBuilder;

fn repo_ext(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../extensions")
        .join(name)
}

fn sibling_binary(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe.parent().expect("binary directory");
    let target = dir.parent().unwrap_or(dir);
    let binary = target.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(
        binary.is_file(),
        "build {} first: {} is missing",
        name,
        binary.display()
    );
    binary
}

fn wait_until(mut done: impl FnMut() -> bool, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if done() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    done()
}

#[test]
#[ignore]
fn real_worker_handshake_ready_then_clean_shutdown() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(sibling_binary("hodo-ext-worker"))
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    assert_eq!(
        host.run_background_scripts(0).expect("spawn worker"),
        "worker started"
    );
    assert!(host.worker_is_live(0), "handshake must bind the channel");
    assert!(host.worker_pid(0).is_some_and(|pid| pid > 0));

    assert!(
        wait_until(
            || {
                host.tick();
                host.take_diagnostics()
                    .iter()
                    .any(|line| line.contains("sandbox:"))
            },
            Duration::from_secs(5)
        ),
        "worker never logged readiness: {:?}",
        host.take_diagnostics()
    );

    host.shutdown_worker(0);
    assert!(
        wait_until(
            || {
                host.tick();
                !host.worker_is_live(0)
            },
            Duration::from_secs(5)
        ),
        "worker ignored Shutdown"
    );
    assert!(host
        .take_diagnostics()
        .iter()
        .any(|line| line.contains("shut down cleanly")));
}

/// The real Host must ship script sources in `__hodo_ready`, not just the
/// handshake fields: without them the worker evaluates nothing.
#[test]
#[ignore]
fn real_worker_evaluates_background_scripts_from_ready_payload() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(sibling_binary("hodo-ext-worker"))
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    assert_eq!(
        host.run_background_scripts(0).expect("spawn worker"),
        "worker started"
    );

    let expected = serde_json::json!("Hello from a Chrome-style MV3 extension");
    assert!(
        wait_until(
            || {
                host.tick();
                matches!(
                    host.dispatch(
                        0,
                        &ApiCall::StorageGet {
                            key: "greeting".into()
                        }
                    ),
                    hodo_ext_api::ApiResult::Ok(value) if value == expected
                )
            },
            Duration::from_secs(10)
        ),
        "background script never ran; diagnostics: {:?}",
        host.take_diagnostics()
    );
    // `console.log` is intentionally inert in the JS prelude, so the storage
    // side-effect above is the observable proof that the script ran.

    host.shutdown_worker(0);
    assert!(wait_until(
        || {
            host.tick();
            !host.worker_is_live(0)
        },
        Duration::from_secs(5)
    ));
}

#[test]
#[ignore]
fn storage_roundtrip_through_the_broker() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(env!("CARGO_BIN_EXE_hodo-ext-fake-worker").to_string())
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn fake worker");

    assert!(
        wait_until(
            || {
                host.tick();
                matches!(
                    host.dispatch(
                        0,
                        &ApiCall::StorageGet {
                            key: "greeting".into()
                        }
                    ),
                    hodo_ext_api::ApiResult::Ok(value) if value == serde_json::json!("hello")
                )
            },
            Duration::from_secs(5)
        ),
        "storage.set/get roundtrip failed: {:?}",
        host.take_diagnostics()
    );
    assert!(host
        .take_diagnostics()
        .iter()
        .any(|line| line.contains("fake worker ready")));

    host.shutdown_worker(0);
    assert!(wait_until(
        || {
            host.tick();
            !host.worker_is_live(0)
        },
        Duration::from_secs(5)
    ));
}

#[test]
#[ignore]
fn killed_worker_restarts_then_crash_budget_disables_extension() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(env!("CARGO_BIN_EXE_hodo-ext-fake-worker").to_string())
        .worker_env("HODO_FAKE_CRASH", "1")
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn fake worker");

    // The fake worker dies mid-task every time; the browser must survive, log
    // each crash, restart with backoff, then disable the extension.
    assert!(wait_until(
        || {
            host.tick();
            host.extension_disabled(0)
        },
        Duration::from_secs(10)
    ));
    assert!(!host.worker_is_live(0));
    let diagnostics = host.take_diagnostics();
    assert!(
        diagnostics.iter().any(|line| line.contains("crashed")),
        "no crash diagnostic: {diagnostics:?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|line| line.contains("extension disabled")),
        "no disable event: {diagnostics:?}"
    );

    // A disabled extension is never woken by a late event.
    host.emit_event(0, "probe", serde_json::json!({}));
    host.tick();
    assert!(!host.worker_is_live(0));
}

#[test]
#[ignore]
fn spawn_failure_exhausts_backoff_and_disables() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path("hodo-worker-that-does-not-exist")
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    assert!(host.run_background_scripts(0).is_err());
    assert!(wait_until(
        || {
            host.tick();
            host.extension_disabled(0)
        },
        Duration::from_secs(5)
    ));
    assert!(host
        .take_diagnostics()
        .iter()
        .any(|line| line.contains("spawn failed")));
}

/// P1: an event arriving inside the shutdown grace window must reach the
/// worker (the `Shutdown` frame is queued only after grace) and cancel the
/// pending shutdown. Driven entirely by the injected clock.
#[test]
#[ignore]
fn event_in_shutdown_grace_is_delivered_and_cancels_shutdown() {
    let clock = Rc::new(FakeClock::new(0));
    let mut host = HostBuilder::new()
        .isolated(true)
        .clock(clock.clone())
        .worker_path(env!("CARGO_BIN_EXE_hodo-ext-fake-worker").to_string())
        .worker_env("HODO_FAKE_NO_STORAGE", "1")
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn fake worker");
    assert!(
        wait_until(
            || {
                host.tick();
                host.take_diagnostics()
                    .iter()
                    .any(|line| line.contains("fake worker ready"))
            },
            Duration::from_secs(5)
        ),
        "fake worker never got ready"
    );

    host.shutdown_worker(0);
    host.emit_event(0, "grace-probe", serde_json::json!({"at": "grace"}));
    clock.set_ms(1_000); // past the 250 ms grace deadline
    assert!(
        wait_until(
            || {
                host.tick();
                host.take_diagnostics()
                    .iter()
                    .any(|line| line.contains("event:grace-probe"))
            },
            Duration::from_secs(5)
        ),
        "event queued during shutdown grace was lost: {:?}",
        host.take_diagnostics()
    );
    assert!(
        host.worker_is_live(0),
        "an event in the grace window must cancel the shutdown"
    );
    host.terminate_worker(0);
}

/// P1: a crash starts a backoff; an event sent during it must be retained and
/// replayed when `poll` restarts the worker. The fake worker crashes once
/// (marker file), then survives to log the replayed event.
#[test]
#[ignore]
fn event_during_backoff_is_replayed_after_clock_driven_restart() {
    let marker = std::env::temp_dir().join(format!(
        "hodo-fake-crash-once-{}.marker",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&marker);
    let clock = Rc::new(FakeClock::new(0));
    let mut host = HostBuilder::new()
        .isolated(true)
        .clock(clock.clone())
        .worker_path(env!("CARGO_BIN_EXE_hodo-ext-fake-worker").to_string())
        .worker_env("HODO_FAKE_CRASH_ONCE", "1")
        .worker_env("HODO_FAKE_CRASH_MARKER", marker.to_string_lossy().as_ref())
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn fake worker");
    assert!(
        wait_until(
            || {
                host.tick();
                host.take_diagnostics()
                    .iter()
                    .any(|line| line.contains("crashed"))
            },
            Duration::from_secs(5)
        ),
        "fake worker never crashed: {:?}",
        host.take_diagnostics()
    );
    assert!(!host.worker_is_live(0));

    // Spawn is refused by the crash budget: the event must be retained.
    host.emit_event(0, "backoff-probe", serde_json::json!({"n": 1}));
    assert!(!host.worker_is_live(0));
    clock.set_ms(1_000); // past the 100/200/400 ms backoff window
    assert!(
        wait_until(
            || {
                host.tick();
                host.take_diagnostics()
                    .iter()
                    .any(|line| line.contains("event:backoff-probe"))
            },
            Duration::from_secs(10)
        ),
        "retained event was never replayed: {:?}",
        host.take_diagnostics()
    );
    host.terminate_worker(0);
    let _ = std::fs::remove_file(&marker);
}

/// P2: repeated RSS observability reads must not seed or trip the watchdog.
#[test]
#[ignore]
fn repeated_rss_diagnostic_reads_do_not_kill_the_worker() {
    let clock = Rc::new(FakeClock::new(0));
    let mut host = HostBuilder::new()
        .isolated(true)
        .clock(clock.clone())
        .worker_path(env!("CARGO_BIN_EXE_hodo-ext-fake-worker").to_string())
        .worker_env("HODO_FAKE_NO_STORAGE", "1")
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn fake worker");
    assert!(wait_until(
        || {
            host.tick();
            host.take_diagnostics()
                .iter()
                .any(|line| line.contains("fake worker ready"))
        },
        Duration::from_secs(5)
    ));
    for _ in 0..10 {
        assert!(host.worker_rss_bytes(0).is_some());
        clock.advance_ms(600);
        host.tick();
    }
    assert!(
        host.worker_is_live(0) && !host.extension_disabled(0),
        "diagnostic reads must not trip the watchdog: {:?}",
        host.take_diagnostics()
    );
    host.terminate_worker(0);
}

/// Manual RSS harness: prints one-worker baseline to stderr for the docs.
#[test]
#[ignore]
fn worker_rss_baseline() {
    let mut host = HostBuilder::new()
        .isolated(true)
        .worker_path(sibling_binary("hodo-ext-worker"))
        .build();
    host.load_dir(&repo_ext("hello-chrome")).expect("load ext");
    host.run_background_scripts(0).expect("spawn worker");
    assert!(wait_until(
        || {
            host.tick();
            host.worker_rss_bytes(0).is_some()
        },
        Duration::from_secs(5)
    ));
    thread::sleep(Duration::from_millis(500));
    let bytes = host.worker_rss_bytes(0).expect("rss sample");
    eprintln!(
        "one-worker RSS baseline: {} bytes ({:.1} MiB); 16-context budget {:.1} MiB",
        bytes,
        bytes as f64 / (1024.0 * 1024.0),
        bytes as f64 * 16.0 / (1024.0 * 1024.0)
    );
    host.shutdown_worker(0);
    let _ = wait_until(
        || {
            host.tick();
            !host.worker_is_live(0)
        },
        Duration::from_secs(5),
    );
}

//! Test-only worker: speaks the frozen IPC wire format without a JS engine.
//! Used by the ignored spawn end-to-end tests; never shipped in the browser.

use std::env;
use std::io::{self, Write};

use hodo_ext_ipc::{encode, FrameReader, Message};
use serde_json::{json, Value};

fn main() {
    let token = env::var("HODO_EXT_TOKEN").expect("token");
    let digest = env::args()
        .skip(1)
        .skip_while(|arg| arg != "--manifest-digest")
        .nth(1)
        .expect("--manifest-digest <digest>");
    let crash = env::var_os("HODO_FAKE_CRASH").is_some();
    // Crash exactly once so a test can observe a retained event replayed
    // after the broker's backoff restart.
    let crash_once = crash_once();
    // Event-delivery tests need a worker that stays in its frame loop; the
    // storage roundtrip is exercised by dedicated tests.
    let no_storage = env::var("HODO_FAKE_NO_STORAGE").is_ok_and(|value| value == "1");

    let stdout = io::stdout();
    let mut out = stdout.lock();
    write(
        &mut out,
        &Message::Hello {
            token,
            manifest_digest: digest,
        },
    );

    let mut reader = FrameReader::new(io::stdin());
    let ready = match reader.read_frame() {
        Ok(Some(Message::Event { name, payload })) if name == "__hodo_ready" => payload,
        Ok(other) => panic!("expected ready, got {other:?}"),
        Err(err) => panic!("ready read failed: {err}"),
    };
    assert!(ready.get("config").is_some(), "ready has no config");
    write(
        &mut out,
        &Message::Event {
            name: "__hodo_log".into(),
            payload: json!({ "message": "fake worker ready" }),
        },
    );

    if !no_storage {
        write(
            &mut out,
            &Message::ApiCall {
                id: 1,
                method: "storage.local.set".into(),
                params: json!(["greeting", "hello"]),
            },
        );
        let set_reply = expect_response(&mut reader, &mut out, 1);
        assert_eq!(set_reply, json!(null));

        if crash || crash_once {
            // Simulate a worker dying in the middle of its task queue.
            std::process::exit(9);
        }

        write(
            &mut out,
            &Message::ApiCall {
                id: 2,
                method: "storage.local.get".into(),
                params: json!(["greeting"]),
            },
        );
        assert_eq!(expect_response(&mut reader, &mut out, 2), json!("hello"));
    }

    loop {
        match reader.read_frame() {
            Ok(Some(Message::Shutdown)) | Ok(None) => break,
            Ok(Some(Message::Event { name, .. })) if name == "__hodo_ping" => {
                write(
                    &mut out,
                    &Message::Event {
                        name: "__hodo_pong".into(),
                        payload: json!({}),
                    },
                );
            }
            Ok(Some(Message::Event { name, .. })) => log_event(&mut out, &name),
            Ok(other) => panic!("unexpected frame: {other:?}"),
            Err(err) => panic!("frame error: {err}"),
        }
    }
    let _ = out.flush();
}

fn expect_response(reader: &mut FrameReader<io::Stdin>, out: &mut impl Write, id: u64) -> Value {
    loop {
        match reader.read_frame() {
            Ok(Some(Message::ApiResponse {
                id: got,
                result: Some(value),
                error: None,
            })) if got == id => return value,
            Ok(Some(Message::Event { name, .. })) if name == "__hodo_ping" => continue,
            // An event replayed while a call is in flight must not kill the
            // worker; log it so the test can observe delivery.
            Ok(Some(Message::Event { name, .. })) => log_event(out, &name),
            Ok(Some(other)) => panic!("expected response {id}, got {other:?}"),
            Ok(None) => panic!("broker closed the pipe"),
            Err(err) => panic!("response read failed: {err}"),
        }
    }
}

fn write(out: &mut impl Write, msg: &Message) {
    out.write_all(&encode(msg)).expect("write frame");
    out.flush().expect("flush frame");
}

/// Make a delivered browser event observable through the broker's log channel.
fn log_event(out: &mut impl Write, name: &str) {
    write(
        out,
        &Message::Event {
            name: "__hodo_log".into(),
            payload: json!({ "message": format!("event:{name}") }),
        },
    );
}

/// `HODO_FAKE_CRASH_ONCE=1` with a marker path crashes the first process only.
fn crash_once() -> bool {
    let (Ok(flag), Some(marker)) = (
        env::var("HODO_FAKE_CRASH_ONCE"),
        env::var_os("HODO_FAKE_CRASH_MARKER"),
    ) else {
        return false;
    };
    if flag != "1" {
        return false;
    }
    let path = std::path::Path::new(&marker);
    if path.exists() {
        return false;
    }
    let _ = std::fs::write(path, b"crashed");
    true
}

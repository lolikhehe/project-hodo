//! End-to-end extension tests against the real unpacked extension directories.

use std::path::PathBuf;

use hodo_ext_api::{ApiCall, ApiResult};
use hodo_ext_host::Host;
use hodo_ext_manifest::BrowserFlavor;
use serde_json::{json, Value};

fn ext_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../extensions")
        .join(name)
}

fn unwrap_ok(result: ApiResult, label: &str) -> Value {
    match result {
        ApiResult::Ok(value) => value,
        ApiResult::Err(message) => panic!("{label}: expected Ok, got Err({message})"),
    }
}

fn assert_denied(result: ApiResult, label: &str) {
    match result {
        ApiResult::Err(message) => assert!(
            message.contains("permission denied"),
            "{label}: unexpected error message: {message}"
        ),
        ApiResult::Ok(_) => panic!("{label}: expected permission-denied Err, got Ok"),
    }
}

#[test]
fn hello_gecko_loads_and_dispatches_allowed_apis() {
    let mut host = Host::new();
    {
        let ext = host
            .load_dir(&ext_dir("hello-gecko"))
            .expect("hello-gecko must load");
        assert_eq!(
            hodo_ext_manifest::flavor(&ext.manifest),
            BrowserFlavor::Gecko
        );
        assert_eq!(ext.manifest.name, "Hello Gecko");
    }
    assert_eq!(host.extensions().len(), 1);

    let missing = unwrap_ok(
        host.dispatch(
            0,
            &ApiCall::StorageGet {
                key: "answer".into(),
            },
        ),
        "StorageGet before set",
    );
    assert_eq!(missing, Value::Null);

    let value = json!({ "answer": 42 });
    assert_eq!(
        unwrap_ok(
            host.dispatch(
                0,
                &ApiCall::StorageSet {
                    key: "answer".into(),
                    value: value.clone(),
                },
            ),
            "StorageSet"
        ),
        Value::Null
    );
    assert_eq!(
        unwrap_ok(
            host.dispatch(
                0,
                &ApiCall::StorageGet {
                    key: "answer".into(),
                },
            ),
            "StorageGet after set"
        ),
        value
    );

    // hello-gecko declares both "storage" and "tabs".
    let tab = unwrap_ok(
        host.dispatch(
            0,
            &ApiCall::TabsCreate {
                url: "https://example.test".into(),
            },
        ),
        "TabsCreate",
    );
    assert!(tab.get("tabId").is_some(), "TabsCreate must return a tabId");
}

#[test]
fn storage_demo_gecko_allows_storage_but_denies_tabs() {
    let mut host = Host::new();
    host.load_dir(&ext_dir("storage-demo-gecko"))
        .expect("storage-demo-gecko must load");

    assert_eq!(
        unwrap_ok(
            host.dispatch(
                0,
                &ApiCall::StorageSet {
                    key: "k".into(),
                    value: json!("v"),
                },
            ),
            "StorageSet"
        ),
        Value::Null
    );
    assert_denied(
        host.dispatch(
            0,
            &ApiCall::TabsCreate {
                url: "https://example.test".into(),
            },
        ),
        "TabsCreate",
    );
}

#[test]
fn tabs_demo_chrome_allows_tabs_but_denies_storage() {
    let mut host = Host::new();
    host.load_dir(&ext_dir("tabs-demo-chrome"))
        .expect("tabs-demo-chrome must load");

    let tab = unwrap_ok(
        host.dispatch(
            0,
            &ApiCall::TabsCreate {
                url: "https://example.test".into(),
            },
        ),
        "TabsCreate",
    );
    assert!(
        tab.get("tabId").is_some(),
        "TabsCreate result must contain tabId: {tab}"
    );

    assert_denied(
        host.dispatch(
            0,
            &ApiCall::StorageSet {
                key: "k".into(),
                value: json!(1),
            },
        ),
        "StorageSet",
    );
}

#[test]
fn hello_chrome_allows_both_storage_and_tabs() {
    let mut host = Host::new();
    host.load_dir(&ext_dir("hello-chrome"))
        .expect("hello-chrome must load");

    assert_eq!(
        unwrap_ok(
            host.dispatch(
                0,
                &ApiCall::StorageSet {
                    key: "k".into(),
                    value: json!("v"),
                },
            ),
            "StorageSet"
        ),
        Value::Null
    );
    let tab = unwrap_ok(
        host.dispatch(
            0,
            &ApiCall::TabsCreate {
                url: "https://example.test".into(),
            },
        ),
        "TabsCreate",
    );
    assert!(
        tab.get("tabId").is_some(),
        "TabsCreate must return a tabId: {tab}"
    );
}

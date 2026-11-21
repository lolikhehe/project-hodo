//! Cross-compat API bridge: `chrome.*` and `browser.*` share one dispatch table.

use std::collections::HashMap;

use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub enum ApiCall {
    TabsCreate {
        url: String,
    },
    TabsQueryActive,
    StorageSet {
        key: String,
        value: serde_json::Value,
    },
    StorageGet {
        key: String,
    },
    SendMessage {
        message: serde_json::Value,
    },
}

#[derive(Debug, Clone)]
pub enum ApiResult {
    Ok(serde_json::Value),
    Err(String),
}

#[derive(Debug, Default)]
pub struct ApiRegistry {
    storage: HashMap<String, serde_json::Value>,
    next_tab_id: u64,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dispatch(&mut self, call: &ApiCall) -> ApiResult {
        match call {
            ApiCall::StorageSet { key, value } => {
                self.storage.insert(key.clone(), value.clone());
                ApiResult::Ok(Value::Null)
            }
            // WebExtension semantics: a missing key yields null, not an error.
            ApiCall::StorageGet { key } => {
                ApiResult::Ok(self.storage.get(key).cloned().unwrap_or(Value::Null))
            }
            ApiCall::TabsCreate { url } => {
                self.next_tab_id += 1;
                ApiResult::Ok(json!({ "tabId": self.next_tab_id, "url": url }))
            }
            // shortcut: mock active tab; the shell wires real tabs later.
            ApiCall::TabsQueryActive => {
                ApiResult::Ok(json!({ "tabId": 1, "index": 0, "url": null }))
            }
            // shortcut: no JS engine receiver yet, so messages are acknowledged but not delivered.
            ApiCall::SendMessage { .. } => ApiResult::Ok(Value::Null),
        }
    }
}

/// Map `"chrome.storage.local"` / `"browser.storage"` to the canonical api name.
pub fn canonical_namespace(ns: &str) -> Option<&'static str> {
    let lower = ns.to_ascii_lowercase();
    let stripped = lower
        .strip_prefix("chrome.")
        .or_else(|| lower.strip_prefix("browser."))
        .unwrap_or(&lower);
    // shortcut: single storage model for MVP, so local/sync/managed/session all fold into "storage".
    match stripped.split('.').next()? {
        "storage" => Some("storage"),
        "tabs" => Some("tabs"),
        "runtime" => Some("runtime"),
        // Gecko-idiomatic `browser.extension.*` is an alias for `runtime`.
        "extension" => Some("runtime"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unwrap_ok(result: ApiResult) -> Value {
        match result {
            ApiResult::Ok(value) => value,
            ApiResult::Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn storage_roundtrip() {
        let mut registry = ApiRegistry::new();
        let value = json!({ "answer": 42 });
        assert!(matches!(
            registry.dispatch(&ApiCall::StorageSet {
                key: "k".into(),
                value: value.clone(),
            }),
            ApiResult::Ok(Value::Null)
        ));
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageGet { key: "k".into() })),
            value
        );
    }

    #[test]
    fn storage_get_absent_is_null() {
        let mut registry = ApiRegistry::new();
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageGet {
                key: "missing".into()
            })),
            Value::Null
        );
    }

    #[test]
    fn tabs_create_increments_tab_id() {
        let mut registry = ApiRegistry::new();
        let first = unwrap_ok(registry.dispatch(&ApiCall::TabsCreate {
            url: "https://a".into(),
        }));
        let second = unwrap_ok(registry.dispatch(&ApiCall::TabsCreate {
            url: "https://b".into(),
        }));
        assert_eq!(first["tabId"], 1);
        assert_eq!(first["url"], "https://a");
        assert_eq!(second["tabId"], 2);
        assert_eq!(second["url"], "https://b");
    }

    #[test]
    fn tabs_query_active_and_send_message() {
        let mut registry = ApiRegistry::new();
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::TabsQueryActive)),
            json!({ "tabId": 1, "index": 0, "url": null })
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::SendMessage {
                message: json!("hi")
            })),
            Value::Null
        );
    }

    #[test]
    fn canonical_namespace_accepts_both_dialects() {
        let cases = [
            ("chrome.storage.local", "storage"),
            ("browser.storage", "storage"),
            ("browser.storage.sync", "storage"),
            ("CHROME.STORAGE.LOCAL", "storage"),
            ("chrome.tabs", "tabs"),
            ("browser.tabs.create", "tabs"),
            ("Browser.Tabs.Create", "tabs"),
            ("runtime", "runtime"),
            ("browser.extension", "runtime"),
            ("chrome.extension.sendMessage", "runtime"),
            ("BROWSER.EXTENSION", "runtime"),
        ];
        for (input, expected) in cases {
            assert_eq!(canonical_namespace(input), Some(expected), "input: {input}");
        }
    }

    #[test]
    fn canonical_namespace_rejects_unknown() {
        assert_eq!(canonical_namespace("chrome.alarms"), None);
        assert_eq!(canonical_namespace("moz-extension://abc"), None);
    }

    #[test]
    fn api_result_is_clone() {
        let result = ApiResult::Ok(json!({ "x": 1 }));
        let cloned = result.clone();
        assert_eq!(unwrap_ok(result), unwrap_ok(cloned));
    }
}

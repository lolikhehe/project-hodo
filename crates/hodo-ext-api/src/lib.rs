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
    StorageSyncSet {
        key: String,
        value: serde_json::Value,
    },
    StorageSyncGet {
        key: String,
    },
    TabsList,
    TabsActivate {
        tab_id: u64,
    },
    NotificationsCreate {
        title: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum ApiResult {
    Ok(serde_json::Value),
    Err(String),
}

/// Events emitted by API calls, drained by the host via [`ApiRegistry::drain_events`].
#[derive(Debug, Clone)]
pub enum ExtEvent {
    StorageChanged { key: String },
    MessageReceived { message: serde_json::Value },
    TabActivated { tab_id: u64 },
    NotificationShown { title: String },
}

#[derive(Debug, Default)]
pub struct ApiRegistry {
    storage: HashMap<String, serde_json::Value>,
    storage_sync: HashMap<String, serde_json::Value>,
    next_tab_id: u64,
    tabs: Vec<(u64, String)>,
    next_notification_id: u64,
    pending_events: Vec<ExtEvent>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take all events queued since the last drain.
    pub fn drain_events(&mut self) -> Vec<ExtEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn dispatch(&mut self, call: &ApiCall) -> ApiResult {
        match call {
            ApiCall::StorageSet { key, value } => {
                self.storage.insert(key.clone(), value.clone());
                self.pending_events
                    .push(ExtEvent::StorageChanged { key: key.clone() });
                ApiResult::Ok(Value::Null)
            }
            // WebExtension semantics: a missing key yields null, not an error.
            ApiCall::StorageGet { key } => {
                ApiResult::Ok(self.storage.get(key).cloned().unwrap_or(Value::Null))
            }
            ApiCall::StorageSyncSet { key, value } => {
                self.storage_sync.insert(key.clone(), value.clone());
                self.pending_events
                    .push(ExtEvent::StorageChanged { key: key.clone() });
                ApiResult::Ok(Value::Null)
            }
            ApiCall::StorageSyncGet { key } => {
                ApiResult::Ok(self.storage_sync.get(key).cloned().unwrap_or(Value::Null))
            }
            ApiCall::TabsCreate { url } => {
                self.next_tab_id += 1;
                let tab_id = self.next_tab_id;
                self.tabs.push((tab_id, url.clone()));
                ApiResult::Ok(json!({ "tabId": tab_id, "url": url }))
            }
            ApiCall::TabsList => {
                let tabs: Vec<Value> = self
                    .tabs
                    .iter()
                    .map(|(tab_id, url)| json!({ "tabId": tab_id, "url": url }))
                    .collect();
                ApiResult::Ok(Value::Array(tabs))
            }
            ApiCall::TabsActivate { tab_id } => {
                self.pending_events
                    .push(ExtEvent::TabActivated { tab_id: *tab_id });
                ApiResult::Ok(Value::Null)
            }
            // shortcut: mock active tab; the shell wires real tabs later.
            ApiCall::TabsQueryActive => {
                ApiResult::Ok(json!({ "tabId": 1, "index": 0, "url": null }))
            }
            // shortcut: no JS engine receiver yet, so messages are acknowledged but queued as events.
            ApiCall::SendMessage { message } => {
                self.pending_events.push(ExtEvent::MessageReceived {
                    message: message.clone(),
                });
                ApiResult::Ok(Value::Null)
            }
            ApiCall::NotificationsCreate { title, .. } => {
                self.next_notification_id += 1;
                let notification_id = self.next_notification_id;
                self.pending_events.push(ExtEvent::NotificationShown {
                    title: title.clone(),
                });
                ApiResult::Ok(json!({ "notificationId": notification_id }))
            }
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
    // shortcut: local/sync/managed/session share the "storage" permission namespace
    // (sync keeps a separate backing map but uses the same permission).
    match stripped.split('.').next()? {
        "storage" => Some("storage"),
        "tabs" => Some("tabs"),
        "runtime" => Some("runtime"),
        // Gecko-idiomatic `browser.extension.*` is an alias for `runtime`.
        "extension" => Some("runtime"),
        "notifications" => Some("notifications"),
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
            ("chrome.notifications", "notifications"),
            ("browser.notifications.create", "notifications"),
            ("BROWSER.NOTIFICATIONS", "notifications"),
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

    #[test]
    fn sync_and_local_storage_are_isolated() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::StorageSet {
            key: "k".into(),
            value: json!("local"),
        });
        registry.dispatch(&ApiCall::StorageSyncSet {
            key: "k".into(),
            value: json!("sync"),
        });
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageGet { key: "k".into() })),
            json!("local")
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageSyncGet { key: "k".into() })),
            json!("sync")
        );
    }

    #[test]
    fn sync_get_absent_is_null() {
        let mut registry = ApiRegistry::new();
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageSyncGet {
                key: "missing".into()
            })),
            Value::Null
        );
    }

    #[test]
    fn tabs_list_returns_all_created_tabs() {
        let mut registry = ApiRegistry::new();
        assert_eq!(unwrap_ok(registry.dispatch(&ApiCall::TabsList)), json!([]));
        registry.dispatch(&ApiCall::TabsCreate {
            url: "https://a".into(),
        });
        registry.dispatch(&ApiCall::TabsCreate {
            url: "https://b".into(),
        });
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::TabsList)),
            json!([
                { "tabId": 1, "url": "https://a" },
                { "tabId": 2, "url": "https://b" }
            ])
        );
    }

    #[test]
    fn events_queue_in_dispatch_order_and_drain_clears() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::StorageSet {
            key: "k".into(),
            value: json!(1),
        });
        registry.dispatch(&ApiCall::SendMessage {
            message: json!("hi"),
        });
        registry.dispatch(&ApiCall::TabsActivate { tab_id: 7 });
        registry.dispatch(&ApiCall::NotificationsCreate {
            title: "t".into(),
            message: "m".into(),
        });

        let events = registry.drain_events();
        assert_eq!(events.len(), 4);
        assert!(matches!(&events[0], ExtEvent::StorageChanged { key } if key == "k"));
        assert!(
            matches!(&events[1], ExtEvent::MessageReceived { message } if *message == json!("hi"))
        );
        assert!(matches!(&events[2], ExtEvent::TabActivated { tab_id } if *tab_id == 7));
        assert!(matches!(&events[3], ExtEvent::NotificationShown { title } if title == "t"));
        assert!(registry.drain_events().is_empty());
    }

    #[test]
    fn sync_set_also_emits_storage_changed() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::StorageSyncSet {
            key: "s".into(),
            value: json!(1),
        });
        let events = registry.drain_events();
        assert!(matches!(&events[0], ExtEvent::StorageChanged { key } if key == "s"));
    }

    #[test]
    fn notification_ids_increment() {
        let mut registry = ApiRegistry::new();
        let first = unwrap_ok(registry.dispatch(&ApiCall::NotificationsCreate {
            title: "a".into(),
            message: "one".into(),
        }));
        let second = unwrap_ok(registry.dispatch(&ApiCall::NotificationsCreate {
            title: "b".into(),
            message: "two".into(),
        }));
        assert_eq!(first, json!({ "notificationId": 1 }));
        assert_eq!(second, json!({ "notificationId": 2 }));
    }

    #[test]
    fn ext_event_is_clone() {
        let event = ExtEvent::StorageChanged { key: "k".into() };
        let cloned = event.clone();
        assert!(matches!(cloned, ExtEvent::StorageChanged { key } if key == "k"));
    }
}

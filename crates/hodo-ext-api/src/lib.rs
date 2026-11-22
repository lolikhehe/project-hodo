//! Cross-compat API bridge: `chrome.*` and `browser.*` share one dispatch table.
//!
//! Also owns the small shared primitives the JS bridge and the extension host
//! need for M2.1: clocks, runtime configuration, storage and alarms.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use hodo_ext_i18n::Messages;
use serde_json::{json, Value};

/// Injectable time source. Tests use [`FakeClock`] and never sleep.
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Wall-clock implementation used by the host in production.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Deterministic clock for tests. Clones share the same value.
#[derive(Debug, Clone, Default)]
pub struct FakeClock {
    now_ms: std::rc::Rc<std::cell::Cell<u64>>,
}

impl FakeClock {
    #[must_use]
    pub fn new(now_ms: u64) -> Self {
        let clock = Self::default();
        clock.set_ms(now_ms);
        clock
    }

    pub fn set_ms(&self, now_ms: u64) {
        self.now_ms.set(now_ms);
    }

    pub fn advance_ms(&self, delta_ms: u64) {
        let now = self.now_ms.get();
        self.now_ms.set(now.saturating_add(delta_ms));
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.get()
    }
}

/// Which API style the injected globals use by default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ApiDialect {
    /// MV2 `chrome.*`: callbacks only.
    ChromeMv2,
    /// MV3 `chrome.*`: callback if the last argument is a function, else Promise.
    #[default]
    ChromeMv3,
    /// `browser.*`: always Promise (Gecko style, any manifest version).
    Gecko,
}

/// Per-context settings passed from the host into the JS bridge.
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfig {
    pub dialect: ApiDialect,
    pub extension_name: String,
    /// Canonical API namespaces the manifest grants (`runtime` is implicit).
    pub allowed_namespaces: Vec<String>,
    /// URL patterns from `host_permissions`, used by `fetch`.
    pub host_permissions: Vec<String>,
    /// `None` means a persistent context (MV2); `Some(ms)` enables MV3 idle unload.
    pub idle_unload_ms: Option<u64>,
}

/// 10 MiB local storage quota, enforced across `local`/`sync`/`session`.
pub const STORAGE_QUOTA_BYTES: usize = 10 * 1024 * 1024;
/// Hard cap on a single inter-context message or API argument blob.
pub const MAX_API_VALUE_BYTES: usize = hodo_types::limits::MAX_RESOURCE_BYTES;
/// Maximum nesting depth accepted across the JS bridge.
pub const MAX_API_VALUE_DEPTH: usize = hodo_types::limits::MAX_MSG_DEPTH;

/// Reject non-JSON-like, oversized or deeply nested values before they reach a
/// registry or another context. Cycles are reported by `serde_json::to_string`
/// failing on a `Value` built by boa's `JsValue::to_json` (which returns `None`).
pub fn validate_api_value(value: &Value) -> Result<(), String> {
    fn walk(value: &Value, depth: usize) -> Result<usize, String> {
        if depth > MAX_API_VALUE_DEPTH {
            return Err("value nesting too deep".to_string());
        }
        let mut bytes: usize = match value {
            Value::Null | Value::Bool(_) | Value::Number(_) => 8,
            Value::String(s) => s.len().saturating_add(8),
            Value::Array(items) => {
                let mut total: usize = 8;
                for item in items {
                    total = total.saturating_add(walk(item, depth + 1)?);
                }
                total
            }
            Value::Object(map) => {
                let mut total: usize = 8;
                for (key, item) in map {
                    total = total.saturating_add(key.len().saturating_add(8));
                    total = total.saturating_add(walk(item, depth + 1)?);
                }
                total
            }
        };
        if bytes > MAX_API_VALUE_BYTES {
            bytes = MAX_API_VALUE_BYTES + 1;
        }
        Ok(bytes)
    }
    if walk(value, 0)? > MAX_API_VALUE_BYTES {
        return Err("value exceeds 2 MiB API limit".to_string());
    }
    Ok(())
}

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
    StorageSessionSet {
        key: String,
        value: serde_json::Value,
    },
    StorageSessionGet {
        key: String,
    },
    StorageSyncSet {
        key: String,
        value: serde_json::Value,
    },
    StorageSyncGet {
        key: String,
    },
    AlarmCreate {
        name: String,
        delay_ms: u64,
        period_ms: Option<u64>,
        /// Clock value at creation, supplied by the JS bridge or host.
        now_ms: u64,
    },
    AlarmClear {
        name: String,
    },
    Fetch {
        url: String,
        method: String,
        body: String,
    },
    SendMessage {
        message: serde_json::Value,
    },
    TabsList,
    TabsActivate {
        tab_id: u64,
    },
    NotificationsCreate {
        title: String,
        message: String,
    },
    /// PURE read: localized message. Answered from the loaded locale tables.
    I18nGetMessage {
        key: String,
        substitutions: Vec<String>,
    },
    /// PURE read: resolve an in-package path against the extension's URL base.
    RuntimeGetUrl {
        path: String,
    },
    /// PURE read: the extension's manifest as JSON.
    RuntimeGetManifest,
    /// PURE read: the extension id.
    RuntimeId,
}

#[derive(Debug, Clone)]
pub enum ApiResult {
    Ok(serde_json::Value),
    Err(String),
}

/// Events emitted by API calls, drained by the host and delivered to JS
/// listeners by each runtime's task queue.
#[derive(Debug, Clone)]
pub enum ExtEvent {
    StorageChanged {
        area: String,
        key: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    AlarmFired {
        name: String,
    },
    MessageReceived {
        message: serde_json::Value,
    },
    TabActivated {
        tab_id: u64,
    },
    NotificationShown {
        title: String,
    },
}

/// Immutable per-extension data behind the four PURE `runtime.*`/`i18n.*`
/// reads. The host installs it from the loaded manifest and locale tables;
/// reads never mutate it, so the JS memo may cache them for a whole tick.
#[derive(Debug, Default)]
pub struct RuntimeInfo {
    pub id: String,
    pub url_base: String,
    pub manifest: Value,
    pub messages: Option<Messages>,
}

#[derive(Debug, Default)]
pub struct ApiRegistry {
    info: Rc<RuntimeInfo>,
    storage: HashMap<String, serde_json::Value>,
    storage_sync: HashMap<String, serde_json::Value>,
    /// RAM only: dropped when the host session ends.
    storage_session: HashMap<String, serde_json::Value>,
    storage_bytes: usize,
    alarms: BTreeMap<String, Alarm>,
    next_tab_id: u64,
    tabs: Vec<(u64, String)>,
    next_notification_id: u64,
    pending_events: Vec<ExtEvent>,
}

#[derive(Debug, Clone)]
struct Alarm {
    next_at_ms: u64,
    period_ms: Option<u64>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install the static context served by the PURE reads. The shared
    /// registry is re-pointed before each extension's dispatch in the host.
    pub fn set_runtime_info(&mut self, info: Rc<RuntimeInfo>) {
        self.info = info;
    }

    /// Take all events queued since the last drain.
    pub fn drain_events(&mut self) -> Vec<ExtEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Advance alarm scheduling to `now_ms`, emitting at most one event per
    /// alarm per call. Called by the host on every event-loop tick.
    pub fn advance_clock(&mut self, now_ms: u64) {
        let due: Vec<String> = self
            .alarms
            .iter()
            .filter(|(_, alarm)| alarm.next_at_ms <= now_ms)
            .map(|(name, _)| name.clone())
            .collect();
        for name in due {
            if let Some(alarm) = self.alarms.get_mut(&name) {
                match alarm.period_ms {
                    Some(period) => {
                        let step = period.max(1);
                        let missed = now_ms.saturating_sub(alarm.next_at_ms) / step;
                        alarm.next_at_ms = alarm
                            .next_at_ms
                            .saturating_add(step.saturating_mul(missed.saturating_add(1)));
                    }
                    None => {
                        self.alarms.remove(&name);
                    }
                }
            }
            self.pending_events.push(ExtEvent::AlarmFired { name });
        }
    }

    fn quota_check(
        &self,
        key: &str,
        value: &Value,
        previous: Option<&Value>,
    ) -> Result<usize, String> {
        validate_api_value(value)?;
        let old = previous.map_or(0, Self::value_bytes);
        let added = key.len().saturating_add(Self::value_bytes(value));
        let used = self.storage_bytes.saturating_sub(old).saturating_add(added);
        if used > STORAGE_QUOTA_BYTES {
            return Err("storage quota exceeded".to_string());
        }
        Ok(used)
    }

    fn value_bytes(value: &Value) -> usize {
        serde_json::to_string(value).map_or(usize::MAX, |s| s.len())
    }

    fn storage_set(&mut self, area: &str, key: &str, value: &Value) -> ApiResult {
        let old = match area {
            "sync" => self.storage_sync.get(key).cloned(),
            "session" => self.storage_session.get(key).cloned(),
            _ => self.storage.get(key).cloned(),
        };
        // `session` is RAM-only and intentionally outside the persistent quota.
        if area != "session" {
            match self.quota_check(key, value, old.as_ref()) {
                Ok(used) => self.storage_bytes = used,
                Err(err) => return ApiResult::Err(err),
            }
        } else if let Err(err) = validate_api_value(value) {
            return ApiResult::Err(err);
        }
        let map = match area {
            "sync" => &mut self.storage_sync,
            "session" => &mut self.storage_session,
            _ => &mut self.storage,
        };
        map.insert(key.to_string(), value.clone());
        self.pending_events.push(ExtEvent::StorageChanged {
            area: area.to_string(),
            key: key.to_string(),
            old_value: old.unwrap_or(Value::Null),
            new_value: value.clone(),
        });
        ApiResult::Ok(Value::Null)
    }

    pub fn dispatch(&mut self, call: &ApiCall) -> ApiResult {
        match call {
            ApiCall::StorageSet { key, value } => self.storage_set("local", key, value),
            ApiCall::StorageSyncSet { key, value } => self.storage_set("sync", key, value),
            ApiCall::StorageSessionSet { key, value } => self.storage_set("session", key, value),
            // WebExtension semantics: a missing key yields null, not an error.
            ApiCall::StorageGet { key } => {
                ApiResult::Ok(self.storage.get(key).cloned().unwrap_or(Value::Null))
            }
            ApiCall::StorageSyncGet { key } => {
                ApiResult::Ok(self.storage_sync.get(key).cloned().unwrap_or(Value::Null))
            }
            ApiCall::StorageSessionGet { key } => ApiResult::Ok(
                self.storage_session
                    .get(key)
                    .cloned()
                    .unwrap_or(Value::Null),
            ),
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
            ApiCall::AlarmCreate {
                name,
                delay_ms,
                period_ms,
                now_ms,
            } => {
                if let Err(err) = validate_api_value(&json!(name)) {
                    return ApiResult::Err(err);
                }
                self.alarms.insert(
                    name.clone(),
                    Alarm {
                        next_at_ms: now_ms.saturating_add(*delay_ms),
                        period_ms: *period_ms,
                    },
                );
                ApiResult::Ok(Value::Null)
            }
            ApiCall::AlarmClear { name } => {
                self.alarms.remove(name);
                ApiResult::Ok(Value::Bool(true))
            }
            // The host performs the permission check and the actual network
            // request; the registry only acknowledges the API call.
            ApiCall::Fetch { .. } => ApiResult::Ok(Value::Null),
            // shortcut: no cross-context receiver yet; messages are acknowledged
            // and queued as events for the popup/worker wiring in M2.2/M3.
            ApiCall::SendMessage { message } => {
                if let Err(err) = validate_api_value(message) {
                    return ApiResult::Err(err);
                }
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
            ApiCall::I18nGetMessage { key, substitutions } => ApiResult::Ok(Value::String(
                self.info
                    .messages
                    .as_ref()
                    .map_or_else(String::new, |m| m.get_message(key, substitutions)),
            )),
            ApiCall::RuntimeGetUrl { path } => {
                let base = self.info.url_base.trim_end_matches('/');
                ApiResult::Ok(Value::String(format!(
                    "{base}/{}",
                    path.trim_start_matches('/')
                )))
            }
            ApiCall::RuntimeGetManifest => ApiResult::Ok(self.info.manifest.clone()),
            ApiCall::RuntimeId => ApiResult::Ok(Value::String(self.info.id.clone())),
        }
    }

    /// Number of distinct keys with stored values across persistent areas.
    pub fn storage_key_count(&self) -> usize {
        self.storage.len() + self.storage_sync.len() + self.storage_session.len()
    }

    /// Drop RAM-only `storage.session` (called when a browser session ends).
    pub fn clear_session(&mut self) {
        self.storage_session.clear();
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
        "alarms" => Some("alarms"),
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
            ("chrome.alarms.create", "alarms"),
        ];
        for (input, expected) in cases {
            assert_eq!(canonical_namespace(input), Some(expected), "input: {input}");
        }
    }

    #[test]
    fn canonical_namespace_rejects_unknown() {
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
        assert!(
            matches!(&events[0], ExtEvent::StorageChanged { area, key, .. } if area == "local" && key == "k")
        );
        assert!(
            matches!(&events[1], ExtEvent::MessageReceived { message } if *message == json!("hi"))
        );
        assert!(matches!(&events[2], ExtEvent::TabActivated { tab_id } if *tab_id == 7));
        assert!(matches!(&events[3], ExtEvent::NotificationShown { title } if title == "t"));
        assert!(registry.drain_events().is_empty());
    }

    #[test]
    fn storage_changed_carries_old_and_new_values() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::StorageSet {
            key: "k".into(),
            value: json!(1),
        });
        registry.dispatch(&ApiCall::StorageSet {
            key: "k".into(),
            value: json!(2),
        });
        let events = registry.drain_events();
        assert!(matches!(
            &events[1],
            ExtEvent::StorageChanged { old_value, new_value, .. }
                if *old_value == json!(1) && *new_value == json!(2)
        ));
    }

    #[test]
    fn session_storage_is_ram_only() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::StorageSessionSet {
            key: "k".into(),
            value: json!("ram"),
        });
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageSessionGet { key: "k".into() })),
            json!("ram")
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageGet { key: "k".into() })),
            Value::Null
        );
        registry.clear_session();
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::StorageSessionGet { key: "k".into() })),
            Value::Null
        );
    }

    #[test]
    fn storage_quota_is_enforced() {
        let mut registry = ApiRegistry::new();
        let chunk = Value::String("x".repeat(1024 * 1024));
        let mut rejected = false;
        for i in 0..24 {
            let result = registry.dispatch(&ApiCall::StorageSet {
                key: format!("k{i}"),
                value: chunk.clone(),
            });
            if let ApiResult::Err(msg) = result {
                assert!(msg.contains("quota"), "unexpected error: {msg}");
                rejected = true;
                break;
            }
        }
        assert!(rejected, "10 MiB quota was never enforced");
    }

    #[test]
    fn deeply_nested_values_are_rejected() {
        let mut nested = json!(1);
        for _ in 0..MAX_API_VALUE_DEPTH + 4 {
            nested = json!([nested]);
        }
        assert!(validate_api_value(&nested).is_err());
        assert!(validate_api_value(&json!({"ok": 1})).is_ok());
    }

    #[test]
    fn alarms_fire_once_then_repeat_on_fake_clock() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::AlarmCreate {
            name: "a".into(),
            delay_ms: 10,
            period_ms: Some(5),
            now_ms: 0,
        });
        registry.advance_clock(9);
        assert!(registry.drain_events().is_empty());
        registry.advance_clock(10);
        assert_eq!(registry.drain_events().len(), 1);
        registry.advance_clock(14);
        assert!(registry.drain_events().is_empty());
        registry.advance_clock(15);
        assert_eq!(registry.drain_events().len(), 1);
        registry.dispatch(&ApiCall::AlarmClear { name: "a".into() });
        registry.advance_clock(1000);
        assert!(registry.drain_events().is_empty());
    }

    #[test]
    fn one_shot_alarm_is_removed_after_firing() {
        let mut registry = ApiRegistry::new();
        registry.dispatch(&ApiCall::AlarmCreate {
            name: "once".into(),
            delay_ms: 2,
            period_ms: None,
            now_ms: 0,
        });
        registry.advance_clock(2);
        assert_eq!(registry.drain_events().len(), 1);
        registry.advance_clock(100);
        assert!(registry.drain_events().is_empty());
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
    fn fake_clock_advances_without_sleeping() {
        let clock = FakeClock::new(5);
        let shared = clock.clone();
        assert_eq!(clock.now_ms(), 5);
        shared.advance_ms(7);
        assert_eq!(clock.now_ms(), 12);
    }

    #[test]
    fn pure_runtime_and_i18n_variants_serve_static_info() {
        let messages =
            hodo_ext_i18n::load_locales(std::path::Path::new("__no_locales__"), None, "en-US")
                .unwrap()
                .with_extension_id("ext-42");
        let mut registry = ApiRegistry::new();
        registry.set_runtime_info(Rc::new(RuntimeInfo {
            id: "ext-42".into(),
            url_base: "moz-extension://ext-42".into(),
            manifest: json!({ "name": "Demo" }),
            messages: Some(messages),
        }));
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::RuntimeId)),
            json!("ext-42")
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::RuntimeGetUrl {
                path: "/popup.html".into()
            })),
            json!("moz-extension://ext-42/popup.html")
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::RuntimeGetManifest)),
            json!({ "name": "Demo" })
        );
        assert_eq!(
            unwrap_ok(registry.dispatch(&ApiCall::I18nGetMessage {
                key: "@@extension_id".into(),
                substitutions: vec![],
            })),
            json!("ext-42")
        );
    }

    #[test]
    fn ext_event_is_clone() {
        let event = ExtEvent::StorageChanged {
            area: "local".into(),
            key: "k".into(),
            old_value: Value::Null,
            new_value: json!(1),
        };
        let cloned = event.clone();
        assert!(matches!(cloned, ExtEvent::StorageChanged { key, .. } if key == "k"));
    }
}

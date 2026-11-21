//! Cross-compat API bridge: `chrome.*` and `browser.*` share one dispatch table.

use std::collections::HashMap;

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
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dispatch(&mut self, call: &ApiCall) -> ApiResult {
        todo!("dispatch({:?})", call)
    }
}

/// Map `"chrome.storage.local"` / `"browser.storage"` to the canonical api name.
pub fn canonical_namespace(ns: &str) -> Option<&'static str> {
    let ns = ns
        .strip_prefix("chrome.")
        .or_else(|| ns.strip_prefix("browser."))
        .unwrap_or(ns);
    match ns {
        "storage" | "storage.local" | "storage.sync" | "storage.managed" | "storage.session" => {
            Some("storage")
        }
        "tabs" => Some("tabs"),
        "runtime" | "extension" => Some("runtime"),
        _ => None,
    }
}

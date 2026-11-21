//! Chrome and Gecko WebExtension manifest parsing (MV2 and MV3).

use std::collections::{HashMap, HashSet};

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVersion {
    V2,
    V3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserFlavor {
    Chrome,
    Gecko,
    Generic,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub version: ManifestVersion,
    pub name: String,
    pub ext_version: String,
    pub permissions: Vec<String>,
    /// URL patterns from `host_permissions`; never grant API namespaces.
    pub host_permissions: Vec<String>,
    /// Gecko/Edge MV2 style background scripts.
    pub background_scripts: Vec<String>,
    /// Chrome MV3 style service worker.
    pub service_worker: Option<String>,
    pub gecko_id: Option<String>,
    /// Top-level `"key"` field used by the Chrome Web Store.
    pub chrome_key: Option<String>,
    pub action_title: Option<String>,
    pub icons: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("invalid json: {0}")]
    Json(String),
    #[error("missing field: {0}")]
    Missing(&'static str),
    #[error("unsupported manifest_version: {0}")]
    UnsupportedVersion(u8),
}

/// Lowercase-normalize and dedup one permission field, skipping non-strings.
fn normalized_list(root: &Value, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    if let Some(values) = root.get(key).and_then(Value::as_array) {
        for value in values.iter().filter_map(Value::as_str) {
            let normalized = value.to_lowercase();
            if seen.insert(normalized.clone()) {
                out.push(normalized);
            }
        }
    }
    out
}

/// Accepts both Chrome-style and Gecko-style WebExtension manifests.
pub fn parse(json: &str) -> Result<Manifest, ManifestError> {
    let root: Value = serde_json::from_str(json).map_err(|e| ManifestError::Json(e.to_string()))?;

    let name = root
        .get("name")
        .and_then(Value::as_str)
        .ok_or(ManifestError::Missing("name"))?
        .to_owned();
    let ext_version = root
        .get("version")
        .and_then(Value::as_str)
        .ok_or(ManifestError::Missing("version"))?
        .to_owned();

    let version = match root.get("manifest_version").and_then(Value::as_u64) {
        Some(2) => ManifestVersion::V2,
        Some(3) => ManifestVersion::V3,
        other => {
            return Err(ManifestError::UnsupportedVersion(
                u8::try_from(other.unwrap_or(0)).unwrap_or(u8::MAX),
            ));
        }
    };

    // Normalize each field independently: host permissions are URL patterns and
    // must never be conflated with the API permissions the host gates on.
    let permissions = normalized_list(&root, "permissions");
    let host_permissions = normalized_list(&root, "host_permissions");

    let mut background_scripts = Vec::new();
    let mut service_worker = None;
    if let Some(background) = root.get("background").and_then(Value::as_object) {
        if let Some(scripts) = background.get("scripts").and_then(Value::as_array) {
            background_scripts.extend(scripts.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        service_worker = background
            .get("service_worker")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }

    let gecko_id = ["browser_specific_settings", "applications"]
        .iter()
        .filter_map(|key| root.get(*key))
        .filter_map(|settings| settings.get("gecko"))
        .filter_map(|gecko| gecko.get("id"))
        .find_map(Value::as_str)
        .map(str::to_owned);

    let chrome_key = root.get("key").and_then(Value::as_str).map(str::to_owned);

    let action_title = ["action", "browser_action"]
        .iter()
        .filter_map(|key| root.get(*key))
        .filter_map(|action| action.get("default_title"))
        .find_map(Value::as_str)
        .map(str::to_owned);

    let icons = match root.get("icons").and_then(Value::as_object) {
        Some(entries) => entries
            .iter()
            .filter_map(|(size, path)| path.as_str().map(|path| (size.clone(), path.to_owned())))
            .collect(),
        None => HashMap::new(),
    };

    Ok(Manifest {
        version,
        name,
        ext_version,
        permissions,
        host_permissions,
        background_scripts,
        service_worker,
        gecko_id,
        chrome_key,
        action_title,
        icons,
    })
}

pub fn flavor(m: &Manifest) -> BrowserFlavor {
    if m.gecko_id.is_some() {
        BrowserFlavor::Gecko
    } else if m.chrome_key.is_some() {
        BrowserFlavor::Chrome
    } else {
        BrowserFlavor::Generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHROME_MV3: &str = r#"{
        "manifest_version": 3,
        "name": "Chrome Ext",
        "version": "1.2.3",
        "key": "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A",
        "permissions": ["Storage", "tabs", 7, null],
        "host_permissions": ["https://EXAMPLE.com/*", "https://example.com/*"],
        "background": { "service_worker": "sw.js" },
        "action": { "default_title": "Chrome Action" },
        "icons": { "48": "icon48.png", "128": "icon128.png", "bad": 5 },
        "unknown_field": 42
    }"#;

    const GECKO_MV2: &str = r#"{
        "manifest_version": 2,
        "name": "Gecko Ext",
        "version": "0.1",
        "browser_specific_settings": { "gecko": { "id": "ext@example.com" } },
        "browser_action": { "default_title": "Gecko Action" },
        "background": { "scripts": ["bg.js", "polyfill.js"] },
        "permissions": ["bookmarks"]
    }"#;

    const LEGACY_APPLICATIONS: &str = r#"{
        "manifest_version": 2,
        "name": "Legacy Ext",
        "version": "1.0",
        "applications": { "gecko": { "id": "legacy@example.com" } }
    }"#;

    const MIXED_MV2: &str = r#"{
        "manifest_version": 2,
        "name": "Mixed Ext",
        "version": "1.0",
        "background": {
            "scripts": ["legacy.js"],
            "service_worker": "modern.js"
        }
    }"#;

    #[test]
    fn chrome_mv3_parses_chrome_fields() {
        let m = parse(CHROME_MV3).expect("valid chrome manifest");
        assert_eq!(m.version, ManifestVersion::V3);
        assert_eq!(m.name, "Chrome Ext");
        assert_eq!(m.ext_version, "1.2.3");
        assert_eq!(m.service_worker.as_deref(), Some("sw.js"));
        assert!(m.background_scripts.is_empty());
        assert_eq!(m.action_title.as_deref(), Some("Chrome Action"));
        assert!(m.chrome_key.is_some());
        assert_eq!(m.gecko_id, None);
        assert_eq!(flavor(&m), BrowserFlavor::Chrome);
        assert_eq!(m.icons.len(), 2);
        assert_eq!(m.icons.get("48").map(String::as_str), Some("icon48.png"));
        assert_eq!(m.permissions, vec!["storage", "tabs"]);
        assert_eq!(m.host_permissions, vec!["https://example.com/*"]);
    }

    #[test]
    fn host_permissions_never_grant_api_namespaces() {
        let m = parse(
            r#"{"manifest_version":3,"name":"x","version":"1",
                "permissions":[],
                "host_permissions":["Storage","https://EXAMPLE.com/*","storage"]}"#,
        )
        .expect("valid manifest");
        assert!(m.permissions.is_empty());
        assert_eq!(m.host_permissions, vec!["storage", "https://example.com/*"]);
    }

    #[test]
    fn gecko_mv2_parses_gecko_fields() {
        let m = parse(GECKO_MV2).expect("valid gecko manifest");
        assert_eq!(m.version, ManifestVersion::V2);
        assert_eq!(m.gecko_id.as_deref(), Some("ext@example.com"));
        assert_eq!(m.action_title.as_deref(), Some("Gecko Action"));
        assert_eq!(m.background_scripts, vec!["bg.js", "polyfill.js"]);
        assert_eq!(m.service_worker, None);
        assert_eq!(m.chrome_key, None);
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn legacy_applications_key_is_accepted() {
        let m = parse(LEGACY_APPLICATIONS).expect("valid legacy manifest");
        assert_eq!(m.gecko_id.as_deref(), Some("legacy@example.com"));
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn mixed_mv2_background_keeps_scripts_and_service_worker() {
        let m = parse(MIXED_MV2).expect("valid mixed manifest");
        assert_eq!(m.version, ManifestVersion::V2);
        assert_eq!(m.background_scripts, vec!["legacy.js"]);
        assert_eq!(m.service_worker.as_deref(), Some("modern.js"));
    }

    #[test]
    fn missing_fields_and_bad_version_are_rejected() {
        assert!(matches!(
            parse(r#"{"manifest_version":2,"version":"1"}"#),
            Err(ManifestError::Missing("name"))
        ));
        assert!(matches!(
            parse(r#"{"manifest_version":2,"name":"x"}"#),
            Err(ManifestError::Missing("version"))
        ));
        assert!(matches!(
            parse(r#"{"name":"x","version":"1"}"#),
            Err(ManifestError::UnsupportedVersion(0))
        ));
        assert!(matches!(
            parse(r#"{"manifest_version":4,"name":"x","version":"1"}"#),
            Err(ManifestError::UnsupportedVersion(4))
        ));
        assert!(matches!(parse("not json"), Err(ManifestError::Json(_))));
    }

    #[test]
    fn flavor_is_generic_without_browser_markers() {
        let m = parse(r#"{"manifest_version":3,"name":"x","version":"1"}"#)
            .expect("valid generic manifest");
        assert_eq!(flavor(&m), BrowserFlavor::Generic);
        assert!(m.permissions.is_empty());
        assert!(m.icons.is_empty());
    }
}

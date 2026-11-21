//! Chrome and Gecko WebExtension manifest parsing (MV2 and MV3).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use hodo_ext_i18n::{I18nError, Messages};
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
    /// Required when the extension ships a `_locales` directory.
    pub default_locale: Option<String>,
    /// Kept so `__MSG_` tokens can be substituted (CSS/HTML are NOT touched).
    pub description: Option<String>,
    /// `name` after `__MSG_` resolution; set by [`resolve_messages`].
    pub resolved_name: Option<String>,
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
    let default_locale = root
        .get("default_locale")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let description = root
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);

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
        default_locale,
        description,
        resolved_name: None,
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

/// Resolve `__MSG_name__` tokens in `name`, `description` and `action_title`.
/// Called by the host/loader once the extension root is known; `parse` stays
/// root-free and cheap. CSS is deliberately not rewritten, and HTML is only
/// localized explicitly through `i18n.getMessage` in the JS bridge.
pub fn resolve_messages(root: &Path, manifest: &mut Manifest) -> Result<(), I18nError> {
    resolve_messages_with_ui(root, manifest, &hodo_ext_i18n::ui_locale())
}

/// Same as [`resolve_messages`] with an explicit UI locale (tests, `--lang`).
pub fn resolve_messages_with_ui(
    root: &Path,
    manifest: &mut Manifest,
    ui_locale: &str,
) -> Result<(), I18nError> {
    let messages =
        hodo_ext_i18n::load_locales(root, manifest.default_locale.as_deref(), ui_locale)?
            .with_extension_id(manifest.gecko_id.as_deref().unwrap_or(""));
    manifest.name = substitute_messages(&manifest.name, &messages);
    manifest.description = manifest
        .description
        .take()
        .map(|text| substitute_messages(&text, &messages));
    manifest.action_title = manifest
        .action_title
        .take()
        .map(|text| substitute_messages(&text, &messages));
    manifest.resolved_name = Some(manifest.name.clone());
    Ok(())
}

/// Single pass: text produced for a token is never rescanned.
fn substitute_messages(text: &str, messages: &Messages) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'_' && text[i..].starts_with("__MSG_") {
            if let Some(offset) = text[i + 6..].find("__") {
                let end = i + 6 + offset + 2;
                let name = &text[i + 6..end - 2];
                if !name.is_empty()
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@')
                {
                    out.push_str(&messages.get_message(name, &[]));
                    i = end;
                    continue;
                }
            }
        }
        match text[i..].chars().next() {
            Some(ch) => {
                out.push(ch);
                i += ch.len_utf8();
            }
            None => break,
        }
    }
    out
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

    #[test]
    fn parse_keeps_default_locale_and_description() {
        let m = parse(
            r#"{"manifest_version":3,"name":"x","version":"1",
                "default_locale":"uk-UA","description":"desc"}"#,
        )
        .expect("valid manifest");
        assert_eq!(m.default_locale.as_deref(), Some("uk-UA"));
        assert_eq!(m.description.as_deref(), Some("desc"));
        assert_eq!(m.resolved_name, None);
    }

    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "hodo-manifest-i18n-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn write_extension(&self, manifest: &str, messages: &str) {
            fs::write(self.0.join("manifest.json"), manifest).expect("write manifest");
            let locale = self.0.join("_locales").join("en");
            fs::create_dir_all(&locale).expect("create locale dir");
            fs::write(locale.join("messages.json"), messages).expect("write messages");
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const MESSAGES: &str = r#"{
        "appName": { "message": "Localized Name" },
        "appDesc": { "message": "Localized Description" }
    }"#;

    #[test]
    fn msg_tokens_resolve_in_chrome_mv3_manifest() {
        let dir = TempDir::new();
        let manifest = r#"{
            "manifest_version": 3,
            "name": "__MSG_appName__",
            "description": "__MSG_appDesc__",
            "version": "1.0",
            "default_locale": "en",
            "action": { "default_title": "__MSG_appName__" }
        }"#;
        dir.write_extension(manifest, MESSAGES);

        let mut m = parse(manifest).expect("valid manifest");
        assert_eq!(m.name, "__MSG_appName__"); // parse stays raw
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve");
        assert_eq!(m.name, "Localized Name");
        assert_eq!(m.resolved_name.as_deref(), Some("Localized Name"));
        assert_eq!(m.description.as_deref(), Some("Localized Description"));
        assert_eq!(m.action_title.as_deref(), Some("Localized Name"));
    }

    #[test]
    fn msg_tokens_resolve_in_gecko_mv2_manifest() {
        let dir = TempDir::new();
        let manifest = r#"{
            "manifest_version": 2,
            "name": "__MSG_appName__",
            "description": "__MSG_appDesc__",
            "version": "1.0",
            "default_locale": "en",
            "applications": { "gecko": { "id": "gecko@example.com" } },
            "browser_action": { "default_title": "__MSG_appName__" }
        }"#;
        dir.write_extension(manifest, MESSAGES);

        let mut m = parse(manifest).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "en").expect("resolve");
        assert_eq!(m.name, "Localized Name");
        assert_eq!(m.description.as_deref(), Some("Localized Description"));
        assert_eq!(m.action_title.as_deref(), Some("Localized Name"));
        assert_eq!(flavor(&m), BrowserFlavor::Gecko);
    }

    #[test]
    fn i18n_gecko_demo_resolves_to_i18n_demo() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/i18n-gecko");
        let json = fs::read_to_string(root.join("manifest.json")).expect("demo manifest");
        let mut m = parse(&json).expect("valid demo manifest");
        resolve_messages_with_ui(&root, &mut m, "en").expect("resolve demo");
        assert_eq!(m.name, "i18n Demo");
        assert_eq!(m.resolved_name.as_deref(), Some("i18n Demo"));
        assert_eq!(m.action_title.as_deref(), Some("i18n Demo"));
    }

    #[test]
    fn resolve_without_locales_leaves_plain_fields_untouched() {
        let dir = TempDir::new();
        let manifest = r#"{"manifest_version":3,"name":"Plain Name","version":"1"}"#;
        fs::write(dir.0.join("manifest.json"), manifest).expect("write manifest");
        let mut m = parse(manifest).expect("valid manifest");
        resolve_messages_with_ui(&dir.0, &mut m, "de-DE").expect("resolve");
        assert_eq!(m.name, "Plain Name");
        assert_eq!(m.resolved_name.as_deref(), Some("Plain Name"));
    }
}

//! In-process extension host with permission gating (sandbox process = roadmap).
//!
//! Owns one shared API registry, the injectable clock used by timers/alarms and
//! the live background contexts. MV2 contexts stay persistent; idle MV3 service
//! workers are unloaded by [`Host::tick`] and woke again by [`Host::emit_event`]
//! (their state lives in `storage`, not in JS globals).

use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use hodo_ext_api::{
    canonical_namespace, ApiCall, ApiDialect, ApiRegistry, ApiResult, Clock, RuntimeConfig,
    SystemClock,
};
use hodo_ext_manifest::{Manifest, ManifestVersion};
use serde_json::{json, Value};

/// Same cap as `hodo_net::MAX_BYTES`; a manifest or script is never allowed to grow unbounded.
const MAX_MANIFEST_BYTES: usize = hodo_types::limits::MAX_MANIFEST_BYTES;
/// MV3 service workers unload after 30 seconds without events or API calls.
const IDLE_UNLOAD_MS: u64 = 30_000;

#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: Manifest,
    pub root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("manifest error: {0}")]
    Manifest(#[from] hodo_ext_manifest::ManifestError),
    #[error("csp: {0}")]
    Csp(String),
    #[error("io: {0}")]
    Io(String),
}

pub struct Host {
    extensions: Vec<Extension>,
    /// Shared with every background-script runtime so JS effects land here too.
    registry: Rc<RefCell<ApiRegistry>>,
    clock: Rc<dyn Clock>,
    /// `None` for MV2 or for an unloaded MV3 service worker.
    contexts: Vec<Option<hodo_js::ExtScriptRuntime>>,
    last_activity: Vec<u64>,
    diagnostics: RefCell<Vec<String>>,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            registry: Rc::new(RefCell::new(ApiRegistry::new())),
            clock: Rc::new(SystemClock),
            contexts: Vec::new(),
            last_activity: Vec::new(),
            diagnostics: RefCell::new(Vec::new()),
        }
    }
}

impl Host {
    pub fn new() -> Self {
        Self::default()
    }

    /// Host with an injected clock: tests drive timers/alarms/idle without sleeping.
    #[must_use]
    pub fn with_clock(clock: Rc<dyn Clock>) -> Self {
        Self {
            clock,
            ..Self::default()
        }
    }

    /// Load an unpacked extension directory containing `manifest.json`.
    pub fn load_dir(&mut self, path: &Path) -> Result<&Extension, HostError> {
        let file = fs::File::open(path.join("manifest.json"))
            .map_err(|err| HostError::Io(err.to_string()))?;
        let mut bytes = Vec::new();
        let read = file
            .take(MAX_MANIFEST_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|err| HostError::Io(err.to_string()))?;
        if read > MAX_MANIFEST_BYTES {
            return Err(HostError::Io("manifest too large".into()));
        }
        let json = String::from_utf8(bytes).map_err(|err| HostError::Io(err.to_string()))?;
        let manifest = hodo_ext_manifest::parse(&json)?;
        validate_csp(&json, manifest.version)?;

        // A missing background script is cosmetic: warn, but still load.
        for script in manifest
            .background_scripts
            .iter()
            .chain(manifest.service_worker.iter())
        {
            if !path.join(script).exists() {
                eprintln!(
                    "hodo-ext-host: warning: background script '{script}' not found in {}",
                    path.display()
                );
            }
        }

        self.extensions.push(Extension {
            manifest,
            root: path.to_path_buf(),
        });
        self.contexts.push(None);
        self.last_activity.push(self.clock.now_ms());
        Ok(self.extensions.last().expect("just pushed an extension"))
    }

    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Whether the MV3 service worker is currently resident.
    #[must_use]
    pub fn context_is_live(&self, ext_index: usize) -> bool {
        self.contexts.get(ext_index).is_some_and(Option::is_some)
    }

    /// Dispatch only if the extension's manifest declares the API namespace
    /// (`runtime` is always allowed). `fetch` uses host permissions instead.
    pub fn dispatch(&mut self, ext_index: usize, call: &ApiCall) -> ApiResult {
        if let ApiCall::Fetch { url, method, body } = call {
            return self.fetch(ext_index, url, method, body);
        }
        let Some(ext) = self.extensions.get(ext_index) else {
            return ApiResult::Err("no such extension".into());
        };
        let ns = Self::required_namespace(call);
        let allowed = ns == "runtime" || ext.manifest.permissions.iter().any(|p| p == ns);
        if !allowed {
            return ApiResult::Err(format!(
                "permission denied: {ns} requires manifest permission"
            ));
        }
        let call = match call {
            ApiCall::AlarmCreate {
                name,
                delay_ms,
                period_ms,
                ..
            } => ApiCall::AlarmCreate {
                name: name.clone(),
                delay_ms: *delay_ms,
                period_ms: *period_ms,
                now_ms: self.clock.now_ms(),
            },
            other => other.clone(),
        };
        self.touch(ext_index);
        self.registry.borrow_mut().dispatch(&call)
    }

    /// Evaluates the extension's background scripts (MV2) / service worker (MV3)
    /// against the host's registry. Also drains the new event loop, so callback
    /// and Promise chains settle before this returns.
    ///
    /// # Errors
    /// Returns a short error when a script is unreadable, too large, or throws.
    pub fn run_background_scripts(&mut self, ext_index: usize) -> Result<String, String> {
        let ext = self
            .extensions
            .get(ext_index)
            .ok_or_else(|| "no such extension".to_string())?;
        let scripts: Vec<&str> = ext
            .manifest
            .background_scripts
            .iter()
            .map(String::as_str)
            .chain(ext.manifest.service_worker.iter().map(String::as_str))
            .collect();

        let mut sources = Vec::new();
        for script in scripts {
            let path = ext.root.join(script);
            // Missing files are cosmetic; `load_dir` already warns about them.
            if !path.is_file() {
                continue;
            }
            let mut bytes = Vec::new();
            let read = fs::File::open(&path)
                .and_then(|file| {
                    file.take(MAX_MANIFEST_BYTES as u64 + 1)
                        .read_to_end(&mut bytes)
                })
                .map_err(|err| format!("background script {script}: {err}"))?;
            if read > MAX_MANIFEST_BYTES {
                return Err("background script too large".into());
            }
            sources.push(
                String::from_utf8(bytes)
                    .map_err(|err| format!("background script {script}: {err}"))?,
            );
        }
        if sources.is_empty() {
            return Ok("no background scripts".into());
        }

        let config = self.runtime_config(ext_index);
        let mut runtime = hodo_js::ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::clone(&self.registry),
            Rc::clone(&self.clock),
        );
        let fetch_ext = self.extensions[ext_index].clone();
        runtime.set_fetch_hook(Rc::new(move |call| match call {
            ApiCall::Fetch { url, method, body } => fetch_for(&fetch_ext, url, method, body),
            other => ApiResult::Err(format!("unexpected hook call: {other:?}")),
        }));
        let result = runtime
            .eval(&sources.join("\n"))
            .map_err(|err| err.lines().next().unwrap_or("js error").to_string());
        self.drain_logs(&mut runtime);
        if result.is_ok() {
            self.contexts[ext_index] = Some(runtime);
            self.touch(ext_index);
        }
        result
    }

    /// Drive every live context: alarms, queued API tasks, timers, microtasks
    /// and MV3 idle unload. Returns the amount of work performed.
    pub fn tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        self.registry.borrow_mut().advance_clock(now);
        let mut work = 0;
        let mut logs = Vec::new();
        for index in 0..self.contexts.len() {
            let Some(runtime) = self.contexts[index].as_mut() else {
                continue;
            };
            work += runtime.tick();
            logs.extend(runtime.take_logs());
            let idle = self.contexts[index]
                .as_ref()
                .is_some_and(|_| now.saturating_sub(self.last_activity[index]) > IDLE_UNLOAD_MS);
            if idle && self.is_mv3(index) {
                self.contexts[index] = None;
            }
        }
        self.diagnostics.borrow_mut().extend(logs);
        work
    }

    /// Alias for [`Host::tick`] used by the shell's event loop.
    pub fn poll(&mut self) -> usize {
        self.tick()
    }

    /// Wake a (possibly unloaded) context and deliver one event to its
    /// listeners. MV3 contexts reload from the script; persistent state lives
    /// in `storage`, so a wake keeps prior results.
    pub fn emit_event(&mut self, ext_index: usize, name: &str, payload: Value) {
        if !self.context_is_live(ext_index) {
            let _ = self.run_background_scripts(ext_index);
        }
        self.touch(ext_index);
        if let Some(runtime) = self.contexts.get_mut(ext_index).and_then(Option::as_mut) {
            runtime.fire_event(name, payload);
            runtime.tick();
            let logs = runtime.take_logs();
            for line in logs {
                eprintln!("hodo-ext-host: {line}");
                self.diagnostics.borrow_mut().push(line);
            }
        }
    }

    fn touch(&mut self, ext_index: usize) {
        if let Some(last) = self.last_activity.get_mut(ext_index) {
            *last = self.clock.now_ms();
        }
    }

    fn is_mv3(&self, ext_index: usize) -> bool {
        self.extensions.get(ext_index).is_some_and(|ext| {
            ext.manifest.version == ManifestVersion::V3 && ext.manifest.service_worker.is_some()
        })
    }

    fn runtime_config(&self, ext_index: usize) -> RuntimeConfig {
        let ext = &self.extensions[ext_index];
        let allowed: Vec<String> = ext
            .manifest
            .permissions
            .iter()
            .filter_map(|p| canonical_namespace(p).map(str::to_string))
            .collect();
        RuntimeConfig {
            dialect: match ext.manifest.version {
                ManifestVersion::V2 => ApiDialect::ChromeMv2,
                ManifestVersion::V3 => ApiDialect::ChromeMv3,
            },
            extension_name: ext.manifest.name.clone(),
            allowed_namespaces: allowed,
            host_permissions: ext.manifest.host_permissions.clone(),
            idle_unload_ms: self.is_mv3(ext_index).then_some(IDLE_UNLOAD_MS),
        }
    }

    fn drain_logs(&self, runtime: &mut hodo_js::ExtScriptRuntime) {
        for line in runtime.take_logs() {
            eprintln!("hodo-ext-host: {line}");
            self.diagnostics.borrow_mut().push(line);
        }
    }

    /// Drain diagnostic lines (unhandled rejections, callback errors, JS
    /// `console.error` calls) collected from all contexts.
    pub fn take_diagnostics(&self) -> Vec<String> {
        std::mem::take(&mut self.diagnostics.borrow_mut())
    }

    fn fetch(&mut self, ext_index: usize, url: &str, method: &str, body: &str) -> ApiResult {
        let Some(ext) = self.extensions.get(ext_index).cloned() else {
            return ApiResult::Err("no such extension".into());
        };
        self.touch(ext_index);
        fetch_for(&ext, url, method, body)
    }

    pub fn required_namespace(call: &ApiCall) -> &'static str {
        match call {
            ApiCall::TabsCreate { .. }
            | ApiCall::TabsQueryActive
            | ApiCall::TabsList
            | ApiCall::TabsActivate { .. } => "tabs",
            ApiCall::StorageSet { .. }
            | ApiCall::StorageGet { .. }
            | ApiCall::StorageSyncSet { .. }
            | ApiCall::StorageSyncGet { .. }
            | ApiCall::StorageSessionSet { .. }
            | ApiCall::StorageSessionGet { .. } => "storage",
            ApiCall::AlarmCreate { .. } | ApiCall::AlarmClear { .. } => "alarms",
            ApiCall::Fetch { .. } => "fetch",
            ApiCall::NotificationsCreate { .. } => "notifications",
            ApiCall::SendMessage { .. } => "runtime",
        }
    }
}

/// Shared by direct host dispatch and the per-runtime JS fetch hook, so the
/// permission check cannot be bypassed by script-originated calls.
fn fetch_for(ext: &Extension, url: &str, method: &str, body: &str) -> ApiResult {
    if !url_allowed(ext, url) {
        return ApiResult::Err("fetch denied: host permission required".into());
    }
    let method = method.to_ascii_uppercase();
    if method != "GET" && method != "POST" {
        return ApiResult::Err(format!("fetch: unsupported method {method}"));
    }
    if body.len() > hodo_net::MAX_BYTES || url.len() > hodo_net::MAX_BYTES {
        return ApiResult::Err("fetch: request exceeds 2 MiB cap".into());
    }
    match fetch_bytes(url, &method, body) {
        Ok((status, bytes)) => {
            let text = bytes
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>();
            ApiResult::Ok(
                json!({ "ok": (200..300).contains(&status), "status": status, "body": text }),
            )
        }
        Err(err) => ApiResult::Err(format!("fetch: {err}")),
    }
}

/// Never weaken the manifest's CSP below the MV3/MV2 defaults.
fn validate_csp(json: &str, version: ManifestVersion) -> Result<(), HostError> {
    let root: Value = serde_json::from_str(json).map_err(|err| HostError::Csp(err.to_string()))?;
    let Some(csp) = root.get("content_security_policy").and_then(Value::as_str) else {
        return Ok(());
    };
    let lower = csp.to_ascii_lowercase();
    for token in [
        "'unsafe-eval'",
        "'unsafe-inline'",
        "'wasm-unsafe-eval'",
        "blob:",
        "data:",
        "http:",
        "https:",
    ] {
        if lower.contains(token) {
            return Err(HostError::Csp(format!("forbidden CSP token {token}")));
        }
    }
    if !lower.contains("'self'") && !lower.contains("'none'") {
        return Err(HostError::Csp(
            "script sources must be 'self' or 'none'".into(),
        ));
    }
    if version == ManifestVersion::V2 && !lower.contains("object-src") {
        return Err(HostError::Csp("MV2 requires object-src 'self'".into()));
    }
    Ok(())
}

/// Match a URL against `host_permissions` (or a `<all_urls>` permission).
/// Credentials in the URL are always rejected.
fn url_allowed(ext: &Extension, url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https" | "file") || url.contains('@') {
        return false;
    }
    let host = rest.split('/').next().unwrap_or("");
    let path = rest.find('/').map_or("", |index| &rest[index..]);
    ext.manifest
        .host_permissions
        .iter()
        .chain(ext.manifest.permissions.iter())
        .any(|pattern| {
            if pattern == "<all_urls>" {
                return matches!(scheme, "http" | "https");
            }
            let Some((pattern_scheme, pattern_rest)) = pattern.split_once("://") else {
                return false;
            };
            if pattern_scheme != "*" && pattern_scheme != scheme {
                return false;
            }
            let pattern_host = pattern_rest.split('/').next().unwrap_or("");
            let host_ok = if let Some(suffix) = pattern_host.strip_prefix("*.") {
                host == suffix || host.ends_with(&format!(".{suffix}"))
            } else {
                pattern_host == "*" || pattern_host.eq_ignore_ascii_case(host)
            };
            if !host_ok {
                return false;
            }
            let pattern_path = pattern_rest
                .find('/')
                .map_or("", |index| &pattern_rest[index..]);
            path.starts_with(pattern_path.trim_end_matches('*'))
        })
}

/// GET goes through the frozen `hodo_net::load` (2 MiB cap, shared agent).
/// POST uses the same pinned ureq version until hodo-net exposes a body API.
fn fetch_bytes(url: &str, method: &str, body: &str) -> Result<(u16, Vec<u8>), String> {
    if method == "GET" {
        // `file:///` + `/C:/...` is a Windows path shape hodo-net cannot open;
        // hand it `file://C:/...` instead (still no host-side path escaping).
        let load_url = if let Some(rest) = url.strip_prefix("file:///") {
            let bytes = rest.as_bytes();
            if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                format!("file://{rest}")
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        };
        let resource = hodo_net::load(&load_url).map_err(|err| err.to_string())?;
        let bytes = match resource {
            hodo_net::Resource::Html(text) => text.into_bytes(),
            hodo_net::Resource::Other { data, .. } => data,
        };
        return Ok((200, bytes));
    }
    let mut response = ureq::post(url)
        .send(body.to_string())
        .map_err(|err| err.to_string())?;
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = response
            .body_mut()
            .as_reader()
            .read(&mut chunk)
            .map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > hodo_net::MAX_BYTES {
            return Err("resource exceeds cap".into());
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    Ok((status, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_ext_api::FakeClock;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    fn repo_ext(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../extensions")
            .join(name)
    }

    fn storage_get(host: &mut Host, key: &str) -> Option<String> {
        match host.dispatch(0, &ApiCall::StorageGet { key: key.into() }) {
            ApiResult::Ok(value) => value.as_str().map(str::to_owned),
            ApiResult::Err(err) => panic!("StorageGet({key}) failed: {err}"),
        }
    }

    struct TempExt {
        root: PathBuf,
    }

    impl TempExt {
        fn new(manifest: &str) -> Self {
            let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("hodo-ext-host-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("manifest.json"), manifest).unwrap();
            Self { root }
        }
    }

    impl Drop for TempExt {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    const CHROME_STORAGE: &str = r#"{
        "manifest_version": 3,
        "name": "Chrome Storage",
        "version": "1.0.0",
        "permissions": ["storage"],
        "background": { "service_worker": "missing-sw.js" }
    }"#;

    const GECKO_TABS: &str = r#"{
        "manifest_version": 2,
        "name": "Gecko Tabs",
        "version": "1.0.0",
        "permissions": ["tabs", "storage"],
        "browser_specific_settings": { "gecko": { "id": "gecko@example.test" } },
        "background": { "scripts": ["missing-bg.js"] }
    }"#;

    const NO_PERMISSIONS: &str = r#"{
        "manifest_version": 3,
        "name": "No Permissions",
        "version": "1.0.0",
        "permissions": []
    }"#;

    const NO_PERMISSIONS_BG: &str = r#"{
        "manifest_version": 3,
        "name": "No Permissions Background",
        "version": "1.0.0",
        "permissions": [],
        "background": { "scripts": ["bg.js"] }
    }"#;

    const HOST_PERMISSION_ONLY: &str = r#"{
        "manifest_version": 3,
        "name": "Host Permission Only",
        "version": "1.0.0",
        "host_permissions": ["storage"],
        "permissions": []
    }"#;

    const NOTIFICATIONS_ONLY: &str = r#"{
        "manifest_version": 3,
        "name": "Notifications Only",
        "version": "1.0.0",
        "permissions": ["notifications"]
    }"#;

    fn assert_ok(result: ApiResult, label: &str) {
        if let ApiResult::Err(msg) = result {
            panic!("{label}: expected Ok, got Err({msg})");
        }
    }

    fn assert_err_contains(result: ApiResult, needle: &str, label: &str) {
        match result {
            ApiResult::Err(msg) => assert!(
                msg.contains(needle),
                "{label}: expected '{msg}' to contain '{needle}'"
            ),
            ApiResult::Ok(_) => panic!("{label}: expected Err, got Ok"),
        }
    }

    #[test]
    fn chrome_storage_allows_storage_but_denies_tabs() {
        let tmp = TempExt::new(CHROME_STORAGE);
        let mut host = Host::new();
        host.load_dir(&tmp.root).expect("chrome ext should load");
        assert_eq!(host.extensions().len(), 1);

        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_ok(host.dispatch(0, &set), "StorageSet");

        let tabs = ApiCall::TabsCreate {
            url: "https://example.test".into(),
        };
        assert_err_contains(host.dispatch(0, &tabs), "permission denied", "TabsCreate");
    }

    #[test]
    fn gecko_tabs_and_storage_both_allowed() {
        let tmp = TempExt::new(GECKO_TABS);
        let mut host = Host::new();
        host.load_dir(&tmp.root).expect("gecko ext should load");

        let tabs = ApiCall::TabsCreate {
            url: "https://example.test".into(),
        };
        assert_ok(host.dispatch(0, &tabs), "TabsCreate");
        assert_ok(
            host.dispatch(0, &ApiCall::TabsQueryActive),
            "TabsQueryActive",
        );

        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_ok(host.dispatch(0, &set), "StorageSet");
        assert_ok(
            host.dispatch(0, &ApiCall::StorageGet { key: "k".into() }),
            "StorageGet",
        );
    }

    #[test]
    fn runtime_always_allowed_without_permissions() {
        let tmp = TempExt::new(NO_PERMISSIONS);
        let mut host = Host::new();
        host.load_dir(&tmp.root).expect("no-perm ext should load");

        let msg = ApiCall::SendMessage {
            message: Default::default(),
        };
        assert_ok(host.dispatch(0, &msg), "SendMessage");
    }

    #[test]
    fn host_permissions_do_not_grant_api_access() {
        let tmp = TempExt::new(HOST_PERMISSION_ONLY);
        let mut host = Host::new();
        host.load_dir(&tmp.root).expect("ext should load");
        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_err_contains(
            host.dispatch(0, &set),
            "permission denied",
            "StorageSet via host_permissions",
        );
    }

    #[test]
    fn notifications_permission_allows_notifications_but_denies_storage() {
        let tmp = TempExt::new(NOTIFICATIONS_ONLY);
        let mut host = Host::new();
        host.load_dir(&tmp.root)
            .expect("notifications ext should load");

        assert_ok(
            host.dispatch(
                0,
                &ApiCall::NotificationsCreate {
                    title: "t".into(),
                    message: "m".into(),
                },
            ),
            "NotificationsCreate",
        );
        assert_err_contains(
            host.dispatch(
                0,
                &ApiCall::StorageSet {
                    key: "k".into(),
                    value: Default::default(),
                },
            ),
            "permission denied",
            "StorageSet with notifications-only permission",
        );
    }

    #[test]
    fn oversized_manifest_is_rejected() {
        let tmp = TempExt::new("");
        fs::write(
            tmp.root.join("manifest.json"),
            vec![b' '; MAX_MANIFEST_BYTES + 1],
        )
        .unwrap();
        let mut host = Host::new();
        assert!(matches!(
            host.load_dir(&tmp.root),
            Err(HostError::Io(msg)) if msg == "manifest too large"
        ));
    }

    #[test]
    fn bad_index_is_an_error() {
        let mut host = Host::new();
        let msg = ApiCall::SendMessage {
            message: Default::default(),
        };
        assert_err_contains(host.dispatch(0, &msg), "no such extension", "bad index");
    }

    #[test]
    fn malformed_manifest_surfaces_manifest_error() {
        let tmp = TempExt::new("{ this is not valid json");
        let mut host = Host::new();
        assert!(matches!(
            host.load_dir(&tmp.root),
            Err(HostError::Manifest(_)) | Err(HostError::Csp(_))
        ));
    }

    #[test]
    fn weaker_csp_is_rejected() {
        let manifest = r#"{
            "manifest_version": 3, "name": "Weak CSP", "version": "1.0.0",
            "content_security_policy": "script-src 'self' 'unsafe-eval' https://cdn.example"
        }"#;
        let tmp = TempExt::new(manifest);
        let mut host = Host::new();
        assert!(matches!(host.load_dir(&tmp.root), Err(HostError::Csp(_))));

        let ok = r#"{
            "manifest_version": 3, "name": "Strong CSP", "version": "1.0.0",
            "content_security_policy": "script-src 'self'; object-src 'none'"
        }"#;
        let tmp_ok = TempExt::new(ok);
        assert!(Host::new().load_dir(&tmp_ok.root).is_ok());
    }

    #[test]
    fn required_namespace_mapping() {
        assert_eq!(
            Host::required_namespace(&ApiCall::StorageSet {
                key: "k".into(),
                value: Default::default(),
            }),
            "storage"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::StorageGet { key: "k".into() }),
            "storage"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::TabsCreate { url: "u".into() }),
            "tabs"
        );
        assert_eq!(Host::required_namespace(&ApiCall::TabsQueryActive), "tabs");
        assert_eq!(
            Host::required_namespace(&ApiCall::SendMessage {
                message: Default::default()
            }),
            "runtime"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::Fetch {
                url: "u".into(),
                method: "GET".into(),
                body: String::new()
            }),
            "fetch"
        );
    }

    #[test]
    fn real_hello_gecko_background_writes_visible_storage() {
        let mut host = Host::new();
        host.load_dir(&repo_ext("hello-gecko"))
            .expect("hello-gecko should load");
        host.run_background_scripts(0)
            .expect("hello-gecko bg.js should evaluate");

        // StorageChanged events are delivered to JS listeners during `tick`,
        // so the registry queue is empty by the time the host looks at it.
        assert!(host.registry.borrow_mut().drain_events().is_empty());
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
    }

    #[test]
    fn real_hello_chrome_service_worker_writes_visible_storage() {
        let mut host = Host::new();
        host.load_dir(&repo_ext("hello-chrome"))
            .expect("hello-chrome should load");
        host.run_background_scripts(0)
            .expect("hello-chrome bg.js should evaluate");
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello from a Chrome-style MV3 extension")
        );
    }

    #[test]
    fn permission_filter_blocks_unpermitted_js_namespace() {
        let tmp = TempExt::new(NO_PERMISSIONS_BG);
        fs::write(
            tmp.root.join("bg.js"),
            "chrome.storage.local.set({ blocked: 1 });",
        )
        .unwrap();
        let mut host = Host::new();
        host.load_dir(&tmp.root).expect("ext should load");

        let err = host
            .run_background_scripts(0)
            .expect_err("storage must be filtered out of the JS globals");
        assert!(
            err.contains("TypeError") || err.contains("undefined"),
            "unexpected error: {err}"
        );
        assert!(host.registry.borrow_mut().drain_events().is_empty());
        let probe = host.registry.borrow_mut().dispatch(&ApiCall::StorageGet {
            key: "blocked".into(),
        });
        match probe {
            ApiResult::Ok(value) => assert!(value.is_null()),
            ApiResult::Err(err) => panic!("unexpected registry error: {err}"),
        }
    }

    #[test]
    fn running_background_twice_replays_events_without_accumulating() {
        let mut host = Host::new();
        host.load_dir(&repo_ext("hello-gecko")).unwrap();
        host.run_background_scripts(0).unwrap();
        let first = host.registry.borrow_mut().drain_events();
        host.run_background_scripts(0).unwrap();
        let second = host.registry.borrow_mut().drain_events();

        assert!(first.is_empty());
        assert!(second.is_empty());
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
    }

    #[test]
    fn no_background_scripts_is_not_an_error() {
        let tmp = TempExt::new(NO_PERMISSIONS);
        let mut host = Host::new();
        host.load_dir(&tmp.root).unwrap();
        assert_eq!(
            host.run_background_scripts(0).unwrap(),
            "no background scripts"
        );
    }

    #[test]
    fn all_bundled_demo_extensions_execute() {
        let demos = [
            "hello-chrome",
            "hello-gecko",
            "i18n-gecko",
            "messages-gecko",
            "notify-chrome",
            "popup-chrome",
            "storage-demo-gecko",
            "tabs-demo-chrome",
            "async-chrome",
            "promises-gecko",
        ];
        let mut host = Host::new();
        for name in demos {
            host.load_dir(&repo_ext(name))
                .unwrap_or_else(|err| panic!("{name}: {err}"));
            let index = host.extensions().len() - 1;
            host.run_background_scripts(index)
                .unwrap_or_else(|err| panic!("{name}: {err}"));
            host.tick();
        }
        assert_eq!(host.extensions().len(), 10);
    }

    #[test]
    fn mv3_idle_unloads_and_event_wakes_context() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        assert!(host.context_is_live(0));
        assert_eq!(storage_get(&mut host, "mode").as_deref(), Some("async"));

        clock.set_ms(IDLE_UNLOAD_MS + 1);
        host.tick();
        assert!(!host.context_is_live(0), "idle MV3 worker must unload");

        host.emit_event(0, "probe", json!({"wake": true}));
        assert!(
            host.context_is_live(0),
            "subscribed event must wake the worker"
        );
        assert_eq!(storage_get(&mut host, "mode").as_deref(), Some("async"));
    }

    #[test]
    fn mv2_context_stays_persistent_when_idle() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("hello-gecko")).unwrap();
        host.run_background_scripts(0).unwrap();
        clock.set_ms(IDLE_UNLOAD_MS * 10);
        host.tick();
        assert!(
            host.context_is_live(0),
            "MV2 background pages are persistent"
        );
    }

    #[test]
    fn alarms_drain_on_tick_without_sleeping() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        host.dispatch(
            0,
            &ApiCall::AlarmCreate {
                name: "tick".into(),
                delay_ms: 5,
                period_ms: None,
                now_ms: 0,
            },
        );
        clock.set_ms(5);
        host.tick();
        match host.dispatch(
            0,
            &ApiCall::StorageGet {
                key: "lastAlarm".into(),
            },
        ) {
            ApiResult::Ok(value) => assert_eq!(value, json!("tick")),
            ApiResult::Err(err) => panic!("alarm listener did not run: {err}"),
        }
    }

    #[test]
    fn fetch_denied_without_host_permission() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "No Fetch", "version": "1.0.0",
                "permissions": [],
                "background": { "service_worker": "bg.js" }
            }"#,
        );
        fs::write(
            tmp.root.join("bg.js"),
            "fetch('https://example.test/data.json').then(function (r) { __hodo_log('unexpected'); }, function (e) { __hodo_log('denied:' + e.message); });",
        )
        .unwrap();
        let mut host = Host::new();
        host.load_dir(&tmp.root).unwrap();
        host.run_background_scripts(0).unwrap();
        host.tick();
        let logs = host.take_diagnostics();
        assert!(
            logs.iter().any(|line| line.contains("fetch denied")),
            "unexpected logs: {logs:?}"
        );
        assert_err_contains(
            host.dispatch(
                0,
                &ApiCall::Fetch {
                    url: "https://example.test/".into(),
                    method: "GET".into(),
                    body: String::new(),
                },
            ),
            "fetch denied",
            "direct fetch dispatch",
        );
    }

    #[test]
    fn fetch_allowed_with_host_permission_reads_local_file() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "Fetch File", "version": "1.0.0",
                "host_permissions": ["file:///*"],
                "background": { "service_worker": "bg.js" }
            }"#,
        );
        let data_path = tmp.root.join("data.txt");
        fs::write(&data_path, "hello from fetch").unwrap();
        let url = format!("file:///{}", data_path.to_string_lossy().replace('\\', "/"));
        let script = format!(
            "fetch('{url}').then(function (r) {{ return r.text(); }}).then(function (t) {{ __hodo_log('text:' + t); }}, function (e) {{ __hodo_log('err:' + e.message); }});"
        );
        fs::write(tmp.root.join("bg.js"), script).unwrap();
        let mut host = Host::new();
        host.load_dir(&tmp.root).unwrap();
        host.run_background_scripts(0).unwrap();
        host.tick();
        let logs = host.take_diagnostics();
        assert!(
            logs.iter()
                .any(|line| line.contains("text:hello from fetch")),
            "unexpected logs: {logs:?}"
        );
    }

    /// Manual timing probe (not part of the fast suite): `cargo test -p
    /// hodo-ext-host api_roundtrip_timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn api_roundtrip_timing() {
        use std::time::Instant;

        const N: u32 = 1000;
        let mut host = Host::new();
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        let mut runtime = hodo_js::ExtScriptRuntime::with_allowed_and_registry(
            &["storage"],
            Rc::clone(&host.registry),
        );

        let started = Instant::now();
        for i in 0..N {
            runtime
                .eval(&format!("browser.storage.local.set({{roundtrip: {i}}});"))
                .unwrap();
        }
        let js_each = started.elapsed().as_secs_f64() * 1e6 / f64::from(N);

        let started = Instant::now();
        for i in 0..N {
            assert_ok(
                host.dispatch(
                    0,
                    &ApiCall::StorageSet {
                        key: "roundtrip".into(),
                        value: json!(i),
                    },
                ),
                "timed dispatch",
            );
        }
        let host_each = started.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!(
            "API roundtrip: JS eval+tick {js_each:.1} us/op, host dispatch {host_each:.1} us/op"
        );
    }

    #[test]
    fn url_permission_matching_rejects_credentials_and_wrong_hosts() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "Match", "version": "1.0.0",
                "host_permissions": ["https://example.com/*"]
            }"#,
        );
        let mut host = Host::new();
        host.load_dir(&tmp.root).unwrap();
        let ext = host.extensions()[0].clone();
        assert!(url_allowed(&ext, "https://example.com/a"));
        assert!(!url_allowed(&ext, "https://example.com.evil.test/a"));
        assert!(!url_allowed(&ext, "https://user:pass@example.com/a"));
        assert!(!url_allowed(&ext, "file:///C:/secret.txt"));
    }
}

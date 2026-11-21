//! In-process extension host with permission gating (sandbox process = roadmap).

use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use hodo_ext_api::{ApiCall, ApiRegistry, ApiResult};
use hodo_ext_manifest::Manifest;

/// Same cap as `hodo_net::MAX_BYTES`; a manifest is never allowed to grow unbounded.
const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: Manifest,
    pub root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("manifest error: {0}")]
    Manifest(#[from] hodo_ext_manifest::ManifestError),
    #[error("io: {0}")]
    Io(String),
}

#[derive(Debug, Default)]
pub struct Host {
    extensions: Vec<Extension>,
    /// Shared with every background-script runtime so JS effects land here too.
    registry: Rc<RefCell<ApiRegistry>>,
}

impl Host {
    pub fn new() -> Self {
        Self::default()
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
        Ok(self.extensions.last().expect("just pushed an extension"))
    }

    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Dispatch only if the extension's manifest declares the API namespace
    /// (`runtime` is always allowed).
    pub fn dispatch(&mut self, ext_index: usize, call: &ApiCall) -> ApiResult {
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
        self.registry.borrow_mut().dispatch(call)
    }

    /// Evaluates the extension's background scripts (MV2) / service worker (MV3)
    /// against the host's registry, returning the last JS expression's value.
    ///
    /// Only namespaces declared in `permissions` are injected into the JS
    /// globals, so the per-call permission gate above is deliberately not
    /// re-applied to JS-originated calls — the filter already removed them.
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

        let allowed: Vec<&str> = ext
            .manifest
            .permissions
            .iter()
            .map(String::as_str)
            .collect();
        // The JS namespace filter is the permission gate for these scripts.
        let mut runtime = hodo_js::ExtScriptRuntime::with_allowed_and_registry(
            &allowed,
            Rc::clone(&self.registry),
        );
        runtime
            .eval(&sources.join("\n"))
            .map_err(|err| err.lines().next().unwrap_or("js error").to_string())
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
            | ApiCall::StorageSyncGet { .. } => "storage",
            ApiCall::NotificationsCreate { .. } => "notifications",
            ApiCall::SendMessage { .. } => "runtime",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_ext_api::ExtEvent;
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
        // Missing service worker is cosmetic: the extension still loads.
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
            Err(HostError::Manifest(_))
        ));
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
    }

    #[test]
    fn real_hello_gecko_background_writes_visible_storage() {
        let mut host = Host::new();
        host.load_dir(&repo_ext("hello-gecko"))
            .expect("hello-gecko should load");
        host.run_background_scripts(0)
            .expect("hello-gecko bg.js should evaluate");

        let events = host.registry.borrow_mut().drain_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, ExtEvent::StorageChanged { key } if key == "greeting")),
            "expected a StorageChanged event for 'greeting', got {events:?}"
        );
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

        assert_eq!(first.len(), 2);
        assert_eq!(second.len(), 2);
        assert!(second
            .iter()
            .any(|e| matches!(e, ExtEvent::StorageChanged { key } if key == "greeting")));
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
}

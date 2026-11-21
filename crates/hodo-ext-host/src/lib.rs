//! In-process extension host with permission gating (sandbox process = roadmap).

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

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
    registry: ApiRegistry,
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
        self.registry.dispatch(call)
    }

    pub fn required_namespace(call: &ApiCall) -> &'static str {
        match call {
            ApiCall::TabsCreate { .. } | ApiCall::TabsQueryActive => "tabs",
            ApiCall::StorageSet { .. } | ApiCall::StorageGet { .. } => "storage",
            ApiCall::SendMessage { .. } => "runtime",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

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

    const HOST_PERMISSION_ONLY: &str = r#"{
        "manifest_version": 3,
        "name": "Host Permission Only",
        "version": "1.0.0",
        "host_permissions": ["storage"],
        "permissions": []
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
}

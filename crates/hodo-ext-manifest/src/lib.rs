//! Chrome and Gecko WebExtension manifest parsing (MV2 and MV3).

use std::collections::HashMap;

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

/// Accepts both Chrome-style and Gecko-style WebExtension manifests.
pub fn parse(json: &str) -> Result<Manifest, ManifestError> {
    todo!("parse({})", json)
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

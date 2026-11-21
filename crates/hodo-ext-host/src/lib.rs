//! In-process extension host with permission gating (sandbox process = roadmap).

use std::path::{Path, PathBuf};

use hodo_ext_api::{ApiCall, ApiRegistry, ApiResult};
use hodo_ext_manifest::Manifest;

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
        todo!("load_dir({})", path.display())
    }

    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Dispatch only if the extension's manifest declares the API namespace
    /// (`runtime` is always allowed).
    pub fn dispatch(&mut self, ext_index: usize, call: &ApiCall) -> ApiResult {
        todo!("dispatch({}, {:?})", ext_index, call)
    }

    pub fn required_namespace(call: &ApiCall) -> &'static str {
        match call {
            ApiCall::TabsCreate { .. } | ApiCall::TabsQueryActive => "tabs",
            ApiCall::StorageSet { .. } | ApiCall::StorageGet { .. } => "storage",
            ApiCall::SendMessage { .. } => "runtime",
        }
    }
}

//! Resource loading with a hard 2 MiB cap.

#[derive(Debug)]
pub enum Resource {
    Html(String),
    Other { data: Vec<u8>, mime: String },
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("invalid url")]
    InvalidUrl,
    #[error("network: {0}")]
    Network(String),
    #[error("resource exceeds cap")]
    TooLarge,
    #[error("io: {0}")]
    Io(String),
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),
}

pub const MAX_BYTES: usize = 2 * 1024 * 1024;

/// http(s) via an HTTP client; local paths and file:// via `std::fs`; 2 MiB cap.
pub fn load(url: &str) -> Result<Resource, FetchError> {
    todo!("load({})", url)
}

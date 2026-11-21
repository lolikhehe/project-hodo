//! Resource loading with a hard 2 MiB cap.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

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
    if url.is_empty() {
        return Err(FetchError::InvalidUrl);
    }
    if url.starts_with("data:") {
        return Err(FetchError::UnsupportedScheme("data".to_string()));
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return load_http(url);
    }
    if let Some(path) = url.strip_prefix("file://") {
        return load_path(Path::new(path));
    }
    load_path(Path::new(url))
}

/// One shared HTTP client for all requests instead of rebuilding it each time.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .into()
    })
}

/// Content types may carry parameters and arbitrary casing (e.g. `Text/HTML; charset=utf-8`).
fn is_html_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .eq_ignore_ascii_case("text/html")
}

/// Decode UTF-8 without copying when the bytes are already valid.
fn decode_utf8(data: &[u8]) -> String {
    std::str::from_utf8(data)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::from_utf8_lossy(data).into_owned())
}

fn load_http(url: &str) -> Result<Resource, FetchError> {
    let mut res = agent()
        .get(url)
        .call()
        .map_err(|e| FetchError::Network(e.to_string()))?;

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    // Dropping the reader on TooLarge closes the connection without buffering the rest.
    let data = read_capped(res.body_mut().as_reader(), |e| {
        FetchError::Network(e.to_string())
    })?;

    if is_html_content_type(&content_type) {
        Ok(Resource::Html(decode_utf8(&data)))
    } else {
        Ok(Resource::Other {
            data,
            mime: content_type,
        })
    }
}

fn load_path(path: &Path) -> Result<Resource, FetchError> {
    let file = File::open(path).map_err(|e| FetchError::Io(e.to_string()))?;
    let data = read_capped(file, |e| FetchError::Io(e.to_string()))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "html" || ext == "htm" {
        Ok(Resource::Html(decode_utf8(&data)))
    } else {
        Ok(Resource::Other {
            data,
            mime: extension_mime(&ext).to_string(),
        })
    }
}

fn extension_mime(ext: &str) -> &'static str {
    match ext {
        "css" => "text/css",
        "js" => "text/javascript",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn read_capped<R: Read>(
    mut reader: R,
    map_err: impl Fn(std::io::Error) -> FetchError,
) -> Result<Vec<u8>, FetchError> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader.read(&mut chunk).map_err(&map_err)?;
        if n == 0 {
            break;
        }
        if data.len() + n > MAX_BYTES {
            return Err(FetchError::TooLarge);
        }
        data.extend_from_slice(&chunk[..n]);
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const HTML: &str = "<!doctype html><html><body>hodo</body></html>";
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n";

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("hodo-net-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn loads_local_and_file_urls() {
        let dir = temp_dir();
        let html_path = dir.join("page.html");
        fs::write(&html_path, HTML).unwrap();

        match load(html_path.to_string_lossy().as_ref()).unwrap() {
            Resource::Html(s) => assert_eq!(s, HTML),
            other => panic!("expected Html from local path, got {:?}", other),
        }

        let file_url = format!("file://{}", html_path.to_string_lossy());
        match load(file_url.as_str()).unwrap() {
            Resource::Html(s) => assert_eq!(s, HTML),
            other => panic!("expected Html from file:// url, got {:?}", other),
        }

        let png_path = dir.join("pixel.png");
        fs::write(&png_path, PNG).unwrap();
        match load(png_path.to_string_lossy().as_ref()).unwrap() {
            Resource::Other { data, mime } => {
                assert_eq!(data, PNG);
                assert_eq!(mime, "image/png");
            }
            other => panic!("expected Other from .png, got {:?}", other),
        }

        let big_path = dir.join("big.bin");
        fs::write(&big_path, vec![0u8; MAX_BYTES + 1]).unwrap();
        assert!(matches!(
            load(big_path.to_string_lossy().as_ref()),
            Err(FetchError::TooLarge)
        ));

        assert!(matches!(
            load("data:text/html,hello"),
            Err(FetchError::UnsupportedScheme(s)) if s == "data"
        ));
        assert!(matches!(load(""), Err(FetchError::InvalidUrl)));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn html_content_type_is_case_insensitive() {
        for ct in [
            "text/html",
            "Text/HTML",
            "TEXT/HTML; charset=utf-8",
            " text/html ",
        ] {
            assert!(is_html_content_type(ct), "input: {ct}");
        }
        assert!(!is_html_content_type("application/xhtml+xml"));
        assert!(!is_html_content_type("text/plain"));
    }

    #[test]
    fn decode_utf8_handles_valid_and_invalid_bytes() {
        assert_eq!(decode_utf8(b"hodo"), "hodo");
        assert!(!decode_utf8(&[0xff, 0xfe]).is_empty());
    }

    #[test]
    #[ignore = "requires live network; run manually"]
    fn loads_example_com() {
        match load("https://example.com") {
            Ok(Resource::Html(_)) => {}
            other => panic!("expected Html, got {:?}", other),
        }
    }
}

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
///
/// ureq's `gzip` feature is enabled by default (we keep default features), so
/// `Content-Encoding: gzip` responses are transparently decompressed before the
/// body is read — no manual decompression is needed here. `max_redirects` is set
/// explicitly; reaching the cap returns a `TooManyRedirects` error (mapped to
/// `FetchError::Network`) instead of following a redirect loop forever.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .max_redirects(5)
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

/// Sniff a byte prefix as HTML (HTML5 "text/html" rule): skip leading ASCII
/// whitespace, then look for a case-insensitive `<!doctype html` or `<html`.
/// Only the first 512 bytes are considered.
pub(crate) fn looks_like_html(prefix: &[u8]) -> bool {
    const WS: &[u8] = b" \t\n\r\x0c";
    let prefix = &prefix[..prefix.len().min(512)];
    let start = prefix
        .iter()
        .position(|b| !WS.contains(b))
        .unwrap_or(prefix.len());
    let tag = &prefix[start..];
    starts_with_ascii_ci(tag, b"<!doctype html") || starts_with_ascii_ci(tag, b"<html")
}

fn starts_with_ascii_ci(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.len() >= needle.len()
        && haystack[..needle.len()]
            .iter()
            .zip(needle)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// Decode UTF-8 without copying, stripping a leading UTF-8 BOM if present.
fn decode_utf8(data: &[u8]) -> String {
    let data = data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data);
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
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty());

    // Fail fast on a declared oversized body before streaming it. ureq strips
    // Content-Length when it transparently decompresses gzip, so this guard only
    // sees the uncompressed size; read_capped still caps the decompressed bytes.
    let declared_len = res
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<usize>().ok());
    if declared_len.is_some_and(|len| len > MAX_BYTES) {
        return Err(FetchError::TooLarge);
    }

    // Dropping the reader on TooLarge closes the connection without buffering the rest.
    let data = read_capped(res.body_mut().as_reader(), |e| {
        FetchError::Network(e.to_string())
    })?;

    let is_html = match &content_type {
        Some(ct) => is_html_content_type(ct),
        None => looks_like_html(&data),
    };
    if is_html {
        Ok(Resource::Html(decode_utf8(&data)))
    } else {
        Ok(Resource::Other {
            data,
            mime: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
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
        // Unique per test: tests run in parallel and must not share a directory.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("hodo-net-test-{}-{n}", std::process::id()));
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
    fn sniffs_html_prefix() {
        assert!(looks_like_html(b"<!doctype html><html>"));
        assert!(looks_like_html(b"<html lang=\"en\">"));
        // leading whitespace (space/tab/newline/CR/form-feed) and mixed case
        assert!(looks_like_html(b" \t\r\n\x0c<!DOCTYPE HTML>"));
        assert!(looks_like_html(b"\n\n<Html>"));
        // not HTML
        assert!(!looks_like_html(b"just plain text"));
        assert!(!looks_like_html(&[0x89, b'P', b'N', b'G', 0x00, 0x01]));
        assert!(!looks_like_html(b""));
        // a tag past the 512-byte sniff window is ignored
        let mut far = vec![b' '; 512];
        far.extend_from_slice(b"<html>");
        assert!(!looks_like_html(&far));
    }

    #[test]
    fn strips_utf8_bom_from_local_html() {
        let dir = temp_dir();
        let path = dir.join("bom.html");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(HTML.as_bytes());
        fs::write(&path, &bytes).unwrap();

        match load(path.to_string_lossy().as_ref()).unwrap() {
            Resource::Html(s) => assert_eq!(s, HTML, "BOM should be stripped"),
            other => panic!("expected Html, got {:?}", other),
        }

        fs::remove_dir_all(&dir).unwrap();
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

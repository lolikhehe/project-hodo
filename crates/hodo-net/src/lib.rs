//! Resource loading with a hard 2 MiB cap, plus an opt-in RFC 9111 cache path.
//!
//! [`load`] is unchanged/frozen (always uncached). [`load_with`] is the new,
//! opt-in API: it uses [`hodo_cache::DiskCache`] for private-browser caching.
//! URL normalization is hand-written; no `url` crate is needed.
//!
//! [`fetch`] is the privacy-aware entry point: tracking-parameter stripping,
//! adblock, HSTS, HTTPS upgrade, double-keyed cookies and warnings.

mod privacy;

pub use privacy::{
    adblock_blocked_count, allow_site, blocked_count, blocked_urls, clear_hsts, cookie_count,
    fetch, hsts_hosts, is_site_allowed, privacy_stats, revoke_site, set_adblock_enabled,
    strip_tracking_params, warned_urls, warning_count, FetchOutcome, PrivacyStats, Warning,
};

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use hodo_adblock::BlockDecision;
use hodo_cache::{
    CacheEntry, CacheKey, Clock, DiskCache, MappedEntry, SystemClock, STREAM_CHUNK_LEN,
};

pub use hodo_cache::{cache_stats, CacheStats};

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
    /// A resolved hop pointed at a URL already seen in this chain.
    #[error("network: too many redirects (cycle: {0})")]
    RedirectLoop(String),
    #[error("resource exceeds cap")]
    TooLarge,
    #[error("io: {0}")]
    Io(String),
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("offline: cache miss")]
    Offline,
    #[error("duplicate in-flight request for the same key")]
    InFlight,
    #[error("https upgrade failed for {url}")]
    HttpsUpgradeFailed { url: String },
    #[error("blocked by filter: {rule}")]
    Blocked { rule: String },
}

pub const MAX_BYTES: usize = 2 * 1024 * 1024;

/// Options for the opt-in cached loader.
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// `false` = behave like the frozen uncached loader.
    pub use_cache: bool,
    /// Cache-only mode: a miss (or stale entry) is an error, never a fetch.
    pub offline: bool,
    /// Top-frame site (or extension id); part of the cache key, prevents
    /// cross-site timing leaks.
    pub partition: String,
    /// `true` when the embedding document is HTTPS (used to flag mixed content).
    pub secure_context: bool,
    /// Registrable domain of the top-level site (empty = top-level navigation).
    pub top_site: String,
    /// Referrer URL; only an origin-only header is ever sent, and only same-origin.
    pub referer: Option<String>,
    /// Private tab: no cookie writes, no HSTS, no cache reads or writes.
    pub private: bool,
    /// Remove known tracking query parameters (default `true`).
    pub strip_tracking: bool,
    /// Disk-cached HTTP(S) misses only: stream the response body straight into
    /// a cache temp file in 64 KiB chunks (RAM peak = one chunk), then serve
    /// the hit through hodo-cache's mmap path. `false` keeps the frozen
    /// buffer-then-put behaviour. file:// and non-cacheable responses are
    /// unchanged. The final `Resource` is still body-sized by necessity.
    pub stream_to_cache: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            use_cache: true,
            offline: false,
            partition: String::new(),
            secure_context: false,
            top_site: String::new(),
            referer: None,
            private: false,
            strip_tracking: true,
            stream_to_cache: false,
        }
    }
}

/// Load a resource without using the cache (frozen behaviour).
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

// Connection warm-up audit (ureq 3.5.2, registry source `src/config.rs` +
// `src/pool.rs`): an `Agent` always owns a `ConnectionPool`; HTTP/1.1
// keep-alive is on by default and there is no on/off switch. Defaults are
// `max_idle_connections = 10`, `max_idle_connections_per_host = 3`,
// `max_idle_age = 15s` (Config::default, config.rs:1088-1090). Requests that
// change transport-level settings (user-agent, buffer sizes, TLS/proxy config)
// bypass the pool (Config::can_share_pool_with). We set the three pool values
// explicitly on both shared agents so the warm-up contract is pinned here and
// survives a ureq upgrade; our requests use only agent-level settings, so they
// stay pool-compatible. Pooled connections are reused across `load_with` calls.

/// One shared HTTP client for all uncached requests.
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
            .max_idle_connections(10)
            .max_idle_connections_per_host(3)
            .max_idle_age(Duration::from_secs(15))
            .build()
            .into()
    })
}

/// Cached path uses an agent with redirects disabled: 301/308 must be visible
/// so they can be stored as entries; redirects are followed by hand instead.
fn cache_agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .max_redirects(0)
            .max_idle_connections(10)
            .max_idle_connections_per_host(3)
            .max_idle_age(Duration::from_secs(15))
            .build()
            .into()
    })
}

fn is_html_content_type(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .eq_ignore_ascii_case("text/html")
}

/// Sniff a byte prefix as HTML (HTML5 "text/html" rule).
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

fn decode_utf8(data: &[u8]) -> String {
    let data = data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data);
    std::str::from_utf8(data)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| String::from_utf8_lossy(data).into_owned())
}

/// Accepts either an owned `Vec<u8>` (cold paths: moved, no copy) or a borrowed
/// `&[u8]` (mmap warm hits: HTML decodes straight from mapped pages; binary
/// resources still need one copy into the owned `Resource::Other`).
fn resource_from<D: AsRef<[u8]> + Into<Vec<u8>>>(
    data: D,
    content_type: Option<&str>,
    url: &str,
) -> Resource {
    let bytes: &[u8] = data.as_ref();
    let mime = content_type
        .filter(|ct| !ct.trim().is_empty())
        .unwrap_or("application/octet-stream");
    let is_html = if mime != "application/octet-stream" {
        is_html_content_type(mime)
    } else {
        looks_like_html(bytes) || url.ends_with(".html") || url.ends_with(".htm")
    };
    if is_html {
        Resource::Html(decode_utf8(bytes))
    } else {
        Resource::Other {
            data: data.into(),
            mime: mime.to_string(),
        }
    }
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

    let declared_len = res
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<usize>().ok());
    if declared_len.is_some_and(|len| len > MAX_BYTES) {
        return Err(FetchError::TooLarge);
    }

    let data = read_capped(res.body_mut().as_reader(), |e| {
        FetchError::Network(e.to_string())
    })?;
    Ok(resource_from(data, content_type.as_deref(), url))
}

fn load_path(path: &Path) -> Result<Resource, FetchError> {
    let file = File::open(path).map_err(|e| FetchError::Io(e.to_string()))?;
    let data = read_capped(file, |e| FetchError::Io(e.to_string()))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = extension_mime(&ext);
    Ok(resource_from(data, Some(mime), &path.to_string_lossy()))
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

// ---------------------------------------------------------------------------
// URL normalization (hand-written, no url crate)
// ---------------------------------------------------------------------------

/// Normalize a URL for use as a cache-key component: strip fragment, lowercase
/// scheme/host, drop default ports, resolve dot segments, uppercase percent
/// escapes and decode unreserved escapes. Path traversal sequences can never
/// reach the filesystem because cache paths are hash-only.
pub fn normalize_url(url: &str) -> Result<String, FetchError> {
    if url.is_empty() {
        return Err(FetchError::InvalidUrl);
    }
    let Some((scheme, rest)) = url.split_once("://") else {
        if url.starts_with("data:") {
            return Err(FetchError::UnsupportedScheme("data".into()));
        }
        // Bare local path: treat as file:// for the cache key.
        return Ok(format!("file://{}", strip_fragment(url)));
    };
    let scheme = scheme.to_ascii_lowercase();
    match scheme.as_str() {
        "http" | "https" => normalize_http(&scheme, rest),
        "file" => Ok(format!("file://{}", strip_fragment(rest))),
        other => Err(FetchError::UnsupportedScheme(other.to_string())),
    }
}

fn strip_fragment(value: &str) -> &str {
    value.split('#').next().unwrap_or(value)
}

fn normalize_http(scheme: &str, rest: &str) -> Result<String, FetchError> {
    let rest = strip_fragment(rest);
    let (authority, tail) = match rest.find('/') {
        Some(pos) => (&rest[..pos], &rest[pos..]),
        None => (rest, ""),
    };
    let (path, query) = match tail.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (tail, None),
    };
    let authority = normalize_authority(authority, scheme);
    if authority.is_empty() {
        return Err(FetchError::InvalidUrl);
    }
    let path = remove_dot_segments(&percent_normalize(&path.replace('\\', "/")));
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        path
    };
    let mut out = format!("{scheme}://{authority}{path}");
    if let Some(query) = query {
        out.push('?');
        out.push_str(query);
    }
    Ok(out)
}

fn normalize_authority(authority: &str, scheme: &str) -> String {
    let (userinfo, hostport) = match authority.rsplit_once('@') {
        Some((u, h)) => (Some(u), h),
        None => (None, authority),
    };
    // Bracketed IPv6 literals keep their brackets; port follows the bracket.
    let (host, port) = if let Some(end) = hostport.find(']') {
        let port_start = end + 1;
        let port = hostport[port_start..].strip_prefix(':');
        (&hostport[..=end], port)
    } else {
        match hostport.rsplit_once(':') {
            Some((h, p)) if p.bytes().all(|b| b.is_ascii_digit()) => (h, Some(p)),
            _ => (hostport, None),
        }
    };
    let host = host.to_ascii_lowercase();
    let default =
        (scheme == "http" && port == Some("80")) || (scheme == "https" && port == Some("443"));
    let mut out = String::new();
    if let Some(userinfo) = userinfo {
        out.push_str(userinfo);
        out.push('@');
    }
    out.push_str(&host);
    if let Some(port) = port {
        if !default && !port.is_empty() {
            out.push(':');
            out.push_str(port);
        }
    }
    out
}

fn percent_normalize(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                let byte = hi * 16 + lo;
                if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                    out.push(byte as char);
                } else {
                    out.push('%');
                    out.push(ascii_hex(hi));
                    out.push(ascii_hex(lo));
                }
                i += 3;
                continue;
            }
        }
        // Input is &str, so copying the whole UTF-8 sequence is safe here.
        let ch = input[i..].chars().next().expect("non-empty UTF-8 slice");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn ascii_hex(value: u8) -> char {
    b"0123456789ABCDEF"[value as usize] as char
}

fn remove_dot_segments(path: &str) -> String {
    let absolute = path.starts_with('/');
    let trailing = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            _ => segments.push(segment),
        }
    }
    let mut out = String::new();
    if absolute {
        out.push('/');
    }
    out.push_str(&segments.join("/"));
    if trailing && !out.ends_with('/') {
        out.push('/');
    }
    out
}

/// Resolve a `Location` header against its request URL (absolute, scheme-relative,
/// absolute-path or relative), then normalize the result. Strips fragments.
pub fn resolve_location(base: &str, location: &str) -> Result<String, FetchError> {
    let location = strip_fragment(location);
    if location.contains("://") {
        return normalize_url(location);
    }
    let Some((scheme, rest)) = base.split_once("://") else {
        return Err(FetchError::InvalidUrl);
    };
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let resolved = if let Some(stripped) = location.strip_prefix("//") {
        format!("{scheme}://{stripped}")
    } else if location.starts_with('/') {
        format!("{scheme}://{authority}{location}")
    } else {
        // Resolve relative to the path after the authority, not the whole rest.
        // Cut the query/fragment first: a `/` inside a query value is not a
        // path separator.
        let tail = &rest[authority_end..];
        let path_part = tail.split(['?', '#']).next().unwrap_or(tail);
        let path = &path_part[..path_part.rfind('/').map(|p| p + 1).unwrap_or(0)];
        format!("{scheme}://{authority}{path}{location}")
    };
    normalize_url(&resolved)
}

// ---------------------------------------------------------------------------
// RFC 9111 header parsers — any malformed input degrades to "uncachable"
// ---------------------------------------------------------------------------

pub(crate) type Headers = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CacheControl {
    max_age: Option<u64>,
    no_store: bool,
    no_cache: bool,
    must_revalidate: bool,
    public: bool,
    malformed: bool,
}

fn parse_cache_control(value: Option<&str>) -> CacheControl {
    let mut out = CacheControl::default();
    let Some(value) = value else { return out };
    for directive in value.split(',') {
        let directive = directive.trim();
        if directive.is_empty() {
            continue;
        }
        let (name, argument) = match directive.split_once('=') {
            Some((n, a)) => (n.trim().to_ascii_lowercase(), Some(a.trim())),
            None => (directive.to_ascii_lowercase(), None),
        };
        match name.as_str() {
            "max-age" => match argument.and_then(|a| a.parse::<u64>().ok()) {
                Some(age) => out.max_age = Some(age),
                None => out.malformed = true,
            },
            "no-store" | "no-cache" | "must-revalidate" | "public"
                if !argument.is_some_and(|a| a.starts_with('"')) =>
            {
                match name.as_str() {
                    "no-store" => out.no_store = true,
                    "no-cache" => out.no_cache = true,
                    "must-revalidate" => out.must_revalidate = true,
                    _ => out.public = true,
                }
            }
            _ => {} // unknown extensions are ignored, not fatal
        }
    }
    out
}

/// Header names we can observe on our own requests; anything else in `Vary`
/// makes the response uncachable (MVP conservatism).
const SUPPORTED_VARY: &[&str] = &[
    "accept-encoding",
    "accept-language",
    "accept",
    "host",
    "user-agent",
    "origin",
    "referer",
];

#[derive(Debug, PartialEq, Eq)]
enum Vary {
    Absent,
    Headers(Vec<String>),
    Unsupported,
}

fn parse_vary(value: Option<&str>) -> Vary {
    let Some(value) = value else {
        return Vary::Absent;
    };
    let mut names = Vec::new();
    for token in value.split(',') {
        let name = token.trim().to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if name == "*" || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Vary::Unsupported;
        }
        if !SUPPORTED_VARY.contains(&name.as_str()) {
            return Vary::Unsupported;
        }
        names.push(name);
    }
    if names.is_empty() {
        Vary::Absent
    } else {
        Vary::Headers(names)
    }
}

fn vary_for(headers: &Headers, request_headers: &Headers) -> Option<BTreeMap<String, String>> {
    match parse_vary(headers.get("vary").map(String::as_str)) {
        Vary::Unsupported => None,
        Vary::Absent => Some(BTreeMap::new()),
        Vary::Headers(names) => Some(
            names
                .into_iter()
                .map(|name| {
                    let value = request_headers.get(&name).cloned().unwrap_or_default();
                    (name, value)
                })
                .collect(),
        ),
    }
}

fn vary_matches(stored: &BTreeMap<String, String>, current: &Headers) -> bool {
    stored.iter().all(|(name, value)| {
        current
            .get(name)
            .is_some_and(|actual| actual.eq_ignore_ascii_case(value))
    })
}

/// Accept weak ETags (`W/"opaque"`) and quoted opaque tags; reject everything
/// else so a malformed validator is never replayed as `If-None-Match`.
fn parse_etag(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let opaque = value.strip_prefix("W/").unwrap_or(value);
    if opaque.len() < 2 || !opaque.starts_with('"') || !opaque.ends_with('"') {
        return None;
    }
    let inner = &opaque[1..opaque.len() - 1];
    if inner.bytes().any(|b| b <= b' ' || b == 0x7f || b == b'"') {
        return None;
    }
    Some(value.to_string())
}

const MONTHS: &[&str] = &[
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// IMF-fixdate only (`Sun, 06 Nov 1994 08:49:37 GMT`); other HTTP-date forms
/// are deliberately treated as unparseable (documented RFC simplification).
fn parse_http_date(value: &str) -> Option<u64> {
    let clean = value.replace(',', " ");
    let parts: Vec<&str> = clean.split_whitespace().collect();
    if parts.len() != 6 || !parts[5].eq_ignore_ascii_case("gmt") {
        return None;
    }
    let day: u32 = parts[1].parse().ok()?;
    let month = MONTHS
        .iter()
        .position(|m| parts[2].eq_ignore_ascii_case(m))? as u32
        + 1;
    let year: i64 = parts[3].parse().ok()?;
    if !(1..=31).contains(&day) || !(1970..=9999).contains(&year) {
        return None;
    }
    let time: Vec<u32> = parts[4]
        .split(':')
        .map(str::parse::<u32>)
        .collect::<Result<_, _>>()
        .ok()?;
    if time.len() != 3 || time[0] > 23 || time[1] > 59 || time[2] > 59 {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(
        (days as u64) * 86_400 + (time[0] as u64) * 3600 + (time[1] as u64) * 60 + (time[2] as u64),
    )
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = i64::from(month) + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

// ---------------------------------------------------------------------------
// Freshness
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Freshness {
    /// Absolute Unix time when the entry goes stale; `None` = no TTL/validator.
    expires_at: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
}

/// All freshness math is clock-injected so tests can use a fake clock.
fn response_freshness(headers: &Headers, clock: &dyn Clock) -> Freshness {
    let now = clock.now_unix();
    let cc = parse_cache_control(headers.get("cache-control").map(String::as_str));
    let date = headers
        .get("date")
        .and_then(|v| parse_http_date(v))
        .unwrap_or(now);
    let age = headers
        .get("age")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let current_age = age.saturating_add(now.saturating_sub(date));
    let expires_at = if cc.no_cache {
        Some(0) // store, but always revalidate
    } else if let Some(max_age) = cc.max_age {
        Some(now.saturating_add(max_age.saturating_sub(current_age)))
    } else if let Some(expires) = headers.get("expires").and_then(|v| parse_http_date(v)) {
        let lifetime = expires.saturating_sub(date);
        Some(now.saturating_add(lifetime.saturating_sub(current_age)))
    } else if let Some(last_modified) = headers
        .get("last-modified")
        .and_then(|v| parse_http_date(v))
    {
        // Heuristic: 10% of the time since Last-Modified, capped at 24 hours.
        let heuristic = (date.saturating_sub(last_modified) / 10).min(86_400);
        Some(now.saturating_add(heuristic.saturating_sub(current_age)))
    } else {
        None
    };
    Freshness {
        expires_at,
        etag: parse_etag(headers.get("etag").map(String::as_str)),
        last_modified: headers.get("last-modified").cloned(),
    }
}

const CACHEABLE_STATUSES: &[u16] = &[200, 203, 204, 301, 308, 404, 410];

fn is_cacheable(status: u16, request_headers: &Headers, response_headers: &Headers) -> bool {
    if !CACHEABLE_STATUSES.contains(&status) {
        return false;
    }
    let cc = parse_cache_control(response_headers.get("cache-control").map(String::as_str));
    if cc.no_store || cc.malformed {
        return false;
    }
    if request_headers.contains_key("authorization") && !cc.public && !cc.must_revalidate {
        return false;
    }
    !matches!(
        parse_vary(response_headers.get("vary").map(String::as_str)),
        Vary::Unsupported
    )
}

/// Returns `(headers, set-cookie values)`. `Set-Cookie` is returned separately
/// so the privacy layer can hand each value to the cookie jar; it is never
/// cached nor replayed from a cache entry.
pub(crate) fn response_headers(res: &ureq::http::Response<ureq::Body>) -> (Headers, Vec<String>) {
    let mut map = Headers::new();
    let mut set_cookies = Vec::new();
    for (name, value) in res.headers().iter() {
        let name = name.as_str().to_ascii_lowercase();
        let value = value.to_str().unwrap_or("").to_string();
        if name == "set-cookie" {
            set_cookies.push(value);
            continue;
        }
        map.entry(name)
            .and_modify(|previous| {
                previous.push_str(", ");
                previous.push_str(&value);
            })
            .or_insert(value);
    }
    (map, set_cookies)
}

fn our_request_headers(url: &str) -> Headers {
    let mut headers = Headers::new();
    // ureq's gzip feature sends Accept-Encoding: gzip by default.
    headers.insert("accept-encoding".into(), "gzip".into());
    if let Some((_, rest)) = url.split_once("://") {
        let end = rest.find('/').unwrap_or(rest.len());
        headers.insert("host".into(), rest[..end].to_string());
    }
    headers
}

fn put_entry(
    key: &CacheKey,
    status: u16,
    headers: &Headers,
    data: &[u8],
    clock: &dyn Clock,
    cache: &mut DiskCache,
    request_headers: &Headers,
) {
    if !is_cacheable(status, request_headers, headers) {
        return;
    }
    let Some(vary) = vary_for(headers, request_headers) else {
        return;
    };
    let freshness = response_freshness(headers, clock);
    let entry = CacheEntry {
        url: key.url.clone(),
        content_type: headers
            .get("content-type")
            .cloned()
            .unwrap_or_else(|| "application/octet-stream".into()),
        data: data.to_vec(),
        status_code: status,
        expires_at: freshness.expires_at,
        etag: freshness.etag,
        last_modified: freshness.last_modified,
        vary,
        headers: headers.clone(),
        partition_key: key.partition.clone(),
        request_method: "GET".into(),
        stored_at: clock.now_unix(),
    };
    // A full cache or a failed write must not turn a successful fetch into an error.
    let _ = cache.put(key, &entry);
}

struct Fetched {
    status: u16,
    headers: Headers,
    data: Vec<u8>,
    final_url: String,
    /// (url, status, headers) of cached 301/308 hops.
    redirects: Vec<(String, u16, Headers)>,
    /// Set-Cookie values from all network responses in this fetch.
    set_cookies: Vec<String>,
    /// (URL of the responding hop, raw Strict-Transport-Security header).
    hsts: Vec<(String, String)>,
    /// Set when the body was written through to this cache key instead of
    /// being buffered in `data`.
    streamed_key: Option<CacheKey>,
}

/// Resource plus the privacy side effects observed while loading it.
pub(crate) struct Loaded {
    pub(crate) resource: Resource,
    pub(crate) set_cookies: Vec<String>,
    /// (URL of the hop that sent the header, raw HSTS value).
    pub(crate) hsts: Vec<(String, String)>,
    pub(crate) final_url: String,
}

const MAX_REDIRECTS: usize = 5;

/// When `HODO_DEBUG_REDIRECTS` is set, print every hop of the redirect chain
/// to stderr. Kept permanently: redirect loops are otherwise invisible.
fn debug_redirects() -> bool {
    std::env::var_os("HODO_DEBUG_REDIRECTS").is_some()
}

pub(crate) fn fetch_network(
    url: &str,
    request_headers: &Headers,
    // `Some(baseline)` forbids following a redirect that leaves that origin
    // (speculative prefetch uses this so no cross-origin request is ever made).
    allowed_origin: Option<&str>,
    private: bool,
) -> Result<Fetched, FetchError> {
    let mut current = url.to_string();
    let mut redirects = Vec::new();
    let mut set_cookies = Vec::new();
    let mut hsts = Vec::new();
    let trace = debug_redirects();
    // Seeded with the start URL so a self-redirect (A -> A) and a cycle
    // (A -> B -> A) both fail immediately instead of burning all five hops.
    let mut visited: Vec<String> = vec![current.clone()];
    for hop in 0..=MAX_REDIRECTS {
        if trace {
            eprintln!("[hodo-redirects] hop {hop}: GET {current}");
        }
        let mut req = cache_agent().get(&current);
        for (name, value) in request_headers {
            // `host` in request_headers belongs to the *original* URL. Sending
            // it on later hops would pin a cross-host redirect to the wrong
            // virtual host (www -> apex self-redirect loop). Let ureq set the
            // Host from `current` on every request.
            if name == "host" {
                continue;
            }
            req = req.header(name.as_str(), value.as_str());
        }
        let mut res = req.call().map_err(|e| FetchError::Network(e.to_string()))?;
        let status = res.status().as_u16();
        let (headers, cookies) = response_headers(&res);
        set_cookies.extend(cookies);
        if let Some(sts) = headers.get("strict-transport-security") {
            hsts.push((current.clone(), sts.clone()));
        }
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let location = headers
                .get("location")
                .cloned()
                .ok_or_else(|| FetchError::Network("redirect without Location".into()))?;
            if matches!(status, 301 | 308) {
                redirects.push((current.clone(), status, headers));
            }
            let (next, _) =
                privacy::upgrade_redirect(&resolve_location(&current, &location)?, private);
            if trace {
                eprintln!("[hodo-redirects] hop {hop}: {status} Location: {location} -> {next}");
            }
            if allowed_origin.is_some_and(|origin| !same_origin(origin, &next)) {
                return Err(FetchError::Network(
                    "refusing cross-origin redirect on same-origin-only fetch".into(),
                ));
            }
            if visited.contains(&next) {
                return Err(FetchError::RedirectLoop(next));
            }
            visited.push(next.clone());
            current = next;
            continue;
        }
        if trace {
            eprintln!("[hodo-redirects] hop {hop}: {status} final {current}");
        }
        let data = read_capped(res.body_mut().as_reader(), |e| {
            FetchError::Network(e.to_string())
        })?;
        return Ok(Fetched {
            status,
            headers,
            data,
            final_url: current,
            redirects,
            set_cookies,
            hsts,
            streamed_key: None,
        });
    }
    Err(FetchError::Network("too many redirects".into()))
}

/// HTTP-only variant of [`fetch_network`] for opt-in streaming writes: the
/// final body is piped into a cache temp file in `STREAM_CHUNK_LEN` pieces and
/// committed, so no body-sized buffer is ever allocated. Non-cacheable
/// responses fall back to the ordinary capped read. Redirects are handled
/// exactly like the buffered path.
fn fetch_network_streaming(
    url: &str,
    request_headers: &Headers,
    cache: &mut DiskCache,
    partition: &str,
    clock: &dyn Clock,
    private: bool,
) -> Result<Fetched, FetchError> {
    let mut current = url.to_string();
    let mut redirects = Vec::new();
    let mut set_cookies = Vec::new();
    let mut hsts = Vec::new();
    let trace = debug_redirects();
    let mut visited: Vec<String> = vec![current.clone()];
    for hop in 0..=MAX_REDIRECTS {
        if trace {
            eprintln!("[hodo-redirects] stream hop {hop}: GET {current}");
        }
        let mut req = cache_agent().get(&current);
        for (name, value) in request_headers {
            // `host` in request_headers belongs to the *original* URL. Sending
            // it on later hops would pin a cross-host redirect to the wrong
            // virtual host (www -> apex self-redirect loop). Let ureq set the
            // Host from `current` on every request.
            if name == "host" {
                continue;
            }
            req = req.header(name.as_str(), value.as_str());
        }
        let mut res = req.call().map_err(|e| FetchError::Network(e.to_string()))?;
        let status = res.status().as_u16();
        let (headers, cookies) = response_headers(&res);
        set_cookies.extend(cookies);
        if let Some(sts) = headers.get("strict-transport-security") {
            hsts.push((current.clone(), sts.clone()));
        }
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let location = headers
                .get("location")
                .cloned()
                .ok_or_else(|| FetchError::Network("redirect without Location".into()))?;
            if matches!(status, 301 | 308) {
                redirects.push((current.clone(), status, headers));
            }
            let next =
                privacy::upgrade_redirect(&resolve_location(&current, &location)?, private).0;
            if trace {
                eprintln!(
                    "[hodo-redirects] stream hop {hop}: {status} Location: {location} -> {next}"
                );
            }
            if visited.contains(&next) {
                return Err(FetchError::RedirectLoop(next));
            }
            visited.push(next.clone());
            current = next;
            continue;
        }
        if trace {
            eprintln!("[hodo-redirects] stream hop {hop}: {status} final {current}");
        }

        let declared_len = res
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<usize>().ok());
        if declared_len.is_some_and(|len| len > MAX_BYTES) {
            return Err(FetchError::TooLarge);
        }

        let store_vary = is_cacheable(status, request_headers, &headers)
            .then(|| vary_for(&headers, request_headers))
            .flatten();
        let Some(vary) = store_vary else {
            // Nothing safe to write through — use the ordinary capped read.
            let data = read_capped(res.body_mut().as_reader(), |e| {
                FetchError::Network(e.to_string())
            })?;
            return Ok(Fetched {
                status,
                headers,
                data,
                final_url: current,
                redirects,
                set_cookies,
                hsts,
                streamed_key: None,
            });
        };

        let freshness = response_freshness(&headers, clock);
        let entry = CacheEntry {
            url: current.clone(),
            content_type: headers
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".into()),
            data: Vec::new(),
            status_code: status,
            expires_at: freshness.expires_at,
            etag: freshness.etag,
            last_modified: freshness.last_modified,
            vary,
            headers: headers.clone(),
            partition_key: partition.to_string(),
            request_method: "GET".into(),
            stored_at: clock.now_unix(),
        };
        let final_url = current;
        let key = CacheKey::new("GET", &final_url, partition);
        let mut writer = cache
            .stream_put(&key, &entry)
            .map_err(|e| FetchError::Io(e.to_string()))?;
        let mut reader = res.body_mut().as_reader();
        let mut chunk = [0u8; STREAM_CHUNK_LEN];
        loop {
            let n = reader
                .read(&mut chunk)
                .map_err(|e| FetchError::Network(e.to_string()))?;
            if n == 0 {
                break;
            }
            writer.write(&chunk[..n]).map_err(|e| match e {
                hodo_cache::CacheError::TooLarge => FetchError::TooLarge,
                other => FetchError::Io(other.to_string()),
            })?;
        }
        writer.commit().map_err(|e| FetchError::Io(e.to_string()))?;
        return Ok(Fetched {
            status,
            headers,
            data: Vec::new(),
            final_url,
            redirects,
            set_cookies,
            hsts,
            streamed_key: Some(key),
        });
    }
    Err(FetchError::Network("too many redirects".into()))
}

enum Revalidated {
    /// Merged response headers plus the Set-Cookie values carried by the 304.
    NotModified(Headers, Vec<String>),
    Modified,
}

fn validator_headers(entry: &CacheEntry) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    if let Some(etag) = &entry.etag {
        headers.push(("if-none-match".into(), etag.clone()));
    }
    if let Some(last_modified) = &entry.last_modified {
        headers.push(("if-modified-since".into(), last_modified.clone()));
    }
    headers
}

/// RFC 9111 §4.3.4: a 304 updates metadata but never the stored body.
fn apply_not_modified(
    mut entry: CacheEntry,
    headers: &Headers,
    request_headers: &Headers,
    clock: &dyn Clock,
) -> CacheEntry {
    entry.merge_headers(headers, request_headers, clock);
    entry.expires_at = response_freshness(&entry.headers, clock).expires_at;
    entry
}

fn revalidate(
    url: &str,
    entry: &CacheEntry,
    request_headers: &Headers,
) -> Result<Revalidated, FetchError> {
    let mut req = cache_agent().get(url);
    for (name, value) in request_headers {
        req = req.header(name.as_str(), value.as_str());
    }
    for (name, value) in validator_headers(entry) {
        req = req.header(name.as_str(), value.as_str());
    }
    let res = req.call().map_err(|e| FetchError::Network(e.to_string()))?;
    if res.status().as_u16() == 304 {
        let (headers, set_cookies) = response_headers(&res);
        return Ok(Revalidated::NotModified(headers, set_cookies));
    }
    // MVP: a revalidation miss falls back to one normal fetch (two requests).
    drop(res);
    Ok(Revalidated::Modified)
}

fn serve_cached(
    first: CacheEntry,
    key: &CacheKey,
    cache: &mut DiskCache,
    clock: &dyn Clock,
) -> Option<Resource> {
    let mut current = key.clone();
    let mut entry = first;
    for _ in 0..=MAX_REDIRECTS {
        if !entry.is_fresh(clock) {
            return None;
        }
        if matches!(entry.status_code, 301 | 308) {
            let location = entry.headers.get("location")?;
            let next_url = resolve_location(&current.url, location).ok()?;
            current = CacheKey::new("GET", &next_url, &current.partition);
            entry = cache.get(&current)?;
            continue;
        }
        return Some(resource_from(
            entry.data,
            Some(&entry.content_type),
            &current.url,
        ));
    }
    None
}

/// mmap-backed twin of [`serve_cached`]: follows cached 301/308 chains without
/// materializing any body into a `Vec`. Parse callers still take `&[u8]`; the
/// mapping simply outlives the borrow.
fn serve_cached_mapped(
    first: MappedEntry,
    key: &CacheKey,
    cache: &mut DiskCache,
    clock: &dyn Clock,
) -> Option<Resource> {
    let mut current = key.clone();
    let mut entry = first;
    for _ in 0..=MAX_REDIRECTS {
        if !entry.is_fresh(clock) {
            return None;
        }
        let status = entry.metadata().status_code;
        if matches!(status, 301 | 308) {
            let location = entry.metadata().headers.get("location")?.clone();
            let next_url = resolve_location(&current.url, &location).ok()?;
            current = CacheKey::new("GET", &next_url, &current.partition);
            entry = cache.get_mapped(&current)?;
            continue;
        }
        let content_type = entry.metadata().content_type.clone();
        let url = current.url.clone();
        return Some(resource_from(
            entry.mapped_body(),
            Some(&content_type),
            &url,
        ));
    }
    None
}

/// MVP single-flight guard: an in-process set of in-flight keys. A duplicate
/// request gets [`FetchError::InFlight`] immediately; a full future-sharing
/// layer is deliberately not built. The `&mut DiskCache` API already serializes
/// requests within one process in practice.
struct PendingGuard(String);

fn pending_requests() -> &'static Mutex<HashSet<String>> {
    static PENDING: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(HashSet::new()))
}

impl PendingGuard {
    fn enter(key: &CacheKey) -> Option<Self> {
        let id = format!("{:016x}", key.hash());
        let mut pending = pending_requests().lock().unwrap_or_else(|e| e.into_inner());
        pending.insert(id.clone()).then_some(Self(id))
    }
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        if let Ok(mut pending) = pending_requests().lock() {
            pending.remove(&self.0);
        }
    }
}

/// Opt-in cached loader. Key = (GET, normalized URL without fragment, partition).
pub fn load_with(
    url: &str,
    opts: &FetchOptions,
    cache: &mut DiskCache,
) -> Result<Resource, FetchError> {
    let normalized = normalize_url(url)?;
    if !opts.use_cache {
        return load(&normalized);
    }
    let key = CacheKey::new("GET", &normalized, &opts.partition);
    let Some(_guard) = PendingGuard::enter(&key) else {
        return Err(FetchError::InFlight);
    };
    Ok(load_cached(&key, opts, cache, &Headers::new())?.resource)
}

fn loaded_from_fetched(fetched: Fetched) -> Loaded {
    let resource = resource_from(
        fetched.data,
        fetched.headers.get("content-type").map(String::as_str),
        &fetched.final_url,
    );
    Loaded {
        resource,
        set_cookies: fetched.set_cookies,
        hsts: fetched.hsts,
        final_url: fetched.final_url,
    }
}

/// Internal load used by [`crate::fetch`]. `extra_headers` carries the
/// privacy layer's `Cookie` / `Referer` headers. Private mode bypasses the
/// cache entirely (no reads, no writes) and just performs one network request.
pub(crate) fn load_with_privacy(
    url: &str,
    opts: &FetchOptions,
    cache: &mut DiskCache,
    extra_headers: &Headers,
) -> Result<Loaded, FetchError> {
    let normalized = normalize_url(url)?;
    if opts.private || !opts.use_cache {
        if let Some(path) = normalized.strip_prefix("file://") {
            return Ok(Loaded {
                resource: load_path(Path::new(path))?,
                set_cookies: Vec::new(),
                hsts: Vec::new(),
                final_url: normalized,
            });
        }
        if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
            return Ok(Loaded {
                resource: load(&normalized)?,
                set_cookies: Vec::new(),
                hsts: Vec::new(),
                final_url: normalized,
            });
        }
        let mut headers = our_request_headers(&normalized);
        headers.extend(extra_headers.iter().map(|(k, v)| (k.clone(), v.clone())));
        return Ok(loaded_from_fetched(fetch_network(
            &normalized,
            &headers,
            None,
            opts.private,
        )?));
    }

    let key = CacheKey::new("GET", &normalized, &opts.partition);
    let Some(_guard) = PendingGuard::enter(&key) else {
        return Err(FetchError::InFlight);
    };
    load_cached(&key, opts, cache, extra_headers)
}

fn load_cached(
    key: &CacheKey,
    opts: &FetchOptions,
    cache: &mut DiskCache,
    extra_headers: &Headers,
) -> Result<Loaded, FetchError> {
    let clock = SystemClock;
    let mut request_headers = our_request_headers(&key.url);
    request_headers.extend(extra_headers.iter().map(|(k, v)| (k.clone(), v.clone())));

    let cached = |resource: Resource, url: String| Loaded {
        resource,
        set_cookies: Vec::new(),
        hsts: Vec::new(),
        final_url: url,
    };

    // file:// entries are immutable: always fresh. Cached here too so warm
    // loads of local files avoid re-reading/decoding the original.
    if key.url.starts_with("file://") {
        if let Some(entry) = cache.get(key) {
            return Ok(cached(
                resource_from(entry.data, Some(&entry.content_type), &key.url),
                key.url.clone(),
            ));
        }
        if opts.offline {
            return Err(FetchError::Offline);
        }
        let path = &key.url["file://".len()..];
        let resource = load_path(Path::new(path))?;
        let (data, content_type) = match &resource {
            Resource::Html(text) => (text.as_bytes().to_vec(), "text/html".to_string()),
            Resource::Other { data, mime } => (data.clone(), mime.clone()),
        };
        let entry = CacheEntry {
            url: key.url.clone(),
            content_type,
            data,
            status_code: 200,
            expires_at: Some(u64::MAX),
            etag: None,
            last_modified: None,
            vary: BTreeMap::new(),
            headers: BTreeMap::new(),
            partition_key: key.partition.clone(),
            request_method: "GET".into(),
            stored_at: clock.now_unix(),
        };
        let _ = cache.put(key, &entry);
        return Ok(cached(resource, key.url.clone()));
    }

    if !key.url.starts_with("http://") && !key.url.starts_with("https://") {
        return Ok(cached(load(&key.url)?, key.url.clone()));
    }

    // Warm hits read metadata + an mmap-backed body slice instead of pulling
    // the whole file into a Vec. The mapping lives inside `MappedEntry` and
    // outlives every `&[u8]` handed to the slice-based resource/parse helpers.
    if let Some(mapped) = cache.get_mapped(key) {
        let meta = mapped.metadata();
        if meta.is_fresh(&clock) && vary_matches(&meta.vary, &request_headers) {
            if let Some(resource) = serve_cached_mapped(mapped, key, cache, &clock) {
                return Ok(cached(resource, key.url.clone()));
            }
            if opts.offline {
                return Err(FetchError::Offline);
            }
            // Broken redirect chain: fall through to a normal fetch.
        } else if opts.offline {
            return Err(FetchError::Offline);
        } else {
            // Revalidation writes metadata back, so materialize the body once.
            let entry = mapped.into_owned();
            if entry.etag.is_some() || entry.last_modified.is_some() {
                if let Revalidated::NotModified(new_headers, cookies) =
                    revalidate(&key.url, &entry, &request_headers)?
                {
                    // A 304 can carry side facts: forward its Set-Cookie (and
                    // any HSTS header) to the privacy layer instead of dropping
                    // them with the discarded response.
                    let hsts = new_headers
                        .get("strict-transport-security")
                        .map(|value| vec![(key.url.clone(), value.clone())])
                        .unwrap_or_default();
                    let merged = apply_not_modified(entry, &new_headers, &request_headers, &clock);
                    let is_redirect = matches!(merged.status_code, 301 | 308);
                    let resource = (!is_redirect).then(|| {
                        resource_from(merged.data.as_slice(), Some(&merged.content_type), &key.url)
                    });
                    let _ = cache.put(key, &merged);
                    if let Some(resource) = resource {
                        return Ok(Loaded {
                            resource,
                            set_cookies: cookies,
                            hsts,
                            final_url: key.url.clone(),
                        });
                    }
                    if let Some(resource) = serve_cached(merged, key, cache, &clock) {
                        return Ok(Loaded {
                            resource,
                            set_cookies: cookies,
                            hsts,
                            final_url: key.url.clone(),
                        });
                    }
                }
            }
        }
    } else if opts.offline {
        return Err(FetchError::Offline);
    }

    let is_http = key.url.starts_with("http://") || key.url.starts_with("https://");
    let fetched = if opts.stream_to_cache && is_http {
        fetch_network_streaming(
            &key.url,
            &request_headers,
            cache,
            &key.partition,
            &clock,
            opts.private,
        )?
    } else {
        fetch_network(&key.url, &request_headers, None, opts.private)?
    };
    let final_key = fetched
        .streamed_key
        .clone()
        .unwrap_or_else(|| CacheKey::new("GET", &fetched.final_url, &key.partition));
    for (url, status, headers) in &fetched.redirects {
        let redirect_key = CacheKey::new("GET", url, &key.partition);
        put_entry(
            &redirect_key,
            *status,
            headers,
            &[],
            &clock,
            cache,
            &request_headers,
        );
    }
    if let Some(streamed_key) = &fetched.streamed_key {
        // The body was committed straight to disk; read it back via mmap so
        // the network path never held a body-sized buffer.
        let mapped = cache.get_mapped(streamed_key).ok_or_else(|| {
            FetchError::Network("streamed cache entry missing after commit".into())
        })?;
        let content_type = mapped.metadata().content_type.clone();
        let url = fetched.final_url.clone();
        let resource = resource_from(mapped.mapped_body(), Some(&content_type), &url);
        return Ok(Loaded {
            resource,
            set_cookies: fetched.set_cookies,
            hsts: fetched.hsts,
            final_url: fetched.final_url,
        });
    }
    put_entry(
        &final_key,
        fetched.status,
        &fetched.headers,
        &fetched.data,
        &clock,
        cache,
        &request_headers,
    );
    Ok(loaded_from_fetched(fetched))
}

// ---------------------------------------------------------------------------
// POST (frozen M7.3 API) — always uncached
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostMethod {
    Post,
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostRedirect {
    AsGet,
    AsPost,
}

/// 301/302/303 lose the body and become GET; 307/308 repeat the POST.
fn post_redirect_action(status: u16) -> Option<PostRedirect> {
    match status {
        301..=303 => Some(PostRedirect::AsGet),
        307 | 308 => Some(PostRedirect::AsPost),
        _ => None,
    }
}

fn require_http_url(url: &str) -> Result<(), FetchError> {
    if url.is_empty() {
        return Err(FetchError::InvalidUrl);
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(());
    }
    let scheme = url
        .split_once("://")
        .map(|(scheme, _)| scheme)
        .or_else(|| url.strip_prefix("data:").map(|_| "data"));
    Err(match scheme {
        Some(scheme) => FetchError::UnsupportedScheme(scheme.to_string()),
        None => FetchError::InvalidUrl,
    })
}

fn post_builder(
    agent: &ureq::Agent,
    url: &str,
    content_type: &str,
) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
    agent.post(url).header("content-type", content_type)
}

/// Send a POST with a byte body. Over-cap bodies are rejected before any
/// bytes leave the process; responses are read through the same 2 MiB cap as
/// [`load`]. Redirects are followed by hand because ureq's automatic redirects
/// would replay the body on 301/302/303. POST responses are never cached.
pub fn post(url: &str, body: &[u8], content_type: &str) -> Result<Resource, FetchError> {
    require_http_url(url)?;
    if body.len() > MAX_BYTES {
        return Err(FetchError::TooLarge);
    }

    let mut current = url.to_string();
    let mut method = PostMethod::Post;
    for _ in 0..=MAX_REDIRECTS {
        require_http_url(&current)?;
        let mut res = match method {
            PostMethod::Post => post_builder(cache_agent(), &current, content_type).send(body),
            PostMethod::Get => cache_agent().get(&current).call(),
        }
        .map_err(|e| FetchError::Network(e.to_string()))?;

        if let Some(action) = post_redirect_action(res.status().as_u16()) {
            let location = res
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
                .ok_or_else(|| FetchError::Network("redirect without Location".into()))?;
            current = resolve_location(&current, &location)?;
            if action == PostRedirect::AsGet {
                method = PostMethod::Get;
            }
            continue;
        }

        let response_type = res
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty());
        let declared_len = res
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<usize>().ok());
        if declared_len.is_some_and(|len| len > MAX_BYTES) {
            return Err(FetchError::TooLarge);
        }
        let data = read_capped(res.body_mut().as_reader(), |e| {
            FetchError::Network(e.to_string())
        })?;
        let resource = resource_from(data, response_type.as_deref(), &current);
        invalidate_cache_for(url);
        return Ok(resource);
    }
    Err(FetchError::Network("too many redirects".into()))
}

/// The shell owns the real disk cache (`hodo-shell::app::shell_cache`), so
/// hodo-net mirrors that owner: one lazily opened cache rooted at the cwd.
/// `DiskCache::invalidate` deletes the file, so the shell's own index self-heals
/// on its next lookup even though it is a separate instance.
fn global_cache() -> Option<&'static Mutex<DiskCache>> {
    static CACHE: OnceLock<Option<Mutex<DiskCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            DiskCache::open(&root, 128 * 1024 * 1024)
                .ok()
                .map(Mutex::new)
        })
        .as_ref()
}

fn cache_key_for_get(url: &str) -> Option<CacheKey> {
    normalize_url(url)
        .ok()
        .map(|normalized| CacheKey::new("GET", &normalized, ""))
}

/// Drop the cached GET for `url` after a mutating request. Called automatically
/// by [`post`] on success; callers using a non-default cache partition must
/// invalidate their own key too.
/// shortcut: the frozen `post` signature has no partition parameter, so only
/// the shell's default (empty) partition is invalidated here.
pub fn invalidate_cache_for(url: &str) {
    let (Some(key), Some(cache)) = (cache_key_for_get(url), global_cache()) else {
        return;
    };
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    guard.invalidate(&key);
}

// ---------------------------------------------------------------------------
// Speculative prefetch (warm the disk cache without returning a body)
// ---------------------------------------------------------------------------

/// `scheme://authority` of an already-normalized URL (authority includes an
/// explicit non-default port, so this is a strict web origin).
fn origin_of_normalized(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    let end = rest.find('/').unwrap_or(rest.len());
    Some(&rest[..end])
}

fn same_origin(a: &str, b: &str) -> bool {
    matches!(
        (origin_of_normalized(a), origin_of_normalized(b)),
        (Some(a), Some(b)) if a.eq_ignore_ascii_case(b)
    )
}

/// Warm the disk cache for `url` at low priority, returning no body. Intended
/// for the shell's link-hover hook (see [`hover_prefetch_hint`]); all errors are
/// deliberately swallowed — prefetch is best-effort and never on a critical
/// path. ureq 3.5.2 exposes no socket-priority knob, so "low priority" means
/// best-effort/off-path on the shared pooled agent; the shell should debounce
/// hover calls.
///
/// # Policy (conservative by design)
///
/// - **Private mode NEVER prefetches** — no network, no cache read or write.
/// - HTTP(S) only; the same 2 MiB body cap and RFC 9111 cacheability rules as
///   [`load_with`] apply. Oversize or uncacheable responses are discarded.
/// - Adblock-gated: blocked (or warned) URLs are never prefetched.
/// - Same-origin only: when `opts.referer` is present the target must share its
///   origin; a redirect that would leave the requested URL's own origin is
///   **refused outright**, so no cross-origin request is ever made, let alone
///   cached. Cross-origin targets are never speculatively fetched.
///
/// Offline mode and `use_cache = false` are also no-ops.
pub fn prefetch(url: &str, opts: &FetchOptions) {
    let Some(cache) = global_cache() else { return };
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    prefetch_with_cache(url, opts, &mut guard);
}

/// Same policy as [`prefetch`], but writes to the caller's cache. Kept crate-
/// visible so tests (and a future shell that owns its cache handle) can wire
/// it without touching the process-wide cache.
pub(crate) fn prefetch_with_cache(url: &str, opts: &FetchOptions, cache: &mut DiskCache) {
    if opts.private || opts.offline || !opts.use_cache {
        return;
    }
    let Ok(normalized) = normalize_url(url) else {
        return;
    };
    if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
        return;
    }
    if let Some(referer) = opts.referer.as_deref() {
        let Ok(referer) = normalize_url(referer) else {
            return;
        };
        if !same_origin(&normalized, &referer) {
            return;
        }
    }
    let third_party = privacy::is_third_party(&normalized, &opts.top_site, opts.referer.as_deref());
    if !matches!(
        hodo_adblock::should_block_record(&normalized, &opts.top_site, third_party, false),
        BlockDecision::Allow
    ) {
        return;
    }

    let request_headers = our_request_headers(&normalized);
    // `fetch_network` enforces MAX_BYTES through `read_capped`.
    let Ok(fetched) = fetch_network(&normalized, &request_headers, Some(&normalized), false) else {
        return;
    };
    if !same_origin(&normalized, &fetched.final_url) {
        return;
    }
    let clock = SystemClock;
    for (hop_url, status, headers) in &fetched.redirects {
        if !same_origin(&normalized, hop_url) {
            continue;
        }
        let hop_key = CacheKey::new("GET", hop_url, &opts.partition);
        put_entry(
            &hop_key,
            *status,
            headers,
            &[],
            &clock,
            cache,
            &request_headers,
        );
    }
    let key = CacheKey::new("GET", &fetched.final_url, &opts.partition);
    put_entry(
        &key,
        fetched.status,
        &fetched.headers,
        &fetched.data,
        &clock,
        cache,
        &request_headers,
    );
}

/// Sugar for the shell's link-hover hook: `prefetch(url, &FetchOptions::default())`.
/// Callers that know the top site / referer / partition should call [`prefetch`]
/// directly so the same-origin and double-key rules can be enforced precisely.
pub fn hover_prefetch_hint(url: &str) {
    prefetch(url, &FetchOptions::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use std::time::Instant;

    const HTML: &str = "<!doctype html><html><body>hodo</body></html>";

    struct FakeClock(std::sync::Mutex<u64>);

    impl FakeClock {
        fn new(now: u64) -> Self {
            Self(std::sync::Mutex::new(now))
        }

        fn advance(&self, seconds: u64) {
            *self.0.lock().unwrap() += seconds;
        }
    }

    impl Clock for FakeClock {
        fn now_unix(&self) -> u64 {
            *self.0.lock().unwrap()
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, AtomicOrdering::Relaxed);
        let dir = std::env::temp_dir().join(format!("hodo-net-{tag}-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    struct TempCache {
        root: PathBuf,
        cache: DiskCache,
    }

    impl TempCache {
        fn new() -> Self {
            let root = temp_dir("cache");
            let cache = DiskCache::open(&root, 8 * 1024 * 1024).unwrap();
            Self { root, cache }
        }
    }

    impl Drop for TempCache {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn options(partition: &str) -> FetchOptions {
        FetchOptions {
            partition: partition.into(),
            ..FetchOptions::default()
        }
    }

    // --- existing frozen-loader tests -------------------------------------

    #[test]
    fn loads_local_and_file_urls() {
        let dir = temp_dir("load");
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
        fs::write(&png_path, b"\x89PNG\r\n\x1a\n").unwrap();
        match load(png_path.to_string_lossy().as_ref()).unwrap() {
            Resource::Other { data, mime } => {
                assert_eq!(data, b"\x89PNG\r\n\x1a\n");
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
    }

    #[test]
    fn decode_utf8_handles_valid_and_invalid_bytes() {
        assert_eq!(decode_utf8(b"hodo"), "hodo");
        assert!(!decode_utf8(&[0xff, 0xfe]).is_empty());
    }

    #[test]
    fn sniffs_html_prefix() {
        assert!(looks_like_html(b"<!doctype html><html>"));
        assert!(looks_like_html(b"\n\n<Html>"));
        assert!(!looks_like_html(b"just plain text"));
        let mut far = vec![b' '; 512];
        far.extend_from_slice(b"<html>");
        assert!(!looks_like_html(&far));
    }

    #[test]
    fn strips_utf8_bom_from_local_html() {
        let dir = temp_dir("bom");
        let path = dir.join("bom.html");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(HTML.as_bytes());
        fs::write(&path, &bytes).unwrap();
        match load(path.to_string_lossy().as_ref()).unwrap() {
            Resource::Html(s) => assert_eq!(s, HTML),
            other => panic!("expected Html, got {:?}", other),
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    // --- URL normalization / path-safety ----------------------------------

    #[test]
    fn normalizes_urls_without_fragment() {
        assert_eq!(
            normalize_url("HTTPS://Example.COM:443/a/./b/../c#frag").unwrap(),
            "https://example.com/a/c"
        );
        assert_eq!(
            normalize_url("http://Example.com:80/x?a=1#f").unwrap(),
            "http://example.com/x?a=1"
        );
        assert_eq!(
            normalize_url("https://x/a%2fb%41c").unwrap(),
            "https://x/a%2FbAc"
        );
        assert!(matches!(
            normalize_url("data:text/html,x"),
            Err(FetchError::UnsupportedScheme(_))
        ));
        assert!(matches!(normalize_url(""), Err(FetchError::InvalidUrl)));
    }

    #[test]
    fn hostile_urls_stay_hash_only_and_long_urls_work() {
        let mut t = TempCache::new();
        let long = format!("https://x/{}", "a".repeat(20_000));
        for url in [
            "https://x/../../etc/passwd".to_string(),
            "https://x/%2e%2e/%2e%2e/secret".to_string(),
            "https://x\\..\\..\\windows\\system32".to_string(),
            long,
        ] {
            let normalized = normalize_url(&url).unwrap();
            let key = CacheKey::new("GET", &normalized, "site");
            let entry = CacheEntry {
                url: normalized,
                content_type: "text/plain".into(),
                data: b"ok".to_vec(),
                status_code: 200,
                expires_at: Some(u64::MAX),
                etag: None,
                last_modified: None,
                vary: BTreeMap::new(),
                headers: BTreeMap::new(),
                partition_key: "site".into(),
                request_method: "GET".into(),
                stored_at: 0,
            };
            t.cache.put(&key, &entry).unwrap();
            assert_eq!(t.cache.get(&key).unwrap().data, b"ok");
        }
        assert_eq!(t.cache.len(), 4);
    }

    // --- parsers / robustness ---------------------------------------------

    #[test]
    fn parses_cache_control_and_rejects_malformed() {
        let cc = parse_cache_control(Some("max-age=60, no-cache, public"));
        assert_eq!(cc.max_age, Some(60));
        assert!(cc.no_cache && cc.public && !cc.no_store && !cc.malformed);
        assert!(parse_cache_control(Some("max-age=abc")).malformed);
        assert!(parse_cache_control(Some("max-age=\"60\"")).malformed);
        assert!(parse_cache_control(Some("no-store")).no_store);
        assert!(!parse_cache_control(None).malformed);
    }

    #[test]
    fn parses_vary_etag_and_dates() {
        assert_eq!(
            parse_vary(Some("Accept-Encoding, accept-language")),
            Vary::Headers(vec!["accept-encoding".into(), "accept-language".into()])
        );
        assert_eq!(parse_vary(Some("*")), Vary::Unsupported);
        assert_eq!(parse_vary(Some("x-secret")), Vary::Unsupported);
        assert_eq!(parse_vary(Some("accept-encoding, !!")), Vary::Unsupported);
        assert_eq!(parse_vary(Some("")), Vary::Absent);
        assert_eq!(parse_etag(Some("W/\"abc\"")).as_deref(), Some("W/\"abc\""));
        assert!(parse_etag(Some("abc")).is_none());
        assert!(parse_etag(Some("\"a\"\"b\"")).is_none());
        assert_eq!(
            parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(784_111_777)
        );
        assert_eq!(parse_http_date("not a date"), None);
    }

    #[test]
    fn malformed_headers_never_panic() {
        let inputs = [
            "",
            ",",
            ",,,,",
            "max-age=",
            "max-age=-1",
            "max-age=999999999999999999999999",
            "no-cache=",
            "vary:\x00",
            "W/",
            "\"",
            "*",
            "\u{0000}",
            " 🍕 ",
        ];
        let clock = FakeClock::new(1_000);
        for value in inputs {
            let mut headers = Headers::new();
            headers.insert("cache-control".into(), value.into());
            headers.insert("vary".into(), value.into());
            headers.insert("etag".into(), value.into());
            headers.insert("date".into(), value.into());
            let result = std::panic::catch_unwind(|| {
                let _ = parse_cache_control(Some(value));
                let _ = parse_vary(Some(value));
                let _ = parse_etag(Some(value));
                let _ = parse_http_date(value);
                let _ = response_freshness(&headers, &clock);
            });
            assert!(result.is_ok(), "panicked on {value:?}");
        }
    }

    // --- freshness matrix --------------------------------------------------

    #[test]
    fn freshness_matrix_cache_control_age_validators() {
        let clock = FakeClock::new(784_111_777); // same instant as `date`
        let date = "Sun, 06 Nov 1994 08:49:37 GMT";
        let cases: &[(&str, u64, &str, bool)] = &[
            ("max-age=60", 0, "", true),
            ("max-age=60", 59, "", true),
            ("max-age=60", 60, "", false),
            ("max-age=60", 61, "", false),
            ("max-age=60", 10, "ETag: \"x\"", true),
            ("no-cache", 0, "ETag: \"x\"", false),
            ("", 0, "Last-Modified: Sat, 05 Nov 1994 08:49:37 GMT", true),
            ("", 0, "Expires: Mon, 07 Nov 1994 08:49:37 GMT", true),
            ("max-age=600", 500, "", true),
            ("max-age=600", 601, "", false),
        ];
        for (cc, age, extra, expected_fresh) in cases {
            let mut headers = Headers::new();
            headers.insert("cache-control".into(), cc.to_string());
            headers.insert("date".into(), date.into());
            headers.insert("age".into(), age.to_string());
            if !extra.is_empty() {
                let (name, value) = extra.split_once(": ").unwrap();
                headers.insert(name.to_ascii_lowercase(), value.into());
            }
            let fresh = response_freshness(&headers, &clock);
            let actually_fresh = fresh.expires_at.is_some_and(|t| clock.now_unix() < t);
            assert_eq!(
                actually_fresh, *expected_fresh,
                "case {cc:?} age={age} {extra:?}"
            );
        }
    }

    #[test]
    fn fake_clock_advance_shifts_freshness_window() {
        let clock = FakeClock::new(100);
        let mut headers = Headers::new();
        headers.insert("cache-control".into(), "max-age=10".into());
        assert_eq!(response_freshness(&headers, &clock).expires_at, Some(110));
        clock.advance(11);
        assert_eq!(response_freshness(&headers, &clock).expires_at, Some(121));
    }

    #[test]
    fn last_modified_heuristic_is_10_percent_capped_at_24h() {
        let clock = FakeClock::new(1_000_000);
        let mut headers = Headers::new();
        headers.insert("date".into(), "Sun, 06 Nov 1994 08:49:37 GMT".into());
        headers.insert(
            "last-modified".into(),
            "Sat, 05 Nov 1994 08:49:37 GMT".into(),
        );
        // 1 day old → heuristic 2.4 hours = 8640s.
        let fresh = response_freshness(&headers, &clock);
        assert_eq!(fresh.expires_at, Some(1_000_000 + 8640));
        headers.insert(
            "last-modified".into(),
            "Sun, 01 Jan 1984 00:00:00 GMT".into(),
        );
        let fresh = response_freshness(&headers, &clock);
        assert_eq!(fresh.expires_at, Some(1_000_000 + 86_400));
    }

    #[test]
    fn no_store_authorization_and_vary_star_are_uncachable() {
        let request = our_request_headers("https://x/a");
        let mut authorized = request.clone();
        authorized.insert("authorization".into(), "Bearer secret".into());
        let mut headers = Headers::new();
        headers.insert("cache-control".into(), "no-store".into());
        assert!(!is_cacheable(200, &request, &headers));
        headers.remove("cache-control");
        assert!(!is_cacheable(200, &authorized, &headers));
        headers.insert("cache-control".into(), "public, max-age=60".into());
        assert!(is_cacheable(200, &authorized, &headers));
        headers.remove("cache-control");
        headers.insert("vary".into(), "*".into());
        assert!(!is_cacheable(200, &request, &headers));
        assert!(!is_cacheable(500, &request, &Headers::new()));
        assert!(is_cacheable(301, &request, &Headers::new()));
    }

    #[test]
    fn vary_matching_uses_stored_request_values() {
        let mut stored = BTreeMap::new();
        stored.insert("accept-encoding".to_string(), "gzip".to_string());
        let mut current = Headers::new();
        current.insert("accept-encoding".into(), "gzip".into());
        assert!(vary_matches(&stored, &current));
        current.insert("accept-encoding".into(), "br".into());
        assert!(!vary_matches(&stored, &current));
    }

    // --- deterministic cache-flow tests -----------------------------------

    #[test]
    fn cache_hit_serves_stored_entry() {
        let clock = FakeClock::new(1_000);
        let mut t = TempCache::new();
        let key = CacheKey::new("GET", "https://x/a", "p");
        let mut entry = CacheEntry {
            url: key.url.clone(),
            content_type: "text/plain".into(),
            data: b"hello".to_vec(),
            status_code: 200,
            expires_at: Some(2_000),
            etag: None,
            last_modified: None,
            vary: BTreeMap::new(),
            headers: BTreeMap::new(),
            partition_key: "p".into(),
            request_method: "GET".into(),
            stored_at: 0,
        };
        entry
            .headers
            .insert("content-type".into(), "text/plain".into());
        t.cache.put(&key, &entry).unwrap();
        let hit = t.cache.get(&key).unwrap(); // the real load path counts this hit
        match serve_cached(hit, &key, &mut t.cache, &clock).unwrap() {
            Resource::Other { data, mime } => {
                assert_eq!(data, b"hello");
                assert_eq!(mime, "text/plain");
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(t.cache.stats().hits, 1);
    }

    #[test]
    fn no_store_and_unknown_vary_are_never_stored() {
        let clock = FakeClock::new(1_000);
        let mut t = TempCache::new();
        let key = CacheKey::new("GET", "https://x/n", "p");
        let request = our_request_headers("https://x/n");
        let mut headers = Headers::new();
        headers.insert("cache-control".into(), "no-store".into());
        put_entry(&key, 200, &headers, b"x", &clock, &mut t.cache, &request);
        headers.remove("cache-control");
        headers.insert("vary".into(), "X-Secret".into());
        put_entry(&key, 200, &headers, b"x", &clock, &mut t.cache, &request);
        assert!(t.cache.is_empty());
    }

    #[test]
    fn redirects_301_and_308_are_stored_and_followed() {
        let clock = FakeClock::new(1_000);
        let mut t = TempCache::new();
        let request = our_request_headers("https://x/start");
        for status in [301u16, 308] {
            let start = CacheKey::new("GET", "https://x/start", "p");
            let mut headers = Headers::new();
            headers.insert("cache-control".into(), "max-age=600".into());
            headers.insert("location".into(), "/target".into());
            put_entry(
                &start,
                status,
                &headers,
                &[],
                &clock,
                &mut t.cache,
                &request,
            );

            let target = CacheKey::new("GET", "https://x/target", "p");
            let mut final_headers = Headers::new();
            final_headers.insert("cache-control".into(), "max-age=600".into());
            final_headers.insert("content-type".into(), "text/plain".into());
            put_entry(
                &target,
                200,
                &final_headers,
                b"final",
                &clock,
                &mut t.cache,
                &request,
            );

            let first = t.cache.get(&start).unwrap();
            match serve_cached(first, &start, &mut t.cache, &clock).unwrap() {
                Resource::Other { data, .. } => assert_eq!(data, b"final"),
                other => panic!("unexpected {other:?}"),
            }
            t.cache.clear();
        }
    }

    #[test]
    fn offline_mode_is_cache_only() {
        let dir = temp_dir("offline");
        let path = dir.join("page.html");
        fs::write(&path, HTML).unwrap();
        let url = format!("file://{}", path.to_string_lossy());
        let mut t = TempCache::new();
        let opts = options("offline");
        assert!(matches!(
            load_with(
                &url,
                &FetchOptions {
                    offline: true,
                    ..opts.clone()
                },
                &mut t.cache
            ),
            Err(FetchError::Offline)
        ));
        let _ = load_with(&url, &opts, &mut t.cache).unwrap();
        match load_with(
            &url,
            &FetchOptions {
                offline: true,
                ..opts
            },
            &mut t.cache,
        )
        .unwrap()
        {
            Resource::Html(text) => assert_eq!(text, HTML),
            other => panic!("unexpected {other:?}"),
        }
        fs::remove_dir_all(&dir).unwrap();
    }
    #[test]
    fn unsafe_method_invalidates_by_key() {
        let mut t = TempCache::new();
        let key = CacheKey::new("GET", "https://x/a", "p");
        let entry = CacheEntry {
            url: key.url.clone(),
            content_type: "text/plain".into(),
            data: b"a".to_vec(),
            status_code: 200,
            expires_at: Some(u64::MAX),
            etag: None,
            last_modified: None,
            vary: BTreeMap::new(),
            headers: BTreeMap::new(),
            partition_key: "p".into(),
            request_method: "GET".into(),
            stored_at: 0,
        };
        t.cache.put(&key, &entry).unwrap();
        // A future non-GET handler calls invalidate with the same key.
        t.cache.invalidate(&key);
        assert!(t.cache.get(&key).is_none());
    }

    #[test]
    fn file_warm_vs_cold_timing() {
        let dir = temp_dir("bench");
        let path = dir.join("page.html");
        let body = HTML.repeat(512); // ~22 KiB, enough to exceed timer noise
        fs::write(&path, &body).unwrap();
        let url = format!("file://{}", path.to_string_lossy());
        let mut t = TempCache::new();
        let opts = options("bench");

        let cold_start = Instant::now();
        for _ in 0..100 {
            let _ = load(&url).unwrap();
        }
        let cold = cold_start.elapsed();

        let _ = load_with(&url, &opts, &mut t.cache).unwrap(); // prime
        let warm_start = Instant::now();
        for _ in 0..100 {
            let _ = load_with(&url, &opts, &mut t.cache).unwrap();
        }
        let warm = warm_start.elapsed();
        eprintln!("file:// cold 100x: {cold:?}; warm 100x: {warm:?}");
        fs::remove_dir_all(&dir).unwrap();
    }

    // --- POST API ----------------------------------------------------------

    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::thread::JoinHandle;

    fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedRequest {
        method: String,
        path: String,
        host: Option<String>,
        content_type: Option<String>,
        body: Vec<u8>,
    }

    fn read_request(stream: &mut TcpStream) -> RecordedRequest {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            if let Some(pos) = find_bytes(&buf, b"\r\n\r\n") {
                break pos + 4;
            }
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break buf.len(),
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
            }
        };
        let head = String::from_utf8_lossy(&buf[..header_end]);
        let mut lines = head.split("\r\n");
        let request_line = lines.next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_string();
        let path = parts.next().unwrap_or_default().to_string();
        let mut content_length = 0usize;
        let mut content_type = None;
        let mut host = None;
        for line in lines {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            match name.trim().to_ascii_lowercase().as_str() {
                "content-length" => content_length = value.trim().parse().unwrap_or(0),
                "content-type" => content_type = Some(value.trim().to_string()),
                "host" => host = Some(value.trim().to_string()),
                _ => {}
            }
        }
        while buf.len() < header_end + content_length {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
            }
        }
        let end = (header_end + content_length).min(buf.len());
        RecordedRequest {
            method,
            path,
            host,
            content_type,
            body: buf[header_end..end].to_vec(),
        }
    }

    struct ScriptedServer {
        url: String,
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    impl ScriptedServer {
        /// Serve one scripted `(status, Location, body)` reply per connection.
        fn start(script: Vec<(u16, Option<String>, &'static str)>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let url = format!("http://127.0.0.1:{port}/start");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let seen = Arc::clone(&requests);
            let _join: JoinHandle<()> = std::thread::spawn(move || {
                for (status, location, body) in script {
                    let Ok((mut stream, _)) = listener.accept() else {
                        return;
                    };
                    seen.lock().unwrap().push(read_request(&mut stream));
                    let mut response = format!(
                        "HTTP/1.1 {status} Scripted\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n",
                        body.len()
                    );
                    if let Some(location) = location {
                        response.push_str("Location: ");
                        response.push_str(&location);
                        response.push_str("\r\n");
                    }
                    response.push_str("\r\n");
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.write_all(body.as_bytes());
                    let _ = stream.flush();
                }
            });
            Self { url, requests }
        }

        fn requests(&self) -> Vec<RecordedRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    /// Scripted server that can set arbitrary response headers (Set-Cookie,
    /// ETag, Vary, HSTS, Location).
    struct HeaderServer {
        url: String,
    }

    impl HeaderServer {
        fn start(script: Vec<(u16, String, String)>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let url = format!("http://127.0.0.1:{port}/r");
            let _join: JoinHandle<()> = std::thread::spawn(move || {
                for (status, extra, body) in script {
                    let Ok((mut stream, _)) = listener.accept() else {
                        return;
                    };
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    loop {
                        if find_bytes(&buf, b"\r\n\r\n").is_some() {
                            break;
                        }
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    let response = format!(
                        "HTTP/1.1 {status} Scripted\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
            });
            Self { url }
        }
    }

    #[test]
    fn post_roundtrip_over_loopback() {
        let server = ScriptedServer::start(vec![(200, None, "pong")]);
        match post(&server.url, b"ping", "text/plain").unwrap() {
            Resource::Other { data, mime } => {
                assert_eq!(data, b"pong");
                assert_eq!(mime, "text/plain");
            }
            other => panic!("unexpected {other:?}"),
        }
        let seen = server.requests();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].method, "POST");
        assert_eq!(seen[0].path, "/start");
        assert_eq!(seen[0].body, b"ping");
        assert_eq!(seen[0].content_type.as_deref(), Some("text/plain"));
    }

    #[test]
    fn post_303_follows_as_get() {
        let server =
            ScriptedServer::start(vec![(303, Some("/final".into()), ""), (200, None, "done")]);
        match post(&server.url, b"ping", "text/plain").unwrap() {
            Resource::Other { data, .. } => assert_eq!(data, b"done"),
            other => panic!("unexpected {other:?}"),
        }
        let seen = server.requests();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].method, "POST");
        assert_eq!(seen[1].method, "GET");
        assert_eq!(seen[1].path, "/final");
        assert!(seen[1].body.is_empty());
        assert!(seen[1].content_type.is_none());
    }

    #[test]
    fn post_307_reposts_body() {
        let server =
            ScriptedServer::start(vec![(307, Some("/again".into()), ""), (200, None, "ok")]);
        match post(&server.url, b"ping", "text/plain").unwrap() {
            Resource::Other { data, .. } => assert_eq!(data, b"ok"),
            other => panic!("unexpected {other:?}"),
        }
        let seen = server.requests();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].method, "POST");
        assert_eq!(seen[1].method, "POST");
        assert_eq!(seen[1].path, "/again");
        assert_eq!(seen[1].body, b"ping");
        assert_eq!(seen[1].content_type.as_deref(), Some("text/plain"));
    }

    #[test]
    fn post_redirect_loop_is_capped() {
        let server =
            ScriptedServer::start(vec![(307, Some("/loop".into()), ""); MAX_REDIRECTS + 1]);
        let err = post(&server.url, b"x", "text/plain").unwrap_err();
        assert!(err.to_string().contains("too many redirects"), "{err}");
        assert_eq!(server.requests().len(), MAX_REDIRECTS + 1);
    }

    #[test]
    fn post_rejects_oversize_body_before_sending() {
        let body = vec![0u8; MAX_BYTES + 1];
        assert!(matches!(
            post("http://127.0.0.1:1/x", &body, "text/plain"),
            Err(FetchError::TooLarge)
        ));
    }

    #[test]
    fn post_rejects_non_http_urls() {
        for (url, scheme) in [
            ("data:text/plain,hi", "data"),
            ("file:///etc/passwd", "file"),
            ("ftp://example.com/x", "ftp"),
        ] {
            match post(url, b"x", "text/plain") {
                Err(FetchError::UnsupportedScheme(actual)) => assert_eq!(actual, scheme, "{url}"),
                other => panic!("expected scheme rejection for {url}, got {other:?}"),
            }
        }
        assert!(matches!(
            post("", b"x", "text/plain"),
            Err(FetchError::InvalidUrl)
        ));
        assert!(matches!(
            post("/relative", b"x", "text/plain"),
            Err(FetchError::InvalidUrl)
        ));
    }

    #[test]
    fn redirect_policy_table_is_frozen() {
        for status in [301, 302, 303] {
            assert_eq!(post_redirect_action(status), Some(PostRedirect::AsGet));
        }
        for status in [307, 308] {
            assert_eq!(post_redirect_action(status), Some(PostRedirect::AsPost));
        }
        for status in [200, 204, 300, 304, 305, 306, 400, 404, 500] {
            assert_eq!(post_redirect_action(status), None);
        }
    }

    #[test]
    fn post_builds_content_type_header() {
        let req = post_builder(
            cache_agent(),
            "http://example.com/x",
            "application/x-www-form-urlencoded",
        );
        let headers = req.headers_ref().unwrap();
        assert_eq!(headers["content-type"], "application/x-www-form-urlencoded");
    }

    #[test]
    fn resolve_location_public_api() {
        assert_eq!(
            resolve_location("https://example.com/a/b?x=1", "../c#f").unwrap(),
            "https://example.com/c"
        );
        assert_eq!(
            resolve_location("https://example.com/a/b", "/c").unwrap(),
            "https://example.com/c"
        );
        assert_eq!(
            resolve_location("https://example.com/a/b", "//other.test/x").unwrap(),
            "https://other.test/x"
        );
        assert_eq!(
            resolve_location("https://example.com/a/b", "http://Other.TEST:80/z").unwrap(),
            "http://other.test/z"
        );
        assert_eq!(
            resolve_location("https://x/dir/page?next=/foo", "bar").unwrap(),
            "https://x/dir/bar",
            "a slash inside the query is not a base-path separator"
        );
        assert!(matches!(
            resolve_location("not a url", "/x"),
            Err(FetchError::InvalidUrl)
        ));
    }

    #[test]
    fn redirect_cycle_fails_fast() {
        // A -> B -> A: the second Location resolves to the start URL.
        let server = ScriptedServer::start(vec![
            (301, Some("/b".into()), ""),
            (301, Some("/start".into()), ""),
        ]);
        let headers = our_request_headers(&server.url);
        let err = fetch_network(&server.url, &headers, None, true)
            .err()
            .expect("expected a redirect-loop error");
        assert!(
            matches!(err, FetchError::RedirectLoop(ref url) if url == &server.url),
            "{err:?}"
        );
        assert_eq!(server.requests().len(), 2, "must not burn the 5-hop budget");
    }

    #[test]
    fn three_hop_redirect_chain_terminates() {
        // rust-lang.org's shape: http apex -> https apex -> https www -> 200.
        // The https upgrade itself is covered by upgrade_redirect's tests; the
        // loopback server stands in for each host with distinct hop paths.
        let server = ScriptedServer::start(vec![
            (301, Some("/https".into()), ""),
            (301, Some("/www".into()), ""),
            (200, None, "final"),
        ]);
        let base = server.url.strip_suffix("/start").unwrap();
        let headers = our_request_headers(&server.url);
        let fetched = fetch_network(&server.url, &headers, None, true).unwrap();
        assert_eq!(fetched.final_url, format!("{base}/www"));
        assert_eq!(fetched.data, b"final");
        assert_eq!(server.requests().len(), 3);
    }

    #[test]
    fn long_distinct_redirect_chains_still_hit_the_five_hop_cap() {
        let script: Vec<(u16, Option<String>, &'static str)> = (0..=MAX_REDIRECTS)
            .map(|hop| (301, Some(format!("/hop{}", hop + 1)), ""))
            .collect();
        let server = ScriptedServer::start(script);
        let headers = our_request_headers(&server.url);
        let err = fetch_network(&server.url, &headers, None, true)
            .err()
            .expect("expected the redirect cap");
        assert!(
            matches!(err, FetchError::Network(ref msg) if msg == "too many redirects"),
            "{err:?}"
        );
        assert_eq!(server.requests().len(), MAX_REDIRECTS + 1);
    }

    #[test]
    fn redirect_hop_uses_the_new_host_not_the_original() {
        // The server-side half of the rust-lang.org bug: reusing the initial
        // `Host` on a cross-host redirect makes the apex redirect to itself.
        let final_server = ScriptedServer::start(vec![(200, None, "ok")]);
        let first_server = ScriptedServer::start(vec![(301, Some(final_server.url.clone()), "")]);
        let headers = our_request_headers(&first_server.url);
        let fetched = fetch_network(&first_server.url, &headers, None, true).unwrap();
        assert_eq!(fetched.data, b"ok");

        let final_host = final_server.url.split_once("://").unwrap().1;
        let final_host = &final_host[..final_host.find('/').unwrap_or(final_host.len())];
        let seen = final_server.requests();
        assert_eq!(seen.len(), 1);
        assert_eq!(
            seen[0].host.as_deref(),
            Some(final_host),
            "a redirect must send the new host, not the original one"
        );
    }

    #[test]
    fn not_modified_304_propagates_set_cookie_to_the_jar() {
        let server = HeaderServer::start(vec![
            (
                200,
                "Cache-Control: max-age=0\r\nETag: \"v1\"\r\n".into(),
                "hello".into(),
            ),
            (
                304,
                "Cache-Control: max-age=600\r\nSet-Cookie: sid=abc; Path=/\r\n".into(),
                "".into(),
            ),
        ]);
        let mut t = TempCache::new();
        let mut jar = hodo_cookie::CookieJar::new();
        let opts = FetchOptions {
            strip_tracking: false,
            ..FetchOptions::default()
        };
        fetch(&server.url, &opts, &mut t.cache, &mut jar).unwrap();
        fetch(&server.url, &opts, &mut t.cache, &mut jar).unwrap();
        let partition = privacy::cookie_partition("http", "127.0.0.1", "");
        assert_eq!(
            jar.cookie_header(&server.url, &partition, "", 1_700_000_000),
            "sid=abc",
            "Set-Cookie on a 304 must reach the jar"
        );
    }

    #[test]
    fn not_modified_304_updates_the_vary_selection() {
        let server = HeaderServer::start(vec![
            (
                200,
                "Cache-Control: max-age=0\r\nETag: \"v1\"\r\n".into(),
                "hello".into(),
            ),
            (
                304,
                "Cache-Control: max-age=600\r\nVary: accept-language\r\n".into(),
                "".into(),
            ),
        ]);
        let mut t = TempCache::new();
        let mut jar = hodo_cookie::CookieJar::new();
        let opts = FetchOptions {
            strip_tracking: false,
            ..FetchOptions::default()
        };
        fetch(&server.url, &opts, &mut t.cache, &mut jar).unwrap();
        fetch(&server.url, &opts, &mut t.cache, &mut jar).unwrap();
        let key = CacheKey::new("GET", &normalize_url(&server.url).unwrap(), "");
        let entry = t.cache.get(&key).unwrap();
        assert!(
            entry.vary.contains_key("accept-language"),
            "a 304 changing Vary must update the stored selection, got {:?}",
            entry.vary
        );
        assert!(!entry.vary.contains_key("accept-encoding"));
    }

    #[test]
    fn hop_hsts_is_recorded_against_its_own_url() {
        let server = HeaderServer::start(vec![
            (
                302,
                "Location: /final\r\nStrict-Transport-Security: max-age=1000\r\n".into(),
                "".into(),
            ),
            (200, "".into(), "done".into()),
        ]);
        let fetched = fetch_network(&server.url, &Headers::new(), None, false).unwrap();
        assert!(fetched.final_url.ends_with("/final"));
        assert!(
            fetched
                .hsts
                .iter()
                .any(|(url, value)| url.ends_with("/r") && value.contains("max-age=1000")),
            "HSTS from a redirect hop must be attributed to that hop: {:?}",
            fetched.hsts
        );
    }

    // --- speculative prefetch / streaming / mmap warm hits -----------------

    /// Loopback server that serves a fixed number of requests, optionally with
    /// chunked transfer encoding, and counts how many it actually received.
    struct CountingServer {
        url: String,
        requests: Arc<AtomicUsize>,
        join: Option<JoinHandle<()>>,
    }

    impl CountingServer {
        fn start(body: Vec<u8>, serves: usize, chunked: bool, content_type: &str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let requests = Arc::new(AtomicUsize::new(0));
            let seen = Arc::clone(&requests);
            let content_type = content_type.to_string();
            let join = std::thread::spawn(move || {
                for _ in 0..serves {
                    let Ok((mut stream, _)) = listener.accept() else {
                        return;
                    };
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    loop {
                        if find_bytes(&buf, b"\r\n\r\n").is_some() {
                            break;
                        }
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    seen.fetch_add(1, AtomicOrdering::Relaxed);
                    let head = if chunked {
                        format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nCache-Control: max-age=600\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n")
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nCache-Control: max-age=600\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                    };
                    let _ = stream.write_all(head.as_bytes());
                    if chunked {
                        for piece in body.chunks(STREAM_CHUNK_LEN) {
                            let _ = write!(stream, "{:X}\r\n", piece.len());
                            let _ = stream.write_all(piece);
                            let _ = stream.write_all(b"\r\n");
                        }
                        let _ = stream.write_all(b"0\r\n\r\n");
                    } else {
                        let _ = stream.write_all(&body);
                    }
                    let _ = stream.flush();
                }
            });
            Self {
                url: format!("http://127.0.0.1:{port}/"),
                requests,
                join: Some(join),
            }
        }

        fn requests(&self) -> usize {
            self.requests.load(AtomicOrdering::Relaxed)
        }
    }

    impl Drop for CountingServer {
        fn drop(&mut self) {
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    #[test]
    fn prefetch_warms_cache_with_zero_network_on_warm_hit() {
        let body = b"prefetched body".to_vec();
        // Only the prefetch itself may reach the network; the later warm hit
        // is served from the mmap path.
        let server = CountingServer::start(body.clone(), 1, false, "application/octet-stream");
        let mut t = TempCache::new();
        let opts = options("prefetch");
        prefetch_with_cache(&server.url, &opts, &mut t.cache);
        assert_eq!(server.requests(), 1);
        assert_eq!(t.cache.len(), 1);

        match load_with(&server.url, &opts, &mut t.cache).unwrap() {
            Resource::Other { data, mime } => {
                assert_eq!(data, body);
                assert_eq!(mime, "application/octet-stream");
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(server.requests(), 1, "warm hit must not touch the network");
    }

    #[test]
    fn private_mode_never_prefetches() {
        let server = CountingServer::start(
            b"must not be fetched".to_vec(),
            0,
            false,
            "application/octet-stream",
        );
        let mut t = TempCache::new();
        let opts = FetchOptions {
            private: true,
            ..options("private-prefetch")
        };
        prefetch_with_cache(&server.url, &opts, &mut t.cache);
        assert_eq!(server.requests(), 0);
        assert_eq!(t.cache.len(), 0);
        assert_eq!(t.cache.total_bytes(), 0);
    }

    #[test]
    fn streaming_caches_2mib_chunked_response_without_body_buffer() {
        let body = vec![b'S'; 2 * 1024 * 1024];
        let server = CountingServer::start(body.clone(), 1, true, "application/octet-stream");
        let mut t = TempCache::new();
        let opts = FetchOptions {
            stream_to_cache: true,
            ..options("stream")
        };
        // Design guarantee: the response body is piped into a cache temp file
        // in STREAM_CHUNK_LEN (64 KiB) pieces; no body-sized Vec is allocated
        // on the network path. Only the final owned Resource is body-sized.
        match load_with(&server.url, &opts, &mut t.cache).unwrap() {
            Resource::Other { data, .. } => assert_eq!(data, body),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(server.requests(), 1);
        assert_eq!(t.cache.len(), 1);
        assert_eq!(t.cache.total_bytes(), 2 * 1024 * 1024);

        // The warm hit is served from the mmap view with zero network traffic.
        match load_with(&server.url, &opts, &mut t.cache).unwrap() {
            Resource::Other { data, .. } => assert_eq!(data, body),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(server.requests(), 1);
    }

    #[test]
    fn big_doc_warm_hit_perf() {
        // Big-doc corpus pattern: 2 MiB HTML served over HTTP, primed once,
        // then 100 warm `load_with` hits through the mmap path.
        let body = HTML.repeat(2 * 1024 * 1024 / HTML.len()).into_bytes();
        let server = CountingServer::start(body.clone(), 1, false, "text/html");
        let mut t = TempCache::new();
        let opts = options("bigdoc");
        let _ = load_with(&server.url, &opts, &mut t.cache).unwrap(); // prime
        for _ in 0..10 {
            let _ = load_with(&server.url, &opts, &mut t.cache).unwrap();
        }
        const N: u32 = 100;
        // "Before": force the read() fallback to model the old full-copy path.
        hodo_cache::set_forced_mmap_fallback(true);
        let start = Instant::now();
        for _ in 0..N {
            let _ = load_with(&server.url, &opts, &mut t.cache).unwrap();
        }
        let read_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
        // "After": the real mmap warm path.
        hodo_cache::set_forced_mmap_fallback(false);
        let start = Instant::now();
        for _ in 0..N {
            let _ = load_with(&server.url, &opts, &mut t.cache).unwrap();
        }
        let mmap_ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
        eprintln!(
            "big-doc warm hit: read {read_ms:.3} ms vs mmap {mmap_ms:.3} ms ({:.2}x)",
            read_ms / mmap_ms
        );
        assert_eq!(server.requests(), 1, "warm hits must not hit the network");
    }

    #[test]
    fn post_invalidation_key_matches_cached_get() {
        let key = cache_key_for_get("HTTP://Example.COM:80/a#frag").unwrap();
        assert_eq!(key.method, "GET");
        assert_eq!(key.url, "http://example.com/a");
        assert_eq!(key.partition, "");

        let mut t = TempCache::new();
        let entry = CacheEntry {
            url: key.url.clone(),
            content_type: "text/plain".into(),
            data: b"stale".to_vec(),
            status_code: 200,
            expires_at: Some(u64::MAX),
            etag: None,
            last_modified: None,
            vary: BTreeMap::new(),
            headers: BTreeMap::new(),
            partition_key: key.partition.clone(),
            request_method: "GET".into(),
            stored_at: 0,
        };
        t.cache.put(&key, &entry).unwrap();
        t.cache.invalidate(&key);
        assert!(t.cache.get(&key).is_none());
    }
}

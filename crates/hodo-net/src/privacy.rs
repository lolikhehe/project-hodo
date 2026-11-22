//! Privacy-aware fetch pipeline: tracking stripping, adblock, HSTS, HTTPS
//! upgrade, double-keyed cookies, referrer policy and warning collection.

use super::*;
use hodo_adblock::BlockDecision;
use hodo_cookie::CookieJar;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Advisory events surfaced by [`fetch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// An HTTPS page requested an `http://` subresource.
    MixedContent(String),
    /// An `http://` request was transparently upgraded to HTTPS.
    InsecureUpgraded(String),
    /// A filter rule matched in advisory (`Warn`) mode.
    Blocked(String),
}

/// Result of a privacy-aware fetch. `resource` is `None` only when a caller
/// deliberately asks for a metadata-only check (no such mode exists yet).
#[derive(Debug)]
pub struct FetchOutcome {
    pub resource: Option<Resource>,
    pub warnings: Vec<Warning>,
}

/// Counters for the future `about:hodo/privacy-report` page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrivacyStats {
    pub cookies: usize,
    pub blocked: u64,
    pub blocked_bytes: u64,
    pub warnings: u64,
    pub hsts_hosts: usize,
    pub blocked_urls: usize,
    pub warned_urls: usize,
}

const TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "utm_id",
    "utm_name",
    "fbclid",
    "gclid",
    "gclsrc",
    "msclkid",
    "dclid",
    "igshid",
    "wbraid",
    "gbraid",
    "ttclid",
    "twclid",
    "ysclid",
    "mc_cid",
    "mc_eid",
    "_ga",
    "_gl",
    "ref",
    "ref_src",
    "spm",
];

const MAX_HSTS_HOSTS: usize = 1000;
const MAX_HSTS_AGE: u64 = 400 * 24 * 60 * 60;
const MAX_WARN_SAMPLE: usize = 50;

// --- globals ---------------------------------------------------------------

static GLOBAL_JAR: OnceLock<Mutex<CookieJar>> = OnceLock::new();
fn global_jar() -> &'static Mutex<CookieJar> {
    GLOBAL_JAR.get_or_init(|| Mutex::new(CookieJar::new()))
}

#[derive(Debug, Clone, Copy)]
struct HstsEntry {
    expires_at: u64,
    inserted_at: u64,
    include_subdomains: bool,
}

static HSTS: OnceLock<Mutex<HashMap<String, HstsEntry>>> = OnceLock::new();
fn hsts_map() -> &'static Mutex<HashMap<String, HstsEntry>> {
    HSTS.get_or_init(|| Mutex::new(HashMap::new()))
}

static WARN_COUNT: AtomicU64 = AtomicU64::new(0);
static WARN_URLS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
fn warned_urls_ring() -> &'static Mutex<VecDeque<String>> {
    WARN_URLS.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_WARN_SAMPLE)))
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn note_warning(url: &str) {
    WARN_COUNT.fetch_add(1, Ordering::Relaxed);
    let mut ring = warned_urls_ring().lock().unwrap_or_else(|e| e.into_inner());
    if ring.len() == MAX_WARN_SAMPLE {
        ring.pop_front();
    }
    ring.push_back(url.to_string());
}

// --- pure helpers ----------------------------------------------------------

/// Remove the frozen tracking-parameter list from a URL's query string.
/// Fragment, path and all non-tracking parameters are preserved in order.
pub fn strip_tracking_params(url: &str) -> String {
    let Some(qpos) = url.find('?') else {
        return url.to_string();
    };
    let after = &url[qpos + 1..];
    let (query, fragment) = match after.find('#') {
        Some(pos) => (&after[..pos], Some(&after[pos..])),
        None => (after, None),
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|pair| {
            let key = pair
                .split('=')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            !TRACKING_PARAMS.contains(&key.as_str())
        })
        .collect();
    let mut out = url[..qpos].to_string();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(fragment) = fragment {
        out.push_str(fragment);
    }
    out
}

/// Lowercased `(scheme, host, path)` without userinfo or port.
fn split_url_parts(url: &str) -> (String, String, String) {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    let rest = rest.split('#').next().unwrap_or(rest);
    let (authority, tail) = match rest.find('/') {
        Some(pos) => (&rest[..pos], &rest[pos..]),
        None => (rest, "/"),
    };
    let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(end) = hostport.find(']') {
        &hostport[..=end]
    } else {
        hostport.rsplit_once(':').map_or(hostport, |(h, _)| h)
    };
    let host = host
        .trim_matches(|c| c == '[' || c == ']')
        .to_ascii_lowercase();
    let path = tail.split('?').next().unwrap_or(tail);
    (
        scheme.to_ascii_lowercase(),
        host,
        if path.is_empty() {
            "/".into()
        } else {
            path.into()
        },
    )
}

fn host_of(url: &str) -> String {
    split_url_parts(url).1
}

/// No PSL: last two labels, or the whole literal for an IP host.
fn registrable(host: &str) -> String {
    if host.parse::<std::net::Ipv4Addr>().is_ok()
        || host
            .trim_matches(|c| c == '[' || c == ']')
            .parse::<std::net::Ipv6Addr>()
            .is_ok()
    {
        return host.to_string();
    }
    let labels: Vec<&str> = host.split('.').filter(|s| !s.is_empty()).collect();
    if labels.len() <= 2 {
        host.to_string()
    } else {
        format!("{}.{}", labels[labels.len() - 2], labels[labels.len() - 1])
    }
}

/// True for IPv4/IPv6 literals (optionally bracketed). RFC 6797 §8.1.2
/// forbids HSTS state for these hosts.
fn is_ip_host(host: &str) -> bool {
    let h = host.trim_matches(|c| c == '[' || c == ']');
    h.parse::<std::net::Ipv4Addr>().is_ok() || h.parse::<std::net::Ipv6Addr>().is_ok()
}

fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let rest = rest.split('#').next().unwrap_or(rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if scheme.is_empty() || hostport.is_empty() {
        return None;
    }
    // Keep the port: different ports are different web origins.
    Some(format!("{}://{}", scheme.to_ascii_lowercase(), hostport))
}

/// Default policy: no Referer cross-origin; origin-only for same-origin.
fn referer_header(url: &str, referer: Option<&str>) -> Option<String> {
    let origin = origin_of(url)?;
    let referer_origin = origin_of(referer?)?;
    (origin == referer_origin).then(|| format!("{referer_origin}/"))
}

pub(crate) fn is_third_party(url: &str, top_site: &str, referer: Option<&str>) -> bool {
    let site = registrable(&host_of(url));
    if !top_site.is_empty() {
        return !site.eq_ignore_ascii_case(top_site);
    }
    referer
        .and_then(origin_of)
        .map(|origin| !registrable(&host_of(&origin)).eq_ignore_ascii_case(&site))
        .unwrap_or(false)
}

fn is_loopback(host: &str) -> bool {
    host == "localhost" || host == "::1" || host.starts_with("127.")
}

/// `http://` -> `https://`. Loopback hosts are exempt so local test/loopback
/// servers (and developer tooling) can still be reached over plain HTTP.
fn upgrade_to_https(url: &str, host: &str) -> Option<String> {
    let rest = url.strip_prefix("http://")?;
    (!is_loopback(host)).then(|| format!("https://{rest}"))
}

/// HSTS forces the upgrade even for otherwise-exempt loopback hosts.
fn upgrade_target(url: &str, host: &str, hsts_forces: bool) -> Option<String> {
    if hsts_forces {
        url.strip_prefix("http://")
            .map(|rest| format!("https://{rest}"))
    } else {
        upgrade_to_https(url, host)
    }
}

/// Re-apply the upgrade policy to a redirect target. Called for every hop so
/// an `https -> http` Location cannot slip through cleartext when HSTS applies.
/// Private fetches never consult the global HSTS map (empty local view).
pub(crate) fn upgrade_redirect(url: &str, private: bool) -> (String, bool) {
    let host = host_of(url);
    let hsts_forces = !private && hsts_applies(&host, now_unix());
    match upgrade_target(url, &host, hsts_forces) {
        Some(https) => (https, true),
        None => (url.to_string(), false),
    }
}

/// CHIPS-style double key: request site plus top-level site. A `Partitioned`
/// cookie set under site A is invisible when the same host loads under site B.
pub(crate) fn cookie_partition(scheme: &str, host: &str, top_site: &str) -> String {
    format!("{}|{}|{}", scheme, registrable(host), top_site)
}

fn parse_hsts(header: &str) -> Option<(u64, bool)> {
    let mut max_age = None;
    let mut include_subdomains = false;
    for directive in header.split(';') {
        let (name, value) = directive
            .trim()
            .split_once('=')
            .unwrap_or((directive.trim(), ""));
        match name.trim().to_ascii_lowercase().as_str() {
            "max-age" => max_age = value.trim().parse::<u64>().ok(),
            "includesubdomains" => include_subdomains = true,
            _ => {}
        }
    }
    max_age.map(|age| (age, include_subdomains))
}

fn hsts_applies(host: &str, now: u64) -> bool {
    let map = hsts_map().lock().unwrap_or_else(|e| e.into_inner());
    if map.get(host).is_some_and(|e| e.expires_at > now) {
        return true;
    }
    // An IP literal has no parent domain: never walk dot-suffixes, or a host
    // like "1" could force an upgrade on 127.0.0.1.
    if is_ip_host(host) {
        return false;
    }
    let mut start = 0;
    loop {
        let parent = &host[start..];
        if map
            .get(parent)
            .is_some_and(|e| e.include_subdomains && e.expires_at > now)
        {
            return true;
        }
        match parent.find('.') {
            Some(rel) => start += rel + 1,
            None => return false,
        }
    }
}

fn observe_hsts(host: &str, max_age: u64, include_subdomains: bool, now: u64) {
    if is_ip_host(host) {
        return;
    }
    let mut map = hsts_map().lock().unwrap_or_else(|e| e.into_inner());
    if max_age == 0 {
        map.remove(host);
        return;
    }
    map.retain(|_, e| e.expires_at > now);
    if !map.contains_key(host) && map.len() >= MAX_HSTS_HOSTS {
        // Evict by soonest expiry, then oldest insertion — not HashMap order.
        if let Some(oldest) = map
            .iter()
            .min_by_key(|(_, e)| (e.expires_at, e.inserted_at))
            .map(|(name, _)| name.clone())
        {
            map.remove(&oldest);
        }
    }
    map.insert(
        host.to_string(),
        HstsEntry {
            expires_at: now.saturating_add(max_age.min(MAX_HSTS_AGE)),
            inserted_at: now,
            include_subdomains,
        },
    );
}

/// Forget all in-memory HSTS state (called at the start of a private fetch).
pub fn clear_hsts() {
    hsts_map().lock().unwrap_or_else(|e| e.into_inner()).clear();
}

pub fn hsts_hosts() -> usize {
    hsts_map().lock().unwrap_or_else(|e| e.into_inner()).len()
}

// --- the pipeline ----------------------------------------------------------

/// Privacy-aware resource fetch. See [`FetchOptions`] for the controls.
pub fn fetch(
    url: &str,
    opts: &FetchOptions,
    cache: &mut DiskCache,
    jar: &mut CookieJar,
) -> Result<FetchOutcome, FetchError> {
    let started_at = now_unix();
    let mut warnings = Vec::new();
    // Private mode must never mutate the process-wide HSTS map: it gets an
    // empty local view via the `!opts.private` guards below.

    // 1. Tracking-parameter stripping (gated).
    let stripped = if opts.strip_tracking {
        strip_tracking_params(url)
    } else {
        url.to_string()
    };

    // 2. Adblock hook — top of the normalization pipeline.
    let third_party = is_third_party(&stripped, &opts.top_site, opts.referer.as_deref());
    match hodo_adblock::should_block_record(&stripped, &opts.top_site, third_party, !opts.private) {
        BlockDecision::Allow => {}
        BlockDecision::Block { rule } => return Err(FetchError::Blocked { rule }),
        BlockDecision::Warn { rule } => {
            warnings.push(Warning::Blocked(rule));
            note_warning(&stripped);
        }
    }

    // 3/4. HSTS check + HTTPS upgrade.
    let host = host_of(&stripped);
    let hsts_forces = !opts.private && hsts_applies(&host, started_at);
    let original = stripped.clone();
    let mut current = stripped;
    let mut upgraded = false;
    // HSTS forces the upgrade even for otherwise-exempt loopback hosts.
    let target = upgrade_target(&current, &host, hsts_forces);
    if let Some(https) = target {
        if opts.secure_context {
            warnings.push(Warning::MixedContent(original.clone()));
        }
        current = https;
        upgraded = true;
        warnings.push(Warning::InsecureUpgraded(original.clone()));
        // Both Warning values are emitted, but the URL is counted once.
        note_warning(&original);
    }

    // 5. Cookie header (private mode sends none) + referrer policy.
    let (scheme, request_host, _) = split_url_parts(&current);
    let cookie_partition = self::cookie_partition(&scheme, &request_host, &opts.top_site);
    let mut headers = Headers::new();
    if !opts.private {
        let cookies = jar.cookie_header(&current, &cookie_partition, &opts.top_site, started_at);
        if !cookies.is_empty() {
            headers.insert("cookie".into(), cookies);
        }
    }
    if let Some(referer) = referer_header(&current, opts.referer.as_deref()) {
        headers.insert("referer".into(), referer);
    }

    // 6. Load through the existing cached path (private = network only).
    let loaded = match load_with_privacy(&current, opts, cache, &headers) {
        Ok(loaded) => loaded,
        Err(err) if upgraded && matches!(err, FetchError::Network(_) | FetchError::Io(_)) => {
            return Err(FetchError::HttpsUpgradeFailed { url: original });
        }
        Err(err) => return Err(err),
    };

    // 7/9. Store Set-Cookie and observe HSTS (never in private mode).
    if !opts.private {
        for set_cookie in &loaded.set_cookies {
            jar.store(set_cookie, &loaded.final_url, &cookie_partition, started_at);
            // Mirror into the process-wide jar so `cookie_count()` is useful
            // for about:hodo even when callers own their tab-local jar.
            global_jar()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .store(set_cookie, &loaded.final_url, &cookie_partition, started_at);
        }
        for (hop_url, header) in &loaded.hsts {
            if let Some((max_age, include_subdomains)) = parse_hsts(header) {
                observe_hsts(&host_of(hop_url), max_age, include_subdomains, started_at);
            }
        }
    }

    Ok(FetchOutcome {
        resource: Some(loaded.resource),
        warnings,
    })
}

// --- report accessors ------------------------------------------------------

pub fn cookie_count() -> usize {
    global_jar().lock().unwrap_or_else(|e| e.into_inner()).len()
}

/// Blocked requests counted by the ad blocker (delegates to its global list).
pub fn blocked_count() -> u64 {
    hodo_adblock::blocked_count()
}

pub fn adblock_blocked_count() -> u64 {
    blocked_count()
}

pub fn blocked_urls() -> Vec<String> {
    hodo_adblock::blocked_urls()
}

pub fn warning_count() -> u64 {
    WARN_COUNT.load(Ordering::Relaxed)
}

pub fn warned_urls() -> Vec<String> {
    warned_urls_ring()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .cloned()
        .collect()
}

pub fn set_adblock_enabled(enabled: bool) {
    hodo_adblock::set_adblock_enabled(enabled);
}

pub fn allow_site(site: &str) {
    hodo_adblock::allow_site(site);
}

pub fn revoke_site(site: &str) {
    hodo_adblock::revoke_site(site);
}

pub fn is_site_allowed(site: &str) -> bool {
    hodo_adblock::is_site_allowed(site)
}

pub fn privacy_stats() -> PrivacyStats {
    PrivacyStats {
        cookies: cookie_count(),
        blocked: blocked_count(),
        blocked_bytes: hodo_adblock::blocked_bytes(),
        warnings: warning_count(),
        hsts_hosts: hsts_hosts(),
        blocked_urls: blocked_urls().len(),
        warned_urls: warned_urls().len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::thread::JoinHandle;

    const HTML: &str = "<!doctype html><html><body>ok</body></html>";

    /// Several tests touch the process-wide HSTS map; serialize them.
    static HSTS_TEST_LOCK: Mutex<()> = Mutex::new(());
    fn hsts_lock() -> std::sync::MutexGuard<'static, ()> {
        HSTS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Tests asserting exact warning-count deltas must not interleave with
    /// other fetches that emit upgrade warnings.
    static WARN_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Tests asserting exact adblock-counter deltas must be serialized.
    static ADBLOCK_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn temp_cache() -> DiskCache {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("hodo-net-privacy-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::create_dir_all(&root);
        DiskCache::open(&root, 8 * 1024 * 1024).unwrap()
    }

    #[test]
    fn strips_the_frozen_tracking_list() {
        let url = "https://example.com/p?utm_source=news&keep=1&fbclid=xyz&gclid=g&keep2=2&_ga=1&ref=x#frag";
        assert_eq!(
            strip_tracking_params(url),
            "https://example.com/p?keep=1&keep2=2#frag"
        );
        assert_eq!(
            strip_tracking_params("https://example.com/p?UTM_MEDIUM=x&a=1"),
            "https://example.com/p?a=1"
        );
        assert_eq!(
            strip_tracking_params("https://example.com/p?utm_source=x"),
            "https://example.com/p"
        );
        for param in TRACKING_PARAMS {
            let stripped = strip_tracking_params(&format!("https://x.test/?{param}=1"));
            assert!(!stripped.contains('?'), "failed to strip {param}");
        }
        assert_eq!(
            strip_tracking_params("https://x.test/a#b"),
            "https://x.test/a#b"
        );
    }

    #[test]
    fn https_upgrade_and_typed_failure() {
        assert_eq!(
            upgrade_to_https("http://example.com/a?x=1", "example.com"),
            Some("https://example.com/a?x=1".into())
        );
        assert_eq!(
            upgrade_to_https("https://example.com/", "example.com"),
            None
        );
        // Loopback stays http so the in-process test server is reachable.
        assert_eq!(upgrade_to_https("http://127.0.0.1:9/", "127.0.0.1"), None);
        assert_eq!(
            upgrade_target("http://127.0.0.1:9/", "127.0.0.1", false),
            None
        );
        assert_eq!(
            upgrade_target("http://127.0.0.1:9/", "127.0.0.1", true),
            Some("https://127.0.0.1:9/".into())
        );

        let err = FetchError::HttpsUpgradeFailed {
            url: "http://example.com/a".into(),
        };
        assert!(err.to_string().contains("example.com"));
    }

    #[test]
    fn upgrading_a_dead_host_is_a_typed_error() {
        let _warn_guard = WARN_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // 203.0.113.0/24 is TEST-NET-3: no real host; a closed port fails fast.
        let url = "http://203.0.113.1:9/dead";
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let opts = FetchOptions::default();
        let result = fetch(url, &opts, &mut cache, &mut jar);
        assert!(
            matches!(result, Err(FetchError::HttpsUpgradeFailed { .. })),
            "expected typed upgrade error, got {result:?}"
        );
    }

    #[test]
    fn mixed_content_and_upgrade_warnings() {
        let _warn_guard = WARN_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let opts = FetchOptions {
            secure_context: true,
            strip_tracking: false,
            ..FetchOptions::default()
        };
        let before = warning_count();
        // Non-loopback so the https upgrade is actually attempted; the dead
        // port makes the fetch fail without depending on a live server.
        let outcome = fetch("http://203.0.113.3:9/mixed", &opts, &mut cache, &mut jar).unwrap_err();
        assert!(matches!(outcome, FetchError::HttpsUpgradeFailed { .. }));
        assert_eq!(warning_count(), before + 1);
        assert!(!warned_urls().is_empty());
    }

    #[test]
    fn hsts_observe_upgrade_and_expiry() {
        let _guard = hsts_lock();
        clear_hsts();
        let now = 1_700_000_000;
        assert!(parse_hsts("max-age=100; includeSubDomains").is_some_and(|(a, s)| a == 100 && s));
        assert!(parse_hsts("max-age=0").is_some_and(|(a, _)| a == 0));
        assert!(parse_hsts("garbage").is_none());

        observe_hsts("example.com", 100, true, now);
        assert_eq!(hsts_hosts(), 1);
        assert!(hsts_applies("example.com", now + 1));
        assert!(hsts_applies("a.b.example.com", now + 1));
        assert!(!hsts_applies("evil-example.com", now + 1));
        assert!(!hsts_applies("example.com", now + 101));

        observe_hsts("example.com", 0, false, now); // max-age=0 clears
        assert_eq!(hsts_hosts(), 0);

        // 400-day cap.
        observe_hsts("capped.test", MAX_HSTS_AGE + 100, false, now);
        let map = hsts_map().lock().unwrap();
        assert_eq!(map["capped.test"].expires_at, now + MAX_HSTS_AGE);
        drop(map);
        clear_hsts();
    }

    #[test]
    fn hsts_host_cap_is_enforced() {
        let _guard = hsts_lock();
        clear_hsts();
        for i in 0..MAX_HSTS_HOSTS + 5 {
            observe_hsts(&format!("h{i}.test"), 1000, false, 100);
        }
        assert_eq!(hsts_hosts(), MAX_HSTS_HOSTS);
        clear_hsts();
    }

    fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }

    /// One-shot loopback HTTP server that records the raw request head.
    struct OneShot {
        url: String,
        head: Arc<Mutex<Vec<u8>>>,
        join: Option<JoinHandle<()>>,
    }

    impl OneShot {
        fn start(extra_headers: &str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let head = Arc::new(Mutex::new(Vec::new()));
            let seen = Arc::clone(&head);
            let extra = extra_headers.to_string();
            let join = std::thread::spawn(move || {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let end = loop {
                    if let Some(pos) = find_bytes(&buf, b"\r\n\r\n") {
                        break pos + 4;
                    }
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break buf.len(),
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                };
                *seen.lock().unwrap() = buf[..end].to_vec();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n{extra}\r\nConnection: close\r\n\r\n{HTML}",
                    HTML.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            });
            Self {
                url: format!("http://127.0.0.1:{port}/"),
                head,
                join: Some(join),
            }
        }

        fn head(&self) -> String {
            String::from_utf8_lossy(&self.head.lock().unwrap()).to_string()
        }
    }

    impl Drop for OneShot {
        fn drop(&mut self) {
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    #[test]
    fn cookie_attach_and_store_roundtrip_over_loopback() {
        let server = OneShot::start("Set-Cookie: sid=abc123; Path=/; SameSite=None; Secure\r\n");
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        // Pre-seed a cookie for this loopback partition.
        jar.store("seed=1", &server.url, "http|127.0.0.1|", 1_700_000_000);
        let opts = FetchOptions {
            use_cache: false,
            strip_tracking: false,
            ..FetchOptions::default()
        };
        let outcome = fetch(&server.url, &opts, &mut cache, &mut jar).unwrap();
        assert!(outcome.resource.is_some());
        let head = server.head();
        assert!(
            head.to_ascii_lowercase().contains("cookie: seed=1"),
            "{head}"
        );
        // The response cookie was stored into the caller's jar (Secure cookies
        // are retained but only sent over https, which loopback cannot do).
        assert_eq!(
            jar.cookie_header(&server.url, "http|127.0.0.1|", "", 1_700_000_000),
            "seed=1"
        );
    }

    #[test]
    fn adblock_block_routing_and_disable_paths() {
        let _guard = ADBLOCK_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_adblock_enabled(true);
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let opts = FetchOptions::default();
        let err = fetch(
            "https://doubleclick.net/banner",
            &opts,
            &mut cache,
            &mut jar,
        )
        .unwrap_err();
        assert!(matches!(err, FetchError::Blocked { .. }), "{err:?}");
        assert!(blocked_count() > 0);

        set_adblock_enabled(false);
        allow_site("doubleclick.net");
        assert!(is_site_allowed("doubleclick.net"));
        revoke_site("doubleclick.net");
        assert!(!is_site_allowed("doubleclick.net"));
        set_adblock_enabled(true);
    }

    #[test]
    fn private_fetch_and_prefetch_do_not_record_blocks() {
        let _guard = ADBLOCK_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_adblock_enabled(true);
        let url = "https://doubleclick.net/banner";
        let before = blocked_count();
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let private = FetchOptions {
            private: true,
            strip_tracking: false,
            ..FetchOptions::default()
        };
        assert!(matches!(
            fetch(url, &private, &mut cache, &mut jar),
            Err(FetchError::Blocked { .. })
        ));
        assert_eq!(
            blocked_count(),
            before,
            "private fetch must not record a block"
        );

        crate::prefetch_with_cache(url, &FetchOptions::default(), &mut cache);
        assert_eq!(blocked_count(), before, "prefetch must not record a block");
    }

    #[test]
    fn private_mode_leaves_no_persistent_side_effects() {
        let server = OneShot::start(
            "Set-Cookie: private=1; Path=/\r\nStrict-Transport-Security: max-age=1000\r\n",
        );
        let _guard = hsts_lock();
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        clear_hsts();
        let hsts_before = hsts_hosts();
        let cookies_before = cookie_count();
        let opts = FetchOptions {
            private: true,
            strip_tracking: false,
            ..FetchOptions::default()
        };
        let outcome = fetch(&server.url, &opts, &mut cache, &mut jar).unwrap();
        assert!(outcome.resource.is_some());
        assert!(outcome.warnings.is_empty());
        assert!(jar.is_empty(), "private jar must not receive Set-Cookie");
        assert_eq!(cookie_count(), cookies_before);
        assert_eq!(hsts_hosts(), hsts_before);
        assert_eq!(cache.len(), 0, "private mode must not write the cache");
        clear_hsts();
    }

    #[test]
    fn privacy_stats_snapshot_is_populated() {
        let stats = privacy_stats();
        assert_eq!(stats.cookies, cookie_count());
        assert_eq!(stats.blocked, blocked_count());
        assert!(stats.warned_urls <= MAX_WARN_SAMPLE);
    }

    #[test]
    fn private_fetch_does_not_erase_global_hsts() {
        let _guard = hsts_lock();
        clear_hsts();
        let now = 1_700_000_000;
        observe_hsts("known.test", 1000, false, now);
        assert!(hsts_applies("known.test", now + 1));

        let server = OneShot::start("");
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let opts = FetchOptions {
            private: true,
            strip_tracking: false,
            use_cache: false,
            ..FetchOptions::default()
        };
        fetch(&server.url, &opts, &mut cache, &mut jar).unwrap();
        assert!(
            hsts_applies("known.test", now + 1),
            "a private fetch must never clear upgrades learned by normal tabs"
        );
        clear_hsts();
    }

    #[test]
    fn cookie_partition_includes_the_top_site() {
        let a = cookie_partition("https", "tracker.test", "sitea.test");
        let b = cookie_partition("https", "tracker.test", "siteb.test");
        assert_ne!(a, b);
        let mut jar = CookieJar::new();
        jar.store(
            "seg=1; SameSite=None; Secure",
            "https://tracker.test/",
            &a,
            1_700_000_000,
        );
        assert_eq!(
            jar.cookie_header("https://tracker.test/", &a, "sitea.test", 1_700_000_000),
            "seg=1"
        );
        assert_eq!(
            jar.cookie_header("https://tracker.test/", &b, "siteb.test", 1_700_000_000),
            "",
            "same request host under a different top site must be isolated"
        );
    }

    #[test]
    fn hsts_never_applies_to_ip_literals() {
        let _guard = hsts_lock();
        clear_hsts();
        let now = 1_700_000_000;
        observe_hsts("1", 1000, true, now); // bogus single-label host
        observe_hsts("127.0.0.1", 1000, true, now); // must be ignored outright
        assert!(hsts_applies("1", now + 1));
        assert!(
            !hsts_applies("127.0.0.1", now + 1),
            "no dot-parent walking for IP literals"
        );
        assert!(!hsts_applies("::1", now + 1));
        assert_eq!(hsts_hosts(), 1, "IP literals must never be stored");
        clear_hsts();
    }

    #[test]
    fn an_upgraded_mixed_content_url_increments_the_warning_count_once() {
        let _warn_guard = WARN_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut cache = temp_cache();
        let mut jar = CookieJar::new();
        let opts = FetchOptions {
            secure_context: true,
            strip_tracking: false,
            ..FetchOptions::default()
        };
        let before = warning_count();
        let err = fetch("http://203.0.113.2:9/mixed", &opts, &mut cache, &mut jar).unwrap_err();
        assert!(matches!(err, FetchError::HttpsUpgradeFailed { .. }));
        assert_eq!(warning_count(), before + 1);
    }

    #[test]
    fn origin_keeps_the_port() {
        assert_eq!(
            origin_of("https://example.com:8443/a?x=1"),
            Some("https://example.com:8443".into())
        );
        assert_ne!(
            origin_of("https://example.com/a"),
            origin_of("https://example.com:8443/a")
        );
    }

    #[test]
    fn redirect_hops_reapply_hsts_and_private_ignores_it() {
        let _guard = hsts_lock();
        clear_hsts();
        let now = now_unix();
        observe_hsts("hsts.test", 1000, false, now);
        observe_hsts("localhost", 1000, false, now);
        let (upgraded, did) = upgrade_redirect("http://hsts.test/next", false);
        assert!(did);
        assert_eq!(upgraded, "https://hsts.test/next");
        // localhost is exempt from the generic upgrade; only HSTS forces it.
        let (forced, did) = upgrade_redirect("http://localhost/next", false);
        assert!(did, "HSTS must force upgrade on an exempt loopback host");
        assert_eq!(forced, "https://localhost/next");
        let (plain, did) = upgrade_redirect("http://localhost/next", true);
        assert!(!did, "private fetches must ignore global HSTS");
        assert_eq!(plain, "http://localhost/next");
        clear_hsts();
    }

    #[test]
    fn hsts_cap_evicts_the_soonest_expiring_entry() {
        let _guard = hsts_lock();
        clear_hsts();
        observe_hsts("short.test", 10, false, 100);
        for i in 0..MAX_HSTS_HOSTS {
            observe_hsts(&format!("n{i}.test"), 1000, false, 100);
        }
        assert_eq!(hsts_hosts(), MAX_HSTS_HOSTS);
        assert!(
            !hsts_applies("short.test", 101),
            "the shortest-lived host must be evicted first"
        );
        assert!(hsts_applies("n0.test", 101));
        clear_hsts();
    }
}

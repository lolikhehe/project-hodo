//! RAM-only RFC 6265bis cookie jar.
//!
//! No disk, no wall clock, no `url` crate: callers inject `now` (Unix seconds)
//! and the caller-supplied partition key. A private tab simply owns a fresh
//! [`CookieJar`] that is dropped when the tab closes.
//!
//! Deliberate simplifications (documented rather than hidden):
//! - no Public Suffix List — "site" is the last two DNS labels (or the whole
//!   IP literal). This is conservative for the SameSite checks we perform.
//! - `HttpOnly` is parsed but not enforced: this client has no script context,
//!   so there is no JavaScript path that could read the `Cookie` header.

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

const MAX_COOKIE_BYTES: usize = 4096;
const MAX_PER_PARTITION: usize = 180;
/// RFC 6265bis §5.6: user agents should limit cookies to ~400 days.
const MAX_LIFETIME_SECS: u64 = 400 * 24 * 60 * 60;

const MONTHS: &[&str] = &[
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone)]
struct Cookie {
    name: String,
    value: String,
    host_key: String,
    host_only: bool,
    domain: String,
    path: String,
    expires_at: Option<u64>,
    secure: bool,
    same_site: SameSite,
    partitioned: bool,
}

/// An in-memory, double-keyed cookie store.
#[derive(Debug, Default)]
pub struct CookieJar {
    /// `partition` -> cookies in insertion order (FIFO eviction).
    partitions: HashMap<String, Vec<Cookie>>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse one `Set-Cookie` value and store it. Invalid or over-limit cookies
    /// are silently ignored; a full partition evicts its oldest cookie.
    pub fn store(&mut self, set_cookie: &str, url: &str, partition: &str, now: u64) {
        let Some((name, value, attrs)) = parse_set_cookie(set_cookie) else {
            return;
        };
        if name.len() + value.len() > MAX_COOKIE_BYTES {
            return;
        }
        let (scheme, host, uri_path) = split_url(url);
        if host.is_empty() {
            return;
        }

        let mut max_age_expiry = None;
        let mut expires_expiry = None;
        let mut domain = String::new();
        let mut path = String::new();
        let mut secure = false;
        let mut same_site = SameSite::Lax;
        let mut partitioned = false;

        for (key, val) in attrs {
            match key.as_str() {
                "max-age" => {
                    if let Some(age) = val.and_then(|v| v.parse::<i64>().ok()) {
                        max_age_expiry = Some(if age <= 0 {
                            0 // already expired
                        } else {
                            now.saturating_add((age as u64).min(MAX_LIFETIME_SECS))
                        });
                    }
                }
                "expires" => {
                    expires_expiry = val
                        .as_deref()
                        .and_then(parse_http_date)
                        .map(|date| date.min(now.saturating_add(MAX_LIFETIME_SECS)))
                }
                "domain" => {
                    if let Some(d) = val {
                        let d = d.trim().trim_start_matches('.').to_ascii_lowercase();
                        if !d.is_empty() && !is_ip_host(&d) {
                            domain = d;
                        }
                    }
                }
                "path" => {
                    if let Some(p) = val {
                        if p.starts_with('/') {
                            path = p.to_string();
                        }
                    }
                }
                "secure" => secure = true,
                "httponly" => {} // no script context in this client
                "samesite" => {
                    same_site = match val.unwrap_or_default().trim().to_ascii_lowercase().as_str() {
                        "strict" => SameSite::Strict,
                        "none" => SameSite::None,
                        _ => SameSite::Lax,
                    }
                }
                "partitioned" => partitioned = true,
                _ => {}
            }
        }
        // Max-Age takes precedence over Expires regardless of attribute order.
        let expires_at = max_age_expiry.or(expires_expiry);

        // SameSite=None and Partitioned both require Secure (6265bis §5.6.7/§5.6.8).
        if (same_site == SameSite::None || partitioned) && !secure {
            return;
        }
        // The request host itself must be secure for a Secure cookie to be set.
        // A Domain cookie must domain-match the request host.
        let mut host_only = domain.is_empty();
        // RFC 6265bis §5.6: Domain equal to the request host is host-only.
        if !host_only && domain == host {
            host_only = true;
            domain.clear();
        }
        if !host_only && !domain_matches(&host, &domain) {
            return;
        }
        if secure && scheme != "https" {
            return;
        }

        let path = if path.is_empty() {
            default_path(&uri_path)
        } else {
            path
        };
        let domain = if host_only { host.clone() } else { domain };
        let expired = expires_at.is_some_and(|exp| exp <= now);
        let cookie = Cookie {
            name,
            value,
            host_key: host,
            host_only,
            domain,
            path,
            expires_at,
            secure,
            same_site,
            partitioned,
        };

        let jar = self.partitions.entry(partition.to_string()).or_default();
        // RFC 6265 §5.3: a new cookie with the same identity replaces the old,
        // keeping its FIFO position. An expired cookie deletes that same
        // identity (logout via Max-Age=0 / past Expires) before the early exit.
        if expired {
            jar.retain(|c| !same_identity(c, &cookie));
            return;
        }
        if let Some(pos) = jar.iter().position(|c| same_identity(c, &cookie)) {
            jar[pos] = cookie;
        } else {
            if jar.len() >= MAX_PER_PARTITION {
                jar.remove(0);
            }
            jar.push(cookie);
        }
    }

    /// Build a `Cookie:` header value for `url`. Empty string means no cookies.
    /// `top_site` is the registrable domain of the top-level site; an empty
    /// string signals a top-level navigation (site-for-cookies = request site).
    pub fn cookie_header(
        &mut self,
        url: &str,
        partition: &str,
        top_site: &str,
        now: u64,
    ) -> String {
        let (scheme, host, uri_path) = split_url(url);
        let Some(jar) = self.partitions.get_mut(partition) else {
            return String::new();
        };
        jar.retain(|c| c.expires_at.is_none_or(|exp| exp > now));

        let site = registrable(&host);
        let same_site = top_site.is_empty() || top_site.eq_ignore_ascii_case(site);
        let scheme = scheme.as_str();

        let mut pair = String::new();
        for cookie in jar.iter() {
            if !same_site {
                match cookie.same_site {
                    SameSite::Strict | SameSite::Lax => continue,
                    SameSite::None => {}
                }
            }
            if cookie.secure && scheme != "https" {
                continue;
            }
            let host_matches = if cookie.host_only {
                cookie.host_key == host
            } else {
                domain_matches(&host, &cookie.domain)
            };
            if !host_matches || !path_matches(&cookie.path, &uri_path) {
                continue;
            }
            if !pair.is_empty() {
                pair.push_str("; ");
            }
            pair.push_str(&cookie.name);
            pair.push('=');
            pair.push_str(&cookie.value);
        }
        pair
    }

    /// Drop every cookie in `partition`, leaving other partitions untouched.
    pub fn clear_partition(&mut self, partition: &str) {
        self.partitions.remove(partition);
    }

    /// Total cookies across all partitions.
    pub fn len(&self) -> usize {
        self.partitions.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of cookies in one partition (used by privacy reports).
    pub fn partition_len(&self, partition: &str) -> usize {
        self.partitions.get(partition).map_or(0, Vec::len)
    }

    /// True if this cookie was marked `Partitioned` when stored.
    pub fn is_partitioned(&self, partition: &str, name: &str) -> bool {
        self.partitions
            .get(partition)
            .is_some_and(|jar| jar.iter().any(|c| c.name == name && c.partitioned))
    }
}

fn same_identity(a: &Cookie, b: &Cookie) -> bool {
    a.name == b.name && a.host_key == b.host_key && a.domain == b.domain && a.path == b.path
}

type Attributes = Vec<(String, Option<String>)>;
type ParsedCookie = (String, String, Attributes);

/// `(name, value, [(attribute, optional_value)])`
fn parse_set_cookie(header: &str) -> Option<ParsedCookie> {
    let mut parts = header.split(';');
    let first = parts.next()?.trim();
    let (name, value) = first.split_once('=')?;
    let name = name.trim();
    let value = value.trim().trim_matches('"');
    if name.is_empty() || name.bytes().any(|b| b <= b' ' || b == b'=' || b == b';') {
        return None;
    }
    let mut attrs = Vec::new();
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, val) = match part.split_once('=') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), Some(v.trim().to_string())),
            None => (part.to_ascii_lowercase(), None),
        };
        attrs.push((key, val));
    }
    Some((name.to_string(), value.to_string(), attrs))
}

/// Lowercased `(scheme, host-without-userinfo-or-port, path)` from an absolute URL.
fn split_url(url: &str) -> (String, String, String) {
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

fn is_ip_host(host: &str) -> bool {
    let h = host.trim_matches(|c| c == '[' || c == ']');
    h.parse::<Ipv4Addr>().is_ok() || h.parse::<Ipv6Addr>().is_ok()
}

/// RFC 6265 §5.1.3. String domain-match. IP hosts are excluded from suffix
/// matching, so `1.2.3.4` can never receive a cookie for `3.4`.
fn domain_matches(host: &str, domain: &str) -> bool {
    if host == domain {
        return true;
    }
    !is_ip_host(host)
        && host.len() > domain.len()
        && host.as_bytes()[host.len() - domain.len() - 1] == b'.'
        && host.ends_with(domain)
}

/// RFC 6265 §5.1.4 default-path algorithm.
fn default_path(uri_path: &str) -> String {
    if !uri_path.starts_with('/') || uri_path.matches('/').count() <= 1 {
        return "/".into();
    }
    let pos = uri_path.rfind('/').unwrap_or(0);
    uri_path[..pos].to_string()
}

fn path_matches(cookie_path: &str, request_path: &str) -> bool {
    if cookie_path == request_path {
        return true;
    }
    if !request_path.starts_with(cookie_path) {
        return false;
    }
    cookie_path.ends_with('/') || request_path.as_bytes().get(cookie_path.len()) == Some(&b'/')
}

/// No PSL: last two labels, or the whole literal for an IP host.
fn registrable(host: &str) -> &str {
    if is_ip_host(host) {
        return host;
    }
    let mut parts = host.rsplitn(3, '.');
    let last = parts.next().unwrap_or(host);
    match parts.next() {
        Some(second) => {
            // A single-label host has no second label; rsplitn gives the host
            // itself in `last` and nothing else.
            let len = second.len() + 1 + last.len();
            &host[host.len() - len..]
        }
        None => last,
    }
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

/// IMF-fixdate only (`Sun, 06 Nov 1994 08:49:37 GMT`).
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

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_700_000_000;

    fn jar_with(cookie: &str, url: &str, partition: &str) -> CookieJar {
        let mut jar = CookieJar::new();
        jar.store(cookie, url, partition, NOW);
        jar
    }

    #[test]
    fn domain_match_subdomain_yes_evil_suffix_no_ip_strict() {
        let mut jar = jar_with("a=1; Domain=example.com", "https://www.example.com/x", "p");
        assert_eq!(
            jar.cookie_header("https://sub.example.com/y", "p", "", NOW),
            "a=1"
        );
        assert_eq!(
            jar.cookie_header("https://example.com/y", "p", "", NOW),
            "a=1"
        );

        let mut evil = jar_with(
            "b=2; Domain=evil-example.com",
            "https://evil-example.com/",
            "p",
        );
        assert_eq!(evil.cookie_header("https://example.com/", "p", "", NOW), "");

        // IP hosts cannot set domain cookies.
        let mut ip = jar_with("c=3; Domain=127.0.0.1", "http://127.0.0.1/", "p");
        assert_eq!(ip.cookie_header("http://127.0.0.1/", "p", "", NOW), "c=3");
        let mut bad_ip = CookieJar::new();
        bad_ip.store("d=4; Domain=1", "http://127.0.0.1/", "p", NOW);
        assert!(bad_ip.is_empty(), "domain must be rejected on an IP host");
    }

    #[test]
    fn secure_cookie_never_sent_over_http() {
        let mut jar = jar_with("s=1; Secure", "https://example.com/", "p");
        assert_eq!(
            jar.cookie_header("https://example.com/a", "p", "", NOW),
            "s=1"
        );
        assert_eq!(jar.cookie_header("http://example.com/a", "p", "", NOW), "");
    }

    #[test]
    fn lax_top_level_yes_subresource_no() {
        let mut jar = jar_with("lax=1; SameSite=Lax", "https://ads.example.com/", "p");
        // Cross-site subresource: not sent.
        assert_eq!(
            jar.cookie_header("https://ads.example.com/pixel", "p", "other.com", NOW),
            ""
        );
        // Top-level navigation (empty top_site): sent.
        assert_eq!(
            jar.cookie_header("https://ads.example.com/page", "p", "", NOW),
            "lax=1"
        );
    }

    #[test]
    fn strict_never_cross_site() {
        let mut jar = jar_with("strict=1; SameSite=Strict", "https://ads.example.com/", "p");
        assert_eq!(
            jar.cookie_header("https://ads.example.com/pixel", "p", "other.com", NOW),
            ""
        );
        assert_eq!(
            jar.cookie_header("https://ads.example.com/page", "p", "example.com", NOW),
            "strict=1"
        );
    }

    #[test]
    fn none_cross_site_only_with_secure() {
        let mut jar = CookieJar::new();
        jar.store("n=0; SameSite=None", "https://example.com/", "p", NOW);
        assert!(
            jar.is_empty(),
            "SameSite=None without Secure must be rejected"
        );

        let mut jar = jar_with(
            "n=1; SameSite=None; Secure",
            "https://ads.example.com/",
            "p",
        );
        assert_eq!(
            jar.cookie_header("https://ads.example.com/pixel", "p", "other.com", NOW),
            "n=1"
        );
    }

    #[test]
    fn max_age_precedence_and_partition_isolation() {
        let mut jar = CookieJar::new();
        jar.store(
            "a=1; Max-Age=1; Expires=Sun, 06 Nov 2094 08:49:37 GMT",
            "https://example.com/",
            "https|a.com",
            NOW,
        );
        jar.store(
            "b=2; Max-Age=100",
            "https://example.com/",
            "https|b.com",
            NOW,
        );
        assert_eq!(
            jar.cookie_header("https://example.com/", "https|a.com", "", NOW),
            "a=1"
        );
        assert_eq!(
            jar.cookie_header("https://example.com/", "https|b.com", "", NOW),
            "b=2"
        );
        assert_eq!(
            jar.cookie_header("https://example.com/", "https|a.com", "", NOW + 2),
            "",
            "Max-Age must beat Expires"
        );
        jar.clear_partition("https|a.com");
        assert_eq!(jar.len(), 1);
        assert_eq!(
            jar.cookie_header("https://example.com/", "https|b.com", "", NOW),
            "b=2"
        );
    }

    #[test]
    fn caps_partitioned_flag_and_default_path() {
        let big = "x=".to_string() + &"a".repeat(MAX_COOKIE_BYTES);
        let mut jar = CookieJar::new();
        jar.store(&big, "https://example.com/", "p", NOW);
        assert!(jar.is_empty(), "over 4096 bytes must be rejected");

        for i in 0..MAX_PER_PARTITION + 5 {
            jar.store(&format!("c{i}={i}"), "https://example.com/", "p", NOW);
        }
        assert_eq!(jar.len(), MAX_PER_PARTITION);
        let header = jar.cookie_header("https://example.com/", "p", "", NOW);
        assert!(header.starts_with("c5=5;"), "{header}");
        assert!(!header.contains("c4=4"), "{header}");

        let mut jar = CookieJar::new();
        jar.store("p=1; Partitioned", "https://example.com/", "p", NOW);
        assert!(
            jar.is_empty(),
            "Partitioned without Secure must be rejected"
        );
        jar.store("p=1; Partitioned; Secure", "https://example.com/", "p", NOW);
        assert!(jar.is_partitioned("p", "p"));

        let mut jar = jar_with("d=1", "https://example.com/a/b/page.html", "p");
        assert_eq!(
            jar.cookie_header("https://example.com/a/b/other", "p", "", NOW),
            "d=1"
        );
        assert_eq!(
            jar.cookie_header("https://example.com/a/c", "p", "", NOW),
            ""
        );
    }

    #[test]
    fn date_parser_and_registrable_domain() {
        assert_eq!(
            parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(784_111_777)
        );
        assert_eq!(parse_http_date("not a date"), None);
        assert_eq!(registrable("sub.www.example.com"), "example.com");
        assert_eq!(registrable("127.0.0.1"), "127.0.0.1");
    }

    #[test]
    fn max_age_zero_and_past_expires_delete_the_identity() {
        let mut jar = jar_with("a=1", "https://example.com/", "p");
        assert_eq!(
            jar.cookie_header("https://example.com/", "p", "", NOW),
            "a=1"
        );
        jar.store("a=2; Max-Age=0", "https://example.com/", "p", NOW);
        assert!(jar.is_empty(), "Max-Age=0 must delete the existing cookie");
        assert_eq!(jar.cookie_header("https://example.com/", "p", "", NOW), "");

        let mut jar = jar_with("b=2", "https://example.com/", "p");
        jar.store(
            "b=2; Expires=Sun, 06 Nov 1994 08:49:37 GMT",
            "https://example.com/",
            "p",
            NOW,
        );
        assert!(
            jar.is_empty(),
            "past Expires must delete the existing cookie"
        );
    }

    #[test]
    fn domain_equal_to_host_is_normalized_to_host_only() {
        let mut jar = jar_with("a=1; Domain=example.com", "https://example.com/", "p");
        assert_eq!(
            jar.cookie_header("https://example.com/x", "p", "", NOW),
            "a=1"
        );
        assert_eq!(
            jar.cookie_header("https://sub.example.com/x", "p", "", NOW),
            "",
            "Domain==host must become host-only"
        );
        assert!(!jar.is_partitioned("p", "a"));
    }

    #[test]
    fn replacement_keeps_fifo_position() {
        let mut jar = CookieJar::new();
        jar.store("a=1", "https://example.com/", "p", NOW);
        jar.store("b=1", "https://example.com/", "p", NOW);
        jar.store("c=1", "https://example.com/", "p", NOW);
        jar.store("b=2", "https://example.com/", "p", NOW);
        assert_eq!(
            jar.cookie_header("https://example.com/", "p", "", NOW),
            "a=1; b=2; c=1",
            "replacement must not move the cookie to the end"
        );
    }

    #[test]
    fn expires_is_capped_to_400_days() {
        let mut jar = CookieJar::new();
        jar.store(
            "a=1; Expires=Sun, 06 Nov 2094 08:49:37 GMT",
            "https://example.com/",
            "p",
            NOW,
        );
        assert_eq!(
            jar.cookie_header("https://example.com/", "p", "", NOW + MAX_LIFETIME_SECS - 1),
            "a=1"
        );
        assert_eq!(
            jar.cookie_header("https://example.com/", "p", "", NOW + MAX_LIFETIME_SECS),
            "",
            "Expires must honour the same 400-day cap as Max-Age"
        );
    }
}

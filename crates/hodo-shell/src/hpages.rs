//! Read-only `hodo://` settings and privacy pages.
//!
//! Wave 1 composes static HTML from the landed privacy / adblock / cache APIs
//! and feeds it through the same shell pipeline as `about:hodo` (synthetic HTML
//! -> normal engine render). Synthetic pages have no click routing yet, so all
//! values are read-only; toggles and allowlist edits ride the page-JS wave.

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::Mutex;

use serde::Deserialize;

use crate::about::{error_page_html, escape_html};

/// Synthetic settings page URL.
pub const HODO_SETTINGS_URL: &str = "hodo://settings";
/// Synthetic privacy report URL.
pub const HODO_PRIVACY_URL: &str = "hodo://privacy";

const HODO_SCHEME: &str = "hodo://";
const UNKNOWN_PAGE_HINT: &str =
    "Unknown hodo:// page. Valid pages: about:hodo, hodo://settings, hodo://privacy";

/// Route a `hodo://` URL to synthetic HTML. `None` means "not one of ours", so
/// the caller falls back to the normal resource loader. An unknown `hodo://`
/// path becomes a synthetic error page, never a network fetch.
pub fn hodo_page_html(url: &str) -> Option<String> {
    match url {
        HODO_SETTINGS_URL => Some(settings_html()),
        HODO_PRIVACY_URL => Some(privacy_html()),
        _ if url.starts_with(HODO_SCHEME) => Some(error_page_html(UNKNOWN_PAGE_HINT)),
        _ => None,
    }
}

/// `hodo://settings`: current adblock, privacy-flag and appearance state.
pub fn settings_html() -> String {
    let enabled = hodo_adblock::adblock_enabled();
    let (parsed, skipped) = hodo_adblock::rule_counts();
    let build_ms = hodo_adblock::automaton_build_millis();
    let stats = hodo_net::privacy_stats();
    let dark = read_dark_enabled();

    format!(
        "<html><head><title>hodo — settings</title></head><body>\
         <h1>hodo settings</h1>\
         <p>Read-only snapshot. Synthetic pages have no click routing yet; \
         toggles and allowlist edits ride the page-JS wave.</p>\
         <h2>Adblock</h2>\
         <ul>\
         <li>Status: {}</li>\
         <li>Rules parsed: {}, skipped: {}</li>\
         <li>Automaton build: {} ms</li>\
         <li>Blocked so far: {} requests, {} bytes</li>\
         <li>Per-site allowlist: individual allowances can be queried with \
         is_site_allowed(site), but the landed API exposes no listing or count; \
         editing rides the page-JS wave.</li>\
         </ul>\
         <h2>Privacy flags</h2>\
         <ul>\
         <li>HODO_NO_ADBLOCK: {} — disables adblock at startup.</li>\
         <li>HODO_NO_ISOLATION: {} — disables extension isolation at startup.</li>\
         </ul>\
         <h2>Appearance</h2>\
         <ul>\
         <li>Dark theme: {} (cache/design.json — set_dark_enabled(bool) writes it)</li>\
         </ul>\
         <h2>Privacy counters</h2>\
         <ul>\
         <li>Cookies: {}, warnings: {}, HSTS hosts: {}</li>\
         </ul>\
         </body></html>",
        if enabled { "Enabled" } else { "Disabled" },
        parsed,
        skipped,
        build_ms,
        stats.blocked,
        stats.blocked_bytes,
        env_flag("HODO_NO_ADBLOCK"),
        env_flag("HODO_NO_ISOLATION"),
        if dark { "on" } else { "off" },
        stats.cookies,
        stats.warnings,
        stats.hsts_hosts,
    )
}

/// `hodo://privacy`: the full report built only from landed accessors.
pub fn privacy_html() -> String {
    let stats = hodo_net::privacy_stats();
    let cache = hodo_cache::cache_stats();
    let blocked = hodo_net::blocked_urls();
    let warned = hodo_net::warned_urls();

    format!(
        "<html><head><title>hodo — privacy</title></head><body>\
         <h1>hodo privacy report</h1>\
         <h2>Overview</h2>\
         <ul>\
         <li>Cookies (global jar): {}</li>\
         <li>Cookie partition breakdown: not exposed by the landed API \
         (only per-partition counts are available).</li>\
         <li>Blocked requests: {} (adblock counter: {})</li>\
         <li>Blocked bytes: {}</li>\
         <li>Warnings: {}</li>\
         <li>HSTS hosts: {} (host list not exposed by the landed API)</li>\
         <li>Blocked URLs sampled: {}, warned URLs sampled: {}</li>\
         </ul>\
         <h2>Last {} blocked URLs</h2>\
         {} \
         <p>The landed ring stores URLs only; matched rule text is surfaced \
         through block errors and warnings, not per URL.</p>\
         <h2>Warned URLs (mixed content / HTTPS upgrades)</h2>\
         {}\
         <h2>Cache</h2>\
         <ul>\
         <li>{} hits, {} misses, {} bytes, {} evictions</li>\
         </ul>\
         </body></html>",
        stats.cookies,
        stats.blocked,
        hodo_net::adblock_blocked_count(),
        stats.blocked_bytes,
        stats.warnings,
        stats.hsts_hosts,
        stats.blocked_urls,
        stats.warned_urls,
        blocked.len(),
        url_list_html(&blocked),
        url_list_html(&warned),
        cache.hits,
        cache.misses,
        cache.bytes,
        cache.evictions,
    )
}

/// Escape and wrap a URL ring as a list, so hostile URLs cannot inject markup.
fn url_list_html(items: &[String]) -> String {
    if items.is_empty() {
        return "<p>None recorded.</p>".to_string();
    }
    let mut out = String::from("<ul>");
    for item in items {
        out.push_str("<li>");
        out.push_str(&escape_html(item));
        out.push_str("</li>");
    }
    out.push_str("</ul>");
    out
}

fn env_flag(name: &str) -> &'static str {
    let on = std::env::var(name).is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if on {
        "set"
    } else {
        "unset"
    }
}

#[derive(Deserialize)]
struct Design {
    dark: bool,
}

fn design_path(root: &Path) -> PathBuf {
    root.join("cache").join("design.json")
}

fn read_dark_enabled_from(root: &Path) -> bool {
    let Ok(text) = fs::read_to_string(design_path(root)) else {
        return false;
    };
    serde_json::from_str::<Design>(&text)
        .map(|design| design.dark)
        .unwrap_or(false)
}

/// Current dark-theme flag. Missing or malformed `cache/design.json` falls
/// back to off. Consumed by this page and by the restyle wave.
pub fn read_dark_enabled() -> bool {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    read_dark_enabled_from(&root)
}

// Consumed by the restyle wave; wave 1 only reads the flag.
#[allow(dead_code)]
fn write_dark_enabled_to(root: &Path, enabled: bool) -> std::io::Result<()> {
    let dir = root.join("cache");
    fs::create_dir_all(&dir)?;
    let path = design_path(root);
    let tmp = dir.join("design.json.tmp");
    let body = format!("{{\"dark\":{enabled}}}");
    fs::write(&tmp, body)?;
    // shortcut: Windows cannot rename over an existing file, so the target is
    // removed first; the brief gap is acceptable for a local theme preference.
    if path.exists() {
        fs::remove_file(&path)?;
    }
    fs::rename(&tmp, &path)
}

/// Persist the dark-theme flag to `cache/design.json` as `{"dark":bool}`.
// Consumed by the restyle wave; wave 1 only reads the flag.
#[allow(dead_code)]
pub fn set_dark_enabled(enabled: bool) {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = write_dark_enabled_to(&root, enabled);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::about::ABOUT_HODO_URL;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Several tests touch process-global adblock state; serialize them.
    pub(crate) static PAGE_TEST_LOCK: Mutex<()> = Mutex::new(());
    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    pub(crate) fn page_lock() -> std::sync::MutexGuard<'static, ()> {
        PAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn temp_root() -> PathBuf {
        let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("hodo-hpages-{}-{n}", std::process::id()))
    }

    #[test]
    fn settings_page_reflects_known_adblock_state() {
        let _guard = page_lock();
        let before = hodo_adblock::adblock_enabled();

        hodo_adblock::set_adblock_enabled(false);
        assert!(settings_html().contains("Status: Disabled"));

        hodo_adblock::set_adblock_enabled(true);
        let page = settings_html();
        assert!(page.contains("Status: Enabled"));
        let (parsed, skipped) = hodo_adblock::rule_counts();
        assert!(page.contains(&format!("Rules parsed: {parsed}, skipped: {skipped}")));
        assert!(page.contains(&format!(
            "Automaton build: {} ms",
            hodo_adblock::automaton_build_millis()
        )));
        assert!(page.contains("HODO_NO_ADBLOCK") && page.contains("HODO_NO_ISOLATION"));
        assert!(page.contains("is_site_allowed") && page.contains("page-JS wave"));
        assert!(page.contains("Dark theme:"));

        hodo_adblock::set_adblock_enabled(before);
    }

    #[test]
    fn privacy_page_reports_landed_stats() {
        let _guard = page_lock();
        let page = privacy_html();
        let stats = hodo_net::privacy_stats();
        assert!(page.contains("hodo privacy report"));
        assert!(page.contains(&format!("Cookies (global jar): {}", stats.cookies)));
        assert!(page.contains(&format!("Blocked requests: {}", stats.blocked)));
        assert!(page.contains(&format!("Blocked bytes: {}", stats.blocked_bytes)));
        assert!(page.contains(&format!("Warnings: {}", stats.warnings)));
        assert!(page.contains(&format!("HSTS hosts: {}", stats.hsts_hosts)));
        assert!(page.contains("hits,") && page.contains("misses,") && page.contains("evictions"));
    }

    #[test]
    fn url_ring_rendering_escapes_markup() {
        let urls = vec!["https://ads.test/<script>alert(1)</script>&x=1".to_string()];
        let list = url_list_html(&urls);
        assert!(list.contains("&lt;script&gt;"));
        assert!(list.contains("&amp;x=1"));
        assert!(!list.contains("<script>"));
        assert_eq!(escape_html("<b>&\""), "&lt;b&gt;&amp;&quot;");
    }

    #[test]
    fn hodo_url_routing_renders_pages_and_errors_unknown_paths() {
        assert!(hodo_page_html("https://example.com/").is_none());
        assert!(hodo_page_html(HODO_SETTINGS_URL)
            .expect("settings route")
            .contains("<title>hodo — settings</title>"));
        assert!(hodo_page_html(HODO_PRIVACY_URL)
            .expect("privacy route")
            .contains("<title>hodo — privacy</title>"));

        let err = hodo_page_html("hodo://nope").expect("unknown hodo path still yields a page");
        assert!(err.contains("hodo cannot load this page"));
        for valid in [ABOUT_HODO_URL, HODO_SETTINGS_URL, HODO_PRIVACY_URL] {
            assert!(err.contains(valid), "error page must list {valid}");
        }

        let (title, list) = crate::app::load_page(HODO_SETTINGS_URL, crate::app::DEFAULT_VIEWPORT);
        assert_eq!(title, "hodo — settings");
        assert!(!list.items.is_empty());
        let (_, err_list) = crate::app::load_page("hodo://nope", crate::app::DEFAULT_VIEWPORT);
        assert!(!err_list.items.is_empty());
    }

    #[test]
    fn design_json_roundtrip_and_corrupt_fallback() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();

        assert!(!read_dark_enabled_from(&root)); // missing file -> off
        write_dark_enabled_to(&root, true).unwrap();
        assert_eq!(
            fs::read_to_string(design_path(&root)).unwrap(),
            "{\"dark\":true}"
        );
        assert!(read_dark_enabled_from(&root));
        write_dark_enabled_to(&root, false).unwrap();
        assert!(!read_dark_enabled_from(&root));

        fs::write(design_path(&root), b"{not json").unwrap();
        assert!(!read_dark_enabled_from(&root)); // corrupt file -> off

        fs::remove_dir_all(&root).ok();
    }
}

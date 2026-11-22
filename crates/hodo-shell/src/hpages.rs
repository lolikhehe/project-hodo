//! Live `hodo://` settings and privacy pages.
//!
//! Settings are real form controls named `setting:<key>`; the shell's form
//! registry hit-tests them, the settings module persists/applies the change and
//! hodo-render paints M3E switches from the `setting:` name prefix.

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

fn switch_row(label: &str, key: &str, on: bool, restart: bool) -> String {
    format!(
        "<div class=\"row\"><input class=\"switch\" type=\"checkbox\" \
         name=\"setting:{key}\" value=\"{}\"{}> <span>{}{}</span></div>",
        if on { "on" } else { "off" },
        if on { " checked" } else { "" },
        escape_html(label),
        if restart { " (restart)" } else { "" }
    )
}

fn radio_row(label: &str, key: &str, value: &str, selected: bool) -> String {
    format!(
        "<div class=\"row\"><input class=\"radio\" type=\"radio\" \
         name=\"setting:{key}\" value=\"{value}\"{}> <span>{label}</span></div>",
        if selected { " checked" } else { "" }
    )
}

fn action_button(label: &str, key: &str) -> String {
    format!(
        "<div class=\"row\"><button type=\"button\" name=\"setting:{key}\">{label}</button></div>"
    )
}

/// `hodo://settings`: live, persisted knobs rendered as M3E switches.
pub fn settings_html() -> String {
    let s = crate::settings::current();
    let enabled = hodo_adblock::adblock_enabled();
    let (parsed, skipped) = hodo_adblock::rule_counts();
    let build_ms = hodo_adblock::automaton_build_millis();
    let stats = hodo_net::privacy_stats();
    let dark = read_dark_enabled();
    let preset = crate::settings::ram_preset(&s.ram_budget);

    let mut out = String::from(
        "<html><head><title>hodo — settings</title><style>\
         body{font-family:sans-serif;font-size:14px}\
         .switch{width:52px;height:44px}\
         .radio{width:20px;height:20px}\
         .row{margin:4px 0}\
         </style></head><body><h1>hodo settings</h1>",
    );
    out.push_str(&format!(
        "<p>Settings loaded from: {} — env vars override file values.</p>",
        escape_html(&crate::settings::settings_path().display().to_string())
    ));

    out.push_str("<h2>Appearance</h2>");
    out.push_str(&switch_row("Dark theme", "theme", dark, false));

    out.push_str("<h2>Privacy &amp; Security</h2>");
    out.push_str(&switch_row(
        "Enable JavaScript",
        "javascript",
        s.javascript,
        true,
    ));
    out.push_str(&switch_row(
        "Block ads &amp; trackers",
        "adblock",
        enabled,
        false,
    ));
    out.push_str(&switch_row("HTTPS only", "https_only", s.https_only, false));
    out.push_str(&switch_row("HSTS", "hsts_enable", s.hsts_enable, false));
    out.push_str(&switch_row(
        "Strip referrer to origin",
        "referrer_strip",
        s.referrer_strip,
        false,
    ));
    out.push_str(&switch_row(
        "Extension isolation",
        "ext_isolation",
        s.ext_isolation,
        true,
    ));
    out.push_str(&format!(
        "<p>Adblock: {} rules parsed, {} skipped, automaton built in {} ms; \
         {} requests / {} bytes blocked.</p>",
        parsed, skipped, build_ms, stats.blocked, stats.blocked_bytes
    ));
    out.push_str("<h3>Adblock allowlist</h3>");
    let sites = hodo_adblock::allowed_sites();
    if sites.is_empty() {
        out.push_str("<p>None.</p>");
    } else {
        for site in sites {
            out.push_str(&format!(
                "<div class=\"row\">{} {}</div>",
                escape_html(&site),
                action_button("Revoke", &format!("revoke:{site}"))
            ));
        }
    }
    out.push_str(&action_button("Clear HSTS hosts", "clear_hsts"));

    out.push_str("<h2>Performance</h2>");
    out.push_str(&format!(
        "<p>RAM budget: {} — spill {}, cache {} MiB, glyph atlas {} MiB, JS budget {} ms.</p>",
        escape_html(&s.ram_budget),
        preset.spill_threshold,
        preset.cache_cap_mb,
        preset.glyph_cache_mb,
        preset.js_budget_ms
    ));
    out.push_str(&radio_row(
        "Low",
        "ram_budget",
        "low",
        s.ram_budget == "low",
    ));
    out.push_str(&radio_row(
        "Balanced",
        "ram_budget",
        "balanced",
        s.ram_budget == "balanced",
    ));
    out.push_str(&radio_row(
        "High",
        "ram_budget",
        "high",
        s.ram_budget == "high",
    ));
    out.push_str(&switch_row(
        "Prefetch on hover",
        "prefetch_enabled",
        s.prefetch_enabled,
        false,
    ));

    out.push_str("<h2>Advanced</h2>");
    out.push_str(&switch_row(
        "Force full repaint",
        "force_full_repaint",
        s.force_full_repaint,
        false,
    ));
    out.push_str(&switch_row(
        "Debug damage",
        "debug_damage",
        s.debug_damage,
        false,
    ));
    out.push_str(&switch_row(
        "Private by default",
        "private_by_default",
        s.private_by_default,
        false,
    ));
    out.push_str(&switch_row(
        "Restore sessions",
        "restore_sessions",
        s.restore_sessions,
        true,
    ));

    out.push_str(&format!(
        "<h2>Privacy counters</h2><ul><li>Cookies: {}, warnings: {}, HSTS hosts: {}</li></ul>",
        stats.cookies, stats.warnings, stats.hsts_hosts
    ));
    out.push_str(
        "<p>Env overrides: HODO_NO_ADBLOCK, HODO_NO_ISOLATION, HODO_NO_PAGE_JS, \
         HODO_SPILL_THRESHOLD.</p></body></html>",
    );
    out
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

/// Persist the dark-theme flag to `cache/design.json` as `{"dark":bool}`.
/// Atomic temp+rename: no remove window where the file is missing.
pub(crate) fn write_dark_enabled_to(root: &Path, enabled: bool) -> std::io::Result<()> {
    let dir = root.join("cache");
    fs::create_dir_all(&dir)?;
    let path = design_path(root);
    let tmp = dir.join("design.json.tmp");
    let body = format!("{{\"dark\":{enabled}}}");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, &path)
}

pub(crate) fn set_dark_enabled(enabled: bool) {
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
    fn settings_page_renders_live_switches() {
        let _guard = page_lock();
        crate::settings::set_bool("adblock", false);
        let page = settings_html();
        assert!(page.contains("name=\"setting:adblock\" value=\"off\""));
        assert!(page.contains("name=\"setting:javascript\" value=\"on\" checked"));
        assert!(page.contains("Enable JavaScript") && page.contains("(restart)"));
        assert!(page.contains("name=\"setting:ram_budget\" value=\"balanced\""));
        assert!(page.contains("HODO_NO_ADBLOCK") && page.contains("HODO_NO_ISOLATION"));
        assert!(page.contains("Settings loaded from:"));
        assert!(page.contains("Rules parsed") || page.contains("rules parsed"));

        crate::settings::set_bool("adblock", true);
        let page = settings_html();
        assert!(page.contains("name=\"setting:adblock\" value=\"on\" checked"));
        assert!(page.contains("Dark theme"));
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
    fn about_page_names_the_settings_file() {
        let page = crate::about::about_hodo_html();
        assert!(page.contains("Settings loaded from:"));
        assert!(page.contains("settings.json"));
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

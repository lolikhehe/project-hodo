//! Persisted browser settings for `hodo://settings`.
//!
//! One file, `<CWD>/cache/settings.json`, written with the same atomic
//! temp+rename pattern as `sessions.json`. A missing or corrupt file falls back
//! to defaults that match the browser's previous hardcoded values, so a fresh
//! start behaves exactly as before.
//!
//! Precedence is **env > file > default**: env vars are applied after loading
//! the file, and engine read sites still consult their own env var first.
//!
//! Knob map, groups, presets and restart flags follow the scout deliverable in
//! the settings-wave brief (wave 1 `hodo://settings` knob map); the shipped
//! default of every knob equals the value that brief records as hardcoded.

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

/// Snapshot of every user-visible knob. `PartialEq` makes roundtrip tests terse.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    // Appearance
    pub theme: String,
    /// Omnibox search engine for non-URL queries. `duckduckgo` uses the
    /// `https://lite.duckduckgo.com/lite/?q=` template, which renders with our
    /// engine today. `google` selects `https://google.com/search?q=` but is
    /// declared-only: Google is JS-walled for us today (blank render).
    pub search_engine: String,
    // Privacy & Security
    pub javascript: bool,
    pub adblock: bool,
    pub adblock_allowlist: Vec<String>,
    pub https_only: bool,
    pub hsts_enable: bool,
    pub referrer_strip: bool,
    pub ext_isolation: bool,
    // Performance
    pub ram_budget: String,
    pub spill_threshold: usize,
    pub cache_cap_mb: usize,
    pub glyph_cache_mb: f32,
    pub js_budget_ms: u64,
    pub prefetch_enabled: bool,
    // Advanced
    pub force_full_repaint: bool,
    pub debug_damage: bool,
    pub private_by_default: bool,
    pub restore_sessions: bool,
}

/// The three RAM presets. Defaults are the values the shell hardcoded before
/// this wave: 8 display lists, a 128 MiB disk cache, a 1 MiB glyph atlas and
/// the 50 ms GUI script budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RamPreset {
    pub spill_threshold: usize,
    pub cache_cap_mb: usize,
    pub glyph_cache_mb: f32,
    pub js_budget_ms: u64,
}

pub(crate) fn ram_preset(name: &str) -> RamPreset {
    match name {
        "low" => RamPreset {
            spill_threshold: 4,
            cache_cap_mb: 64,
            glyph_cache_mb: 0.5,
            js_budget_ms: 50,
        },
        "high" => RamPreset {
            spill_threshold: 16,
            cache_cap_mb: 256,
            glyph_cache_mb: 2.0,
            js_budget_ms: 50,
        },
        _ => RamPreset {
            spill_threshold: 8,
            cache_cap_mb: 128,
            glyph_cache_mb: 1.0,
            js_budget_ms: 50,
        },
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            search_engine: "duckduckgo".into(),
            javascript: true,
            adblock: true,
            adblock_allowlist: Vec::new(),
            https_only: true,
            hsts_enable: true,
            referrer_strip: true,
            ext_isolation: true,
            ram_budget: "balanced".into(),
            spill_threshold: 8,
            cache_cap_mb: 128,
            glyph_cache_mb: 1.0,
            js_budget_ms: 50,
            prefetch_enabled: true,
            force_full_repaint: false,
            debug_damage: false,
            private_by_default: false,
            restore_sessions: true,
        }
    }
}

fn env_flag(name: &str) -> Option<bool> {
    std::env::var(name)
        .ok()
        .and_then(|value| match value.as_str() {
            "1" => Some(true),
            "0" => Some(false),
            other if other.eq_ignore_ascii_case("true") => Some(true),
            other if other.eq_ignore_ascii_case("false") => Some(false),
            _ => None,
        })
}

fn env_usize(name: &str) -> Option<usize> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

/// Apply env overrides to a loaded snapshot. Kept separate so unit tests can
/// check precedence without touching the process-global settings lock.
fn with_env_overrides(mut s: Settings) -> Settings {
    if let Ok(v) = std::env::var("HODO_THEME") {
        if v == "dark" || v == "light" {
            s.theme = v;
        }
    }
    if env_flag("HODO_NO_PAGE_JS") == Some(true) {
        s.javascript = false;
    }
    if env_flag("HODO_NO_ADBLOCK") == Some(true) {
        s.adblock = false;
    }
    if env_flag("HODO_NO_ISOLATION") == Some(true) {
        s.ext_isolation = false;
    }
    if env_flag("HODO_NO_HTTPS_ONLY") == Some(true) {
        s.https_only = false;
    }
    if env_flag("HODO_NO_HSTS") == Some(true) {
        s.hsts_enable = false;
    }
    if env_flag("HODO_NO_REFERRER_STRIP") == Some(true) {
        s.referrer_strip = false;
    }
    if env_flag("HODO_NO_PREFETCH") == Some(true) {
        s.prefetch_enabled = false;
    }
    if env_flag("HODO_FORCE_FULL") == Some(true) {
        s.force_full_repaint = true;
    }
    if env_flag("HODO_DEBUG_DAMAGE") == Some(true) {
        s.debug_damage = true;
    }
    if env_flag("HODO_PRIVATE") == Some(true) {
        s.private_by_default = true;
    }
    if env_flag("HODO_NO_RESTORE_SESSIONS") == Some(true) {
        s.restore_sessions = false;
    }
    if let Some(v) = env_usize("HODO_SPILL_THRESHOLD") {
        s.spill_threshold = v;
    }
    if let Some(v) = env_usize("HODO_CACHE_CAP_MB") {
        s.cache_cap_mb = v;
    }
    if let Some(v) = std::env::var("HODO_GLYPH_CACHE_MB")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
    {
        s.glyph_cache_mb = v;
    }
    if let Some(v) = env_usize("HODO_JS_BUDGET_MS") {
        if v > 0 {
            s.js_budget_ms = v as u64;
        }
    }
    if let Ok(v) = std::env::var("HODO_RAM_BUDGET") {
        if matches!(v.as_str(), "low" | "balanced" | "high") {
            s.ram_budget = v;
            let preset = ram_preset(&s.ram_budget);
            s.spill_threshold = preset.spill_threshold;
            s.cache_cap_mb = preset.cache_cap_mb;
            s.glyph_cache_mb = preset.glyph_cache_mb;
            s.js_budget_ms = preset.js_budget_ms;
        }
    }
    s
}

fn cache_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("cache")
}

pub(crate) fn settings_path() -> PathBuf {
    cache_dir().join("settings.json")
}

/// Load from disk once, fall back to defaults on any error, then push every
/// knob into the engine. Safe to call at every startup.
pub(crate) fn load() -> Settings {
    let s = load_from(&settings_path());
    let mut guard = SETTINGS
        .get_or_init(|| Mutex::new(s.clone()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *guard = s.clone();
    drop(guard);
    apply_all(&s);
    s
}

fn load_from(path: &std::path::Path) -> Settings {
    // One exit: whichever fallback we take, env overrides are applied last so
    // a corrupt/missing file can never silently discard them.
    let loaded = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
        .unwrap_or_default();
    with_env_overrides(loaded)
}

static SETTINGS: OnceLock<Mutex<Settings>> = OnceLock::new();

/// `HODO_SPILL_THRESHOLD` is process-global (also read by `sessions`), so the
/// test that sets it and every test that reads it must share this guard.
#[cfg(test)]
pub(crate) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Cheap read of the current snapshot. Never panics, even before `load()`.
pub(crate) fn current() -> Settings {
    SETTINGS
        .get()
        .map(|slot| slot.lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_default()
}

/// Temp file for an atomic save. The pid keeps two concurrent writers (e.g. a
/// running browser and a CLI invocation) from renaming each other's partial
/// file out from under the rename step.
fn temp_path_for(path: &std::path::Path, pid: u32) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!("{name}.tmp-{pid}"))
}

/// Atomically persist a snapshot (write temp, then rename — no remove window).
pub(crate) fn save(s: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(json) = serde_json::to_string_pretty(s) else {
        return;
    };
    let temp = temp_path_for(&path, std::process::id());
    if fs::write(&temp, json).is_ok() {
        let _ = fs::rename(&temp, &path);
    } else {
        let _ = fs::remove_file(&temp);
    }
}

/// Push all knobs into the engine. Called on startup and after each toggle so
/// the read sites never need a dependency on this crate.
fn apply_all(s: &Settings) {
    hodo_adblock::set_adblock_enabled(s.adblock);
    for site in &s.adblock_allowlist {
        hodo_adblock::allow_site(site);
    }
    hodo_net::set_https_only(s.https_only);
    hodo_net::set_hsts_enable(s.hsts_enable);
    hodo_net::set_referrer_strip(s.referrer_strip);
    hodo_net::set_prefetch_enabled(s.prefetch_enabled);
    hodo_render::set_glyph_cache_cap_mb(s.glyph_cache_mb);
    hodo_ext_host::set_isolation_enabled(s.ext_isolation);
}

fn mutate_with(f: impl FnOnce(&mut Settings), persist: bool) {
    let slot = SETTINGS.get_or_init(|| Mutex::new(Settings::default()));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard);
    let next = guard.clone();
    drop(guard);
    apply_all(&next);
    if persist {
        save(&next);
    }
}

fn mutate(f: impl FnOnce(&mut Settings)) {
    mutate_with(f, true);
}

fn set_bool_field(s: &mut Settings, name: &str, value: bool) {
    match name {
        "javascript" => s.javascript = value,
        "adblock" => s.adblock = value,
        "https_only" => s.https_only = value,
        "hsts_enable" => s.hsts_enable = value,
        "referrer_strip" => s.referrer_strip = value,
        "ext_isolation" => s.ext_isolation = value,
        "prefetch_enabled" => s.prefetch_enabled = value,
        "force_full_repaint" => s.force_full_repaint = value,
        "debug_damage" => s.debug_damage = value,
        "private_by_default" => s.private_by_default = value,
        "restore_sessions" => s.restore_sessions = value,
        _ => {}
    }
}

/// Toggle a named knob and persist it. `value` carries the radio value for
/// preset controls; `checked` drives booleans and the switch visual.
pub(crate) fn apply_control(name: &str, checked: bool, value: &str) {
    match name {
        "theme" => {
            crate::hpages::set_dark_enabled(checked);
            mutate(|s| s.theme = if checked { "dark" } else { "light" }.into());
        }
        "search_engine" if matches!(value, "duckduckgo" | "google") => {
            mutate(|s| s.search_engine = value.into());
        }
        "javascript" | "adblock" | "https_only" | "hsts_enable" | "referrer_strip"
        | "ext_isolation" | "prefetch_enabled" | "force_full_repaint" | "debug_damage"
        | "private_by_default" | "restore_sessions" => {
            mutate(|s| set_bool_field(s, name, checked));
        }
        "ram_budget" => {
            let preset = ram_preset(value);
            mutate(|s| {
                s.ram_budget = value.into();
                s.spill_threshold = preset.spill_threshold;
                s.cache_cap_mb = preset.cache_cap_mb;
                s.glyph_cache_mb = preset.glyph_cache_mb;
                s.js_budget_ms = preset.js_budget_ms;
            });
        }
        "clear_hsts" => hodo_net::clear_hsts(),
        other if other.starts_with("revoke:") => {
            let site = &other["revoke:".len()..];
            hodo_adblock::revoke_site(site);
            mutate(|s| s.adblock_allowlist.retain(|entry| entry != site));
        }
        _ => {}
    }
}

/// CLI `--private` sets the default in memory for this launch only: it must not
/// silently rewrite the user's persisted preference.
pub(crate) fn set_runtime_private_by_default(value: bool) {
    if let Some(slot) = SETTINGS.get() {
        slot.lock()
            .unwrap_or_else(|e| e.into_inner())
            .private_by_default = value;
    }
}

/// Test hook: set a boolean knob in memory only (no disk write, no cross-test
/// pollution of the workspace `cache/` directory).
#[cfg(test)]
pub(crate) fn set_bool(key: &str, value: bool) {
    mutate_with(|s| set_bool_field(s, key, value), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(tag: &str) -> PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "hodo-settings-{tag}-{}-{n}.json",
            std::process::id()
        ))
    }

    #[test]
    fn defaults_match_the_hardcoded_baseline() {
        let s = Settings::default();
        assert_eq!(s.theme, "light");
        assert_eq!(s.search_engine, "duckduckgo");
        assert!(s.javascript && s.adblock && s.https_only && s.hsts_enable);
        assert!(s.referrer_strip && s.ext_isolation && s.prefetch_enabled);
        assert_eq!(s.spill_threshold, 8);
        assert_eq!(s.cache_cap_mb, 128);
        assert_eq!(s.glyph_cache_mb, 1.0);
        assert_eq!(s.js_budget_ms, 50);
        assert!(s.restore_sessions);
        assert!(!s.private_by_default);
    }

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let _env = env_lock();
        let p = path("roundtrip");
        let s = Settings {
            adblock: false,
            ram_budget: "high".into(),
            search_engine: "google".into(),
            adblock_allowlist: vec!["ads.test".into()],
            ..Settings::default()
        };
        save_to(&p, &s);
        assert_eq!(load_from(&p), s);

        let expected = with_env_overrides(Settings::default());
        fs::write(&p, b"{not json").unwrap();
        assert_eq!(
            load_from(&p),
            expected,
            "corrupt file must still honour env"
        );
        assert_eq!(load_from(&path("missing")), expected);
        let _ = fs::remove_file(&p);
    }

    /// Save to an explicit path (production `save` uses `settings_path`).
    fn save_to(path: &std::path::Path, s: &Settings) {
        let json = serde_json::to_string_pretty(s).unwrap();
        let temp = temp_path_for(path, std::process::id());
        fs::write(&temp, json).unwrap();
        fs::rename(&temp, path).unwrap();
    }

    #[test]
    fn ram_budget_presets_map_exactly() {
        assert_eq!(
            ram_preset("low"),
            RamPreset {
                spill_threshold: 4,
                cache_cap_mb: 64,
                glyph_cache_mb: 0.5,
                js_budget_ms: 50
            }
        );
        assert_eq!(
            ram_preset("balanced"),
            RamPreset {
                spill_threshold: 8,
                cache_cap_mb: 128,
                glyph_cache_mb: 1.0,
                js_budget_ms: 50
            }
        );
        assert_eq!(
            ram_preset("high"),
            RamPreset {
                spill_threshold: 16,
                cache_cap_mb: 256,
                glyph_cache_mb: 2.0,
                js_budget_ms: 50
            }
        );
    }

    #[test]
    fn private_default_reaches_new_tabs() {
        let _guard = crate::hpages::tests::page_lock();
        set_bool("private_by_default", true);
        let app = crate::tabs::App::new(None);
        assert!(
            app.active_tab().private,
            "private_by_default must make new tabs private"
        );
        set_bool("private_by_default", false);
    }

    #[test]
    fn atomic_save_leaves_no_temp_file() {
        let _env = env_lock();
        let p = path("atomic");
        let s = Settings::default();
        save_to(&p, &s);
        assert!(!temp_path_for(&p, std::process::id()).exists());
        assert_eq!(load_from(&p), s);
        // shortcut: kill-mid-write is not simulated; temp+rename guarantees the
        // reader sees either the old file or the new one, never a partial file.
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn temp_path_is_pid_scoped() {
        let p = path("tmp-name");
        let mine = temp_path_for(&p, 1234);
        let other = temp_path_for(&p, 5678);
        assert_ne!(mine, other, "concurrent writers must not share a temp file");
        assert!(mine.to_string_lossy().ends_with("tmp-1234"));
    }

    #[test]
    fn env_overrides_file_value() {
        // Serialise with the page tests (they call `settings::current()`) and
        // with every reader of this process-global env var, including the
        // spill-threshold test in `tabs`.
        let _guard = crate::hpages::tests::page_lock();
        let _env = env_lock();
        std::env::set_var("HODO_SPILL_THRESHOLD", "3");
        let s = with_env_overrides(Settings::default());
        assert_eq!(s.spill_threshold, 3);
        // The same overrides must survive the corrupt-file fallback.
        let p = path("corrupt-env");
        fs::write(&p, b"{not json").unwrap();
        assert_eq!(load_from(&p).spill_threshold, 3);
        std::env::remove_var("HODO_SPILL_THRESHOLD");
        let _ = fs::remove_file(&p);
    }
}

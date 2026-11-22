//! Session persistence for normal tabs.
//!
//! Snapshot lives at `<cache_dir>/sessions.json`, where `cache_dir` is the
//! same `<CWD>/cache` directory the shell's [`DiskCache`](hodo_cache::DiskCache)
//! owner uses — one on-disk location, no second cache subsystem. Private tabs
//! are excluded from both the snapshot and the restore.
//!
//! Writes are atomic (write temp + rename) so a crash cannot leave a truncated
//! file that would be mistaken for a valid session on the next launch.
//!
//! # Inactive-tab display-list spill
//!
//! `cache/spill/<uuid>.bin.lz4` holds the display list of an inactive tab that
//! exceeded `HODO_SPILL_THRESHOLD` (default **8**; **0** disables spilling).
//! Payload is `serde` + `bincode` 2.0.1, compressed with `lz4_flex` 0.14.2
//! (safe, pure-Rust block codec) behind an 8-byte length header. A spilled tab
//! keeps only its URL/title and a `Body::Spilled` marker in RAM; switching to
//! it decompresses in place, and a missing/corrupt file falls back to a normal
//! page load. Orphan files are swept at startup against the saved session.

use std::fs;
use std::path::{Path, PathBuf};

use hodo_layout::{ControlKind, DisplayItem, DisplayList};
use hodo_types::{Color, Rect};
use serde::{Deserialize, Serialize};

use crate::tabs::{App, Body, Tab};

/// Maximum number of tabs written to / restored from a session.
pub const MAX_SESSION_TABS: usize = 32;
/// Maximum history entries kept per tab (oldest entries are dropped first).
pub const MAX_HISTORY_ENTRIES: usize = 512;

/// Serialized form of one normal tab. `history_pos` is the index into
/// `history` of the currently displayed URL. No display data is persisted
/// here; a spilled tab is identified by `uuid` and rebuilt from its spill file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
    pub history: Vec<String>,
    pub history_pos: usize,
    /// Spill-file id for an inactive tab; empty for a live tab or an old
    /// session written before spilling existed (`#[serde(default)]` keeps
    /// those snapshots loadable).
    #[serde(default)]
    pub uuid: String,
}

/// The directory holding both the disk cache and `sessions.json`.
fn cache_dir() -> PathBuf {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    root.join("cache")
}

/// Path of the session snapshot. Exposed so callers can inspect it in logs.
pub fn sessions_path() -> PathBuf {
    cache_dir().join("sessions.json")
}

/// Drop the oldest history entries beyond the cap and keep `history_pos`
/// pointing at a valid entry in the retained slice.
fn cap_tab(mut tab: SessionTab) -> SessionTab {
    let dropped = tab.history.len().saturating_sub(MAX_HISTORY_ENTRIES);
    if dropped != 0 {
        tab.history.drain(..dropped);
    }
    let last = tab.history.len().saturating_sub(1);
    tab.history_pos = tab.history_pos.saturating_sub(dropped).min(last);
    tab
}

/// Build the snapshot from an [`App`], skipping private tabs and the tabs past
/// the 32-tab budget.
pub fn snapshot(app: &App) -> Vec<SessionTab> {
    app.tabs
        .iter()
        .filter(|tab| !tab.private)
        .take(MAX_SESSION_TABS)
        .map(|tab| {
            cap_tab(SessionTab {
                url: tab.url.clone(),
                title: tab.title.clone(),
                history: tab.history.clone(),
                history_pos: tab.history_cursor,
                uuid: tab.spill_uuid().unwrap_or("").to_string(),
            })
        })
        .collect()
}

/// Persist `app`'s normal tabs. All I/O errors are ignored: a failed session
/// write must never take down the browser.
pub fn save_sessions(app: &App) {
    let tabs = snapshot(app);
    let path = sessions_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(json) = serde_json::to_string_pretty(&tabs) else {
        return;
    };
    let temp = path.with_extension("json.tmp");
    if fs::write(&temp, json).is_ok() {
        let _ = fs::rename(&temp, &path);
    } else {
        let _ = fs::remove_file(&temp);
    }
}

/// Load a snapshot from `path`. Missing, unreadable or malformed files yield
/// an empty session so the caller starts fresh — this function never panics.
fn load_from(path: &Path) -> Vec<SessionTab> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(tabs) = serde_json::from_str::<Vec<SessionTab>>(&text) else {
        return Vec::new();
    };
    tabs.into_iter()
        .take(MAX_SESSION_TABS)
        .map(cap_tab)
        .collect()
}

/// Restore the snapshot written by [`save_sessions`]. Missing/corrupt file →
/// empty `Vec`, i.e. a fresh start.
pub fn restore_sessions() -> Vec<SessionTab> {
    load_from(&sessions_path())
}

// ---------------------------------------------------------------------------
// Inactive-tab display-list spill
// ---------------------------------------------------------------------------

/// Refuse to allocate more than this for a decompressed spill payload.
const MAX_SPILL_BYTES: usize = 256 * 1024 * 1024;
/// Bump when the on-disk format changes; older versions are ignored/reloaded.
const SPILL_VERSION: u32 = 1;
const SPILL_SUFFIX: &str = ".bin.lz4";

/// `HODO_SPILL_THRESHOLD`: how many display lists stay in RAM; `0` disables
/// spilling. Unparsable values fall back to the default.
pub(crate) fn spill_threshold() -> usize {
    std::env::var("HODO_SPILL_THRESHOLD")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| crate::settings::current().spill_threshold)
}

#[cfg(test)]
thread_local! {
    /// Per-test spill directory: an unrelated `App::restore` on another test
    /// thread must not sweep a test's spill files as orphans.
    static TEST_SPILL_DIR: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Spill directory, under the same `cache/` root as `sessions.json`.
pub(crate) fn spill_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(dir) = TEST_SPILL_DIR.with(|slot| slot.borrow().clone()) {
        return dir;
    }
    cache_dir().join("spill")
}

/// Point this test thread at a private spill directory.
#[cfg(test)]
pub(crate) fn test_spill_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("hodo-spill-{tag}-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    TEST_SPILL_DIR.with(|slot| *slot.borrow_mut() = Some(dir.clone()));
    dir
}

/// Drop this thread's spill-dir override and its files.
#[cfg(test)]
pub(crate) fn clear_test_spill_dir(dir: &Path) {
    TEST_SPILL_DIR.with(|slot| *slot.borrow_mut() = None);
    let _ = fs::remove_dir_all(dir);
}

fn spill_path(uuid: &str) -> PathBuf {
    spill_dir().join(format!("{uuid}{SPILL_SUFFIX}"))
}

/// True when a session entry's spill file is still on disk.
pub(crate) fn spill_file_exists(uuid: &str) -> bool {
    spill_path(uuid).exists()
}

/// Process-unique, filename-safe id. Generated locally so no `uuid` crate is
/// needed for a handful of hex characters.
fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let seq = NEXT.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    format!(
        "{:08x}-{:08x}-{:08x}-{:08x}",
        std::process::id(),
        (now >> 32) as u32,
        now as u32,
        seq as u32
    )
}

// Serde mirrors of the engine's value types: hodo-layout/types/dom deliberately
// carry no serde derives, so the on-disk shape is defined here instead.
#[derive(Clone, Serialize, Deserialize)]
struct DtoRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Clone, Serialize, Deserialize)]
struct DtoColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Clone, Serialize, Deserialize)]
enum DtoItem {
    Rect {
        rect: DtoRect,
        color: DtoColor,
        border_color: Option<DtoColor>,
        radius: f32,
    },
    Text {
        rect: DtoRect,
        text: String,
        color: DtoColor,
        font_size: f32,
        weight: u16,
        href: Option<String>,
        underline: bool,
        line_through: bool,
    },
    Image {
        rect: DtoRect,
        path: String,
    },
    Control {
        rect: DtoRect,
        kind: u8,
        text: String,
        name: String,
        masked: bool,
        disabled: bool,
        readonly: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct SpilledPayload {
    version: u32,
    items: Vec<DtoItem>,
    node_map: Vec<u32>,
    fixed_items: Vec<DtoItem>,
    fixed_node_map: Vec<u32>,
    scroll_offset: f32,
    page_height: f32,
    url: String,
    title: String,
    history: Vec<String>,
    history_pos: usize,
}

fn dto_rect(rect: Rect) -> DtoRect {
    DtoRect {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
    }
}

fn rect_from_dto(rect: DtoRect) -> Rect {
    Rect {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
    }
}

fn dto_color(color: Color) -> DtoColor {
    DtoColor {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

fn color_from_dto(color: DtoColor) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

fn kind_code(kind: ControlKind) -> u8 {
    match kind {
        ControlKind::TextInput => 0,
        ControlKind::Checkbox => 1,
        ControlKind::Radio => 2,
        ControlKind::Button => 3,
        ControlKind::Select => 4,
        ControlKind::Textarea => 5,
        ControlKind::Option => 6,
    }
}

fn kind_from_code(code: u8) -> ControlKind {
    match code {
        1 => ControlKind::Checkbox,
        2 => ControlKind::Radio,
        3 => ControlKind::Button,
        4 => ControlKind::Select,
        5 => ControlKind::Textarea,
        6 => ControlKind::Option,
        _ => ControlKind::TextInput,
    }
}

fn dto_item(item: &DisplayItem) -> DtoItem {
    match item {
        DisplayItem::Rect {
            rect,
            color,
            border_color,
            radius,
        } => DtoItem::Rect {
            rect: dto_rect(*rect),
            color: dto_color(*color),
            border_color: border_color.map(dto_color),
            radius: *radius,
        },
        DisplayItem::Text {
            rect,
            text,
            color,
            font_size,
            weight,
            href,
            underline,
            line_through,
        } => DtoItem::Text {
            rect: dto_rect(*rect),
            text: text.clone(),
            color: dto_color(*color),
            font_size: *font_size,
            weight: *weight,
            href: href.clone(),
            underline: *underline,
            line_through: *line_through,
        },
        DisplayItem::Image { rect, path } => DtoItem::Image {
            rect: dto_rect(*rect),
            path: path.clone(),
        },
        DisplayItem::Control {
            rect,
            kind,
            text,
            name,
            masked,
            disabled,
            readonly,
        } => DtoItem::Control {
            rect: dto_rect(*rect),
            kind: kind_code(*kind),
            text: text.clone(),
            name: name.clone(),
            masked: *masked,
            disabled: *disabled,
            readonly: *readonly,
        },
    }
}

fn item_from_dto(item: DtoItem) -> DisplayItem {
    match item {
        DtoItem::Rect {
            rect,
            color,
            border_color,
            radius,
        } => DisplayItem::Rect {
            rect: rect_from_dto(rect),
            color: color_from_dto(color),
            border_color: border_color.map(color_from_dto),
            radius,
        },
        DtoItem::Text {
            rect,
            text,
            color,
            font_size,
            weight,
            href,
            underline,
            line_through,
        } => DisplayItem::Text {
            rect: rect_from_dto(rect),
            text,
            color: color_from_dto(color),
            font_size,
            weight,
            href,
            underline,
            line_through,
        },
        DtoItem::Image { rect, path } => DisplayItem::Image {
            rect: rect_from_dto(rect),
            path,
        },
        DtoItem::Control {
            rect,
            kind,
            text,
            name,
            masked,
            disabled,
            readonly,
        } => DisplayItem::Control {
            rect: rect_from_dto(rect),
            kind: kind_from_code(kind),
            text,
            name,
            masked,
            disabled,
            readonly,
        },
    }
}

/// Serialize + compress one live tab's display list, then free it in RAM.
/// Returns `false` (leaving the tab untouched) on any codec or I/O error.
pub(crate) fn spill_tab(tab: &mut Tab) -> bool {
    if !matches!(tab.body, Body::Live) {
        return false;
    }
    let payload = SpilledPayload {
        version: SPILL_VERSION,
        items: tab.display_list.items.iter().map(dto_item).collect(),
        node_map: tab
            .display_list
            .node_map
            .iter()
            .map(|owner| owner.0)
            .collect(),
        fixed_items: tab.display_list.fixed_items.iter().map(dto_item).collect(),
        fixed_node_map: tab
            .display_list
            .fixed_node_map
            .iter()
            .map(|owner| owner.0)
            .collect(),
        scroll_offset: tab.scroll_offset,
        page_height: tab.page_height,
        url: tab.url.clone(),
        title: tab.title.clone(),
        history: tab.history.clone(),
        history_pos: tab.history_cursor,
    };
    let Ok(bytes) = bincode::serde::encode_to_vec(&payload, bincode::config::standard()) else {
        return false;
    };
    let mut blob = (bytes.len() as u64).to_le_bytes().to_vec();
    blob.extend_from_slice(&lz4_flex::compress(&bytes));
    let uuid = new_id();
    let path = spill_path(&uuid);
    if fs::create_dir_all(spill_dir()).is_err() || fs::write(&path, &blob).is_err() {
        return false;
    }
    tab.display_list = DisplayList::default();
    tab.body = Body::Spilled { uuid };
    true
}

/// Rebuild a live display list from its spill file. `false` means missing,
/// corrupt, oversized or wrong-version data and leaves `tab` untouched, so the
/// caller can reload the URL instead of panicking.
pub(crate) fn restore_tab(tab: &mut Tab) -> bool {
    let Body::Spilled { uuid } = &tab.body else {
        return true;
    };
    let Ok(blob) = fs::read(spill_path(uuid)) else {
        return false;
    };
    if blob.len() < 8 {
        return false;
    }
    let Ok(header) = <[u8; 8]>::try_from(&blob[..8]) else {
        return false;
    };
    let size = u64::from_le_bytes(header) as usize;
    if size > MAX_SPILL_BYTES {
        return false;
    }
    let Ok(bytes) = lz4_flex::decompress(&blob[8..], size) else {
        return false;
    };
    let Ok((payload, _read)) =
        bincode::serde::decode_from_slice::<SpilledPayload, _>(&bytes, bincode::config::standard())
    else {
        return false;
    };
    if payload.version != SPILL_VERSION {
        return false;
    }
    tab.display_list = DisplayList {
        items: payload.items.iter().cloned().map(item_from_dto).collect(),
        node_map: payload.node_map.into_iter().map(hodo_dom::NodeId).collect(),
        fixed_items: payload
            .fixed_items
            .iter()
            .cloned()
            .map(item_from_dto)
            .collect(),
        fixed_node_map: payload
            .fixed_node_map
            .into_iter()
            .map(hodo_dom::NodeId)
            .collect(),
    };
    tab.scroll_offset = payload.scroll_offset;
    tab.page_height = payload.page_height;
    tab.url = payload.url;
    tab.title = payload.title;
    tab.history = payload.history;
    tab.history_cursor = payload.history_pos;
    tab.body = Body::Live;
    true
}

/// Remove the spill file of a closed tab.
pub(crate) fn delete_spill(uuid: &str) {
    let _ = fs::remove_file(spill_path(uuid));
}

/// Startup/session-clear sweep: delete every spill file whose id is not named
/// by the restored session. Ignores I/O errors — spilling is an optimization.
pub(crate) fn sweep_orphans(keep: &[String]) {
    let Ok(entries) = fs::read_dir(spill_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(uuid) = name.strip_suffix(SPILL_SUFFIX) else {
            continue;
        };
        if !keep.iter().any(|id| id == uuid) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_path(tag: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "hodo-session-{tag}-{}-{id}.json",
            std::process::id()
        ))
    }

    fn sample(pos: usize) -> SessionTab {
        SessionTab {
            url: "/c".into(),
            title: "Page C".into(),
            history: vec!["/a".into(), "/b".into(), "/c".into()],
            history_pos: pos,
            uuid: String::new(),
        }
    }

    fn write(path: &Path, tabs: &[SessionTab]) {
        fs::write(path, serde_json::to_string(tabs).unwrap()).unwrap();
    }

    #[test]
    fn save_restore_roundtrip_keeps_history_and_cursor() {
        let path = temp_path("roundtrip");
        let tabs = vec![sample(0), sample(1), sample(2)];
        write(&path, &tabs);
        assert_eq!(load_from(&path), tabs);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_or_missing_file_starts_fresh() {
        let missing = temp_path("missing");
        assert!(load_from(&missing).is_empty());

        let corrupt = temp_path("corrupt");
        fs::write(&corrupt, b"{not json at all").unwrap();
        assert!(load_from(&corrupt).is_empty());
        let _ = fs::remove_file(&corrupt);
    }

    #[test]
    fn oversized_history_is_capped_and_cursor_rebased() {
        let path = temp_path("cap");
        let history: Vec<String> = (0..600).map(|i| format!("/p{i}")).collect();
        let tab = SessionTab {
            url: "/p599".into(),
            title: String::new(),
            history,
            history_pos: 590,
            uuid: String::new(),
        };
        write(&path, &[tab]);
        let restored = load_from(&path);
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].history.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(restored[0].history.first().unwrap(), "/p88");
        assert_eq!(restored[0].history.last().unwrap(), "/p599");
        assert_eq!(restored[0].history_pos, 502); // 590 - 88 dropped
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cursor_is_clamped_into_retained_history() {
        let path = temp_path("clamp");
        let tab = SessionTab {
            url: "/a".into(),
            title: String::new(),
            history: vec!["/a".into()],
            history_pos: 99,
            uuid: String::new(),
        };
        write(&path, &[tab]);
        let restored = load_from(&path);
        assert_eq!(restored[0].history_pos, 0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn private_tabs_are_excluded_and_tab_budget_is_enforced() {
        let dir = test_spill_dir("budget");
        let mut app = App::new(Some("about:normal0"));
        // 45 appended tabs: 3 private, 42 normal → well over the 32-tab budget.
        for i in 1..46 {
            app.new_tab();
            let private = i == 5 || i == 15 || i == 25;
            let url = if private {
                format!("about:private{i}")
            } else {
                format!("about:normal{i}")
            };
            let tab = &mut app.tabs[app.active];
            tab.url = url;
            tab.history = vec![tab.url.clone()];
            tab.private = private;
        }
        let saved = snapshot(&app);
        assert_eq!(saved.len(), MAX_SESSION_TABS);
        assert!(
            saved.iter().all(|t| !t.url.contains("private")),
            "no private tab may appear in the snapshot"
        );
        // The first (normal) tab is kept and the 32-tab budget is respected.
        assert_eq!(saved[0].url, "about:normal0");
        clear_test_spill_dir(&dir);
    }

    #[test]
    fn session_tab_never_serializes_display_data() {
        let json = serde_json::to_string(&sample(1)).unwrap();
        for forbidden in ["items", "node_map", "fixed_items", "display_list"] {
            assert!(
                !json.contains(forbidden),
                "session leaked {forbidden}: {json}"
            );
        }
        assert!(json.contains("history_pos"));
        assert!(json.contains("uuid"));
    }

    #[test]
    fn orphan_spill_files_are_swept_but_kept_ids_survive() {
        let dir = test_spill_dir("orphan");
        let keep = new_id();
        let orphan = new_id();
        let keep_path = spill_path(&keep);
        let orphan_path = spill_path(&orphan);
        fs::write(&keep_path, b"keep").unwrap();
        fs::write(&orphan_path, b"orphan").unwrap();

        sweep_orphans(std::slice::from_ref(&keep));
        assert!(
            keep_path.exists(),
            "a tab in the session keeps its spill file"
        );
        assert!(
            !orphan_path.exists(),
            "an orphan spill file must be deleted"
        );

        clear_test_spill_dir(&dir);
    }
}

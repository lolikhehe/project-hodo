//! Tab model + history stack.
//!
//! Design goal: keep RAM flat regardless of tab count. A `Tab` stores only
//! the URL, title and a flat `DisplayList`; the DOM/stylesheet/style map are
//! created and dropped inside [`load_page`](crate::app::load_page). Only the
//! active tab ever has a pixel buffer, rebuilt on navigation, tab switch and
//! resize (~33 ms).
//!
//! # Inactive-tab spill
//! `HODO_SPILL_THRESHOLD` (default 8, `0` disables) caps how many tabs keep a
//! display list in RAM. Excess least-recently-used *inactive* tabs are written
//! to `cache/spill/<uuid>.bin.lz4`; their `Body` becomes `Spilled` and the
//! in-memory list is emptied. Switching to such a tab restores it in place, or
//! reloads its URL through the normal pipeline if the file is missing/corrupt.
//! The on-disk format and the orphan sweep live in [`crate::sessions`].

use std::cell::Cell;

use hodo_layout::DisplayList;
use hodo_types::Size;

use crate::about::ABOUT_HODO_URL;
use crate::app::{load_page, DEFAULT_VIEWPORT};
use crate::ui::{clamp_scroll_offset, compose_display_list, display_bottom, tab_counter};

/// Where a tab's page content lives.
///
/// `Live` means [`Tab::display_list`] holds the items. `Spilled` means the list
/// is empty in RAM and the serialized bytes live in `cache/spill/<uuid>`. The
/// display list stays a plain field so the raster path can borrow it directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Live,
    Spilled { uuid: String },
}

/// A browser tab. Deliberately holds no DOM, stylesheet or pixels.
pub struct Tab {
    pub url: String,
    pub title: String,
    pub display_list: DisplayList,
    /// Spill state of `display_list`: live in RAM or on disk under `uuid`.
    pub body: Body,
    /// Monotonic LRU stamp; bumped whenever the tab is used.
    pub last_used: u64,
    /// Current vertical scroll, in pixels (tab-local, applied at compose time).
    pub scroll_offset: f32,
    /// Bottom edge of the page before scrolling; source of the scroll clamp.
    pub page_height: f32,
    /// Visited URLs, oldest first.
    pub history: Vec<String>,
    /// Index into `history` of the current URL.
    pub history_cursor: usize,
    /// Private tab: loads skip the disk cache, and the tab is never saved.
    pub private: bool,
}

impl Tab {
    fn blank() -> Self {
        Self {
            url: String::new(),
            title: String::new(),
            display_list: DisplayList::default(),
            body: Body::Live,
            last_used: 0,
            scroll_offset: 0.0,
            page_height: 0.0,
            history: Vec::new(),
            history_cursor: 0,
            private: false,
        }
    }

    /// Spill-file id, if this tab's body is currently on disk.
    pub(crate) fn spill_uuid(&self) -> Option<&str> {
        match &self.body {
            Body::Spilled { uuid } => Some(uuid),
            Body::Live => None,
        }
    }
}

// Private state of the tab whose load is in flight. A thread-local keeps
// parallel unit tests isolated; the GUI drives everything on one thread.
thread_local! {
    static PRIVATE_LOAD: Cell<bool> = const { Cell::new(false) };
}

/// Read by `app::load_resource` when building `FetchOptions`.
pub(crate) fn private_load() -> bool {
    PRIVATE_LOAD.with(|flag| flag.get())
}

fn set_private_load(private: bool) {
    PRIVATE_LOAD.with(|flag| flag.set(private));
}

/// True when hodo was launched with `--private`.
fn startup_private() -> bool {
    std::env::args().any(|arg| arg == "--private")
}

/// Build the display list and page height for a restored tab without adding
/// history or touching the URL bar.
fn hydrate(tab: &mut Tab) {
    let url = tab
        .history
        .get(tab.history_cursor)
        .cloned()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| ABOUT_HODO_URL.to_string());
    set_private_load(tab.private);
    let (title, display_list) = load_page(&url, DEFAULT_VIEWPORT);
    tab.url = url;
    tab.title = title;
    tab.display_list = display_list;
    tab.scroll_offset = 0.0;
    tab.page_height = display_bottom(&tab.display_list);
    tab.body = Body::Live;
}

/// Build the restored tab vector from a session snapshot.
///
/// `private` is stamped on **every** tab before hydration: `hydrate` reads
/// `tab.private` to set the cache-bypass flag. Patching only tab 0 afterwards
/// let the other tabs' startup loads go through the shared disk cache.
/// Session restore is kept under `--private` (users still expect their tabs
/// back); only the cache path changes.
fn hydrate_saved(saved: Vec<crate::sessions::SessionTab>, private: bool) -> Vec<Tab> {
    saved
        .into_iter()
        .take(crate::sessions::MAX_SESSION_TABS)
        .map(|entry| {
            let mut tab = Tab::blank();
            tab.private = private;
            tab.url = entry.url;
            tab.title = entry.title;
            tab.history = entry.history;
            tab.history_cursor = entry.history_pos;
            // A spill id in the session resumes spilled: no eager parse,
            // style or layout for an inactive tab at startup.
            if !entry.uuid.is_empty() && crate::sessions::spill_file_exists(&entry.uuid) {
                tab.body = Body::Spilled { uuid: entry.uuid };
            } else {
                hydrate(&mut tab);
            }
            tab
        })
        .collect()
}

/// Rebuild a spilled tab in place; a missing/corrupt file reloads the URL via
/// the normal pipeline instead of panicking. Returns true on a clean restore.
fn restore_or_reload(tab: &mut Tab) -> bool {
    if !matches!(tab.body, Body::Spilled { .. }) {
        return true;
    }
    let restored = crate::sessions::restore_tab(tab);
    if !restored {
        // Drop the unusable file so the orphan sweep cannot resurrect it.
        if let Some(uuid) = tab.spill_uuid().map(str::to_string) {
            crate::sessions::delete_spill(&uuid);
        }
        hydrate(tab);
    }
    restored
}

/// The whole browser state, independent of any windowing system.
pub struct App {
    pub tabs: Vec<Tab>,
    pub active: usize,
    /// Editable content of the URL bar (shown in the UI overlay).
    pub url_bar: String,
    pub url_focused: bool,
    /// Last known surface size, used to compose lists for hit-testing.
    viewport: Size,
    /// Monotonic stamp used to order spill candidates by recency.
    clock: u64,
}

impl App {
    /// Create one tab, navigating it to `initial_url` or the welcome page.
    pub fn new(initial_url: Option<&str>) -> Self {
        let mut app = Self {
            tabs: vec![Tab::blank()],
            active: 0,
            url_bar: String::new(),
            url_focused: true,
            viewport: DEFAULT_VIEWPORT,
            clock: 0,
        };
        let welcome = initial_url.is_none();
        // Same toggle the future UI uses: `--private` opens tab 0 private.
        if startup_private() {
            app.toggle_private();
        }
        app.navigate(initial_url.unwrap_or(ABOUT_HODO_URL), DEFAULT_VIEWPORT);
        // On a bare launch keep the URL bar focused so typing works at once.
        app.url_focused = welcome;
        app
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active]
    }

    /// Flip private browsing for the active tab; returns the new state.
    /// Shared by the CLI startup flag and the future UI toggle.
    pub fn toggle_private(&mut self) -> bool {
        let tab = &mut self.tabs[self.active];
        tab.private = !tab.private;
        tab.private
    }

    /// Persist the normal tabs. A failed write is deliberately ignored.
    ///
    /// Also the single spill hook: every open/close/navigate/back/forward path
    /// the sessions agent wired already calls `save`, so the LRU cap is
    /// enforced here with no extra call sites.
    pub fn save(&mut self) {
        self.clock += 1;
        self.tabs[self.active].last_used = self.clock;
        self.spill_excess();
        crate::sessions::save_sessions(self);
    }

    /// Spill the least-recently-used inactive tabs until at most
    /// `HODO_SPILL_THRESHOLD` display lists remain in RAM. The active tab is
    /// never spilled: it is the only one with pixels to paint.
    fn spill_excess(&mut self) {
        let threshold = crate::sessions::spill_threshold();
        if threshold == 0 {
            return;
        }
        let live = self
            .tabs
            .iter()
            .filter(|tab| matches!(tab.body, Body::Live))
            .count();
        let excess = live.saturating_sub(threshold);
        if excess == 0 {
            return;
        }
        // Private tabs are never persisted (same contract as sessions), so
        // their page content must not be written to disk either.
        let mut candidates: Vec<usize> = (0..self.tabs.len())
            .filter(|&index| {
                index != self.active
                    && !self.tabs[index].private
                    && matches!(self.tabs[index].body, Body::Live)
            })
            .collect();
        candidates.sort_by_key(|&index| self.tabs[index].last_used);
        for index in candidates.into_iter().take(excess) {
            crate::sessions::spill_tab(&mut self.tabs[index]);
        }
    }

    /// Load a saved session (if any) into the tab vector. Tabs are hydrated
    /// eagerly so each one is ready to show; private tabs are never present in
    /// a snapshot. The `--private` state of the initial tab is applied to every
    /// restored tab *before* hydration, so no startup load touches the disk
    /// cache. Entries carrying a valid spill id stay spilled until first
    /// switched to; the active tab is restored now.
    pub fn restore(&mut self) {
        let private = self.tabs[self.active].private;
        let saved = crate::sessions::restore_sessions();
        // Startup sweep: any spill file not named by this session is an orphan.
        let keep: Vec<String> = saved
            .iter()
            .filter(|entry| !entry.uuid.is_empty())
            .map(|entry| entry.uuid.clone())
            .collect();
        crate::sessions::sweep_orphans(&keep);
        if saved.is_empty() {
            return;
        }
        let mut tabs = hydrate_saved(saved, private);
        if tabs.is_empty() {
            return;
        }
        // Tab 0 becomes active and must hold a live list to paint.
        restore_or_reload(&mut tabs[0]);
        self.tabs = tabs;
        self.active = 0;
        self.clock += 1;
        self.tabs[0].last_used = self.clock;
        self.url_bar = self.tabs[0].url.clone();
        self.url_focused = false;
    }

    /// Open a new tab on the welcome page and make it active.
    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::blank());
        self.active = self.tabs.len() - 1;
        self.navigate(ABOUT_HODO_URL, DEFAULT_VIEWPORT);
        self.save();
    }

    /// Close the active tab, keeping at least one tab open at all times.
    pub fn close_active(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        if let Some(uuid) = self.tabs[self.active].spill_uuid().map(str::to_string) {
            crate::sessions::delete_spill(&uuid);
        }
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.url_bar = self.tabs[self.active].url.clone();
        self.url_focused = false;
        self.save();
    }

    /// Make `index` active, restoring its display list from disk if spilled.
    /// This is the switch hook future UI (and the tests) use; the trailing
    /// `save` keeps the live-list cap and the session snapshot in sync.
    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() || index == self.active {
            return false;
        }
        restore_or_reload(&mut self.tabs[index]);
        self.active = index;
        self.clock += 1;
        self.tabs[index].last_used = self.clock;
        self.url_bar = self.tabs[index].url.clone();
        self.url_focused = false;
        self.save();
        true
    }

    /// Navigate the active tab and return its resolved title for the window.
    ///
    /// A new entry is pushed after the cursor and any forward entries are
    /// dropped, matching standard browser history behaviour.
    pub fn navigate(&mut self, url: &str, viewport: Size) -> String {
        let tab = &mut self.tabs[self.active];
        tab.history.truncate(tab.history_cursor + 1);
        tab.history.push(url.to_string());
        tab.history_cursor = tab.history.len() - 1;
        self.load_history_entry(viewport)
    }

    /// Load the active tab's current history entry without adding history.
    fn load_history_entry(&mut self, viewport: Size) -> String {
        set_private_load(self.tabs[self.active].private);
        let url = self.tabs[self.active].history[self.tabs[self.active].history_cursor].clone();
        let (title, display_list) = load_page(&url, viewport);
        let tab = &mut self.tabs[self.active];
        tab.url = url.clone();
        tab.title = title.clone();
        tab.display_list = display_list;
        tab.scroll_offset = 0.0;
        tab.page_height = display_bottom(&tab.display_list);
        self.url_bar = url;
        self.url_focused = false;
        title
    }

    /// Move one entry back in history. No-op at the oldest entry.
    pub fn back(&mut self, viewport: Size) -> String {
        let cursor = self.tabs[self.active].history_cursor;
        if cursor == 0 {
            return self.tabs[self.active].title.clone();
        }
        self.tabs[self.active].history_cursor = cursor - 1;
        let title = self.load_history_entry(viewport);
        self.save();
        title
    }

    /// Move one entry forward in history. No-op at the newest entry.
    pub fn forward(&mut self, viewport: Size) -> String {
        let tab = &mut self.tabs[self.active];
        if tab.history_cursor + 1 >= tab.history.len() {
            return tab.title.clone();
        }
        tab.history_cursor += 1;
        let title = self.load_history_entry(viewport);
        self.save();
        title
    }

    /// Re-run the pipeline for the active tab (F5). No-op on a blank tab.
    pub fn reload(&mut self, viewport: Size) -> String {
        let url = self.tabs[self.active].url.clone();
        if url.is_empty() {
            return String::new();
        }
        set_private_load(self.tabs[self.active].private);
        let (title, display_list) = load_page(&url, viewport);
        let tab = &mut self.tabs[self.active];
        tab.title = title.clone();
        tab.display_list = display_list;
        tab.scroll_offset = 0.0;
        tab.page_height = display_bottom(&tab.display_list);
        self.save();
        title
    }

    /// Scroll the active tab by a pixel delta, clamped to the page.
    pub fn scroll_by(&mut self, delta: f32, viewport: Size) {
        let tab = &mut self.tabs[self.active];
        tab.scroll_offset =
            clamp_scroll_offset(tab.scroll_offset + delta, tab.page_height, viewport.h);
    }

    /// Scroll the active tab to an absolute offset, clamped to the page.
    pub fn scroll_to(&mut self, offset: f32, viewport: Size) {
        let tab = &mut self.tabs[self.active];
        tab.scroll_offset = clamp_scroll_offset(offset, tab.page_height, viewport.h);
    }

    /// Keep hit-testing in sync with the GUI surface size.
    pub fn set_viewport(&mut self, viewport: Size) {
        self.viewport = viewport;
    }

    /// Click at `(x, y)` (viewport pixel coordinates). Navigates if a link is
    /// hit, otherwise just drops URL-bar focus. Returns true on navigation.
    pub fn handle_click(&mut self, x: f32, y: f32) -> bool {
        let tab = &self.tabs[self.active];
        let label = tab_counter(self.active + 1, self.tabs.len());
        let composed = compose_display_list(
            &tab.display_list,
            self.viewport,
            &self.url_bar,
            &[],
            tab.scroll_offset,
            &label,
        );
        // Consult both node maps: a fixed element wins over page items at the
        // same pixel, so only its own link (if any) can navigate there.
        match crate::ui::hit_test(&composed, x, y).0 {
            Some(href) => {
                self.navigate(&href, self.viewport);
                self.save();
                true
            }
            None => {
                self.url_focused = false;
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_dom::NodeId;
    use hodo_layout::{ControlKind, DisplayItem};
    use hodo_types::{Color, Rect};
    use std::time::Instant;

    fn synthetic_list(n: usize) -> DisplayList {
        let mut list = DisplayList::default();
        for i in 0..n {
            let rect = Rect {
                x: (i % 100) as f32,
                y: i as f32 * 1.5,
                w: 90.0,
                h: 18.0,
            };
            let owner = NodeId((i % 17) as u32);
            match i % 4 {
                0 => list.items.push(DisplayItem::Text {
                    rect,
                    text: format!("row {i}"),
                    color: Color::BLACK,
                    font_size: 16.0,
                    weight: 400,
                    href: if i % 8 == 0 {
                        Some(format!("/link{i}"))
                    } else {
                        None
                    },
                    underline: false,
                    line_through: false,
                }),
                1 => list.items.push(DisplayItem::Rect {
                    rect,
                    color: Color::WHITE,
                    border_color: Some(Color::BLACK),
                    radius: 2.0,
                }),
                2 => list.items.push(DisplayItem::Image {
                    rect,
                    path: format!("img{i}.png"),
                }),
                _ => list.items.push(DisplayItem::Control {
                    rect,
                    kind: ControlKind::TextInput,
                    text: format!("value {i}"),
                    name: format!("field{i}"),
                    masked: i % 8 == 3,
                    disabled: false,
                    readonly: i % 16 == 7,
                }),
            }
            list.node_map.push(owner);
        }
        // Exercise the fixed channel too.
        list.fixed_items.push(DisplayItem::Rect {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 10.0,
            },
            color: Color::BLACK,
            border_color: None,
            radius: 0.0,
        });
        list.fixed_node_map.push(NodeId(999));
        list
    }

    fn count_spilled(app: &App) -> usize {
        app.tabs
            .iter()
            .filter(|tab| matches!(tab.body, Body::Spilled { .. }))
            .count()
    }

    fn cleanup_spills(app: &App) {
        for tab in &app.tabs {
            if let Some(uuid) = tab.spill_uuid() {
                crate::sessions::delete_spill(uuid);
            }
        }
    }

    #[test]
    fn private_toggle_marks_only_the_active_tab_and_its_loads() {
        set_private_load(false);
        let mut app = App::new(Some(ABOUT_HODO_URL));
        assert!(!app.active_tab().private);

        assert!(app.toggle_private());
        assert!(app.active_tab().private);
        app.navigate("/private", DEFAULT_VIEWPORT);
        assert!(
            private_load(),
            "the load following a private tab navigation must be flagged private"
        );

        // A fresh tab is normal again, and its load clears the flag.
        app.new_tab();
        assert!(!app.active_tab().private);
        assert!(!private_load());
    }

    #[test]
    fn private_restore_hydrates_every_tab_through_the_cache_bypass() {
        use crate::sessions::SessionTab;
        let entries: Vec<SessionTab> = (0..3)
            .map(|_| SessionTab {
                url: ABOUT_HODO_URL.to_string(),
                title: String::new(),
                history: vec![ABOUT_HODO_URL.to_string()],
                history_pos: 0,
                uuid: String::new(),
            })
            .collect();

        let tabs = hydrate_saved(entries.clone(), true);
        assert_eq!(tabs.len(), 3);
        assert!(
            tabs.iter().all(|tab| tab.private),
            "--private must reach every restored tab, not just tab 0"
        );
        assert!(
            private_load(),
            "the last hydrate must still take the uncached loader path"
        );

        // The flag is per-load: a normal restore clears it again.
        let tabs = hydrate_saved(entries, false);
        assert!(tabs.iter().all(|tab| !tab.private));
        assert!(!private_load());
    }

    #[test]
    fn spill_roundtrip_is_byte_identical() {
        let dir = crate::sessions::test_spill_dir("roundtrip");
        let list = synthetic_list(256);
        let mut tab = Tab::blank();
        tab.url = "/roundtrip".into();
        tab.title = "Roundtrip".into();
        tab.display_list = list.clone();
        tab.scroll_offset = 123.5;
        tab.page_height = display_bottom(&list);
        tab.history = vec!["/roundtrip".into()];
        tab.history_cursor = 0;

        assert!(crate::sessions::spill_tab(&mut tab));
        let uuid = tab.spill_uuid().unwrap().to_string();
        assert!(tab.display_list.items.is_empty());
        assert!(!uuid.contains(['/', '\\']), "generated id is filename-safe");
        assert!(crate::sessions::restore_tab(&mut tab));
        assert!(matches!(tab.body, Body::Live));
        assert_eq!(tab.display_list.items.len(), list.items.len());
        assert_eq!(
            format!("{:?}", tab.display_list.items),
            format!("{:?}", list.items),
            "every display item field survives the roundtrip"
        );
        assert_eq!(tab.display_list.node_map, list.node_map);
        assert_eq!(
            format!("{:?}", tab.display_list.fixed_items),
            format!("{:?}", list.fixed_items)
        );
        assert_eq!(tab.display_list.fixed_node_map, list.fixed_node_map);
        assert_eq!(tab.scroll_offset, 123.5);
        assert_eq!(tab.page_height, display_bottom(&list));

        crate::sessions::delete_spill(&uuid);
        crate::sessions::clear_test_spill_dir(&dir);
    }

    #[test]
    fn twelve_tabs_spill_four_and_switch_restores() {
        let dir = crate::sessions::test_spill_dir("twelve");
        let mut app = App::new(Some(ABOUT_HODO_URL));
        // Capture tab 0 before any spill: the about page embeds live cache
        // stats, so later loads are not textually identical.
        for _ in 1..8 {
            app.new_tab();
        }
        let original_items = format!("{:?}", app.tabs[0].display_list.items);
        for _ in 8..12 {
            app.new_tab();
        }
        assert_eq!(app.tabs.len(), 12);
        let spilled: Vec<usize> = (0..12)
            .filter(|&i| matches!(app.tabs[i].body, Body::Spilled { .. }))
            .collect();
        assert_eq!(spilled, vec![0, 1, 2, 3], "LRU inactive tabs spill first");
        assert!(matches!(app.tabs[app.active].body, Body::Live));
        assert!(app.tabs[0].display_list.items.is_empty());

        assert!(app.switch_tab(0), "switching to a spilled tab restores it");
        assert!(matches!(app.tabs[0].body, Body::Live));
        assert_eq!(
            format!("{:?}", app.tabs[0].display_list.items),
            original_items
        );
        assert_eq!(
            count_spilled(&app),
            4,
            "the live cap is maintained on switch"
        );
        assert_eq!(app.active, 0);
        assert_eq!(app.url_bar, app.tabs[0].url);

        cleanup_spills(&app);
        crate::sessions::clear_test_spill_dir(&dir);
    }

    #[test]
    fn corrupt_spill_file_falls_back_to_a_url_reload() {
        let dir = crate::sessions::test_spill_dir("corrupt");
        let mut tab = Tab::blank();
        tab.url = "/corrupt".into();
        tab.history = vec!["/corrupt".into()];
        tab.history_cursor = 0;
        tab.display_list = synthetic_list(4);
        assert!(crate::sessions::spill_tab(&mut tab));
        let uuid = tab.spill_uuid().unwrap().to_string();
        let path = crate::sessions::spill_dir().join(format!("{uuid}.bin.lz4"));
        std::fs::write(&path, b"not a spill file").unwrap();

        assert!(!crate::sessions::restore_tab(&mut tab));
        assert!(
            matches!(tab.body, Body::Spilled { .. }),
            "failed restore leaves the tab untouched"
        );
        assert!(
            !restore_or_reload(&mut tab),
            "report that the file was unusable"
        );
        assert!(matches!(tab.body, Body::Live));
        assert!(
            !tab.display_list.items.is_empty(),
            "the URL pipeline rebuilt the page"
        );
        assert_eq!(tab.page_height, display_bottom(&tab.display_list));
        crate::sessions::delete_spill(&uuid);
        crate::sessions::clear_test_spill_dir(&dir);
    }

    #[test]
    fn default_spill_threshold_is_eight() {
        // The env var is read at each save; avoid mutating process-wide state
        // in a parallel test by checking the default path only here.
        let dir = crate::sessions::test_spill_dir("threshold");
        assert_eq!(crate::sessions::spill_threshold(), 8);
        let mut app = App::new(Some(ABOUT_HODO_URL));
        for _ in 1..12 {
            app.new_tab();
        }
        assert_eq!(count_spilled(&app), 4);
        cleanup_spills(&app);
        crate::sessions::clear_test_spill_dir(&dir);
    }

    #[test]
    fn spill_restore_perf_on_25k_items() {
        let dir = crate::sessions::test_spill_dir("perf");
        let list = synthetic_list(25_000);
        let mut tab = Tab::blank();
        tab.url = "/perf".into();
        tab.history = vec!["/perf".into()];
        tab.history_cursor = 0;
        tab.display_list = list;

        let start = Instant::now();
        assert!(crate::sessions::spill_tab(&mut tab));
        let spill_ms = start.elapsed().as_secs_f64() * 1000.0;
        let uuid = tab.spill_uuid().unwrap().to_string();

        let start = Instant::now();
        assert!(crate::sessions::restore_tab(&mut tab));
        let restore_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Full re-pipe baseline: the same item count produced by parse/style/
        // layout from a generated HTML page.
        let html = format!(
            "<html><body>{}</body></html>",
            (0..25_000)
                .map(|i| format!("<div>row {i}</div>"))
                .collect::<String>()
        );
        let page_dir = std::env::temp_dir().join(format!("hodo-spill-html-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&page_dir);
        let page = page_dir.join("big.html");
        std::fs::write(&page, html).unwrap();
        let page_url = page.to_string_lossy().into_owned();
        let start = Instant::now();
        let (_, piped) = load_page(&page_url, DEFAULT_VIEWPORT);
        let pipeline_ms = start.elapsed().as_secs_f64() * 1000.0;
        let _ = std::fs::remove_dir_all(&page_dir);

        eprintln!(
            "25k items — spill: {spill_ms:.1} ms | restore: {restore_ms:.1} ms | full re-pipe ({} items): {pipeline_ms:.1} ms",
            piped.items.len()
        );
        eprintln!(
            "RAM unit sizes: DisplayItem {} B, NodeId {} B, Tab {} B",
            std::mem::size_of::<DisplayItem>(),
            std::mem::size_of::<NodeId>(),
            std::mem::size_of::<Tab>()
        );
        assert_eq!(tab.display_list.items.len(), 25_000);
        assert!(
            restore_ms < pipeline_ms,
            "restore ({restore_ms:.1} ms) must beat a full re-pipe ({pipeline_ms:.1} ms)"
        );
        crate::sessions::delete_spill(&uuid);
        crate::sessions::clear_test_spill_dir(&dir);
    }
}

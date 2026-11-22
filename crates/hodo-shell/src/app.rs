//! Browser application: the winit handler, event routing and the page pipeline.
//!
//! The window-agnostic tab model lives in [`tabs`]; chrome composition in
//! [`ui`]; the softbuffer draw path in [`present`].

use std::cell::RefCell;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use hodo_cache::CacheKey;
use hodo_css::{parse_stylesheet_with, Stylesheet};
use hodo_dom::{Dom, NodeId};
use hodo_ext_api::{ApiCall, ApiResult};
use hodo_js::ExtScriptRuntime;
use hodo_layout::DisplayList;
use hodo_net::Resource;
use hodo_types::Size;
use serde_json::{json, Value};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::about::ABOUT_HODO_URL;
use crate::forms::{FormAction, FormPage};
use crate::popup::{PopupAction, PopupState};
use crate::present::RasterState;
use crate::tabs::App;

// Re-export the moved helpers so the in-place test module keeps its original
// `use super::*` view of the module.
#[cfg(test)]
use crate::about::{about_hodo_html, error_page_html};
#[cfg(test)]
use crate::ui::{
    clamp_scroll_offset, compose_display_list, display_bottom, hit_test_link, item_rect,
    tab_counter, BOTTOM_BAR_H,
};
#[cfg(test)]
use hodo_layout::DisplayItem;
#[cfg(test)]
use hodo_types::{Color, Rect};

/// Viewport used by the headless screenshot mode and as the initial GUI size.
pub const DEFAULT_VIEWPORT: Size = Size {
    w: 1200.0,
    h: 800.0,
};

/// Pixels scrolled per wheel line.
pub const SCROLL_LINE: f32 = 40.0;
/// Pixels scrolled per PageUp / PageDown key press.
pub const SCROLL_PAGE: f32 = 400.0;

/// State owned by the winit event loop. Graphics objects live here; the tab
/// model (`tabs::App`) stays window-agnostic and unit-testable.
pub(crate) struct BrowserApp {
    pub(crate) window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    pub(crate) surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    pub(crate) state: App,
    modifiers: ModifiersState,
    pub(crate) ext_titles: Vec<String>,
    /// Startup toolbar actions, indexed the same way as `ext_titles`.
    popup_actions: Vec<PopupAction>,
    /// The single open popup overlay, or `None` when it is closed.
    pub(crate) popup: Option<PopupState>,
    pub(crate) viewport: Size,
    /// Last cursor position in physical pixels (same space as the viewport).
    cursor: (f32, f32),
    /// The only pixel owner: one persistent pixmap + ARGB frame for the active
    /// tab. Tabs themselves remain pixel-free.
    pub(crate) raster: RasterState,
    /// Set by paths that invalidate every pixel (navigation, tab switch,
    /// resize, popup open/close) and consumed by the next draw.
    pub(crate) force_full: bool,
    /// Live form state for the active page; `None` when it has no controls.
    forms: Option<FormPage>,
}

impl BrowserApp {
    pub(crate) fn new(
        initial_url: Option<&str>,
        ext_titles: Vec<String>,
        popup_actions: Vec<PopupAction>,
    ) -> Self {
        let mut state = App::new(initial_url);
        state.restore();
        let mut app = Self {
            window: None,
            context: None,
            surface: None,
            state,
            modifiers: ModifiersState::default(),
            ext_titles,
            popup_actions,
            popup: None,
            viewport: DEFAULT_VIEWPORT,
            cursor: (0.0, 0.0),
            raster: RasterState::new(),
            force_full: true,
            forms: None,
        };
        app.sync_forms();
        app
    }

    fn set_window_title(&self, title: &str) {
        if let Some(window) = &self.window {
            window.set_title(if title.is_empty() { "hodo" } else { title });
        }
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn navigate(&mut self, url: String) {
        let title = self.state.navigate(&url, self.viewport);
        self.sync_forms();
        self.state.save();
        self.set_window_title(&title);
        self.force_full = true;
        self.request_redraw();
    }

    /// Rebuild form state for the active tab. The display-list scan is the
    /// zero-cost guard: pages without controls never pay for a second parse.
    fn sync_forms(&mut self) {
        let has_controls = self
            .state
            .active_tab()
            .display_list
            .items
            .iter()
            .any(|item| matches!(item, hodo_layout::DisplayItem::Control { .. }));
        if !has_controls {
            self.forms = None;
            return;
        }
        let url = self.state.active_tab().url.clone();
        self.forms = FormPage::load(&url, self.viewport);
    }

    /// Copy the live form page's layout into the active tab.
    fn sync_forms_list(&mut self) {
        if let (Some(tab), Some(form)) = (
            self.state.tabs.get_mut(self.state.active),
            self.forms.as_ref(),
        ) {
            tab.display_list = form.display_list().clone();
            tab.page_height = crate::ui::display_bottom(&tab.display_list);
        }
    }

    /// Commit a control edit: dirty fingerprint → re-style/layout/render, then
    /// hand the new list to the tab so the normal compose path paints it.
    fn tick_forms(&mut self) {
        if self.forms.as_mut().is_some_and(FormPage::tick) {
            self.sync_forms_list();
            self.request_redraw();
        }
    }

    /// GET navigates; POST replaces the page with the rendered response body.
    /// Wave 1 deliberately adds no history entry for POST responses.
    fn submit_form(&mut self, action: FormAction) {
        match action {
            FormAction::Navigate(url) => self.navigate(url),
            FormAction::Response { title, list, form } => {
                // The response body replaces the document: its old realm must
                // not keep delivering events into a DOM nobody paints.
                self.state.page_js = None;
                let tab = &mut self.state.tabs[self.state.active];
                tab.display_list = list;
                tab.title = title.clone();
                tab.page_height = crate::ui::display_bottom(&tab.display_list);
                tab.scroll_offset = 0.0;
                // The response may contain controls: adopt its live registry
                // through the normal sync path instead of orphaning form state.
                self.forms = form.map(|boxed| *boxed);
                self.sync_forms_list();
                self.set_window_title(&title);
                self.state.save();
                self.force_full = true;
                self.request_redraw();
            }
        }
    }

    /// Open the popup declared by toolbar action `index`. Actions without a
    /// `popup_path` are title-only and do nothing here.
    fn open_popup(&mut self, index: usize) {
        let Some(action) = self.popup_actions.get(index).cloned() else {
            return;
        };
        if action.popup_path.is_none() {
            // Title-only slot: no tooltip in wave 1; dismiss any open popup.
            self.close_popup();
            return;
        }
        match PopupState::open(&action, index, self.viewport) {
            Ok(popup) => self.popup = Some(popup),
            Err(err) => eprintln!("hodo: popup: {err}"),
        }
        // Present path consumes this once; the panel is overlaid per-present
        // and never baked into the raster cache.
        self.force_full = true;
        self.request_redraw();
    }

    fn close_popup(&mut self) {
        if self.popup.take().is_some() {
            self.force_full = true;
            self.request_redraw();
        }
    }

    /// Per-frame popup context tick, including dirty re-layout and reply drain.
    /// Returns true when its painted frame changed.
    fn tick_popup(&mut self) -> bool {
        self.popup.as_mut().is_some_and(PopupState::tick)
    }

    /// Take message replies produced by the popup's `runtime.onMessage`
    /// listeners, for the ext-host broker to correlate. Called by the broker
    /// wiring once that side lands.
    #[allow(dead_code)]
    pub(crate) fn take_popup_replies(&mut self) -> Vec<(serde_json::Value, serde_json::Value)> {
        self.popup
            .as_mut()
            .map_or_else(Vec::new, PopupState::take_replies)
    }

    /// Single resize entry point, shared by the winit handler and tests. Keeps
    /// page layout, form hit rects and the popup anchor on the same viewport.
    fn set_viewport(&mut self, viewport: Size) {
        self.viewport = viewport;
        self.state.set_viewport(viewport);
        if let (Some(width), Some(height)) = (
            NonZeroU32::new(viewport.w.round().max(1.0) as u32),
            NonZeroU32::new(viewport.h.round().max(1.0) as u32),
        ) {
            if let Some(surface) = self.surface.as_mut() {
                if let Err(err) = surface.resize(width, height) {
                    eprintln!("hodo: surface resize failed: {err}");
                }
            }
        }
        if let Some(form) = self.forms.as_mut() {
            form.set_viewport(viewport);
        }
        self.sync_forms_list();
        // The form list is a different DOM snapshot; for a JS page the live
        // realm is authoritative, so re-pipe it after the form copy.
        if self.state.page_js_relayout(self.viewport) {
            self.force_full = true;
        }
        if let Some(popup) = self.popup.as_mut() {
            popup.set_viewport(viewport);
        }
        self.force_full = true;
        self.request_redraw();
    }

    /// Esc dismisses the popup before any page/chrome key handling.
    fn dismisses_popup(&self, key: Key<&str>) -> bool {
        self.popup.is_some() && matches!(key, Key::Named(NamedKey::Escape))
    }

    /// Route a left-click: toolbar → popup → form control → page JS → link.
    fn click_at(&mut self, x: f32, y: f32) {
        if let Some(index) = crate::ui::hit_test_action(self.viewport, &self.ext_titles, x, y) {
            self.open_popup(index);
        } else if self.popup.is_some() {
            // Click inside the panel is consumed; outside dismisses it.
            let inside = self.popup.as_mut().is_some_and(|popup| popup.click(x, y));
            if !inside {
                self.close_popup();
            }
        } else {
            // Resolve the NodeId before form state can replace the display
            // list. Forms own inputs first; page listeners still see the click
            // afterwards, and can mutate the same DOM the forms registry reads.
            let node = self.state.hit_node(x, y);
            let form_hit = self
                .forms
                .as_ref()
                .is_some_and(|form| form.hit(x, y, self.state.active_tab().scroll_offset));
            let mut action = None;
            if form_hit {
                let scroll = self.state.active_tab().scroll_offset;
                action = self
                    .forms
                    .as_mut()
                    .and_then(|form| form.click(x, y, scroll));
                self.state.url_focused = false;
                self.tick_forms();
            }
            if let Some(node) = node {
                if self.state.page_js_dispatch(
                    node,
                    "click",
                    json!({ "x": x, "y": y }),
                    self.viewport,
                ) {
                    self.force_full = true;
                    self.request_redraw();
                }
            }
            if let Some(action) = action {
                self.submit_form(action);
            } else if !form_hit && self.state.handle_click(x, y) {
                let title = self.state.active_tab().title.clone();
                self.set_window_title(&title);
            }
        }
    }

    fn on_key(&mut self, event: KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }
        if self.dismisses_popup(event.logical_key.as_ref()) {
            self.close_popup();
            return;
        }
        let ctrl = self.modifiers.control_key();
        // Page controls own keys while the URL bar is not focused. Tab works
        // even before any control is focused (it selects the first one).
        if !self.state.url_focused && !ctrl && self.popup.is_none() {
            let mut submitted = None;
            let mut handled = false;
            if let Some(form) = self.forms.as_mut() {
                match event.logical_key.as_ref() {
                    Key::Named(NamedKey::Tab) => handled = form.tab(),
                    Key::Named(NamedKey::Backspace) => handled = form.backspace(),
                    Key::Named(NamedKey::Enter) => submitted = form.enter(),
                    Key::Named(NamedKey::Space) => handled = form.insert(" "),
                    Key::Character(text) => handled = form.insert(text),
                    _ => {}
                }
            }
            if let Some(action) = submitted {
                self.dispatch_page_key(&event);
                self.submit_form(action);
                return;
            }
            if handled {
                self.tick_forms();
                self.dispatch_page_key(&event);
                return;
            }
        }
        self.dispatch_page_key(&event);
        match event.logical_key.as_ref() {
            Key::Named(NamedKey::Enter) => {
                if self.state.url_focused {
                    let url = self.state.url_bar.clone();
                    self.navigate(url);
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if self.state.url_focused && self.state.url_bar.pop().is_some() {
                    self.request_redraw();
                }
            }
            Key::Named(NamedKey::F5) => {
                let title = self.state.reload(self.viewport);
                self.sync_forms();
                if !title.is_empty() {
                    self.set_window_title(&title);
                }
                self.force_full = true;
                self.request_redraw();
            }
            Key::Named(NamedKey::ArrowLeft) if self.modifiers.alt_key() => {
                let title = self.state.back(self.viewport);
                self.sync_forms();
                self.set_window_title(&title);
                self.force_full = true;
                self.request_redraw();
            }
            Key::Named(NamedKey::ArrowRight) if self.modifiers.alt_key() => {
                let title = self.state.forward(self.viewport);
                self.sync_forms();
                self.set_window_title(&title);
                self.force_full = true;
                self.request_redraw();
            }
            Key::Named(NamedKey::PageUp) => {
                self.state.scroll_by(-SCROLL_PAGE, self.viewport);
                self.request_redraw();
            }
            Key::Named(NamedKey::PageDown) => {
                self.state.scroll_by(SCROLL_PAGE, self.viewport);
                self.request_redraw();
            }
            Key::Named(NamedKey::Home) => {
                self.state.scroll_to(0.0, self.viewport);
                self.request_redraw();
            }
            Key::Named(NamedKey::End) => {
                let end = self.state.active_tab().page_height;
                self.state.scroll_to(end, self.viewport);
                self.request_redraw();
            }
            Key::Named(NamedKey::Tab) if ctrl => {
                // Switch hook: restores a spilled tab and re-enforces the
                // live-list cap through `App::switch_tab` -> `App::save`.
                let count = self.state.tabs.len();
                if count > 1 {
                    let next = (self.state.active + 1) % count;
                    self.state.switch_tab(next);
                    self.sync_forms();
                    let title = self.state.active_tab().title.clone();
                    self.set_window_title(&title);
                    self.force_full = true;
                    self.request_redraw();
                }
            }
            Key::Character(text) if ctrl => match text.to_ascii_lowercase().as_str() {
                "l" => {
                    self.state.url_focused = true;
                    self.state.url_bar.clear();
                    self.request_redraw();
                }
                "t" => {
                    self.state.new_tab();
                    self.sync_forms();
                    let title = self.state.active_tab().title.clone();
                    self.set_window_title(&title);
                    self.force_full = true;
                    self.request_redraw();
                }
                "w" => {
                    self.state.close_active();
                    self.sync_forms();
                    let title = self.state.active_tab().title.clone();
                    self.set_window_title(&title);
                    self.force_full = true;
                    self.request_redraw();
                }
                _ => {}
            },
            Key::Character(text) if self.state.url_focused => {
                self.state.url_bar.push_str(text);
                self.request_redraw();
            }
            _ => {}
        }
    }

    /// Page key events target the document root. Form controls keep their own
    /// registry handling first (see `on_key`); the URL bar and popup swallow
    /// keys so typing there never reaches the page.
    fn dispatch_page_key(&mut self, event: &KeyEvent) {
        if self.state.url_focused || self.popup.is_some() {
            return;
        }
        let Some(root) = self.state.page_js_root() else {
            return;
        };
        let key = match event.logical_key.as_ref() {
            Key::Named(named) => format!("{named:?}"),
            Key::Character(text) => text.to_string(),
            _ => String::new(),
        };
        if self
            .state
            .page_js_dispatch(root, "keydown", json!({ "key": key }), self.viewport)
        {
            self.force_full = true;
            self.request_redraw();
        }
    }
}

impl ApplicationHandler for BrowserApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("hodo")
            .with_inner_size(PhysicalSize::new(1200u32, 800u32));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("hodo: cannot create window: {err}");
                event_loop.exit();
                return;
            }
        };
        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(err) => {
                eprintln!("hodo: cannot create softbuffer context: {err}");
                event_loop.exit();
                return;
            }
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                eprintln!("hodo: cannot create softbuffer surface: {err}");
                event_loop.exit();
                return;
            }
        };
        let inner = window.inner_size();
        self.viewport = Size {
            w: inner.width as f32,
            h: inner.height as f32,
        };
        self.state.set_viewport(self.viewport);
        let title = self.state.active_tab().title.clone();
        window.set_title(if title.is_empty() { "hodo" } else { &title });
        self.window = Some(window.clone());
        self.context = Some(context);
        self.surface = Some(surface);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                self.set_viewport(Size {
                    w: size.width as f32,
                    h: size.height as f32,
                });
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                let (x, y) = self.cursor;
                self.click_at(x, y);
                self.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event),
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, lines),
                ..
            } => {
                self.state.scroll_by(lines * SCROLL_LINE, self.viewport);
                self.request_redraw();
            }
            // One owner for every pixel: present.rs composites an open popup
            // onto the cached raster frame and consumes force_full exactly once.
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.tick_popup() {
            self.request_redraw();
        }
        // No ext-host broker consumes replies yet. Drain them on every idle
        // tick so a chatty popup cannot accumulate pairs indefinitely; the
        // queue is also capped in PopupState for bursts within one tick. When a
        // broker lands it must call take_popup_replies() before this drain.
        let _ = self.take_popup_replies();
        // Same dirty-fingerprint path the popup uses; edits are already applied
        // synchronously, so this is a safety net for future mutation sources.
        self.tick_forms();
        // One page-JS tick per frame: drains timers/promises/events, detects
        // native DOM mutation by fingerprint and re-pipes styles/layout once.
        if self.state.page_js_tick(self.viewport) {
            self.force_full = true;
            self.request_redraw();
        }
        // Idle costs nothing without scripts: redraws are event-driven. An open
        // popup or a page with live timers keeps the 33 ms tick until it closes.
        let flow = if self.popup.is_some() || self.state.page_js_has_timers() {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(33))
        } else {
            ControlFlow::Wait
        };
        event_loop.set_control_flow(flow);
    }
}

/// Resolve a stylesheet/script/image URL against the page's **final** URL.
///
/// For http(s) bases this delegates to hodo-net's RFC-style resolver so a
/// root-relative `/static/x.css` keeps the authority instead of producing
/// `https:///static/x.css`. Local Windows paths fall back to the old
/// last-segment replacement (they have no URL authority).
pub fn join_url(base: &str, href: &str) -> String {
    if href.contains("://") || href.starts_with("file:") || href.starts_with("data:") {
        return href.to_string();
    }
    if base.starts_with("http://") || base.starts_with("https://") {
        if let Ok(resolved) = hodo_net::resolve_location(base, href) {
            return resolved;
        }
    }
    match base.rfind(['/', '\\']) {
        Some(index) => format!("{}{}", &base[..=index], href),
        None => href.to_string(),
    }
}

/// The shell's single disk-cache owner: one `DiskCache` per process, guarded
/// by a mutex so each page load can briefly borrow it mutably. `open` creates
/// the `cache/` directory under this root (the current working directory).
fn shell_cache() -> Option<&'static Mutex<hodo_cache::DiskCache>> {
    static CACHE: OnceLock<Option<Mutex<hodo_cache::DiskCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            hodo_cache::DiskCache::open(&root, 128 * 1024 * 1024)
                .ok()
                .map(Mutex::new)
        })
        .as_ref()
}

/// Load a page resource through the shared cache. If the cache cannot be
/// opened, degrade to the frozen uncached loader rather than failing the page.
pub(crate) fn load_resource(url: &str) -> Result<Resource, hodo_net::FetchError> {
    let opts = hodo_net::FetchOptions {
        private: crate::tabs::private_load(),
        ..Default::default()
    };
    // shortcut: `load_with` ignores FetchOptions.private for cache writes; use
    // the uncached loader until the integrator routes loads via hodo_net::fetch.
    if opts.private {
        return hodo_net::load(url);
    }
    match shell_cache() {
        Some(cache) => {
            let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
            hodo_net::load_with(url, &opts, &mut guard)
        }
        None => hodo_net::load(url),
    }
}

/// Follow redirect entries left in the shared cache by `load_with` and return
/// the URL that actually served the body. `hodo_net::Resource` drops its
/// `final_url`, so relative DOM URLs must be resolved through this seam or a
/// redirected page resolves against a half-empty authority (`https:///…`).
/// Private loads bypass the cache entirely and keep the request URL.
fn final_base_url(url: &str) -> String {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return url.to_string();
    }
    let normalized = hodo_net::normalize_url(url).unwrap_or_else(|_| url.to_string());
    let Some(cache) = shell_cache() else {
        return normalized;
    };
    resolve_cached_redirects(normalized, |hop| {
        let mut guard = cache.lock().unwrap_or_else(|err| err.into_inner());
        let entry = guard.get(&CacheKey::new("GET", hop, ""))?;
        Some((entry.status_code, entry.headers.get("location")?.clone()))
    })
}

/// Pure redirect walk over a `(status, Location)` lookup, capped like hodo-net.
fn resolve_cached_redirects<F>(mut current: String, mut hop: F) -> String
where
    F: FnMut(&str) -> Option<(u16, String)>,
{
    for _ in 0..5 {
        let Some((status, location)) = hop(&current) else {
            break;
        };
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            break;
        }
        let Ok(next) = hodo_net::resolve_location(&current, &location) else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
    }
    current
}

/// Script count cap. The central limit table has no page-script entry; reuse
/// the live-context cap so a single document cannot spawn an unbounded realm.
const MAX_PAGE_SCRIPTS: usize = hodo_types::limits::MAX_EXT_CONTEXTS;
/// Per-script source cap, shared with every other untrusted resource.
const MAX_SCRIPT_BYTES: usize = hodo_types::limits::MAX_RESOURCE_BYTES;
/// Wall budget for a *series* of script evals. A single script is still bounded
/// by hodo-js's loop (1e6) and recursion (256) runtime limits.
///
/// The GUI keeps 50 ms so navigation stays snappy; headless CLI runs
/// (`--screenshot`, and the future `--eval`/`--dump-dom`) use 250 ms so a
/// large real-world bundle finishes instead of truncating at the first 50 ms.
/// Both flow through `page_script_budget_or`, so an explicit
/// `HODO_JS_BUDGET_MS` still overrides either default.
const GUI_PAGE_SCRIPT_BUDGET_MS: u64 = 50;
const HEADLESS_PAGE_SCRIPT_BUDGET_MS: u64 = 250;

fn page_script_budget(headless: bool) -> Duration {
    let default_ms = if headless {
        HEADLESS_PAGE_SCRIPT_BUDGET_MS
    } else {
        GUI_PAGE_SCRIPT_BUDGET_MS
    };
    hodo_js::page_script_budget_or(default_ms)
}

/// `HODO_NO_PAGE_JS=1` disables the whole page-JS path, including collection:
/// no realm is built and no extra work is done on script-free pages.
fn page_js_enabled() -> bool {
    !std::env::var_os("HODO_NO_PAGE_JS").is_some_and(|value| value == "1")
}

#[derive(Debug)]
enum ScriptSource {
    Inline(String),
    External(String),
}

/// Attributes whose mutation can change style, layout or hit-testing. Interned
/// DOMs have no attribute iterator, so enumerate the names (same list as the
/// popup overlay's fingerprint).
const FINGERPRINT_ATTRS: &[&str] = &[
    "id",
    "class",
    "style",
    "href",
    "src",
    "rel",
    "action",
    "method",
    "name",
    "value",
    "type",
    "checked",
    "selected",
    "disabled",
    "readonly",
    "placeholder",
    "maxlength",
    "rows",
    "cols",
    "width",
    "height",
    "colspan",
    "rowspan",
];

/// Cheap change probe over the reachable DOM. hodo-js has no mutation counter
/// and the DOM crate is not in this zone, so (like the popup overlay) we walk
/// the live document once per tick rather than thread a dirty flag through the
/// native bindings. Script mutation is detected before the next paint.
fn dom_fingerprint(dom: &Dom) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    dom.walk(|id, node| {
        id.0.hash(&mut hasher);
        match &node.kind {
            hodo_dom::NodeKind::Document => "document".hash(&mut hasher),
            hodo_dom::NodeKind::Element { .. } => {
                dom.tag(id).hash(&mut hasher);
                for name in FINGERPRINT_ATTRS {
                    dom.attr(id, name).hash(&mut hasher);
                }
            }
            hodo_dom::NodeKind::Text(_) => dom.text(id).hash(&mut hasher),
        }
    });
    hasher.finish()
}

/// Host side of `__hodoFetch`: GET only, through the same cached loader as
/// stylesheets and `<script src>`. The page prelude never sees chrome/browser.
fn page_fetch_hook(call: &ApiCall, base_url: &str) -> ApiResult {
    let ApiCall::Fetch { url, method, body } = call else {
        return ApiResult::Err("page: unsupported API call".to_string());
    };
    if method != "GET" || !body.is_empty() {
        return ApiResult::Err("page fetch: only empty GET is supported".to_string());
    }
    let resolved = join_url(base_url, url);
    match load_resource(&resolved) {
        Ok(Resource::Html(text)) => {
            ApiResult::Ok(json!({ "ok": true, "status": 200, "body": text }))
        }
        Ok(Resource::Other { data, .. }) => ApiResult::Ok(json!({
            "ok": true,
            "status": 200,
            "body": String::from_utf8_lossy(&data)
        })),
        Err(err) => ApiResult::Err(format!("page fetch {resolved}: {err}")),
    }
}

/// The JS realm of the active tab. Dropping it closes the realm, clears timers
/// and unregisters every DOM handle owned by the page (hodo-js `Drop`).
pub(crate) struct PageContext {
    dom: Rc<RefCell<Dom>>,
    runtime: ExtScriptRuntime,
    base_url: String,
    fingerprint: u64,
}

impl PageContext {
    fn new(dom: Rc<RefCell<Dom>>, base_url: String, runtime: ExtScriptRuntime) -> Self {
        let fingerprint = dom_fingerprint(&dom.borrow());
        Self {
            dom,
            runtime,
            base_url,
            fingerprint,
        }
    }

    pub(crate) fn has_timers(&self) -> bool {
        self.runtime.timer_count() != 0
    }

    pub(crate) fn root(&self) -> NodeId {
        self.dom.borrow().root()
    }

    /// Last `document.title` assigned by a page script (empty until one does).
    /// hodo-js has no shell dependency, so the host mirrors this into the tab.
    pub(crate) fn document_title(&self) -> String {
        self.runtime.document_title()
    }

    /// Drain timers/promises/queued events, then relayout only if a native
    /// actually changed the DOM. One fingerprint check per frame, so changes
    /// from several tasks coalesce into one relayout.
    pub(crate) fn tick(&mut self, viewport: Size) -> Option<DisplayList> {
        self.runtime.tick();
        let _ = self.runtime.take_message_replies();
        for line in self.runtime.take_logs() {
            eprintln!("hodo: page script: {line}");
        }
        self.recompute_if_changed(viewport)
    }

    /// Queue a DOM event and deliver it in this frame, so click/key handlers
    /// are reflected in the very next paint.
    pub(crate) fn dispatch(
        &mut self,
        node: NodeId,
        kind: &str,
        payload: Value,
        viewport: Size,
    ) -> Option<DisplayList> {
        self.runtime.dispatch_page_event(node, kind, payload);
        self.tick(viewport)
    }

    fn recompute_if_changed(&mut self, viewport: Size) -> Option<DisplayList> {
        let fingerprint = dom_fingerprint(&self.dom.borrow());
        if fingerprint == self.fingerprint {
            return None;
        }
        self.fingerprint = fingerprint;
        Some(self.relayout(viewport))
    }

    /// Full re-pipe against the live DOM (M5.3 incremental layout stays future).
    pub(crate) fn relayout(&self, viewport: Size) -> DisplayList {
        let dom = self.dom.borrow();
        let stylesheet = collect_styles(&dom, &self.base_url, viewport);
        let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
        hodo_layout::layout(&dom, &styles, viewport)
    }
}

/// Resolve inline sources and `<script src>` in document order with hard caps.
fn collect_scripts(found: Vec<ScriptSource>, page_url: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    for source in found {
        if scripts.len() >= MAX_PAGE_SCRIPTS {
            eprintln!("hodo: page script cap of {MAX_PAGE_SCRIPTS} reached on {page_url}");
            break;
        }
        let code = match source {
            ScriptSource::Inline(text) => text,
            ScriptSource::External(src) => {
                let url = join_url(page_url, &src);
                match load_resource(&url) {
                    Ok(Resource::Html(text)) => text,
                    Ok(Resource::Other { data, .. }) => String::from_utf8_lossy(&data).into_owned(),
                    Err(err) => {
                        eprintln!("hodo: page script {url}: {err}");
                        continue;
                    }
                }
            }
        };
        if code.len() > MAX_SCRIPT_BYTES {
            eprintln!("hodo: page script on {page_url} exceeds {MAX_SCRIPT_BYTES} bytes");
            continue;
        }
        scripts.push(code);
    }
    scripts
}

/// Full pipeline with an explicit JS toggle. The GUI keeps the returned realm;
/// headless callers drop it, but still observe the scripts' DOM mutations in
/// the display list (the `--screenshot` path relies on this).
fn load_page_with_flag(
    url: &str,
    viewport: Size,
    js_enabled: bool,
    budget: Duration,
) -> (String, DisplayList, Option<PageContext>) {
    // A local directory is a site shorthand (`--screenshot examples/js-site`);
    // resolve it before loading so relative script/src URLs get the right base.
    let resolved = if !url.contains("://") && Path::new(url).extension().is_none() {
        if Path::new(url).is_dir() {
            Path::new(url)
                .join("index.html")
                .to_string_lossy()
                .into_owned()
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };
    let url = resolved.as_str();
    let html = if let Some(page) = crate::hpages::hodo_page_html(url) {
        page
    } else if url == ABOUT_HODO_URL {
        crate::about::about_hodo_html()
    } else {
        match load_resource(url) {
            Ok(Resource::Html(text)) => text,
            Ok(Resource::Other { data, .. }) => String::from_utf8_lossy(&data).into_owned(),
            Err(err) => crate::about::error_page_html(&err.to_string()),
        }
    };
    // The load above may have followed redirects; every relative URL below
    // must resolve against the URL that actually served this body.
    let base_url = final_base_url(url);
    // Parse with the same scripting flag the rest of the pipeline uses: with
    // HODO_NO_PAGE_JS=1 `<noscript>` content becomes real, renderable markup.
    let dom = Rc::new(RefCell::new(hodo_html::parse_document_with(
        js_enabled, &html,
    )));
    // Layout has no base-URL parameter, so a relative img[src] would resolve
    // against the process CWD. Rewrite it against the final page URL.
    {
        let mut doc = dom.borrow_mut();
        let mut relative_images = Vec::new();
        doc.walk(|id, _| {
            if doc.tag(id).is_some_and(|t| t.eq_ignore_ascii_case("img")) {
                if let Some(src) = doc.attr(id, "src") {
                    if !src.contains("://") && !src.starts_with("data:") {
                        relative_images.push((id, join_url(&base_url, src)));
                    }
                }
            }
        });
        for (id, src) in relative_images {
            doc.set_attr(id, "src", src);
        }
    }

    // `scan_document` folds script discovery into the existing title walk, so
    // a script-free page pays no extra traversal at all.
    let (title, found) = {
        let doc = dom.borrow();
        scan_document(&doc, js_enabled)
    };
    let scripts = if js_enabled {
        collect_scripts(found, &base_url)
    } else {
        Vec::new()
    };

    // realm-drop → parse → collect → eval → DOMContentLoaded → styles/layout
    let page = if scripts.is_empty() {
        None
    } else {
        let root = dom.borrow().root();
        // Thread the final (post-redirect) URL so `window.location.href` reports
        // the page the user actually navigated to, not a `<base>` hint.
        let mut runtime = ExtScriptRuntime::new_page_with_url(Rc::clone(&dom), base_url.clone());
        let fetch_base = base_url.clone();
        runtime.set_fetch_hook(Rc::new(move |call| page_fetch_hook(call, &fetch_base)));
        // Page scripts cannot exchange extension messages, but drain replies
        // anyway so a stray sendResponse cannot grow the runtime's queue.
        runtime.set_message_reply_hook(Some(Rc::new(|_: &Value, _: &Value| {})));
        let started = Instant::now();
        for script in scripts {
            if started.elapsed() > budget {
                eprintln!("hodo: page script budget exceeded on {url}");
                break;
            }
            if let Err(err) = runtime.eval(&script) {
                eprintln!("hodo: page script on {url}: {err}");
            }
        }
        runtime.dispatch_page_event(root, "DOMContentLoaded", json!({}));
        let _ = runtime.tick();
        for line in runtime.take_logs() {
            eprintln!("hodo: page script on {url}: {line}");
        }
        Some(PageContext::new(Rc::clone(&dom), base_url.clone(), runtime))
    };

    let display_list = {
        let doc = dom.borrow();
        let stylesheet = collect_styles(&doc, &base_url, viewport);
        let styles = hodo_style::compute_styles(&doc, &[stylesheet]);
        hodo_layout::layout(&doc, &styles, viewport)
    };
    (title, display_list, page)
}

/// Navigation/reload/back entry point that keeps the new realm alive for
/// events and post-load mutation relayout.
pub(crate) fn load_page_live(
    url: &str,
    viewport: Size,
) -> (String, DisplayList, Option<PageContext>) {
    load_page_with_flag(url, viewport, page_js_enabled(), page_script_budget(false))
}

/// Run the whole pipeline for one URL and return `(title, display_list)`.
///
/// The DOM, stylesheet, style map — and any page JS realm created to produce
/// the mutations — are local to this function and dropped before it returns.
/// Headless screenshot mode uses this path.
pub fn load_page(url: &str, viewport: Size) -> (String, DisplayList) {
    // Headless has no frame deadline, so give scripts the longer budget.
    let (title, display_list, _) =
        load_page_with_flag(url, viewport, page_js_enabled(), page_script_budget(true));
    (title, display_list)
}

/// Source-over blend of straight-alpha RGBA onto opaque white.
pub fn composite_over_white(rgba: &mut [u8]) {
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = pixel[3] as u32;
        if alpha == 255 {
            continue;
        }
        let inv = 255 - alpha;
        pixel[0] = ((pixel[0] as u32 * alpha + 127) / 255 + inv).min(255) as u8;
        pixel[1] = ((pixel[1] as u32 * alpha + 127) / 255 + inv).min(255) as u8;
        pixel[2] = ((pixel[2] as u32 * alpha + 127) / 255 + inv).min(255) as u8;
        pixel[3] = 255;
    }
}

/// Premultiply straight RGBA into the format `tiny_skia::Pixmap` expects.
pub fn premultiply_rgba(rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len());
    for pixel in rgba.as_chunks::<4>().0 {
        let alpha = pixel[3] as u32;
        out.push(((pixel[0] as u32 * alpha + 127) / 255) as u8);
        out.push(((pixel[1] as u32 * alpha + 127) / 255) as u8);
        out.push(((pixel[2] as u32 * alpha + 127) / 255) as u8);
        out.push(pixel[3]);
    }
    out
}

/// Only classic JavaScript executes: no `type`, an empty `type`, or a JS MIME
/// type. Per the HTML spec every other value (including `module`, data blocks
/// like `application/json`, `importmap`, and unknown types) is NOT JavaScript
/// and must not be eval'd — feeding JSON to the JS parser yields SyntaxErrors.
fn is_js_script_type(kind: Option<&str>) -> bool {
    match kind {
        None => true,
        Some(value) => {
            let value = value.trim();
            value.is_empty()
                || value.eq_ignore_ascii_case("text/javascript")
                || value.eq_ignore_ascii_case("application/javascript")
        }
    }
}

/// One walk for the page title and its script sources. Inline text and
/// `src=` URLs are collected in document order; non-JS `type`s are skipped.
fn scan_document(dom: &Dom, js_enabled: bool) -> (String, Vec<ScriptSource>) {
    let mut title = String::new();
    let mut found_title = false;
    let mut scripts = Vec::new();
    dom.walk(|id, _| {
        if !found_title && dom.tag(id) == Some("title") {
            found_title = true;
            for &child in dom.children(id) {
                if let Some(text) = dom.text(child) {
                    title.push_str(text);
                }
            }
        }
        // The kill switch skips even script discovery inside this walk.
        if !js_enabled || dom.tag(id) != Some("script") {
            return;
        }
        if !is_js_script_type(dom.attr(id, "type")) {
            return;
        }
        if let Some(src) = dom.attr(id, "src") {
            scripts.push(ScriptSource::External(src.to_string()));
        } else {
            let mut source = String::new();
            for &child in dom.children(id) {
                if let Some(text) = dom.text(child) {
                    source.push_str(text);
                }
            }
            scripts.push(ScriptSource::Inline(source));
        }
    });
    (title, scripts)
}

pub(crate) fn collect_styles(dom: &Dom, page_url: &str, viewport: Size) -> Stylesheet {
    // The engine's UA sheet does not hide <head>; without this the <style>
    // and <title> text would spill into the page as visible text.
    let mut sources: Vec<String> =
        vec!["head, style, link, meta, title, script { display: none }".to_owned()];

    // One document-order walk: inline <style> text and fetched <link> bodies
    // are appended as they appear. A later sheet now wins a same-specificity
    // tie over an earlier one, matching the browser cascade. dom.walk visits
    // nodes in html5ever append order, so children are already in order.
    dom.walk(|id, _| {
        if dom.tag(id) == Some("style") {
            for &child in dom.children(id) {
                if let Some(text) = dom.text(child) {
                    sources.push(text.to_owned());
                }
            }
        } else if dom.tag(id) == Some("link") && dom.attr(id, "rel") == Some("stylesheet") {
            // Wave-1 simplification: only `media="print"` is non-matching.
            // `all`/`screen`/absent (and any other value) load; a future wave
            // can evaluate the full media list against `css_viewport`.
            if dom
                .attr(id, "media")
                .is_some_and(|m| m.trim().eq_ignore_ascii_case("print"))
            {
                return;
            }
            let Some(href) = dom.attr(id, "href") else {
                return;
            };
            let resolved = join_url(page_url, href);
            match load_resource(&resolved) {
                Ok(Resource::Html(text)) => sources.push(text),
                Ok(Resource::Other { data, .. }) => {
                    sources.push(String::from_utf8_lossy(&data).into_owned());
                }
                Err(err) => eprintln!("hodo: stylesheet {resolved}: {err}"),
            }
        }
    });

    let css_viewport = hodo_css::Viewport {
        width: viewport.w.round().max(1.0) as u32,
        height: viewport.h.round().max(1.0) as u32,
        media_type: hodo_css::MediaType::Screen,
        scripting_enabled: true,
        // Integrator hook: the chrome dark flag drives prefers-color-scheme.
        dark: crate::hpages::read_dark_enabled(),
    };
    let mut rules = Vec::new();
    for source in sources {
        rules.extend(parse_stylesheet_with(css_viewport, &source).rules);
    }
    Stylesheet { rules }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Tests that toggle `HODO_NO_PAGE_JS` hold this lock; every test that
    /// loads the JS fixture takes it too, so a parallel test never sees the
    /// kill switch mid-flip.
    pub(crate) static PAGE_JS_ENV_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn js_site_url() -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/js-site/index.html")
            .to_string_lossy()
            .into_owned()
    }

    pub(crate) fn list_has(list: &DisplayList, fragment: &str) -> bool {
        list.items
            .iter()
            .any(|item| format!("{item:?}").contains(fragment))
    }

    fn linked_text(x: f32, y: f32, w: f32, h: f32, href: Option<&str>) -> DisplayItem {
        DisplayItem::Text {
            rect: Rect { x, y, w, h },
            text: "link".into(),
            color: Color::BLACK,
            font_size: 16.0,
            weight: 400,
            href: href.map(str::to_string),
            underline: false,
            line_through: false,
        }
    }

    #[test]
    fn url_join_handles_absolute_and_relative_links() {
        assert_eq!(
            join_url("https://example.com/dir/page.html", "style.css"),
            "https://example.com/dir/style.css"
        );
        assert_eq!(
            join_url("https://example.com/page.html", "https://cdn.example/x.css"),
            "https://cdn.example/x.css"
        );
        assert_eq!(
            join_url("D:\\site\\index.html", "style.css"),
            "D:\\site\\style.css"
        );
        assert_eq!(join_url("page.html", "style.css"), "style.css");
    }

    /// One local `<link>` sheet + arbitrary `<head>` markup; returns the
    /// computed color of `<p class="target">` after the real shell pipeline.
    fn stylesheet_color(tag: &str, sheet: &str, head: &str, viewport: Size) -> Color {
        let dir = std::env::temp_dir().join(format!("hodo-css-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("sheet.css"), sheet).unwrap();
        let page = dir.join("index.html").to_string_lossy().into_owned();
        let html =
            format!("<html><head>{head}</head><body><p class=\"target\">x</p></body></html>");
        let dom = hodo_html::parse_document_with(page_js_enabled(), &html);
        let stylesheet = collect_styles(&dom, &page, viewport);
        let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
        let mut target = None;
        dom.walk(|id, _| {
            if dom.attr(id, "class") == Some("target") {
                target = Some(id);
            }
        });
        let color = styles
            .get(target.expect("fixture has .target"))
            .unwrap()
            .color;
        let _ = std::fs::remove_dir_all(&dir);
        color
    }

    #[test]
    fn stylesheet_merge_order_follows_document_order() {
        let sheet = "p.target { color: rgb(255, 0, 0) }";
        let inline = "<style>p.target { color: rgb(0, 128, 0) }</style>";
        // <link> first, inline later: the inline rule wins the same-specificity tie.
        assert_eq!(
            stylesheet_color(
                "link-first",
                sheet,
                &format!("<link rel=\"stylesheet\" href=\"sheet.css\">{inline}"),
                DEFAULT_VIEWPORT,
            ),
            Color::rgb(0, 128, 0),
        );
        // Inline first, <link> later: the external rule wins the tie.
        assert_eq!(
            stylesheet_color(
                "style-first",
                sheet,
                &format!("{inline}<link rel=\"stylesheet\" href=\"sheet.css\">"),
                DEFAULT_VIEWPORT,
            ),
            Color::rgb(255, 0, 0),
        );
    }

    #[test]
    fn print_media_link_is_skipped_but_other_media_load() {
        let sheet = "p.target { color: rgb(255, 0, 0) }";
        assert_eq!(
            stylesheet_color(
                "print",
                sheet,
                "<link rel=\"stylesheet\" href=\"sheet.css\" media=\"print\">",
                DEFAULT_VIEWPORT,
            ),
            Color::BLACK,
            "a print-only sheet must not style the screen",
        );
        assert_eq!(
            stylesheet_color(
                "screen",
                sheet,
                "<link rel=\"stylesheet\" href=\"sheet.css\" media=\"screen\">",
                DEFAULT_VIEWPORT,
            ),
            Color::rgb(255, 0, 0),
        );
    }

    #[test]
    fn stylesheet_media_evaluates_against_the_shell_viewport() {
        let sheet = "p.target { color: rgb(0, 0, 255) } \
                     @media (max-width: 500px) { p.target { color: rgb(255, 0, 0) } }";
        let link = "<link rel=\"stylesheet\" href=\"sheet.css\">";
        assert_eq!(
            stylesheet_color("narrow", sheet, link, Size { w: 400.0, h: 300.0 }),
            Color::rgb(255, 0, 0),
        );
        assert_eq!(
            stylesheet_color("wide", sheet, link, DEFAULT_VIEWPORT),
            Color::rgb(0, 0, 255),
        );
    }

    #[test]
    fn failed_stylesheet_link_warns_and_the_page_still_renders() {
        let dir = std::env::temp_dir().join(format!("hodo-css-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = dir.join("index.html").to_string_lossy().into_owned();
        let html = "<html><head><link rel=\"stylesheet\" href=\"missing.css\"></head>\
                    <body><p>still here</p></body></html>";
        let dom = hodo_html::parse_document_with(page_js_enabled(), html);
        let stylesheet = collect_styles(&dom, &page, DEFAULT_VIEWPORT);
        let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
        let list = hodo_layout::layout(&dom, &styles, DEFAULT_VIEWPORT);
        assert!(
            list_has(&list, "still here"),
            "one bad sheet must not drop the page"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn demo_page_image_src_is_resolved_against_the_page_directory() {
        let page = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/demo-site/index.html")
            .to_string_lossy()
            .into_owned();
        let (_, list) = load_page(&page, DEFAULT_VIEWPORT);
        let image = list
            .items
            .iter()
            .find_map(|item| match item {
                DisplayItem::Image { path, .. } => Some(path.as_str()),
                _ => None,
            })
            .expect("demo page contains an <img>");
        assert!(
            Path::new(image).is_absolute(),
            "image src should be resolved to an absolute path: {image}"
        );
        assert!(
            Path::new(image).exists(),
            "resolved image path must point at the real file: {image}"
        );
    }

    #[test]
    fn error_page_builder_is_style_free_and_escapes_input() {
        let page = error_page_html("io: no <file> & friends");
        assert!(page.contains("hodo cannot load this page"));
        assert!(page.contains("io: no &lt;file&gt; &amp; friends"));
        assert!(!page.contains("<style"));
        assert!(!page.contains("<file>"));
    }

    #[test]
    fn tabs_keep_at_least_one_and_follow_active_index() {
        let mut app = App::new(None);
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active, 0);

        app.new_tab();
        app.new_tab();
        assert_eq!(app.tabs.len(), 3);
        assert_eq!(app.active, 2);

        app.close_active();
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active, 1);

        app.close_active();
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active, 0);

        // Closing the last remaining tab is a no-op.
        app.close_active();
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active < app.tabs.len());
    }

    #[test]
    fn history_push_back_forward_and_truncate() {
        let mut app = App::new(Some("/a"));
        assert_eq!(app.active_tab().history, vec!["/a"]);
        assert_eq!(app.active_tab().history_cursor, 0);

        app.navigate("/b", DEFAULT_VIEWPORT);
        app.navigate("/c", DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().history, vec!["/a", "/b", "/c"]);
        assert_eq!(app.active_tab().history_cursor, 2);

        app.back(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/b");
        app.back(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/a");
        app.back(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/a"); // no-op at the start
        app.forward(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/b");
        app.forward(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/c");
        app.forward(DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().url, "/c"); // no-op at the end

        app.back(DEFAULT_VIEWPORT); // /b
        app.back(DEFAULT_VIEWPORT); // /a
        app.navigate("/d", DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().history, vec!["/a", "/d"]);
        assert_eq!(app.active_tab().history_cursor, 1);
        assert_eq!(app.active_tab().url, "/d");
    }

    #[test]
    fn scroll_offset_is_clamped_to_the_page() {
        assert_eq!(clamp_scroll_offset(-10.0, 1000.0, 800.0), 0.0);
        assert_eq!(clamp_scroll_offset(50.0, 1000.0, 800.0), 50.0);
        assert_eq!(clamp_scroll_offset(500.0, 1000.0, 800.0), 268.0);
        assert_eq!(clamp_scroll_offset(1000.0, 500.0, 800.0), 0.0);

        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![
                DisplayItem::Text {
                    rect: Rect {
                        x: 0.0,
                        y: 10.0,
                        w: 100.0,
                        h: 20.0,
                    },
                    text: "a".into(),
                    color: Color::BLACK,
                    font_size: 16.0,
                    weight: 400,
                    href: None,
                    underline: false,
                    line_through: false,
                },
                DisplayItem::Rect {
                    rect: Rect {
                        x: 0.0,
                        y: 400.0,
                        w: 100.0,
                        h: 50.0,
                    },
                    color: Color::WHITE,
                    border_color: None,
                    radius: 0.0,
                },
            ],
        };
        assert_eq!(display_bottom(&page), 450.0);

        let mut app = App::new(Some(ABOUT_HODO_URL));
        app.tabs[0].page_height = 450.0;
        app.scroll_by(SCROLL_PAGE, DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().scroll_offset, 0.0); // 450 < 800
        app.scroll_to(1000.0, DEFAULT_VIEWPORT);
        assert_eq!(app.active_tab().scroll_offset, 0.0);
    }

    #[test]
    fn about_hodo_html_has_expected_content() {
        let html = about_hodo_html();
        assert!(html.contains("<h1>hodo</h1>"));
        assert!(html.contains("<title>hodo — new tab</title>"));
        assert!(html.contains("Ctrl+T opens a new tab"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("Cache:") && html.contains("hits"));

        let (title, list) = load_page(ABOUT_HODO_URL, DEFAULT_VIEWPORT);
        assert_eq!(title, "hodo — new tab");
        assert!(!list.items.is_empty());
    }

    #[test]
    fn tab_counter_formats_one_based_position() {
        assert_eq!(tab_counter(1, 3), "tab 1/3");
        assert_eq!(tab_counter(2, 3), "tab 2/3");
        assert_eq!(tab_counter(1, 1), "tab 1/1");
    }

    #[test]
    fn compositing_and_premultiply_are_byte_exact() {
        let mut rgba = vec![255, 0, 0, 128, 0, 0, 0, 0, 10, 20, 30, 255];
        composite_over_white(&mut rgba);
        assert_eq!(
            rgba,
            vec![255, 127, 127, 255, 255, 255, 255, 255, 10, 20, 30, 255]
        );

        let premul = premultiply_rgba(&[100, 50, 25, 128]);
        assert_eq!(premul, vec![50, 25, 13, 128]);
    }

    #[test]
    fn image_and_text_rects_shift_together_but_bars_stay_fixed() {
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![
                DisplayItem::Image {
                    rect: Rect {
                        x: 0.0,
                        y: 900.0,
                        w: 100.0,
                        h: 100.0,
                    },
                    path: "pic.png".into(),
                },
                linked_text(0.0, 950.0, 100.0, 20.0, None),
            ],
        };
        let composed = compose_display_list(&page, DEFAULT_VIEWPORT, "", &[], 30.0, "tab 1/1");
        let y = |index: usize| item_rect(&composed.items[index]).unwrap().y;

        assert_eq!(y(0), 870.0); // image shifted
        assert_eq!(y(1), 920.0); // text shifted by the same amount
        assert_eq!(y(2), 0.0); // top bar fixed
        assert_eq!(y(5), DEFAULT_VIEWPORT.h - BOTTOM_BAR_H); // bottom bar fixed
        assert_eq!(display_bottom(&page), 1000.0); // images count toward height
    }

    #[test]
    fn hit_test_link_uses_containment_and_reverse_priority() {
        let items = vec![
            linked_text(0.0, 0.0, 100.0, 20.0, Some("/first")),
            linked_text(0.0, 0.0, 100.0, 20.0, None), // ignored
            linked_text(0.0, 0.0, 100.0, 20.0, Some("/second")),
        ];
        assert_eq!(
            hit_test_link(&items, 10.0, 10.0).as_deref(),
            Some("/second")
        );
        assert_eq!(hit_test_link(&items, 200.0, 10.0), None); // outside x
        assert_eq!(hit_test_link(&items, 10.0, 25.0), None); // outside y
        assert_eq!(hit_test_link(&items, 100.0, 20.0), None); // exclusive edge

        // An unlinked overlapping item does not mask a link below it.
        let single = vec![
            linked_text(0.0, 0.0, 100.0, 20.0, Some("/below")),
            linked_text(0.0, 0.0, 100.0, 20.0, None),
        ];
        assert_eq!(hit_test_link(&single, 5.0, 5.0).as_deref(), Some("/below"));
    }

    #[test]
    fn handle_click_navigates_or_unfocuses_the_url_bar() {
        let mut app = App::new(Some(ABOUT_HODO_URL));
        app.tabs[0].display_list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![linked_text(0.0, 100.0, 100.0, 20.0, Some("/clicked"))],
        };
        app.tabs[0].page_height = 120.0;
        app.url_focused = true;

        assert!(app.handle_click(10.0, 110.0));
        assert_eq!(app.active_tab().url, "/clicked");
        assert_eq!(app.active_tab().history, vec![ABOUT_HODO_URL, "/clicked"]);
        assert_eq!(app.active_tab().history_cursor, 1);
        assert!(!app.url_focused);

        app.url_focused = true;
        assert!(!app.handle_click(500.0, 110.0)); // miss
        assert_eq!(app.active_tab().url, "/clicked");
        assert_eq!(app.active_tab().history.len(), 2);
        assert!(!app.url_focused);
    }

    fn popup_test_app(viewport: Size) -> BrowserApp {
        let action = crate::popup::PopupAction {
            title: "Popup Demo".into(),
            root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/popup-chrome"),
            popup_path: Some("popup.html".into()),
            icon_path: None,
        };
        let mut app = BrowserApp::new(None, vec![action.title.clone()], vec![action]);
        app.viewport = viewport;
        app.state.set_viewport(viewport);
        app
    }

    #[test]
    fn popup_toolbar_click_opens_and_esc_or_outside_click_dismisses() {
        let mut app = popup_test_app(DEFAULT_VIEWPORT);
        assert!(app.popup.is_none());

        app.click_at(20.0, 790.0); // action slot 0 in the bottom strip
        assert!(app.popup.is_some());
        assert!(!app.dismisses_popup(Key::Character("a")));
        assert!(app.dismisses_popup(Key::Named(NamedKey::Escape)));
        app.close_popup();
        assert!(app.popup.is_none());

        app.open_popup(0);
        app.click_at(1100.0, 100.0); // outside the panel, not a toolbar slot
        assert!(app.popup.is_none());
    }

    #[test]
    fn post_response_rebuilds_form_registry_and_click_still_works() {
        let mut app = BrowserApp::new(None, Vec::new(), Vec::new());
        let html = "<html><title>saved</title>\
            <form action='/next'><input name='q' value='hi'>\
            <input type='submit' value='Go'></form></html>";
        let form = FormPage::from_html(html, "http://example.test/next", DEFAULT_VIEWPORT)
            .expect("response has controls");
        let list = form.display_list().clone();
        let rect = form.registry.controls[0].rect;
        app.submit_form(FormAction::Response {
            title: "saved".into(),
            list,
            form: Some(Box::new(form)),
        });
        let forms = app.forms.as_ref().expect("registry survived the POST");
        assert_eq!(
            forms.registry.controls.len(),
            2,
            "response controls are live"
        );
        assert!(
            forms.hit(rect.x + 1.0, rect.y + 1.0, 0.0),
            "clicking a control in the POST response must land"
        );
        assert_eq!(app.state.active_tab().title, "saved");
    }

    #[test]
    fn resize_reanchors_the_open_popup() {
        let mut app = popup_test_app(DEFAULT_VIEWPORT);
        app.open_popup(0);
        let viewport = Size { w: 800.0, h: 600.0 };
        app.set_viewport(viewport);
        let popup = app.popup.as_ref().expect("popup stays open across resize");
        assert_eq!(popup.rect, crate::popup::popup_rect(viewport, 8.0));
        assert!(popup.rect.x + popup.rect.w <= viewport.w);
        assert!(popup.rect.y + popup.rect.h <= viewport.h - BOTTOM_BAR_H);
    }

    #[test]
    fn popup_composite_fits_a_window_smaller_than_the_panel() {
        let viewport = Size { w: 200.0, h: 300.0 };
        let popup = crate::popup::PopupState::open(
            &crate::popup::PopupAction {
                title: "Popup Demo".into(),
                root: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/popup-chrome"),
                popup_path: Some("popup.html".into()),
                icon_path: None,
            },
            0,
            viewport,
        )
        .unwrap();
        assert_eq!((popup.rect.w, popup.rect.h), (200.0, 300.0));

        let mut frame = vec![0xFF00_00FFu32; 200 * 300];
        popup.composite(&mut frame, viewport);
        assert!(
            frame.iter().any(|&pixel| pixel != 0xFF00_00FF),
            "the clamped panel must paint into the main frame"
        );
        assert!(frame.iter().all(|&pixel| pixel >> 24 == 255));
    }

    // ---- page JS ---------------------------------------------------------

    #[test]
    fn js_site_scripts_mutate_dom_before_first_paint() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (_, list, ctx) = load_page_live(&js_site_url(), DEFAULT_VIEWPORT);
        assert!(ctx.is_some(), "the fixture declares a script");
        assert!(
            list_has(&list, "JS-OK"),
            "script mutation must reach the first paint"
        );
        assert!(!list_has(&list, "JS-PENDING"));
    }

    #[test]
    fn js_site_click_from_node_map_updates_counter_via_page_listener() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (_, list, ctx) = load_page_live(&js_site_url(), DEFAULT_VIEWPORT);
        let index = list
            .items
            .iter()
            .position(|item| {
                matches!(
                    item,
                    DisplayItem::Control {
                        kind: hodo_layout::ControlKind::Button,
                        name,
                        ..
                    } if name == "btn"
                )
            })
            .expect("the fixture button is in the display list");
        let node = list.node_map[index];
        let mut ctx = ctx.expect("the fixture has a live page realm");
        let fresh = ctx
            .dispatch(
                node,
                "click",
                json!({ "x": 10.0, "y": 10.0 }),
                DEFAULT_VIEWPORT,
            )
            .expect("the click listener mutated the DOM");
        assert!(list_has(&fresh, "count: 1"));
        let fresh = ctx
            .dispatch(node, "click", json!({}), DEFAULT_VIEWPORT)
            .expect("the second click mutated the DOM");
        assert!(list_has(&fresh, "count: 2"));
    }

    #[test]
    fn redirect_final_url_drives_relative_resource_resolution() {
        let final_url =
            resolve_cached_redirects("https://example.com/sub/".to_string(), |hop| match hop {
                "https://example.com/sub/" => {
                    Some((301, "https://other.example.com/deep/".to_string()))
                }
                "https://other.example.com/deep/" => {
                    Some((308, "https://final.example.com/root/".to_string()))
                }
                _ => None,
            });
        assert_eq!(final_url, "https://final.example.com/root/");
        assert_eq!(
            join_url(&final_url, "img/x.png"),
            "https://final.example.com/root/img/x.png"
        );
        assert_eq!(
            join_url(&final_url, "/static/site.css"),
            "https://final.example.com/static/site.css",
            "root-relative URLs must keep the authority"
        );
    }

    #[test]
    fn cached_redirect_entry_supplies_the_final_base_url() {
        let dir = std::env::temp_dir().join(format!("hodo-redirect-base-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut cache = hodo_cache::DiskCache::open(&dir, 1024 * 1024).unwrap();
        let key = CacheKey::new("GET", "https://example.com/sub/", "");
        let entry = hodo_cache::CacheEntry {
            url: key.url.clone(),
            content_type: String::new(),
            data: Vec::new(),
            status_code: 301,
            expires_at: None,
            etag: None,
            last_modified: None,
            vary: Default::default(),
            headers: [(
                "location".to_string(),
                "https://other.example.com/deep/".to_string(),
            )]
            .into_iter()
            .collect(),
            partition_key: String::new(),
            request_method: "GET".to_string(),
            stored_at: 0,
        };
        cache.put(&key, &entry).unwrap();
        let final_url = resolve_cached_redirects("https://example.com/sub/".to_string(), |hop| {
            let found = cache.get(&CacheKey::new("GET", hop, ""))?;
            Some((found.status_code, found.headers.get("location")?.clone()))
        });
        assert_eq!(final_url, "https://other.example.com/deep/");
        drop(cache);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn script_free_demo_has_no_page_js_overhead() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let demo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/demo-site/index.html")
            .to_string_lossy()
            .into_owned();
        let (_, _, ctx) = load_page_live(&demo, DEFAULT_VIEWPORT);
        assert!(ctx.is_none(), "a script-free page creates no realm");

        let bench = |enabled: bool| {
            let start = Instant::now();
            for _ in 0..24 {
                let _ = load_page_with_flag(
                    &demo,
                    DEFAULT_VIEWPORT,
                    enabled,
                    page_script_budget(false),
                );
            }
            start.elapsed().as_secs_f64()
        };
        let _ = bench(true); // warm the shared cache and allocator
        let on = bench(true);
        let off = bench(false);
        let delta = if off > 0.0 {
            (on - off) / off * 100.0
        } else {
            0.0
        };
        eprintln!(
            "script-free demo — page JS on: {:.2} ms | off: {:.2} ms | delta: {delta:.1}%",
            on * 1000.0,
            off * 1000.0
        );
        assert!(
            on <= off * 1.75 + 0.01,
            "script-free page must not pay for page JS: {on} vs {off}"
        );
    }

    #[test]
    fn hodo_no_page_js_kills_the_whole_path() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        std::env::set_var("HODO_NO_PAGE_JS", "1");
        let enabled = page_js_enabled();
        let (_, list, ctx) = load_page_live(&js_site_url(), DEFAULT_VIEWPORT);
        std::env::remove_var("HODO_NO_PAGE_JS");
        assert!(!enabled);
        assert!(ctx.is_none(), "the kill switch must build no realm");
        assert!(list_has(&list, "JS-PENDING"));
        assert!(!list_has(&list, "JS-OK"));
        assert!(page_js_enabled(), "the switch must be scoped to its value");
    }

    #[test]
    fn script_caps_skip_modules_and_beyond_16() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-js-site-caps-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut html = String::from(
            "<div id=\"out\">none</div>\
             <script type=\"module\">document.getElementById('out').textContent='module';</script>",
        );
        for i in 0..(MAX_PAGE_SCRIPTS + 1) {
            html.push_str(&format!(
                "<script>document.getElementById('out').textContent='s{i}';</script>"
            ));
        }
        let path = dir.join("index.html");
        std::fs::write(&path, html).unwrap();
        let url = path.to_string_lossy().into_owned();
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        let last = MAX_PAGE_SCRIPTS - 1;
        assert!(
            list_has(&list, &format!("s{last}")),
            "16 scripts run in order"
        );
        assert!(!list_has(&list, &format!("s{MAX_PAGE_SCRIPTS}")));
        assert!(!list_has(&list, "module"), "type=module is skipped");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_js_script_types_are_not_executed() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-js-site-types-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Every script writes a distinct sentinel into its own element, so a
        // data block that was wrongly eval'd is observable even if another
        // script runs later.
        let mut html = String::from(
            "<div id='none'>none0</div><div id='empty'>empty0</div>\
             <div id='textjs'>textjs0</div><div id='appjs'>appjs0</div>\
             <div id='json'>json0</div><div id='ldjson'>ldjson0</div>\
             <div id='importmap'>importmap0</div><div id='spec'>spec0</div>\
             <div id='unknown'>unknown0</div><div id='module'>module0</div>",
        );
        let cases = [
            ("", "none", "NONE-RAN"),
            (" type=\"\"", "empty", "EMPTY-RAN"),
            (" type=\"text/javascript\"", "textjs", "TEXTJS-RAN"),
            (" type=\"application/javascript\"", "appjs", "APPJS-RAN"),
            (" type=\"application/json\"", "json", "JSON-RAN"),
            (" type=\"application/ld+json\"", "ldjson", "LDJSON-RAN"),
            (" type=\"importmap\"", "importmap", "IMPORTMAP-RAN"),
            (" type=\"speculationrules\"", "spec", "SPEC-RAN"),
            (" type=\"x/unknown\"", "unknown", "UNKNOWN-RAN"),
            (" type=\"module\"", "module", "MODULE-RAN"),
        ];
        for (attr, id, marker) in cases {
            html.push_str(&format!(
                "<script{attr}>document.getElementById('{id}').textContent='{marker}';</script>"
            ));
        }
        let path = dir.join("index.html");
        std::fs::write(&path, html).unwrap();
        let url = path.to_string_lossy().into_owned();
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some(), "classic scripts declare a realm");
        // Classic JS types run.
        for marker in ["NONE-RAN", "EMPTY-RAN", "TEXTJS-RAN", "APPJS-RAN"] {
            assert!(list_has(&list, marker), "{marker} must execute");
        }
        // Data blocks (and module) are skipped silently.
        for (_, _, marker) in cases.iter().skip(4) {
            assert!(!list_has(&list, marker), "{marker} must NOT execute");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_url_loads_index_html_and_runs_scripts() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/js-site")
            .to_string_lossy()
            .into_owned();
        let (title, list, ctx) = load_page_live(&dir, DEFAULT_VIEWPORT);
        assert_eq!(title, "JS site");
        assert!(ctx.is_some());
        assert!(list_has(&list, "JS-OK"));
    }

    #[test]
    fn js_site_painted_frame_contains_the_mutated_text_region() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let url = js_site_url();
        let (_, js_list, _) = load_page_live(&url, DEFAULT_VIEWPORT);
        let (_, raw_list, _) =
            load_page_with_flag(&url, DEFAULT_VIEWPORT, false, page_script_budget(false));
        let mut js_frame = hodo_render::render(&js_list, DEFAULT_VIEWPORT);
        let mut raw_frame = hodo_render::render(&raw_list, DEFAULT_VIEWPORT);
        composite_over_white(&mut js_frame);
        composite_over_white(&mut raw_frame);
        assert_ne!(
            js_frame, raw_frame,
            "the script's DOM mutation must reach the painted pixels"
        );
        let rect = js_list
            .items
            .iter()
            .find_map(|item| match item {
                DisplayItem::Text { rect, text, .. } if text.contains("JS-OK") => Some(*rect),
                _ => None,
            })
            .expect("JS-OK is laid out as a text item");
        let dark_pixels = |frame: &[u8]| -> usize {
            let mut count = 0;
            let w = DEFAULT_VIEWPORT.w as usize;
            let h = DEFAULT_VIEWPORT.h as usize;
            for y in (rect.y.max(0.0) as usize)..((rect.y + rect.h) as usize).min(h) {
                for x in (rect.x.max(0.0) as usize)..((rect.x + rect.w) as usize).min(w) {
                    let index = (y * w + x) * 4;
                    if frame[index] < 200 {
                        count += 1;
                    }
                }
            }
            count
        };
        assert!(
            dark_pixels(&js_frame) > 0,
            "the JS-OK glyph band must contain dark pixels"
        );
    }

    #[test]
    fn js_site_keydown_reaches_document_listener() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (_, _, ctx) = load_page_live(&js_site_url(), DEFAULT_VIEWPORT);
        let root = ctx.as_ref().unwrap().root();
        let list = ctx
            .unwrap()
            .dispatch(root, "keydown", json!({ "key": "Enter" }), DEFAULT_VIEWPORT)
            .expect("the keydown listener mutated the DOM");
        assert!(list_has(&list, "key:Enter"));
    }

    #[test]
    fn oversized_script_source_is_dropped() {
        let scripts = collect_scripts(
            vec![
                ScriptSource::Inline("small".to_string()),
                ScriptSource::Inline("x".repeat(MAX_SCRIPT_BYTES + 1)),
            ],
            "about:test",
        );
        assert_eq!(scripts, ["small"]);
    }

    // ---- shell seam wiring regressions -----------------------------------

    #[test]
    fn no_page_js_renders_noscript_content_to_pixels() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-noscript-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = dir.join("index.html");
        std::fs::write(
            &page,
            "<html><body><noscript><p>NOSCRIPT-NOTICE-42</p></noscript></body></html>",
        )
        .unwrap();
        let url = page.to_string_lossy().into_owned();

        std::env::set_var("HODO_NO_PAGE_JS", "1");
        let (_, off_list) = load_page(&url, DEFAULT_VIEWPORT);
        std::env::remove_var("HODO_NO_PAGE_JS");
        assert!(
            list_has(&off_list, "NOSCRIPT-NOTICE-42"),
            "with page scripting off, <noscript> content must be real markup"
        );
        let mut frame = hodo_render::render(&off_list, DEFAULT_VIEWPORT);
        composite_over_white(&mut frame);
        assert!(
            frame.as_chunks::<4>().0.iter().any(|px| px[0] < 200),
            "the noscript notice must reach painted pixels"
        );

        let (_, on_list) = load_page(&url, DEFAULT_VIEWPORT);
        assert!(
            !list_has(&on_list, "NOSCRIPT-NOTICE-42"),
            "with page scripting on, <noscript> content stays hidden"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn page_script_sees_the_navigated_url_as_window_location_href() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-js-location-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = dir.join("index.html");
        std::fs::write(
            &page,
            "<div id='out'>none</div>\
             <script>document.getElementById('out').textContent = window.location.href;</script>",
        )
        .unwrap();
        let url = page.to_string_lossy().into_owned();
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some(), "the fixture declares a script");
        assert!(
            list_has(&list, "hodo-js-location-"),
            "window.location.href must be the navigated page URL, not a <base> hint"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn page_script_document_title_mirrors_into_tab_title() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-js-title-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = dir.join("index.html");
        std::fs::write(
            &page,
            "<html><head><title>Old title</title></head><body>\
             <script>document.title = 'New';</script></body></html>",
        )
        .unwrap();
        let url = page.to_string_lossy().into_owned();
        let mut app = App::new(Some(&url));
        assert_eq!(app.active_tab().title, "Old title");
        app.page_js_tick(DEFAULT_VIEWPORT);
        assert_eq!(
            app.active_tab().title,
            "New",
            "a script-assigned document.title must reach the tab title"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn headless_page_script_budget_is_longer_than_gui() {
        assert_eq!(GUI_PAGE_SCRIPT_BUDGET_MS, 50);
        assert_eq!(HEADLESS_PAGE_SCRIPT_BUDGET_MS, 250);
    }
}

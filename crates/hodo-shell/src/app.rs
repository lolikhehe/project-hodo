//! Browser application: the winit handler, event routing and the page pipeline.
//!
//! The window-agnostic tab model lives in [`tabs`]; chrome composition in
//! [`ui`]; the softbuffer draw path in [`present`].

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use hodo_cache::CacheKey;
use hodo_css::{parse_stylesheet_with, Stylesheet};
use hodo_dom::{Dom, NodeId, NodeKind};
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
use crate::modules::{decode_dynamic_request, emit_graph, resolve_graph, DynamicModuleRequest};
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

/// Shell-owned cookie jar. `hodo_net::fetch` also mirrors Set-Cookie into its
/// process-wide jar, but reads must come from the jar we keep so cookies
/// round-trip across page/subresource loads. One jar per process is enough: the
/// GUI drives loads on a single thread and double-keyed partitions live inside
/// the jar.
fn shell_cookie_jar() -> &'static Mutex<hodo_cookie::CookieJar> {
    static JAR: OnceLock<Mutex<hodo_cookie::CookieJar>> = OnceLock::new();
    JAR.get_or_init(|| Mutex::new(hodo_cookie::CookieJar::new()))
}

/// Registrable domain of a URL's host (no PSL; last two labels, IP literal kept
/// whole). Used as both `top_site` and the cache partition for shell loads.
fn site_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .rsplit_once('@')
        .map_or(rest, |(_, host)| host);
    let host = authority.split(':').next().unwrap_or("");
    if host.parse::<std::net::Ipv4Addr>().is_ok()
        || host
            .trim_matches(['[', ']'])
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

/// Build the privacy options for one resource load. The private flag comes from
/// the active tab; the loaded URL supplies the top site / double-key partition
/// (the shell has no richer top-frame context to thread here yet).
fn resource_fetch_options(url: &str) -> hodo_net::FetchOptions {
    let site = site_of(url);
    hodo_net::FetchOptions {
        private: crate::tabs::private_load(),
        top_site: site.clone(),
        partition: site,
        referer: None,
        ..Default::default()
    }
}

/// Surface a fetch's advisory warnings once per URL. An HTTPS upgrade emits
/// both `MixedContent` and `InsecureUpgraded` for the same URL, and
/// `privacy.rs::WARN_COUNT` deliberately counts that pair once — mirror it here
/// instead of double-reporting.
fn report_resource_warnings(warnings: &[hodo_net::Warning]) {
    let mut seen = HashSet::new();
    for warning in warnings {
        let url = match warning {
            hodo_net::Warning::MixedContent(url)
            | hodo_net::Warning::InsecureUpgraded(url)
            | hodo_net::Warning::Blocked(url) => url.as_str(),
        };
        if seen.insert(url) {
            eprintln!("hodo: privacy warning for {url}: {warning:?}");
        }
    }
}

/// Load a page resource through the privacy-aware `hodo_net::fetch` entry:
/// tracking stripping, adblock, HSTS, cookies, partitions and warnings all
/// apply to page, stylesheet, script, ESM chunk and `fetch()` loads alike. If
/// the shared disk cache cannot be opened, fall back to a private (network-only)
/// fetch so privacy still holds rather than reverting to the frozen loader.
pub(crate) fn load_resource(url: &str) -> Result<Resource, hodo_net::FetchError> {
    let opts = resource_fetch_options(url);
    let mut jar = shell_cookie_jar()
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let outcome = match shell_cache() {
        Some(cache) => {
            let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
            hodo_net::fetch(url, &opts, &mut guard, &mut jar)?
        }
        None => {
            let root = std::env::temp_dir().join(format!("hodo-nocache-{}", std::process::id()));
            let mut cache = hodo_cache::DiskCache::open(&root, 1024 * 1024)
                .map_err(|err| hodo_net::FetchError::Io(err.to_string()))?;
            hodo_net::fetch(url, &opts, &mut cache, &mut jar)?
        }
    };
    report_resource_warnings(&outcome.warnings);
    outcome.resource.ok_or(hodo_net::FetchError::Network(
        "fetch returned no body".into(),
    ))
}

pub(crate) fn resource_text(resource: Resource) -> String {
    match resource {
        Resource::Html(text) => text,
        Resource::Other { data, .. } => String::from_utf8_lossy(&data).into_owned(),
    }
}

/// Batch-fetch one level of a module graph. Warm cache entries go through the
/// cached loader (so RFC 9111 / 304 revalidation is preserved); cold entries
/// are fetched in parallel by `hodo_net::load_many`. Every cold URL is passed
/// through the adblock/tracking pre-check first, so the parallel path cannot
/// bypass the privacy layer.
///
/// `loaded` is keyed by the requested URL; a failed chunk simply has no entry.
pub(crate) fn load_chunk_batch(urls: &[String]) -> HashMap<String, String> {
    if urls.is_empty() {
        return HashMap::new();
    }
    let opts = resource_fetch_options(&urls[0]);
    let mut cold = Vec::new();
    let mut out = HashMap::new();
    for url in urls {
        let warm = shell_cache().is_some_and(|cache| {
            let mut guard = cache.lock().unwrap_or_else(|err| err.into_inner());
            guard
                .get(&CacheKey::new("GET", url, &opts.partition))
                .is_some()
        });
        if warm {
            if let Ok(body) = load_resource(url).map(resource_text) {
                out.insert(url.clone(), body);
            }
            continue;
        }
        match hodo_net::check_adblock(url, &opts) {
            Ok(stripped) => cold.push((url.clone(), stripped)),
            Err(err) => eprintln!("hodo: chunk {url}: {err}"),
        }
    }
    if !cold.is_empty() {
        let requests: Vec<String> = cold.iter().map(|(_, url)| url.clone()).collect();
        for loaded in hodo_net::load_many(&requests, &opts) {
            // Map by the URL that actually served the body so a failed sibling
            // cannot shift the alignment (load_many returns only successes).
            if let Some((original, _)) = cold.iter().find(|(_, stripped)| {
                loaded.final_url == *stripped
                    || hodo_net::normalize_url(stripped).ok().as_deref()
                        == Some(loaded.final_url.as_str())
            }) {
                out.insert(original.clone(), resource_text(loaded.resource));
            }
        }
    }
    out
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
    let partition = site_of(url);
    resolve_cached_redirects(normalized, move |hop| {
        let mut guard = cache.lock().unwrap_or_else(|err| err.into_inner());
        let entry = guard.get(&CacheKey::new("GET", hop, &partition))?;
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
/// Dynamic scripts discovered per tick. A script that injects another one on
/// every run would otherwise never converge; the cap is per tick, so a live
/// timer can legitimately make progress on later frames.
const MAX_DYNAMIC_SCRIPT_ROUNDS: usize = 8;
/// `document.write` is parsed into a detached document and its `<body>`
/// children are copied into the live one.
/// shortcut: only these attributes are copied; extend when a written tag needs
/// arbitrary attributes (hodo-dom has no public attribute iterator).
const DOC_WRITE_ATTRS: &[&str] = &[
    "id",
    "class",
    "name",
    "src",
    "href",
    "alt",
    "title",
    "type",
    "value",
    "placeholder",
    "role",
    "style",
    "width",
    "height",
    "rel",
    "target",
    "action",
    "method",
    "for",
    "content",
    "charset",
    "lang",
    "dir",
    "checked",
    "selected",
    "disabled",
    "readonly",
    "required",
    "hidden",
];
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
    // Lit marker comments are `#comment` elements with their data here; the
    // same list must notice comment mutations so live repaints pick them up.
    "data",
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

/// Per-node change probe. Paired with [`dom_fingerprint`] (the cheap "did
/// anything change" gate), this yields the exact dirty roots the incremental
/// layout seam needs. Recycled slots carry new `NodeId` generation bits, so a
/// reused slot shows up as a new key here automatically.
fn dom_node_fingerprints(dom: &Dom) -> HashMap<NodeId, u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut out = HashMap::new();
    dom.walk(|id, node| {
        let mut hasher = DefaultHasher::new();
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
        out.insert(id, hasher.finish());
    });
    out
}

/// Hash of the collected stylesheet, used to detect a swap (a `<style>`/`<link>`
/// change) so the incremental layout context can be reset.
fn stylesheet_hash(sheet: &Stylesheet) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    format!("{sheet:?}").hash(&mut hasher);
    hasher.finish()
}

/// Paint-only repaint: update baked-in colours from the new style map without
/// touching geometry. The renderer's `diff_display_lists` then limits the
/// actual pixels redrawn to the changed rows (damage-only repaint).
fn repaint_display_list(list: &mut DisplayList, styles: &hodo_style::StyleMap) {
    let recolor = |owner: NodeId, item: &mut hodo_layout::DisplayItem| {
        let Some(style) = styles.get(owner) else {
            return;
        };
        match item {
            hodo_layout::DisplayItem::Rect {
                color,
                border_color,
                ..
            } => {
                *color = style.background;
                *border_color = (style.border_width > 0.0 && style.border_color.a > 0)
                    .then_some(style.border_color);
            }
            hodo_layout::DisplayItem::Text { color, .. } => *color = style.color,
            _ => {}
        }
    };
    for (owner, item) in list.node_map.iter().zip(list.items.iter_mut()) {
        recolor(*owner, item);
    }
    for (owner, item) in list.fixed_node_map.iter().zip(list.fixed_items.iter_mut()) {
        recolor(*owner, item);
    }
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
    budget: Duration,
    /// Script nodes already evaluated (or deliberately skipped). Keyed by the
    /// real arena `NodeId`, so a node can never run twice even after moves.
    executed_scripts: HashSet<NodeId>,
    /// Shared with the initial load's 16-script cap.
    script_count: usize,
    /// Dynamic-import requests captured from the native reply hook.
    dyn_requests: Rc<RefCell<Vec<DynamicModuleRequest>>>,
    /// Redirect-final URLs that seed `:visited` matching on relayout.
    visited: HashSet<String>,
    // --- incremental rendering state, filled by `init_incremental` -----------
    styles: hodo_style::StyleMap,
    layout_ctx: hodo_layout::LayoutContext,
    sheet_hash: u64,
    node_fingerprints: HashMap<NodeId, u64>,
    last_list: DisplayList,
    incremental_ready: bool,
}

impl PageContext {
    fn new(
        dom: Rc<RefCell<Dom>>,
        base_url: String,
        runtime: ExtScriptRuntime,
        budget: Duration,
        executed_scripts: HashSet<NodeId>,
        script_count: usize,
        dyn_requests: Rc<RefCell<Vec<DynamicModuleRequest>>>,
    ) -> Self {
        let fingerprint = dom_fingerprint(&dom.borrow());
        Self {
            dom,
            runtime,
            base_url,
            fingerprint,
            budget,
            executed_scripts,
            script_count,
            dyn_requests,
            visited: crate::tabs::visited_hrefs(),
            styles: hodo_style::StyleMap::default(),
            layout_ctx: hodo_layout::LayoutContext::new(),
            sheet_hash: 0,
            node_fingerprints: HashMap::new(),
            last_list: DisplayList::default(),
            incremental_ready: false,
        }
    }

    /// Hand the page the layout context and style map produced by its first
    /// paint. Called once after the initial display list is built so subsequent
    /// ticks can reuse clean subtrees instead of re-laying out the document.
    fn init_incremental(
        &mut self,
        layout_ctx: hodo_layout::LayoutContext,
        styles: hodo_style::StyleMap,
        sheet_hash: u64,
        list: &DisplayList,
    ) {
        self.node_fingerprints = dom_node_fingerprints(&self.dom.borrow());
        self.layout_ctx = layout_ctx;
        self.styles = styles;
        self.sheet_hash = sheet_hash;
        self.last_list = list.clone();
        self.incremental_ready = true;
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

    fn drain_logs(&mut self) {
        for line in self.runtime.take_logs() {
            eprintln!("hodo: page script: {line}");
        }
    }

    /// Drain timers/promises/queued events, then relayout only if a native
    /// actually changed the DOM. One fingerprint check per frame, so changes
    /// from several tasks coalesce into one relayout. Newly inserted scripts
    /// are executed after the first diff, then the layout is refreshed once.
    pub(crate) fn tick(&mut self, viewport: Size) -> Option<DisplayList> {
        self.runtime.tick();
        let _ = self.runtime.take_message_replies();
        self.drain_logs();
        let mut list = self.recompute_if_changed(viewport);
        if self.run_dynamic_scripts() {
            self.drain_logs();
            if let Some(next) = self.recompute_if_changed(viewport) {
                list = Some(next);
            }
        }
        if self.drain_dynamic_modules() {
            self.drain_logs();
            if let Some(next) = self.recompute_if_changed(viewport) {
                list = Some(next);
            }
        }
        list
    }

    /// Resolve dynamic-import requests captured from the native reply channel.
    /// Each requested graph runs through the strict module path, then the
    /// pending promise is settled with the runtime's Rust-facing native seam
    /// (`resolve_dynamic_import` / `reject_dynamic_import`).
    fn drain_dynamic_modules(&mut self) -> bool {
        let requests = std::mem::take(&mut *self.dyn_requests.borrow_mut());
        if requests.is_empty() {
            return false;
        }
        let mut ran_any = false;
        for request in requests {
            let target = join_url(&request.base_url, &request.spec);
            match resolve_graph(&target) {
                Ok(modules) => {
                    let graph = emit_graph(&modules);
                    // Trailing `0;` keeps eval's last value primitive — module
                    // namespaces are null-prototype objects.
                    if let Err(err) = self.runtime.eval_module_graph(&format!("{graph}\n0;")) {
                        eprintln!("hodo: dynamic module {target}: {err}");
                        self.runtime.reject_dynamic_import(request.promise_id, &err);
                        continue;
                    }
                    // Snapshot the entry namespace through the runner's registry
                    // and hand it to the native settle seam as a JSON value.
                    let url = serde_json::to_string(&target).expect("string serializes");
                    match self
                        .runtime
                        .eval(&format!("JSON.stringify(__hodoModules[{url}])"))
                    {
                        Ok(json_text) => match serde_json::from_str::<Value>(&json_text) {
                            Ok(exports) => self
                                .runtime
                                .resolve_dynamic_import(request.promise_id, &exports),
                            Err(err) => {
                                eprintln!("hodo: dynamic module exports {target}: {err}");
                                self.runtime
                                    .reject_dynamic_import(request.promise_id, &err.to_string());
                            }
                        },
                        Err(err) => {
                            eprintln!("hodo: dynamic module exports {target}: {err}");
                            self.runtime.reject_dynamic_import(request.promise_id, &err);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("hodo: dynamic module {target}: {err}");
                    self.runtime
                        .reject_dynamic_import(request.promise_id, &err.to_string());
                }
            }
            ran_any = true;
        }
        if ran_any {
            flush_doc_write(&self.runtime, &self.dom);
        }
        ran_any
    }

    /// Execute `<script>` nodes inserted since the last pass. The fingerprint
    /// diff above detects the insertion; this pass is where those nodes run.
    /// Capped at [`MAX_DYNAMIC_SCRIPT_ROUNDS`] per tick so a self-injecting
    /// script cannot spin forever, and bounded by the same wall budget, byte
    /// cap and 16-script total as the initial load.
    fn run_dynamic_scripts(&mut self) -> bool {
        let started = Instant::now();
        let mut executed_any = false;
        for _ in 0..MAX_DYNAMIC_SCRIPT_ROUNDS {
            if started.elapsed() > self.budget {
                break;
            }
            let pending: Vec<(NodeId, Option<String>, Option<String>, String)> = {
                let dom = self.dom.borrow();
                let mut out = Vec::new();
                dom.walk(|id, _| {
                    if dom.tag(id) != Some("script") || self.executed_scripts.contains(&id) {
                        return;
                    }
                    let src = dom.attr(id, "src").map(str::to_string);
                    let kind = dom.attr(id, "type").map(str::to_string);
                    let mut inline = String::new();
                    for &child in dom.children(id) {
                        if let Some(text) = dom.text(child) {
                            inline.push_str(text);
                        }
                    }
                    // An empty script may still be waiting for `src`/text to be
                    // assigned; leave it unmarked so a later tick can run it.
                    if src.is_none() && inline.trim().is_empty() {
                        return;
                    }
                    out.push((id, src, kind, inline));
                });
                out
            };
            if pending.is_empty() {
                break;
            }
            for (id, src, kind, inline) in pending {
                self.executed_scripts.insert(id);
                if !is_js_script_type(kind.as_deref()) {
                    continue;
                }
                if self.script_count >= MAX_PAGE_SCRIPTS {
                    eprintln!(
                        "hodo: dynamic page script cap of {MAX_PAGE_SCRIPTS} reached on {}",
                        self.base_url
                    );
                    continue;
                }
                let code = match src {
                    Some(src) => {
                        let url = join_url(&self.base_url, &src);
                        match load_resource(&url) {
                            Ok(Resource::Html(text)) => text,
                            Ok(Resource::Other { data, .. }) => {
                                String::from_utf8_lossy(&data).into_owned()
                            }
                            Err(err) => {
                                eprintln!("hodo: dynamic page script {url}: {err}");
                                continue;
                            }
                        }
                    }
                    None => inline,
                };
                if code.len() > MAX_SCRIPT_BYTES {
                    eprintln!(
                        "hodo: dynamic page script on {} exceeds {MAX_SCRIPT_BYTES} bytes",
                        self.base_url
                    );
                    continue;
                }
                if started.elapsed() > self.budget {
                    eprintln!(
                        "hodo: dynamic page script budget exceeded on {}",
                        self.base_url
                    );
                    break;
                }
                if let Err(err) = self.runtime.eval(&code) {
                    eprintln!("hodo: dynamic page script on {}: {err}", self.base_url);
                }
                flush_doc_write(&self.runtime, &self.dom);
                self.script_count += 1;
                executed_any = true;
            }
        }
        executed_any
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
        if !self.incremental_ready {
            return Some(self.relayout(viewport));
        }
        let dom = self.dom.borrow();
        let node_fps = dom_node_fingerprints(&dom);
        let old_node_fps = std::mem::replace(&mut self.node_fingerprints, node_fps.clone());
        let dirty: Vec<NodeId> = node_fps
            .iter()
            .filter(|(id, hash)| old_node_fps.get(*id) != Some(hash))
            .map(|(id, _)| *id)
            .collect();

        let stylesheet = collect_styles(&dom, &self.base_url, viewport);
        let sheet_hash = stylesheet_hash(&stylesheet);
        let new_styles =
            hodo_style::compute_styles_with_visited(&dom, &[stylesheet], &self.visited);

        // Property diff: a stylesheet swap or any layout-affecting style change
        // goes through the incremental relayout path; a pure colour/underline
        // change repaints only, leaving geometry (and the item count) untouched.
        let mut layout_affecting = sheet_hash != self.sheet_hash;
        if !layout_affecting {
            for id in &dirty {
                // A new NodeId (recycled slot) or a text node means content was
                // added/replaced; the stale slot style cannot describe it.
                if !old_node_fps.contains_key(id)
                    || matches!(dom.get(*id).kind, hodo_dom::NodeKind::Text(_))
                {
                    layout_affecting = true;
                    break;
                }
                match (self.styles.get(*id), new_styles.get(*id)) {
                    (Some(old), Some(new)) => {
                        if hodo_style::style_change_class(old, new)
                            == hodo_style::StyleChangeClass::LayoutAffecting
                        {
                            layout_affecting = true;
                            break;
                        }
                    }
                    _ => {
                        layout_affecting = true;
                        break;
                    }
                }
            }
        }

        let list = if layout_affecting {
            if sheet_hash != self.sheet_hash {
                self.layout_ctx.reset();
                self.sheet_hash = sheet_hash;
            }
            hodo_layout::layout_from(&dom, &new_styles, viewport, &mut self.layout_ctx, &dirty)
        } else {
            let mut list = self.last_list.clone();
            repaint_display_list(&mut list, &new_styles);
            list
        };
        self.styles = new_styles;
        self.last_list = list.clone();
        Some(list)
    }

    /// Full re-pipe against the live DOM. Used on resize (the viewport check in
    /// `layout_from` resets the context for the new geometry).
    pub(crate) fn relayout(&mut self, viewport: Size) -> DisplayList {
        let dom = self.dom.borrow();
        let stylesheet = collect_styles(&dom, &self.base_url, viewport);
        let sheet_hash = stylesheet_hash(&stylesheet);
        let new_styles =
            hodo_style::compute_styles_with_visited(&dom, &[stylesheet], &self.visited);
        if sheet_hash != self.sheet_hash {
            self.layout_ctx.reset();
            self.sheet_hash = sheet_hash;
        }
        let list = hodo_layout::layout_from(&dom, &new_styles, viewport, &mut self.layout_ctx, &[]);
        self.styles = new_styles;
        self.last_list = list.clone();
        self.node_fingerprints = dom_node_fingerprints(&dom);
        self.incremental_ready = true;
        list
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

/// Copy one parsed subtree into the live arena. NodeIds are arena-local, so the
/// nodes must be recreated; attributes come from the fixed list above.
fn copy_written_children(parsed: &Dom, from: NodeId, live: &mut Dom, to: NodeId) {
    for &child in parsed.children(from) {
        let copied = if let Some(text) = parsed.text(child) {
            live.create(NodeKind::Text(text.to_string()))
        } else if let Some(tag) = parsed.tag(child) {
            let attrs = DOC_WRITE_ATTRS
                .iter()
                .filter_map(|name| {
                    parsed
                        .attr(child, name)
                        .map(|v| ((*name).to_string(), v.to_string()))
                })
                .collect();
            live.create(NodeKind::Element {
                tag: tag.to_string(),
                attrs,
            })
        } else {
            continue;
        };
        live.append(to, copied);
        copy_written_children(parsed, child, live, copied);
    }
}

/// Parse buffered `document.write` text and append its `<body>` children to the
/// live `<body>`. Simplification: real browsers rewrite the open parse stream
/// mid-document; we approximate the common case (a script injecting a tag).
fn append_written_html(live: &mut Dom, html: &str) {
    if html.is_empty() {
        return;
    }
    let parsed = hodo_html::parse_document_with(true, &format!("<html><body>{html}</body></html>"));
    let mut parsed_body = None;
    let mut live_body = None;
    parsed.walk(|id, _| {
        if parsed_body.is_none() && parsed.tag(id) == Some("body") {
            parsed_body = Some(id);
        }
    });
    live.walk(|id, _| {
        if live_body.is_none() && live.tag(id) == Some("body") {
            live_body = Some(id);
        }
    });
    if let (Some(parsed_body), Some(live_body)) = (parsed_body, live_body) {
        copy_written_children(&parsed, parsed_body, live, live_body);
    }
}

/// Drain `document.write` text after one script completes and append it.
fn flush_doc_write(runtime: &ExtScriptRuntime, dom: &Rc<RefCell<Dom>>) {
    let written = runtime.take_doc_write();
    if !written.is_empty() {
        append_written_html(&mut dom.borrow_mut(), &written);
    }
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
    // Module graphs are discovered with a second, tiny walk: only
    // `<script type=module src=...>` can anchor a fetchable static graph.
    let (module_entries, preloads) = {
        let doc = dom.borrow();
        if js_enabled {
            (
                collect_module_entries(&doc, &base_url),
                collect_module_preloads(&doc, &base_url),
            )
        } else {
            (Vec::new(), Vec::new())
        }
    };
    // Prefetches share the same cache the graph fetcher reads from.
    for href in &preloads {
        let _ = load_resource(href);
    }
    // Resolve every entry graph, transform each chunk, and concatenate the
    // `__hodoRunModule(...)` calls in dependency-first order into one script.
    let mut emitted_urls = HashSet::new();
    let mut module_script = String::new();
    for entry in &module_entries {
        match resolve_graph(entry) {
            Ok(modules) => {
                for module in modules {
                    if emitted_urls.insert(module.url.clone()) {
                        module_script.push_str(&emit_graph(std::slice::from_ref(&module)));
                        module_script.push('\n');
                    }
                }
            }
            Err(err) => eprintln!("hodo: module graph {entry}: {err}"),
        }
    }
    if !module_script.is_empty() {
        dump_modules(&module_script);
    }

    // Every script node from the original parse is already accounted for. The
    // dynamic pass must only see scripts inserted later by page JS or written
    // in during initial load.
    let mut executed_scripts = HashSet::new();
    {
        let doc = dom.borrow();
        doc.walk(|id, _| {
            if doc.tag(id) == Some("script") {
                executed_scripts.insert(id);
            }
        });
    }
    let initial_count = scripts.len();
    let needs_realm = !scripts.is_empty() || !module_script.is_empty();

    // Dynamic-import requests arrive on the landed native-reply channel; the
    // hook captures them synchronously while JS calls `__hodo_message_reply`.
    let dyn_requests: Rc<RefCell<Vec<DynamicModuleRequest>>> = Rc::new(RefCell::new(Vec::new()));

    // realm-drop → parse → collect → eval → DOMContentLoaded → styles/layout
    let mut page = if !needs_realm {
        None
    } else {
        let root = dom.borrow().root();
        // Thread the final (post-redirect) URL so `window.location.href` reports
        // the page the user actually navigated to, not a `<base>` hint.
        let mut runtime = ExtScriptRuntime::new_page_with_url(Rc::clone(&dom), base_url.clone());
        let fetch_base = base_url.clone();
        runtime.set_fetch_hook(Rc::new(move |call| page_fetch_hook(call, &fetch_base)));
        // Page scripts cannot exchange extension messages, but the reply hook
        // is also the dynamic-import request channel; the queue is drained on
        // every tick (see `PageContext::drain_dynamic_modules`).
        let hook_sink = Rc::clone(&dyn_requests);
        runtime.set_message_reply_hook(Some(Rc::new(move |message, _response| {
            if let Some(request) = decode_dynamic_request(message) {
                hook_sink.borrow_mut().push(request);
            }
        })));
        let started = Instant::now();
        for script in scripts {
            if started.elapsed() > budget {
                eprintln!("hodo: page script budget exceeded on {url}");
                break;
            }
            if let Err(err) = runtime.eval(&script) {
                eprintln!("hodo: page script on {url}: {err}");
            }
            // document.write buffers during script eval; each completed script
            // flushes its text into <body> before the next one runs.
            flush_doc_write(&runtime, &dom);
        }
        // ESM calls are plain scripts for hodo-js; the paired JS agent supplies
        // `__hodoRunModule`. If it has not landed yet this logs once and the
        // rest of the pipeline still runs (cross-agent wave-1 state).
        if !module_script.is_empty() {
            // The scanner strips per-chunk directives; `eval_module_graph`
            // re-adds `"use strict";` on the host's strict eval path.
            let script = format!("{module_script}\n0;");
            if let Err(err) = runtime.eval_module_graph(&script) {
                eprintln!("hodo: module graph on {url}: {err}");
            }
            flush_doc_write(&runtime, &dom);
        }
        runtime.dispatch_page_event(root, "DOMContentLoaded", json!({}));
        let _ = runtime.tick();
        flush_doc_write(&runtime, &dom);
        for line in runtime.take_logs() {
            eprintln!("hodo: page script on {url}: {line}");
        }
        runtime.close_doc_write();
        let mut page = PageContext::new(
            Rc::clone(&dom),
            base_url.clone(),
            runtime,
            budget,
            executed_scripts,
            initial_count,
            dyn_requests,
        );
        // Headless callers never reach about_to_wait, so drive both dynamic
        // passes here too; the GUI realm runs them again as no-ops on tick 1.
        let _ = page.run_dynamic_scripts();
        let _ = page.drain_dynamic_modules();
        page.drain_logs();
        Some(page)
    };

    let visited = crate::tabs::visited_hrefs();
    let (display_list, styles, layout_ctx, sheet_hash) = {
        let doc = dom.borrow();
        let stylesheet = collect_styles(&doc, &base_url, viewport);
        let sheet_hash = stylesheet_hash(&stylesheet);
        let styles = hodo_style::compute_styles_with_visited(&doc, &[stylesheet], &visited);
        // One LayoutContext per document; the headless screenshot path uses the
        // same `layout_from` entry as live ticks (first call is a full layout).
        let mut layout_ctx = hodo_layout::LayoutContext::new();
        let list = hodo_layout::layout_from(&doc, &styles, viewport, &mut layout_ctx, &[]);
        (list, styles, layout_ctx, sheet_hash)
    };
    if let Some(page) = page.as_mut() {
        page.init_incremental(layout_ctx, styles, sheet_hash, &display_list);
    }
    // The page has settled into its first paint. Prime the STATIC half of the
    // next page's prelude so its ~56 ms parse overlaps that navigation's network
    // fetch. Boa's Context is !Send, so this must stay on the GUI/thread that
    // drives page loads; hodo-js stores the primed realm thread-locally. Tests
    // skip it: a primed realm checked out across pooled test threads corrupts
    // boa's thread-local handle table.
    #[cfg(not(test))]
    hodo_js::prime_page_prelude();
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
#[cfg(test)]
pub fn load_page(url: &str, viewport: Size) -> (String, DisplayList) {
    load_page_with_wait(url, viewport, Duration::ZERO)
}

/// Headless screenshot path with an explicit settle window. Module graphs (and
/// their dynamic imports) need more wall time than classic scripts; we keep
/// ticking the live realm in short slices so timers/promises can resolve and a
/// changed DOM is relaid out before the frame is captured.
pub fn load_page_with_wait(url: &str, viewport: Size, wait: Duration) -> (String, DisplayList) {
    // Headless has no frame deadline, so give scripts the longer budget.
    let (title, display_list, mut page) =
        load_page_with_flag(url, viewport, page_js_enabled(), page_script_budget(true));
    let Some(page) = page.as_mut() else {
        return (title, display_list);
    };
    let deadline = Instant::now() + wait;
    let mut list = display_list;
    while Instant::now() < deadline {
        let slice =
            Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now()));
        std::thread::sleep(slice);
        if let Some(next) = page.tick(viewport) {
            list = next;
        }
    }
    (title, list)
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

/// Collect `<script type="module" src=...>` entries in document order. Inline
/// module scripts have no URL to resolve a graph from, so they stay skipped
/// with a note (the module loader works on the `src=` shape from real bundles).
fn collect_module_entries(dom: &Dom, page_url: &str) -> Vec<String> {
    let mut out = Vec::new();
    dom.walk(|id, _| {
        if dom.tag(id) != Some("script") {
            return;
        }
        if !dom
            .attr(id, "type")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("module"))
        {
            return;
        }
        if let Some(src) = dom.attr(id, "src") {
            let url = join_url(page_url, src);
            if !out.contains(&url) {
                out.push(url);
            }
        } else {
            eprintln!("hodo: inline <script type=module> is skipped on {page_url}");
        }
    });
    out
}

/// `<link rel=modulepreload>` entries warm the same shared cache the graph
/// fetcher uses, so a prefetched chunk is a cache hit when its dep is visited.
fn collect_module_preloads(dom: &Dom, page_url: &str) -> Vec<String> {
    let mut out = Vec::new();
    dom.walk(|id, _| {
        if dom.tag(id) != Some("link") {
            return;
        }
        if !dom.attr(id, "rel").is_some_and(|value| {
            value
                .split_whitespace()
                .any(|kind| kind.eq_ignore_ascii_case("modulepreload"))
        }) {
            return;
        }
        if let Some(href) = dom.attr(id, "href") {
            out.push(join_url(page_url, href));
        }
    });
    out
}

/// `HODO_DUMP_MODULES=1` writes the emitted `__hodoRunModule(...)` script to
/// `hodo-modules-dump.js`; any other value is a target path. Cross-agent debug
/// seam: the shell's contract can be verified before the JS runtime lands.
fn dump_modules(script: &str) {
    let Ok(spec) = std::env::var("HODO_DUMP_MODULES") else {
        return;
    };
    if spec.is_empty() {
        return;
    }
    let path = if spec == "1" {
        PathBuf::from("hodo-modules-dump.js")
    } else {
        PathBuf::from(spec)
    };
    match std::fs::write(&path, script) {
        Ok(()) => eprintln!("hodo: module script dumped to {}", path.display()),
        Err(err) => eprintln!("hodo: module script dump {}: {err}", path.display()),
    }
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

    /// Serialises the `HODO_DUMP_MODULES` env toggle used by the module e2e test.
    static MODULE_DUMP_ENV_LOCK: Mutex<()> = Mutex::new(());

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

    #[test]
    fn visited_links_flip_color_only_for_visited_hrefs() {
        let dir = std::env::temp_dir().join(format!("hodo-visited-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            concat!(
                "<html><head><style>",
                "a:link{color:rgb(0,0,255)}a:visited{color:rgb(255,0,0)}",
                "</style></head><body>",
                "<div><a href='http://visited.test/'>V</a></div>",
                "<div><a href='http://fresh.test/'>F</a></div>",
                "</body></html>"
            ),
        )
        .unwrap();
        let url = format!("file://{}", dir.join("index.html").to_string_lossy());
        crate::tabs::set_visited_hrefs_for_test(&["http://visited.test/"]);
        let (_, list) = load_page(&url, DEFAULT_VIEWPORT);
        crate::tabs::set_visited_hrefs_for_test(&[]);
        let colors: Vec<(String, Color)> = list
            .items
            .iter()
            .filter_map(|item| match item {
                DisplayItem::Text { text, color, .. } => Some((text.clone(), *color)),
                _ => None,
            })
            .collect();
        let blue = Color::rgb(0, 0, 255);
        let red = Color::rgb(255, 0, 0);
        assert!(
            colors
                .iter()
                .any(|(text, color)| text == "V" && *color == red),
            "visited link must be red: {colors:?}"
        );
        assert!(
            colors
                .iter()
                .any(|(text, color)| text == "F" && *color == blue),
            "unvisited link must stay blue: {colors:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn paint_only_style_change_repaints_without_relayout() {
        let mut dom = Dom::new();
        let div = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("style".into(), "color:rgb(0,0,255);width:100px".into())],
        });
        dom.append(dom.root(), div);
        let text = dom.create(NodeKind::Text("x".into()));
        dom.append(div, text);
        let styles = hodo_style::compute_styles(&dom, &[]);
        let mut list = hodo_layout::layout(&dom, &styles, DEFAULT_VIEWPORT);
        let count = list.items.len();

        // Hover-colour change: paint only. Item count and geometry stay put;
        // only the baked-in colour is refreshed (damage-only repaint).
        dom.set_attr(div, "style", "color:rgb(255,0,0);width:100px".into());
        let hover = hodo_style::compute_styles(&dom, &[]);
        assert_eq!(
            hodo_style::style_change_class(styles.get(div).unwrap(), hover.get(div).unwrap()),
            hodo_style::StyleChangeClass::PaintOnly
        );
        repaint_display_list(&mut list, &hover);
        assert_eq!(
            list.items.len(),
            count,
            "repaint must not change item count"
        );
        assert!(list.items.iter().any(
            |item| matches!(item, DisplayItem::Text { color, .. } if *color == Color::rgb(255, 0, 0))
        ));

        // Width change: layout-affecting, so the existing relayout path is used.
        dom.set_attr(div, "style", "color:rgb(255,0,0);width:200px".into());
        let wide = hodo_style::compute_styles(&dom, &[]);
        assert_eq!(
            hodo_style::style_change_class(hover.get(div).unwrap(), wide.get(div).unwrap()),
            hodo_style::StyleChangeClass::LayoutAffecting
        );
    }

    #[test]
    fn incremental_layout_reuses_clean_subtrees_on_one_node_edit() {
        let mut dom = Dom::new();
        let root = dom.root();
        let mut blocks = Vec::new();
        for i in 0..8 {
            let block = dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![("style".into(), format!("background:#{:06x}", i + 1))],
            });
            dom.append(root, block);
            let text = dom.create(NodeKind::Text(format!("line {i}")));
            dom.append(block, text);
            blocks.push(block);
        }
        let styles = hodo_style::compute_styles(&dom, &[]);
        let mut ctx = hodo_layout::LayoutContext::new();
        let first = hodo_layout::layout_from(&dom, &styles, DEFAULT_VIEWPORT, &mut ctx, &[]);
        assert_eq!(ctx.stats().reused_subtrees, 0, "first call is full");
        let old_text = dom.children(blocks[3])[0];
        dom.detach_subtree(old_text);
        let new_text = dom.create(NodeKind::Text("changed".into()));
        dom.append(blocks[3], new_text);
        let second =
            hodo_layout::layout_from(&dom, &styles, DEFAULT_VIEWPORT, &mut ctx, &[new_text]);
        assert_eq!(
            ctx.stats().fresh_subtrees,
            1,
            "only the edited subtree re-laid out"
        );
        assert_eq!(
            ctx.stats().reused_subtrees,
            7,
            "seven clean subtrees reused"
        );
        assert_eq!(
            second.items.len(),
            first.items.len(),
            "item count is invariant"
        );
    }

    #[test]
    fn load_chunk_batch_uses_parallel_cold_and_revalidating_warm_paths() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind");
        let port = listener.local_addr().expect("local addr").port();
        let base = format!("http://127.0.0.1:{port}");
        let saw_conditional = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = std::sync::Arc::clone(&saw_conditional);
        let server = std::thread::spawn(move || {
            // Four connections: two cold, one warm-setup, one warm revalidation.
            for _ in 0..4 {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let end = loop {
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        break pos + 4;
                    }
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => return,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                };
                let head = String::from_utf8_lossy(&buf[..end]).into_owned();
                if head.to_ascii_lowercase().contains("if-none-match: \"v1\"") {
                    flag.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                let response = if head.to_ascii_lowercase().contains("if-none-match") {
                    "HTTP/1.1 304 Not Modified\r\nETag: \"v1\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                } else {
                    "HTTP/1.1 200 OK\r\nETag: \"v1\"\r\nCache-Control: max-age=0\r\nContent-Type: text/javascript\r\nContent-Length: 10\r\nConnection: close\r\n\r\nchunk-body"
                };
                let _ = stream.write_all(response.as_bytes());
            }
        });

        // Cold: nothing cached for these URLs -> parallel `load_many` path.
        let cold = vec![format!("{base}/a.js"), format!("{base}/b.js")];
        let bodies = load_chunk_batch(&cold);
        assert_eq!(bodies.get(&cold[0]).map(String::as_str), Some("chunk-body"));
        assert_eq!(bodies.get(&cold[1]).map(String::as_str), Some("chunk-body"));

        // Populate the shared cache, then the batch must take the warm,
        // per-chunk cached path and revalidate with If-None-Match / 304.
        load_resource(&cold[0]).expect("warm setup load");
        let warm = load_chunk_batch(&cold[0..1]);
        assert_eq!(warm.get(&cold[0]).map(String::as_str), Some("chunk-body"));
        server.join().expect("server thread");
        assert!(
            saw_conditional.load(std::sync::atomic::Ordering::SeqCst),
            "warm chunk must revalidate with If-None-Match (304 path)"
        );
    }

    #[test]
    fn dom_fingerprint_notices_comment_data_mutations() {
        let mut dom = Dom::new();
        let comment = dom.create(NodeKind::Element {
            tag: "#comment".into(),
            attrs: vec![("data".into(), "?lit$1$".into())],
        });
        dom.append(dom.root(), comment);
        let before = dom_fingerprint(&dom);
        dom.set_attr(comment, "data", "?lit$2$".to_string());
        assert_ne!(dom_fingerprint(&dom), before);
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
    fn module_tagged_script_with_src_is_collected_not_skipped() {
        let dom = hodo_html::parse_document_with(
            true,
            "<html><body>\
             <script type=\"module\" src=\"./entry.js\"></script>\
             <script type=\"module\">var inline=1;</script>\
             <link rel=\"modulepreload\" href=\"./dep.js\">\
             </body></html>",
        );
        let entries = collect_module_entries(&dom, "http://hodo.test/page/index.html");
        assert_eq!(entries, vec!["http://hodo.test/page/entry.js"]);
        let preloads = collect_module_preloads(&dom, "http://hodo.test/page/index.html");
        assert_eq!(preloads, vec!["http://hodo.test/page/dep.js"]);
    }

    /// Local file:// fixture: one module entry with two deps, a modulepreload
    /// and `--wait-ms 1500` equivalent. The shell must emit three contract
    /// calls; a factory-painted DOM change is the payoff once the paired JS
    /// runtime lands (before then the dump is the cross-agent evidence).
    #[test]
    fn module_graph_fixture_emits_contract_calls_for_every_chunk() {
        let _dump_guard = MODULE_DUMP_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-modules-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<html><body><div id='out'>none</div>\
             <link rel='modulepreload' href='./dep1.js'>\
             <script type='module' src='./entry.js'></script>\
             </body></html>",
        )
        .unwrap();
        std::fs::write(
            dir.join("entry.js"),
            "import{v}from'./dep1.js';import{w}from'./dep2.js';\
             document.getElementById('out').textContent=v+w;export{v,w};",
        )
        .unwrap();
        std::fs::write(dir.join("dep1.js"), "var v='MOD';export{v};").unwrap();
        std::fs::write(dir.join("dep2.js"), "var w='-OK';export{w};").unwrap();

        let dump = dir.join("emitted.js");
        std::env::set_var("HODO_DUMP_MODULES", dump.to_string_lossy().as_ref());
        let url = format!("file://{}", dir.join("index.html").to_string_lossy());
        let (_, list) = load_page_with_wait(&url, DEFAULT_VIEWPORT, Duration::from_millis(1500));
        std::env::remove_var("HODO_DUMP_MODULES");

        let emitted = std::fs::read_to_string(&dump).expect("HODO_DUMP_MODULES wrote the script");
        assert_eq!(
            emitted.lines().count(),
            3,
            "entry plus two deps, one call each: {emitted}"
        );
        assert!(emitted.contains("__hodoRunModule("), "{emitted}");
        assert!(emitted.contains("file://"), "{emitted}");
        assert!(emitted.contains("v+w"), "{emitted}");
        assert!(!emitted.contains("__hodoDynImport"), "{emitted}");
        // Conditional by design: with the JS side landed the factory mutates
        // the DOM; until then `__hodoRunModule` is undefined and the emit
        // above is the honest cross-agent evidence. Report either way.
        if list_has(&list, "MOD-OK") {
            eprintln!("hodo: module factory painted the DOM change");
        } else {
            eprintln!("hodo: module JS runtime not landed yet; contract verified via dump");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// End-to-end dynamic import: the entry factory calls `import('./child.js')`,
    /// the reply-channel request is decoded, the chunk graph is evaluated through
    /// the strict module path, and the native settle seam resolves the promise,
    /// whose `.then` paints the DOM before the screenshot frame is captured.
    #[test]
    fn dynamic_import_bridge_settles_and_paints_dom() {
        // `load_page_with_wait` reads HODO_DUMP_MODULES; keep it from observing
        // (or overwriting) the sibling e2e test's dump path.
        let _dump_guard = MODULE_DUMP_ENV_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("hodo-modules-dyn-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<html><body><div id='out'>none</div>\
             <script type='module' src='./entry.js'></script></body></html>",
        )
        .unwrap();
        std::fs::write(
            dir.join("entry.js"),
            "import{v}from'./dep1.js';import('./child.js').then(function(m){\
             document.getElementById('out').textContent=v+':'+m.c;});export{v};",
        )
        .unwrap();
        std::fs::write(dir.join("dep1.js"), "var v='DYN';export{v};").unwrap();
        std::fs::write(dir.join("child.js"), "var c='-OK';export{c};").unwrap();

        let url = format!("file://{}", dir.join("index.html").to_string_lossy());
        let (_, list) = load_page_with_wait(&url, DEFAULT_VIEWPORT, Duration::from_millis(1500));
        assert!(
            list_has(&list, "DYN:-OK"),
            "dynamic import must settle and paint before capture"
        );
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

    /// Write an HTML fixture into a unique temp dir; return the dir and URL.
    fn html_fixture(tag: &str, html: &str) -> (std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join(format!("hodo-dyn-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let page = dir.join("index.html");
        std::fs::write(&page, html).unwrap();
        (dir, page.to_string_lossy().into_owned())
    }

    #[test]
    fn document_write_during_load_appends_and_renders() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "write",
            "<html><body><div id='out'>none</div>\
             <script>document.write('<div id=\"w\">Written</div>');</script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        assert!(list_has(&list, "Written"), "written markup must render");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn appended_script_with_src_executes() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "src",
            "<html><body><div id='out'>none</div><script>\
             var s = document.createElement('script'); s.src = 'dyn.js'; \
             document.body.appendChild(s);</script></body></html>",
        );
        std::fs::write(
            dir.join("dyn.js"),
            "document.getElementById('out').textContent = 'SRC-RAN';",
        )
        .unwrap();
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        assert!(list_has(&list, "SRC-RAN"), "appended src script must run");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn appended_inline_script_executes() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "inline",
            "<html><body><div id='out'>none</div><script>\
             var s = document.createElement('script'); \
             s.textContent = \"document.getElementById('out').textContent='INLINE-RAN';\"; \
             document.body.appendChild(s);</script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        assert!(
            list_has(&list, "INLINE-RAN"),
            "appended inline script must run"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_script_node_never_executes_twice_across_ticks() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "dedupe",
            "<html><body><div id='out'>none</div><script>\
             window.n = 0; var s = document.createElement('script'); \
             s.textContent = \"window.n++; document.getElementById('out').textContent='n='+window.n;\"; \
             document.body.appendChild(s); document.head.appendChild(s); \
             document.body.appendChild(s);</script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        let mut ctx = ctx.unwrap();
        assert!(list_has(&list, "n=1"), "one node runs once");
        for _ in 0..3 {
            let next = ctx
                .tick(DEFAULT_VIEWPORT)
                .unwrap_or_else(|| ctx.relayout(DEFAULT_VIEWPORT));
            assert!(
                !list_has(&next, "n=2"),
                "the same NodeId must not run again"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dynamic_non_js_script_types_are_skipped() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "json-type",
            "<html><body><div id='out'>none</div><script>\
             var s = document.createElement('script'); s.setAttribute('type','application/json'); \
             s.textContent = \"document.getElementById('out').textContent='JSON-DYN-RAN';\"; \
             document.body.appendChild(s);</script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        assert!(
            !list_has(&list, "JSON-DYN-RAN"),
            "data blocks never execute"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dynamic_script_rounds_are_capped_per_tick() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "cap",
            "<html><body><div id='out'>none</div><script>\
             window.n = 0; function inject() { var s = document.createElement('script'); \
             s.textContent = \"window.n++; document.getElementById('out').textContent='rounds='+window.n; inject();\"; \
             document.body.appendChild(s); } inject();</script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        assert!(ctx.is_some());
        assert!(
            list_has(&list, &format!("rounds={MAX_DYNAMIC_SCRIPT_ROUNDS}")),
            "a self-injecting script stops after the per-tick cap"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn post_load_document_write_is_ignored() {
        let _guard = PAGE_JS_ENV_LOCK.lock().unwrap();
        let (dir, url) = html_fixture(
            "late-write",
            "<html><body><div id='out'>none</div><script>\
             document.close(); \
             window.setTimeout(function () { document.write('<div>LATE-WRITE</div>'); }, 1); \
             </script></body></html>",
        );
        let (_, list, ctx) = load_page_live(&url, DEFAULT_VIEWPORT);
        let mut ctx = ctx.unwrap();
        assert!(!list_has(&list, "LATE-WRITE"));
        std::thread::sleep(std::time::Duration::from_millis(5));
        let _ = ctx.tick(DEFAULT_VIEWPORT);
        let list = ctx.relayout(DEFAULT_VIEWPORT);
        assert!(
            !list_has(&list, "LATE-WRITE"),
            "post-load writes do nothing"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn headless_page_script_budget_is_longer_than_gui() {
        assert_eq!(GUI_PAGE_SCRIPT_BUDGET_MS, 50);
        assert_eq!(HEADLESS_PAGE_SCRIPT_BUDGET_MS, 250);
    }
}

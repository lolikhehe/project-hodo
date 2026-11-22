//! Browser application: the winit handler, event routing and the page pipeline.
//!
//! The window-agnostic tab model lives in [`tabs`]; chrome composition in
//! [`ui`]; the softbuffer draw path in [`present`].

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use hodo_css::{parse_stylesheet, Stylesheet};
use hodo_dom::Dom;
use hodo_layout::DisplayList;
use hodo_net::Resource;
use hodo_types::Size;
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

    /// Route a left-click: toolbar action → popup click → dismiss → page link.
    fn click_at(&mut self, x: f32, y: f32) {
        if let Some(index) = crate::ui::hit_test_action(self.viewport, &self.ext_titles, x, y) {
            self.open_popup(index);
        } else if self.popup.is_some() {
            // Click inside the panel is consumed; outside dismisses it.
            let inside = self.popup.as_mut().is_some_and(|popup| popup.click(x, y));
            if !inside {
                self.close_popup();
            }
        } else if self
            .forms
            .as_ref()
            .is_some_and(|form| form.hit(x, y, self.state.active_tab().scroll_offset))
        {
            let scroll = self.state.active_tab().scroll_offset;
            let action = self
                .forms
                .as_mut()
                .and_then(|form| form.click(x, y, scroll));
            self.state.url_focused = false;
            self.tick_forms();
            if let Some(action) = action {
                self.submit_form(action);
            }
        } else if self.state.handle_click(x, y) {
            let title = self.state.active_tab().title.clone();
            self.set_window_title(&title);
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
                self.submit_form(action);
                return;
            }
            if handled {
                self.tick_forms();
                return;
            }
        }
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
        // Idle costs nothing: redraws are event-driven. A popup still owns a JS
        // context with timers, so keep its 33 ms tick until it closes.
        let flow = if self.popup.is_some() {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(33))
        } else {
            ControlFlow::Wait
        };
        event_loop.set_control_flow(flow);
    }
}

/// Resolve a stylesheet `href` against the page URL.
///
/// Naive by design: absolute URLs (with a scheme) pass through unchanged;
/// otherwise the last path segment of `base` is replaced. Handles both `/`
/// and `\` separators so local Windows paths work.
pub fn join_url(base: &str, href: &str) -> String {
    if href.contains("://") || href.starts_with("file:") || href.starts_with("data:") {
        return href.to_string();
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

/// Run the whole pipeline for one URL and return `(title, display_list)`.
///
/// The DOM, stylesheet and style map are local to this function and dropped
/// before it returns — that is the low-RAM contract of the shell.
pub fn load_page(url: &str, viewport: Size) -> (String, DisplayList) {
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
    let mut dom = hodo_html::parse_document(&html);
    // Layout has no base-URL parameter, so a relative img[src] would resolve
    // against the process CWD. Rewrite it against the page we actually loaded.
    let mut relative_images = Vec::new();
    dom.walk(|id, _| {
        if dom.tag(id).is_some_and(|t| t.eq_ignore_ascii_case("img")) {
            if let Some(src) = dom.attr(id, "src") {
                if !src.contains("://") && !src.starts_with("data:") {
                    relative_images.push((id, join_url(url, src)));
                }
            }
        }
    });
    for (id, src) in relative_images {
        dom.set_attr(id, "src", src);
    }
    let title = first_title(&dom);
    let stylesheet = collect_styles(&dom, url);
    let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
    let display_list = hodo_layout::layout(&dom, &styles, viewport);
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

fn first_title(dom: &Dom) -> String {
    let mut title = String::new();
    let mut found = false;
    dom.walk(|id, _| {
        if !found && dom.tag(id) == Some("title") {
            found = true;
            for &child in dom.children(id) {
                if let Some(text) = dom.text(child) {
                    title.push_str(text);
                }
            }
        }
    });
    title
}

pub(crate) fn collect_styles(dom: &Dom, page_url: &str) -> Stylesheet {
    // The engine's UA sheet does not hide <head>; without this the <style>
    // and <title> text would spill into the page as visible text.
    let mut sources: Vec<String> =
        vec!["head, style, link, meta, title, script { display: none }".to_owned()];

    // Every <style> element contributes the text of its children.
    dom.walk(|id, _| {
        if dom.tag(id) == Some("style") {
            for &child in dom.children(id) {
                if let Some(text) = dom.text(child) {
                    sources.push(text.to_owned());
                }
            }
        }
    });

    // Every <link rel="stylesheet" href="..."> is resolved and loaded.
    // Load failures are ignored; the rest of the page still renders.
    dom.walk(|id, _| {
        if dom.tag(id) == Some("link") && dom.attr(id, "rel") == Some("stylesheet") {
            if let Some(href) = dom.attr(id, "href") {
                let resolved = join_url(page_url, href);
                if let Ok(resource) = load_resource(&resolved) {
                    match resource {
                        Resource::Html(text) => sources.push(text),
                        Resource::Other { data, .. } => {
                            sources.push(String::from_utf8_lossy(&data).into_owned());
                        }
                    }
                }
            }
        }
    });

    let mut rules = Vec::new();
    for source in sources {
        rules.extend(parse_stylesheet(&source).rules);
    }
    Stylesheet { rules }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
}

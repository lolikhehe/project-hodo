//! Tab model + rendering pipeline for the hodo shell.
//!
//! Design goal: keep RAM flat regardless of tab count. A `Tab` stores only
//! the URL, title and a flat `DisplayList`; the DOM/stylesheet/style map are
//! created and dropped inside [`load_page`]. Only the active tab ever has a
//! pixel buffer, rebuilt on navigation, tab switch and resize (~33 ms).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use hodo_css::{parse_stylesheet, Stylesheet};
use hodo_dom::Dom;
use hodo_layout::{DisplayItem, DisplayList};
use hodo_net::Resource;
use hodo_types::{Color, Rect, Size};

/// Viewport used by the headless screenshot mode and as the initial GUI size.
pub const DEFAULT_VIEWPORT: Size = Size {
    w: 1200.0,
    h: 800.0,
};

/// Height of the top URL bar strip.
const TOP_BAR_H: f32 = 40.0;
/// Height of the bottom extension toolbar strip.
const BOTTOM_BAR_H: f32 = 28.0;

/// Synthetic welcome page used for new tabs and bare launches.
pub const ABOUT_HODO_URL: &str = "about:hodo";
/// Pixels scrolled per wheel line.
pub const SCROLL_LINE: f32 = 40.0;
/// Pixels scrolled per PageUp / PageDown key press.
pub const SCROLL_PAGE: f32 = 400.0;

/// A browser tab. Deliberately holds no DOM, stylesheet or pixels.
pub struct Tab {
    pub url: String,
    pub title: String,
    pub display_list: DisplayList,
    /// Current vertical scroll, in pixels (tab-local, applied at compose time).
    pub scroll_offset: f32,
    /// Bottom edge of the page before scrolling; source of the scroll clamp.
    pub page_height: f32,
    /// Visited URLs, oldest first.
    pub history: Vec<String>,
    /// Index into `history` of the current URL.
    pub history_cursor: usize,
}

impl Tab {
    fn blank() -> Self {
        Self {
            url: String::new(),
            title: String::new(),
            display_list: DisplayList::default(),
            scroll_offset: 0.0,
            page_height: 0.0,
            history: Vec::new(),
            history_cursor: 0,
        }
    }
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
    /// ARGB8888 frame for the *active* tab only: `(255<<24)|(r<<16)|(g<<8)|b`.
    frame: Vec<u32>,
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
            frame: Vec::new(),
        };
        let welcome = initial_url.is_none();
        app.navigate(initial_url.unwrap_or(ABOUT_HODO_URL), DEFAULT_VIEWPORT);
        // On a bare launch keep the URL bar focused so typing works at once.
        app.url_focused = welcome;
        app
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active]
    }

    /// Open a new tab on the welcome page and make it active.
    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::blank());
        self.active = self.tabs.len() - 1;
        self.frame.clear();
        self.navigate(ABOUT_HODO_URL, DEFAULT_VIEWPORT);
    }

    /// Close the active tab, keeping at least one tab open at all times.
    pub fn close_active(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        self.url_bar = self.tabs[self.active].url.clone();
        self.url_focused = false;
        self.frame.clear();
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
        self.frame.clear();
        title
    }

    /// Move one entry back in history. No-op at the oldest entry.
    pub fn back(&mut self, viewport: Size) -> String {
        let cursor = self.tabs[self.active].history_cursor;
        if cursor == 0 {
            return self.tabs[self.active].title.clone();
        }
        self.tabs[self.active].history_cursor = cursor - 1;
        self.load_history_entry(viewport)
    }

    /// Move one entry forward in history. No-op at the newest entry.
    pub fn forward(&mut self, viewport: Size) -> String {
        let tab = &mut self.tabs[self.active];
        if tab.history_cursor + 1 >= tab.history.len() {
            return tab.title.clone();
        }
        tab.history_cursor += 1;
        self.load_history_entry(viewport)
    }

    /// Re-run the pipeline for the active tab (F5). No-op on a blank tab.
    pub fn reload(&mut self, viewport: Size) -> String {
        let url = self.tabs[self.active].url.clone();
        if url.is_empty() {
            return String::new();
        }
        let (title, display_list) = load_page(&url, viewport);
        let tab = &mut self.tabs[self.active];
        tab.title = title.clone();
        tab.display_list = display_list;
        tab.scroll_offset = 0.0;
        tab.page_height = display_bottom(&tab.display_list);
        self.frame.clear();
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
        match hit_test_link(&composed.items, x, y) {
            Some(href) => {
                self.navigate(&href, self.viewport);
                true
            }
            None => {
                self.url_focused = false;
                false
            }
        }
    }

    /// Compose the full frame for the active tab, with the UI overlay on top.
    ///
    /// The returned slice is ARGB8888, row-major, `w * h` entries. Rebuilding
    /// it every redraw is cheap (the page is already a flat display list) and
    /// guarantees a tab switch or resize shows up within one 33 ms tick.
    pub fn frame(&mut self, viewport: Size, ext_titles: &[String]) -> &[u32] {
        self.viewport = viewport;
        let tab = &self.tabs[self.active];
        let label = tab_counter(self.active + 1, self.tabs.len());
        let list = compose_display_list(
            &tab.display_list,
            viewport,
            &self.url_bar,
            ext_titles,
            tab.scroll_offset,
            &label,
        );
        let mut rgba = hodo_render::render(&list, viewport);
        composite_over_white(&mut rgba);
        self.frame.clear();
        self.frame.reserve(rgba.len() / 4);
        for pixel in rgba.as_chunks::<4>().0 {
            self.frame.push(
                (255u32 << 24)
                    | ((pixel[0] as u32) << 16)
                    | ((pixel[1] as u32) << 8)
                    | pixel[2] as u32,
            );
        }
        &self.frame
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
fn load_resource(url: &str) -> Result<Resource, hodo_net::FetchError> {
    let opts = hodo_net::FetchOptions::default();
    match shell_cache() {
        Some(cache) => {
            let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
            hodo_net::load_with(url, &opts, &mut guard)
        }
        None => hodo_net::load(url),
    }
}

/// Synthetic welcome page for `about:hodo` (no network involved).
pub fn about_hodo_html() -> String {
    let stats = hodo_net::cache_stats();
    format!(
        "<html><head><title>hodo — new tab</title></head><body>\
         <h1>hodo</h1><p>A small browser, built from small crates.</p>\
         <ul>\
         <li>Type a URL above and press Enter to visit it.</li>\
         <li>Ctrl+T opens a new tab; Ctrl+W closes it.</li>\
         <li>Alt+Left / Alt+Right walk your history.</li>\
         <li>Scroll, PageUp / PageDown and Home / End move the page.</li>\
         <li>Cache: {} hits, {} misses, {} bytes, {} evictions.</li>\
         </ul></body></html>",
        stats.hits, stats.misses, stats.bytes, stats.evictions
    )
}

/// Status text in the top strip, e.g. `tab 2/3`.
pub fn tab_counter(active: usize, total: usize) -> String {
    format!("tab {active}/{total}")
}

/// Clamp a scroll offset to the visible page range.
pub fn clamp_scroll_offset(offset: f32, page_height: f32, viewport_h: f32) -> f32 {
    offset.clamp(0.0, (page_height - viewport_h).max(0.0))
}

/// Borrow the rect of a display item; every current variant has one.
fn item_rect(item: &DisplayItem) -> Option<&Rect> {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. } => Some(rect),
    }
}

fn item_rect_mut(item: &mut DisplayItem) -> Option<&mut Rect> {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. } => Some(rect),
    }
}

/// Topmost linked text under `(x, y)`, or `None`. Items later in the list are
/// painted on top, so they win; unlinked text is ignored.
pub fn hit_test_link(items: &[DisplayItem], x: f32, y: f32) -> Option<String> {
    items.iter().rev().find_map(|item| match item {
        DisplayItem::Text {
            rect,
            href: Some(href),
            ..
        } if x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h => {
            Some(href.clone())
        }
        _ => None,
    })
}

/// Bottom edge of the page content, before any scroll offset is applied.
pub fn display_bottom(page: &DisplayList) -> f32 {
    page.items
        .iter()
        .filter_map(item_rect)
        .map(|rect| rect.y + rect.h)
        .fold(0.0, f32::max)
}

/// Synthetic error page. The engine has no inline-style support, so the
/// error text is emitted as plain text (and HTML-escaped to keep the markup).
pub fn error_page_html(err: &str) -> String {
    format!(
        "<html><body><h1>hodo cannot load this page</h1><p>{}</p></body></html>",
        escape_html(err)
    )
}

/// Run the whole pipeline for one URL and return `(title, display_list)`.
///
/// The DOM, stylesheet and style map are local to this function and dropped
/// before it returns — that is the low-RAM contract of the shell.
pub fn load_page(url: &str, viewport: Size) -> (String, DisplayList) {
    let html = if url == ABOUT_HODO_URL {
        about_hodo_html()
    } else {
        match load_resource(url) {
            Ok(Resource::Html(text)) => text,
            Ok(Resource::Other { data, .. }) => String::from_utf8_lossy(&data).into_owned(),
            Err(err) => error_page_html(&err.to_string()),
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

/// Page display list (shifted by the scroll offset) + the fixed UI strips,
/// rendered in a single pass.
pub fn compose_display_list(
    page: &DisplayList,
    viewport: Size,
    url: &str,
    ext_titles: &[String],
    scroll_offset: f32,
    tab_label: &str,
) -> DisplayList {
    let offset = clamp_scroll_offset(scroll_offset, display_bottom(page), viewport.h);
    let mut items: Vec<DisplayItem> = page
        .items
        .iter()
        .cloned()
        .map(|mut item| {
            if let Some(rect) = item_rect_mut(&mut item) {
                rect.y -= offset;
            }
            item
        })
        .collect();

    items.push(DisplayItem::Rect {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            w: viewport.w,
            h: TOP_BAR_H,
        },
        color: Color::WHITE,
        border_color: None,
        radius: 0.0,
    });
    items.push(DisplayItem::Text {
        rect: Rect {
            x: 8.0,
            y: 6.0,
            w: (viewport.w - 16.0).max(1.0),
            h: 28.0,
        },
        text: url.to_string(),
        color: Color::BLACK,
        font_size: 16.0,
        weight: 400,
        href: None,
    });
    items.push(DisplayItem::Text {
        rect: Rect {
            x: (viewport.w - 108.0).max(0.0),
            y: 6.0,
            w: 100.0,
            h: 28.0,
        },
        text: tab_label.to_string(),
        color: Color::BLACK,
        font_size: 14.0,
        weight: 400,
        href: None,
    });

    let bottom_y = viewport.h - BOTTOM_BAR_H;
    items.push(DisplayItem::Rect {
        rect: Rect {
            x: 0.0,
            y: bottom_y,
            w: viewport.w,
            h: BOTTOM_BAR_H,
        },
        color: Color::rgb(211, 211, 211),
        border_color: None,
        radius: 0.0,
    });
    items.push(DisplayItem::Text {
        rect: Rect {
            x: 8.0,
            y: bottom_y + 4.0,
            w: (viewport.w - 16.0).max(1.0),
            h: 20.0,
        },
        text: ext_titles.join(" | "),
        color: Color::BLACK,
        font_size: 14.0,
        weight: 400,
        href: None,
    });

    DisplayList { items }
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

fn collect_styles(dom: &Dom, page_url: &str) -> Stylesheet {
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

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
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
        assert_eq!(clamp_scroll_offset(500.0, 1000.0, 800.0), 200.0);
        assert_eq!(clamp_scroll_offset(1000.0, 500.0, 800.0), 0.0);

        let page = DisplayList {
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
}

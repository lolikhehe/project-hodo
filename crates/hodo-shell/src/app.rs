//! Tab model + rendering pipeline for the hodo shell.
//!
//! Design goal: keep RAM flat regardless of tab count. A `Tab` stores only
//! the URL, title and a flat `DisplayList`; the DOM/stylesheet/style map are
//! created and dropped inside [`load_page`]. Only the active tab ever has a
//! pixel buffer, rebuilt on navigation, tab switch and resize (~33 ms).

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

/// A browser tab. Deliberately holds no DOM, stylesheet or pixels.
pub struct Tab {
    pub url: String,
    pub title: String,
    pub display_list: DisplayList,
}

impl Tab {
    fn blank() -> Self {
        Self {
            url: String::new(),
            title: String::new(),
            display_list: DisplayList::default(),
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
    /// ARGB8888 frame for the *active* tab only: `(255<<24)|(r<<16)|(g<<8)|b`.
    frame: Vec<u32>,
}

impl App {
    /// Create one blank tab, optionally navigating it immediately.
    pub fn new(initial_url: Option<&str>) -> Self {
        let mut app = Self {
            tabs: vec![Tab::blank()],
            active: 0,
            url_bar: String::new(),
            url_focused: true,
            frame: Vec::new(),
        };
        if let Some(url) = initial_url {
            app.navigate(url, DEFAULT_VIEWPORT);
        }
        app
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active]
    }

    /// Open a new blank tab and make it active.
    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::blank());
        self.active = self.tabs.len() - 1;
        self.url_bar.clear();
        self.url_focused = true;
        self.frame.clear();
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
    pub fn navigate(&mut self, url: &str, viewport: Size) -> String {
        let (title, display_list) = load_page(url, viewport);
        let tab = &mut self.tabs[self.active];
        tab.url = url.to_string();
        tab.title = title.clone();
        tab.display_list = display_list;
        self.url_bar = url.to_string();
        self.url_focused = false;
        self.frame.clear();
        title
    }

    /// Re-run the pipeline for the active tab (F5). No-op on a blank tab.
    pub fn reload(&mut self, viewport: Size) -> String {
        let url = self.tabs[self.active].url.clone();
        if url.is_empty() {
            return String::new();
        }
        self.navigate(&url, viewport)
    }

    /// Compose the full frame for the active tab, with the UI overlay on top.
    ///
    /// The returned slice is ARGB8888, row-major, `w * h` entries. Rebuilding
    /// it every redraw is cheap (the page is already a flat display list) and
    /// guarantees a tab switch or resize shows up within one 33 ms tick.
    pub fn frame(&mut self, viewport: Size, ext_titles: &[String]) -> &[u32] {
        let tab = &self.tabs[self.active];
        let list = compose_display_list(&tab.display_list, viewport, &self.url_bar, ext_titles);
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
    let html = match hodo_net::load(url) {
        Ok(Resource::Html(text)) => text,
        Ok(Resource::Other { data, .. }) => String::from_utf8_lossy(&data).into_owned(),
        Err(err) => error_page_html(&err.to_string()),
    };
    let dom = hodo_html::parse_document(&html);
    let title = first_title(&dom);
    let stylesheet = collect_styles(&dom, url);
    let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
    let display_list = hodo_layout::layout(&dom, &styles, viewport);
    (title, display_list)
}

/// Page display list + the two UI strips, rendered in a single pass.
pub fn compose_display_list(
    page: &DisplayList,
    viewport: Size,
    url: &str,
    ext_titles: &[String],
) -> DisplayList {
    let mut items = page.items.clone();

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
                if let Ok(resource) = hodo_net::load(&resolved) {
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
}

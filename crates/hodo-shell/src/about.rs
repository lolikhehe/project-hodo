//! `about:hodo` welcome page and synthetic error pages.

use std::sync::atomic::{AtomicU64, Ordering};

/// Synthetic welcome page used for new tabs and bare launches.
pub const ABOUT_HODO_URL: &str = "about:hodo";

// M5.1/M5.2 repaint counters. Atomics keep them reachable from the free
// `load_page` pipeline without threading state through every caller.
static FULL_REPAINTS: AtomicU64 = AtomicU64::new(0);
static PARTIAL_REPAINTS: AtomicU64 = AtomicU64::new(0);
static SCROLL_BLITS: AtomicU64 = AtomicU64::new(0);
static LAST_DAMAGE_RECTS: AtomicU64 = AtomicU64::new(0);
static LAST_DAMAGE_AREA: AtomicU64 = AtomicU64::new(0);
static PIXELS_PAINTED: AtomicU64 = AtomicU64::new(0);

/// Called by the present path once per redraw.
pub(crate) fn record_repaint(full: bool, scroll: bool, rects: usize, area: u64, pixels: u64) {
    if full {
        FULL_REPAINTS.fetch_add(1, Ordering::Relaxed);
    } else if scroll {
        SCROLL_BLITS.fetch_add(1, Ordering::Relaxed);
    } else {
        PARTIAL_REPAINTS.fetch_add(1, Ordering::Relaxed);
    }
    LAST_DAMAGE_RECTS.store(rects as u64, Ordering::Relaxed);
    LAST_DAMAGE_AREA.store(area, Ordering::Relaxed);
    PIXELS_PAINTED.fetch_add(pixels, Ordering::Relaxed);
}

/// `(full, partial, scroll, last rects, last area, total pixels)`.
pub(crate) fn repaint_stats() -> (u64, u64, u64, u64, u64, u64) {
    (
        FULL_REPAINTS.load(Ordering::Relaxed),
        PARTIAL_REPAINTS.load(Ordering::Relaxed),
        SCROLL_BLITS.load(Ordering::Relaxed),
        LAST_DAMAGE_RECTS.load(Ordering::Relaxed),
        LAST_DAMAGE_AREA.load(Ordering::Relaxed),
        PIXELS_PAINTED.load(Ordering::Relaxed),
    )
}

/// Synthetic welcome page for `about:hodo` (no network involved).
pub fn about_hodo_html() -> String {
    let stats = hodo_net::cache_stats();
    let (full, partial, scroll, rects, area, pixels) = repaint_stats();
    format!(
        "<html><head><title>hodo — new tab</title></head><body>\
         <h1>hodo</h1><p>A small browser, built from small crates.</p>\
         <ul>\
         <li>Type a URL above and press Enter to visit it.</li>\
         <li>Ctrl+T opens a new tab; Ctrl+W closes it.</li>\
         <li>Alt+Left / Alt+Right walk your history.</li>\
         <li>Scroll, PageUp / PageDown and Home / End move the page.</li>\
         <li>Cache: {} hits, {} misses, {} bytes, {} evictions.</li>\
         <li>Repaint: {} full, {} partial, {} scroll blits; last {} damage rects / {} px; {} px painted.</li>\
         <li>Settings loaded from: {}</li>\
         </ul></body></html>",
        stats.hits, stats.misses, stats.bytes, stats.evictions, full, partial, scroll, rects, area, pixels,
        crate::settings::settings_path().display()
    )
}

/// Synthetic error page. The engine has no inline-style support, so the
/// error text is emitted as plain text (and HTML-escaped to keep the markup).
pub fn error_page_html(err: &str) -> String {
    format!(
        "<html><body><h1>hodo cannot load this page</h1><p>{}</p></body></html>",
        escape_html(err)
    )
}

pub(crate) fn escape_html(input: &str) -> String {
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

//! End-to-end pipeline: HTML + CSS -> DOM -> computed styles -> display list -> RGBA frame.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use hodo_css::parse_stylesheet;
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_layout::DisplayItem;
use hodo_net::Resource;
use hodo_types::{Rect, Size};

const VIEWPORT: Size = Size {
    w: 1200.0,
    h: 800.0,
};
const FRAME_BYTES: usize = 1200 * 800 * 4;

/// Background declared by `.card` in style.css.
const CARD_BG: (u8, u8, u8) = (0xee, 0xf4, 0xfb);
/// Border color declared by `.card` in style.css (`border: 2px solid #1d5fa8`).
const CARD_BORDER: (u8, u8, u8) = (0x1d, 0x5f, 0xa8);

fn demo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site")
}

fn text_content(dom: &Dom, id: NodeId) -> String {
    fn collect(dom: &Dom, id: NodeId, out: &mut String) {
        if let Some(text) = dom.text(id) {
            out.push_str(text);
        }
        for &child in dom.children(id) {
            collect(dom, child, out);
        }
    }
    let mut out = String::new();
    collect(dom, id, &mut out);
    out
}

fn is_style(dom: &Dom, id: NodeId) -> bool {
    dom.tag(id)
        .is_some_and(|tag| tag.eq_ignore_ascii_case("style"))
}

/// Return the href of a `<link rel="stylesheet" href="...">`, if this is one.
fn stylesheet_href(dom: &Dom, id: NodeId) -> Option<String> {
    if !dom.tag(id)?.eq_ignore_ascii_case("link") {
        return None;
    }
    let rel = dom.attr(id, "rel")?;
    if !rel
        .split_whitespace()
        .any(|value| value.eq_ignore_ascii_case("stylesheet"))
    {
        return None;
    }
    Some(dom.attr(id, "href")?.to_string())
}

/// Mirror the stylesheet gathering a shell would do: inline <style> text plus
/// linked stylesheets resolved against the document's directory.
fn collect_css(dom: &Dom, html_path: &Path) -> String {
    let mut css = String::new();

    let mut style_ids = Vec::new();
    let mut hrefs = Vec::new();
    dom.walk(|id, _| {
        if is_style(dom, id) {
            style_ids.push(id);
        }
        if let Some(href) = stylesheet_href(dom, id) {
            hrefs.push(href);
        }
    });

    for id in style_ids {
        css.push_str(&text_content(dom, id));
        css.push('\n');
    }

    let base = html_path.parent().unwrap_or_else(|| Path::new("."));
    for href in hrefs {
        let path = base.join(href);
        let resource = hodo_net::load(&path.to_string_lossy())
            .unwrap_or_else(|err| panic!("linked stylesheet {} failed: {err}", path.display()));
        match resource {
            Resource::Other { data, .. } => css.push_str(&String::from_utf8_lossy(&data)),
            Resource::Html(text) => css.push_str(&text),
        }
        css.push('\n');
    }
    css
}

#[test]
fn interned_dom_text_content_reads_pool_backed_strings() {
    // Mirror of text_content with per-node strings stripped by Dom::with_interning().
    let mut dom = Dom::with_interning();
    let root = dom.root();
    let p = dom.create(NodeKind::Element {
        tag: "p".into(),
        attrs: Vec::new(),
    });
    let text = dom.create(NodeKind::Text("hello".into()));
    dom.append(root, p);
    dom.append(p, text);

    assert_eq!(text_content(&dom, p), "hello");
}

/// Run the complete pipeline from the on-disk demo fixture.
fn render_demo() -> (hodo_layout::DisplayList, Vec<u8>) {
    let html_path = demo_dir().join("index.html");
    assert!(
        html_path.exists(),
        "missing fixture: {}",
        html_path.display()
    );

    let html = match hodo_net::load(&html_path.to_string_lossy()).expect("load demo HTML") {
        Resource::Html(text) => text,
        other => panic!("expected Html resource, got {other:?}"),
    };
    let dom = hodo_html::parse_document(&html);
    let css = collect_css(&dom, &html_path);
    assert!(!css.is_empty(), "no CSS collected from the demo document");

    // Parsing both inline and linked sources into one sheet merges their rules.
    let sheets = [parse_stylesheet(&css)];
    let styles = hodo_style::compute_styles(&dom, &sheets);
    let list = hodo_layout::layout(&dom, &styles, VIEWPORT);
    let buffer = hodo_render::render(&list, VIEWPORT);
    (list, buffer)
}

/// Inclusive-safe integer pixel bounds of a rect, clipped to the framebuffer.
fn pixel_bounds(rect: &Rect, width: usize, height: usize) -> (usize, usize, usize, usize) {
    let x0 = rect.x.round().max(0.0) as usize;
    let y0 = rect.y.round().max(0.0) as usize;
    let x1 = (rect.x + rect.w).round().min(width as f32) as usize;
    let y1 = (rect.y + rect.h).round().min(height as f32) as usize;
    (x0, y0, x1.max(x0), y1.max(y0))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[test]
fn pipeline_renders_expected_frame() {
    let (list, buffer) = render_demo();
    assert_eq!(buffer.len(), FRAME_BYTES, "RGBA8 framebuffer size");

    // Colored cards/text must yield more than one opaque color.
    let mut opaque_colors = HashSet::new();
    for pixel in buffer.as_chunks::<4>().0 {
        if pixel[3] == 255 {
            opaque_colors.insert((pixel[0], pixel[1], pixel[2]));
        }
    }
    assert!(
        opaque_colors.len() >= 2,
        "expected at least two distinct opaque colors, got {opaque_colors:?}"
    );

    // A sane amount of the viewport is painted, but not literally all of it:
    // the body has a 24px margin, so the outer edge stays transparent.
    let painted = buffer
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0 && (p[0], p[1], p[2]) != (255, 255, 255))
        .count();
    let ratio = painted as f64 / (1200.0 * 800.0);
    assert!(ratio > 0.05, "painted non-white ratio too low: {ratio:.3}");
    assert!(ratio < 1.0, "painted non-white ratio too high: {ratio:.3}");

    // Text survived HTML -> layout -> display list.
    assert!(
        list.items
            .iter()
            .any(|item| matches!(item, DisplayItem::Text { .. })),
        "display list must contain Text items"
    );

    // A .card background rect must line up with the pixels the rasterizer wrote.
    // Scan the whole rect (not just its center: nested content covers the middle).
    let card = list
        .items
        .iter()
        .find_map(|item| match item {
            DisplayItem::Rect { rect, color, .. }
                if (color.r, color.g, color.b) == CARD_BG && color.a == 255 =>
            {
                Some(*rect)
            }
            _ => None,
        })
        .expect(".card background rect (#eef4fb) missing from display list");
    let (x0, y0, x1, y1) = pixel_bounds(&card, 1200, 800);
    let matched = (x0..x1).any(|x| {
        (y0..y1).any(|y| {
            let i = (y * 1200 + x) * 4;
            [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
                == [CARD_BG.0, CARD_BG.1, CARD_BG.2, 255]
        })
    });
    assert!(matched, "no exact #eef4fb pixel inside card rect {card:?}");
}

#[test]
fn card_border_color_lines_the_inside_edge() {
    let (list, buffer) = render_demo();
    let card = list
        .items
        .iter()
        .find_map(|item| match item {
            DisplayItem::Rect { rect, color, .. }
                if (color.r, color.g, color.b) == CARD_BG && color.a == 255 =>
            {
                Some(*rect)
            }
            _ => None,
        })
        .expect(".card background rect (#eef4fb) missing from display list");
    let (x0, y0, x1, y1) = pixel_bounds(&card, 1200, 800);
    let band = 3usize;
    let matched = (x0..x1).any(|x| {
        (y0..y1).any(|y| {
            let near_edge = x < x0 + band || x + band >= x1 || y < y0 + band || y + band >= y1;
            if !near_edge {
                return false;
            }
            let i = (y * 1200 + x) * 4;
            [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
                == [CARD_BORDER.0, CARD_BORDER.1, CARD_BORDER.2, 255]
        })
    });
    assert!(
        matched,
        "no exact #1d5fa8 border pixel in the 3px band just inside card rect {card:?}"
    );
}

#[test]
fn pipeline_is_deterministic_without_extension_overlay() {
    let (_, first) = render_demo();
    let (_, second) = render_demo();
    assert_eq!(first.len(), second.len());
    assert_eq!(
        fnv1a(&first),
        fnv1a(&second),
        "identical input must produce byte-identical frames"
    );
}

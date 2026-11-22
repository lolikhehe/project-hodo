//! Pixel-level checks against the `examples/position-site` fixture.
//!
//! The fixture's float/clear sections are covered by the float pixel tests
//! below: each float keeps its own edge and the following text wraps between
//! them until both bottoms have passed.
//!
//! An integration test keeps the crate identity the same for layout and
//! render (a unit test would link two copies through the dev-dependency cycle).

use std::sync::OnceLock;

use hodo_dom::{Dom, NodeKind};
use hodo_layout::layout;
use hodo_style::compute_styles;
use hodo_types::{Color, Size};

static BASE_CSS: OnceLock<String> = OnceLock::new();

fn base_css() -> &'static str {
    BASE_CSS
        .get_or_init(|| {
            std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/position-site/style.css"
            ))
            .expect("examples/position-site/style.css must exist")
        })
        .as_str()
}

fn elem(dom: &mut Dom, parent: hodo_dom::NodeId, tag: &str, class: &str) -> hodo_dom::NodeId {
    let attrs = if class.is_empty() {
        Vec::new()
    } else {
        vec![("class".to_string(), class.to_string())]
    };
    let id = dom.create(NodeKind::Element {
        tag: tag.to_string(),
        attrs,
    });
    dom.append(parent, id);
    id
}

fn txt(dom: &mut Dom, parent: hodo_dom::NodeId, s: &str) {
    let id = dom.create(NodeKind::Text(s.to_string()));
    dom.append(parent, id);
}

fn heading(dom: &mut Dom, parent: hodo_dom::NodeId, s: &str) {
    let h2 = elem(dom, parent, "h2", "");
    txt(dom, h2, s);
}

/// DOM shape mirroring examples/position-site/index.html.
fn fixture_dom() -> Dom {
    let mut dom = Dom::new();
    let root = dom.root();
    let html = elem(&mut dom, root, "html", "");
    let body = elem(&mut dom, html, "body", "");

    let h1 = elem(&mut dom, body, "h1", "");
    txt(&mut dom, h1, "Position / float demo (M8)");

    heading(&mut dom, body, "Fixed banner");
    let banner = elem(&mut dom, body, "div", "banner");
    txt(&mut dom, banner, "Fixed banner at top-left");

    heading(&mut dom, body, "Relative offset box");
    let rel = elem(&mut dom, body, "div", "rel-box");
    txt(&mut dom, rel, "Shifted 30px right and 20px down");

    heading(&mut dom, body, "Absolute positioned at parent corners");
    let frame = elem(&mut dom, body, "div", "frame");
    for (class, label) in [
        ("corner tl", "TL"),
        ("corner tr", "TR"),
        ("corner bl", "BL"),
        ("corner br", "BR"),
    ] {
        let corner = elem(&mut dom, frame, "div", class);
        txt(&mut dom, corner, label);
    }

    heading(&mut dom, body, "overflow: hidden with long content");
    let clip = elem(&mut dom, body, "div", "clip");
    txt(
        &mut dom,
        clip,
        "This content is much too long for the box and must be clipped at the \
         edges instead of spilling out into the rest of the page layout.",
    );

    heading(&mut dom, body, "min-width and max-width clamps");
    let minw = elem(&mut dom, body, "div", "minw");
    txt(&mut dom, minw, "min-width 200px");
    let maxw = elem(&mut dom, body, "div", "maxw");
    txt(
        &mut dom,
        maxw,
        "max-width 300px with enough text to reach the clamp and stop growing.",
    );

    heading(&mut dom, body, "Floats with text wrapping beside them");
    let left = elem(&mut dom, body, "div", "floatleft");
    txt(&mut dom, left, "LEFT");
    let right = elem(&mut dom, body, "div", "floatright");
    txt(&mut dom, right, "RIGHT");
    let wrap = elem(&mut dom, body, "p", "wraptext");
    txt(
        &mut dom,
        wrap,
        "Text flows around both floated boxes: it sits to the right of the left box \
         and to the left of the right box, then continues across the full width once \
         both floats end. More words fill the lines so the wrapping is clearly \
         visible in a pixel test.",
    );
    let cleared = elem(&mut dom, body, "div", "cleared");
    txt(&mut dom, cleared, "clear: both");
    dom
}

fn render(extra_css: &str, viewport_w: f32) -> Vec<u8> {
    let css = format!("{}\n{}", base_css(), extra_css);
    let sheet = hodo_css::parse_stylesheet(&css);
    let dom = fixture_dom();
    let styles = compute_styles(&dom, &[sheet]);
    let size = Size {
        w: viewport_w,
        // Tall enough that the narrow-viewport sections stay inside the frame.
        h: 2000.0,
    };
    let list = layout(&dom, &styles, size);
    hodo_render::render(&list, size)
}

type BBox = (usize, usize, usize, usize);

fn bbox(frame: &[u8], w: usize, color: Color) -> Option<BBox> {
    let mut box_: Option<BBox> = None;
    for y in 0..frame.len() / 4 / w {
        for x in 0..w {
            let i = (y * w + x) * 4;
            if [frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]
                == [color.r, color.g, color.b, color.a]
            {
                box_ = Some(match box_ {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                });
            }
        }
    }
    box_
}

fn count(frame: &[u8], color: Color) -> usize {
    frame
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| [px[0], px[1], px[2], px[3]] == [color.r, color.g, color.b, color.a])
        .count()
}

/// Count exact-colour pixels inside an inclusive rectangle.
fn count_region(
    frame: &[u8],
    w: usize,
    color: Color,
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
) -> usize {
    let h = frame.len() / 4 / w;
    let mut n = 0;
    for y in y0..=y1.min(h - 1) {
        for x in x0..=x1.min(w - 1) {
            let i = (y * w + x) * 4;
            if [frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]
                == [color.r, color.g, color.b, color.a]
            {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn relative_box_is_shifted_in_pixels_without_moving_siblings() {
    const W: usize = 500;
    let plain = render(".rel-box { position: static; }", W as f32);
    let shifted = render("", W as f32);
    let c = Color::rgb(0xcf, 0xe6, 0xe0);
    let (px0, py0, _, _) = bbox(&plain, W, c).expect("static rel-box paints");
    let (sx0, sy0, _, _) = bbox(&shifted, W, c).expect("relative rel-box paints");
    assert_eq!(
        (sx0, sy0),
        (px0 + 30, py0 + 20),
        "left/top must move the painted box by exactly (30, 20)"
    );
    // The heading below keeps its position: relative offsets keep space.
    let text = Color::rgb(0x22, 0x22, 0x22);
    assert!(
        count(&plain, text) == count(&shifted, text),
        "shifting one box must not repaint or reflow other text"
    );
}

#[test]
fn absolute_corners_do_not_push_following_sections() {
    const W: usize = 500;
    let normal = render("", W as f32);
    let without = render(".corner { display: none; }", W as f32);
    let bg = Color::rgb(0xee, 0xee, 0xee);
    assert_eq!(
        bbox(&normal, W, bg),
        bbox(&without, W, bg),
        "the clip section must land at the same y with and without absolute corners"
    );
    let corner = Color::rgb(0xf0, 0xcc, 0xcc);
    let (x0, y0, x1, y1) = bbox(&normal, W, corner).expect("corner paints");
    assert!(
        y1 - y0 >= 20 && x1 - x0 >= 50,
        "the 60x30 absolute corner must keep its size: {x0},{y0}..{x1},{y1}"
    );
}

#[test]
fn overflow_hidden_removes_out_of_box_text_pixels() {
    const W: usize = 500;
    // A max-height keeps the box at 60px so the long paragraph really
    // overflows; without it the block grows to fit and nothing is clipped.
    // A unique background colour isolates the box from grey text antialiasing.
    let hidden = render(".clip { max-height: 60px; background: #ff00ff; }", W as f32);
    let visible = render(
        ".clip { max-height: 60px; overflow: visible; background: #ff00ff; }",
        W as f32,
    );
    let text = Color::rgb(0x22, 0x22, 0x22);
    assert!(
        count(&visible, text) > count(&hidden, text),
        "overflow:hidden must discard text pixels that overflow:visible paints"
    );
    let bg = Color::rgb(0xff, 0x00, 0xff);
    let box_ = bbox(&hidden, W, bg).expect("clip background paints");
    assert!(
        box_.3 - box_.1 >= 58 && box_.3 - box_.1 <= 62,
        "the clip box keeps its 60px height: {box_:?}"
    );
}

#[test]
fn min_and_max_width_show_up_in_pixels() {
    // min-width only bites when the available column is narrower; max-width
    // needs a wide column, so each needs its own viewport.
    let min_frame = render("", 230.0);
    let max_frame = render("", 500.0);
    let min_bg = Color::rgb(0xcc, 0xd8, 0xf0);
    let max_bg = Color::rgb(0xf0, 0xe0, 0xcc);
    let (_, _, min_x1, _) = bbox(&min_frame, 230, min_bg).expect("min-width box paints");
    let (max_x0, _, max_x1, _) = bbox(&max_frame, 500, max_bg).expect("max-width box paints");
    assert!(
        (219..=221).contains(&min_x1),
        "min-width 200px from x=20 ends near 220: {min_x1}"
    );
    assert!(
        max_x1 - max_x0 <= 301,
        "max-width 300px must cap the box: {max_x0}..{max_x1}"
    );
}

#[test]
fn floats_render_on_opposite_edges_and_clear_drops_below_both() {
    const W: usize = 500;
    let frame = render("", W as f32);
    let blue = Color::rgb(0x99, 0xcc, 0xff);
    let orange = Color::rgb(0xff, 0xcc, 0x99);
    let purple = Color::rgb(0xdd, 0xd0, 0xe8);
    let (lx0, ly0, lx1, ly1) = bbox(&frame, W, blue).expect("left float paints");
    let (rx0, ry0, rx1, ry1) = bbox(&frame, W, orange).expect("right float paints");
    let (_, cy0, _, _) = bbox(&frame, W, purple).expect("cleared block paints");

    // body margin 20: content runs x=20..480. Left float width 120 at x=20;
    // right float width 120 hugging x=480. Both start on the same y.
    assert_eq!(
        (lx0, lx1 - lx0 + 1),
        (20, 120),
        "left float box: {lx0}..{lx1}"
    );
    assert_eq!(
        (rx1, rx1 - rx0 + 1),
        (479, 120),
        "right float box: {rx0}..{rx1}"
    );
    assert_eq!(ly0, ry0, "floats share a top line: {ly0} vs {ry0}");
    assert_eq!(ly1 - ly0 + 1, 80, "float height from CSS");
    // The left float has margin-bottom 10; clear:both must land below its
    // margin box, not merely below the painted background.
    assert!(
        cy0 >= ly1 + 11 && cy0 > ry1 + 10,
        "cleared block must drop below both float margin boxes: {cy0} vs {ly1}/{ry1}"
    );
}

#[test]
fn wrapping_text_stays_out_of_the_float_gutter() {
    const W: usize = 500;
    // Colour only the paragraph so its glyphs are distinguishable from the
    // "LEFT"/"RIGHT" labels painted inside the float boxes themselves.
    let extra = ".wraptext { color: #00ff00; }";
    let with_floats = render(extra, W as f32);
    let without = render(
        ".floatleft, .floatright { float: none; display: none; } .wraptext { color: #00ff00; }",
        W as f32,
    );
    let blue = Color::rgb(0x99, 0xcc, 0xff);
    let green = Color::rgb(0x00, 0xff, 0x00);
    let (_, ly0, _, ly1) = bbox(&with_floats, W, blue).expect("left float paints");

    // x=20..149 is the left float's margin box; while it is active the
    // paragraph's text may only appear from x=150 onward.
    let beside = count_region(&with_floats, W, green, 20, 149, ly0, ly1);
    let plain = count_region(&without, W, green, 20, 149, ly0, ly1);
    assert_eq!(
        beside, 0,
        "no paragraph glyphs inside the left float's box: {beside}"
    );
    assert!(
        plain > 0,
        "the same band has paragraph text when the floats are removed: {plain}"
    );
    // The gutter between the two floats (150..349) must receive text.
    let gutter = count_region(&with_floats, W, green, 150, 349, ly0, ly1);
    assert!(
        gutter > 0,
        "wrapping text must occupy the middle gutter: {gutter}"
    );
}

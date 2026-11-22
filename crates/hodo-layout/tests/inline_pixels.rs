//! Pixel-level check of per-fragment inline backgrounds through hodo-render.
//! An integration test keeps the crate identity the same for layout and render
//! (a unit test would link two copies through the dev-dependency cycle).

use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_layout::{layout, DisplayItem};
use hodo_style::compute_styles;
use hodo_types::{Color, Size};
use std::path::PathBuf;

#[test]
fn fragment_backgrounds_rasterize_in_two_bands() {
    let mut dom = Dom::new();
    let root = dom.root();
    let div = dom.create(NodeKind::Element {
        tag: "div".into(),
        attrs: Vec::new(),
    });
    dom.append(root, div);
    let span = dom.create(NodeKind::Element {
        tag: "span".into(),
        attrs: Vec::new(),
    });
    dom.append(div, span);
    let t = dom.create(NodeKind::Text("abcdefghijklmnopqrst".into()));
    dom.append(span, t);

    let rule = |tag: &str, props: &[(&str, &str)]| StyleRule {
        selector: Selector {
            tag: Some(tag.into()),
            id: None,
            classes: Vec::new(),
            universal: false,
        },
        declarations: props
            .iter()
            .map(|&(p, v)| Declaration {
                property: p.into(),
                value: v.into(),
            })
            .collect(),
    };
    let styles = compute_styles(
        &dom,
        &[Stylesheet {
            rules: vec![
                rule("div", &[("font-size", "10px")]),
                rule("span", &[("background", "#0000ff")]),
            ],
        }],
    );

    let list = layout(&dom, &styles, Size { w: 60.0, h: 40.0 });
    let frame = hodo_render::render(&list, Size { w: 60.0, h: 40.0 });
    let blue = Color::rgb(0, 0, 255);
    let count_band = |y0: usize, y1: usize| -> usize {
        (y0..y1)
            .flat_map(|y| (0..60usize).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * 60 + x) * 4;
                [frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]
                    == [blue.r, blue.g, blue.b, 255]
            })
            .count()
    };
    let first = count_band(0, 12);
    let second = count_band(12, 24);
    assert!(
        first > 100 && second > 100,
        "each line fragment must paint its own background band: {first}, {second}"
    );
}

// ---- M7.2-props: examples/inline-site ------------------------------------

/// Parse the fixture's real stylesheet and drive a small DOM with its classes.
/// This keeps the integration check tied to `examples/inline-site/style.css`
/// without adding an html parser dev-dependency to hodo-layout.
fn fixture_styles(dom: &Dom) -> hodo_style::StyleMap {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("inline-site");
    let css = std::fs::read_to_string(dir.join("style.css"))
        .expect("examples/inline-site/style.css must exist");
    let sheet = hodo_css::parse_stylesheet(&css);
    compute_styles(dom, &[sheet])
}

fn elem(dom: &mut Dom, parent: NodeId, tag: &str, class: Option<&str>) -> NodeId {
    let attrs = class
        .into_iter()
        .map(|c| ("class".to_string(), c.to_string()))
        .collect();
    let id = dom.create(NodeKind::Element {
        tag: tag.to_string(),
        attrs,
    });
    dom.append(parent, id);
    id
}

fn text(dom: &mut Dom, parent: NodeId, value: &str) {
    let id = dom.create(NodeKind::Text(value.to_string()));
    dom.append(parent, id);
}

fn text_rects(list: &hodo_layout::DisplayList) -> Vec<(hodo_types::Rect, String)> {
    list.items
        .iter()
        .filter_map(|it| match it {
            DisplayItem::Text { rect, text, .. } => Some((*rect, text.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn inline_site_indent_and_alignment_from_fixture_css() {
    let mut dom = Dom::new();
    let root = dom.root();
    let indent = elem(&mut dom, root, "p", Some("indent"));
    text(
        &mut dom,
        indent,
        "The first line of this paragraph starts forty pixels in. Following lines return.",
    );
    let right = elem(&mut dom, root, "p", Some("right"));
    text(&mut dom, right, "Right aligned line.");

    let styles = fixture_styles(&dom);
    let list = layout(&dom, &styles, Size { w: 200.0, h: 600.0 });
    let tr = text_rects(&list);

    let right_idx = tr
        .iter()
        .position(|(_, t)| t == "Right aligned line.")
        .expect("right paragraph text");
    let indented = &tr[..right_idx];
    assert!(indented.len() >= 2, "indent paragraph must wrap: {tr:?}");
    assert!(
        (indented[0].0.x - 40.0).abs() < 1.0,
        "fixture text-indent:40px on first line: {:?}",
        indented[0]
    );
    assert!(
        indented[1..].iter().all(|(rect, _)| rect.x.abs() < 1.0),
        "later lines back at the left edge: {indented:?}"
    );

    let r = &tr[right_idx];
    // No body element in this DOM, so the content right edge is the viewport.
    assert!(
        (r.0.x + r.0.w - 200.0).abs() < 1.0,
        "fixture text-align:right must hug the content edge: {:?}",
        r.0
    );
}

#[test]
fn inline_site_pre_and_inline_block_from_fixture_css() {
    let mut dom = Dom::new();
    let root = dom.root();
    let pre = elem(&mut dom, root, "div", Some("pre"));
    text(&mut dom, pre, "+---+---+\n| A | B |\n+---+---+");
    let p = elem(&mut dom, root, "p", None);
    let ib = elem(&mut dom, p, "span", Some("ib"));
    text(&mut dom, ib, "Box A");

    let styles = fixture_styles(&dom);
    let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
    let tr = text_rects(&list);
    assert!(
        tr.iter().any(|(_, t)| t == "| A | B |"),
        "white-space:pre preserves inner spaces: {tr:?}"
    );

    let rects: Vec<_> = list
        .items
        .iter()
        .filter_map(|it| match it {
            DisplayItem::Rect { rect, .. } => Some(*rect),
            _ => None,
        })
        .collect();
    assert!(
        rects.iter().any(|r| (r.w - 80.0).abs() < 1.0),
        "fixture .ib width:80px emits one atomic 80px slot: {rects:?}"
    );
}

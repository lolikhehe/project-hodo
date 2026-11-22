//! Pixel-level table check through hodo-render. Mirrors the small grid in
//! examples/table-site/index.html (caption, border-spacing, separate borders).
//! An integration test keeps crate identity the same for layout and render
//! (a unit test would link two copies through the dev-dependency cycle).

use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
use hodo_dom::{Dom, NodeKind};
use hodo_layout::{layout, DisplayItem};
use hodo_style::compute_styles;
use hodo_types::{Color, Size};

fn elem(dom: &mut Dom, parent: hodo_dom::NodeId, tag: &str) -> hodo_dom::NodeId {
    let id = dom.create(NodeKind::Element {
        tag: tag.into(),
        attrs: Vec::new(),
    });
    dom.append(parent, id);
    id
}

fn text(dom: &mut Dom, parent: hodo_dom::NodeId, value: &str) {
    let id = dom.create(NodeKind::Text(value.into()));
    dom.append(parent, id);
}

fn rule(tag: &str, props: &[(&str, &str)]) -> StyleRule {
    StyleRule {
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
    }
}

fn pixel(buf: &[u8], w: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * w + x) * 4;
    [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
}

#[test]
fn table_grid_rasterizes_with_visible_spacing_gaps() {
    let mut dom = Dom::new();
    let root = dom.root();
    let table = elem(&mut dom, root, "table");
    let cap = elem(&mut dom, table, "caption");
    text(&mut dom, cap, "Demo table");
    for _ in 0..2 {
        let tr = elem(&mut dom, table, "tr");
        for _ in 0..2 {
            let td = elem(&mut dom, tr, "td");
            text(&mut dom, td, "cell");
        }
    }
    let styles = compute_styles(
        &dom,
        &[Stylesheet {
            rules: vec![
                rule(
                    "table",
                    &[("width", "200px"), ("border-spacing", "8px 8px")],
                ),
                rule("caption", &[("font-size", "10px")]),
                rule("td", &[("background", "#0000ff"), ("font-size", "10px")]),
            ],
        }],
    );
    let list = layout(&dom, &styles, Size { w: 200.0, h: 120.0 });
    let mut cell_bg: Vec<_> = list
        .items
        .iter()
        .filter_map(|it| match it {
            DisplayItem::Rect { rect, color, .. } if *color == Color::rgb(0, 0, 255) => Some(*rect),
            _ => None,
        })
        .collect();
    assert_eq!(
        cell_bg.len(),
        4,
        "one blue background per cell: {cell_bg:?}"
    );
    cell_bg.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });
    let first = cell_bg[0];
    let second = cell_bg[1];
    let gap = second.x - (first.x + first.w);
    assert!((gap - 8.0).abs() < 0.01, "spacing gap must be 8px: {gap}");

    let buf = hodo_render::render(&list, Size { w: 200.0, h: 120.0 });
    let blue = [0, 0, 255, 255];
    let cx = (first.x + first.w * 0.5) as usize;
    let cy = (first.y + first.h * 0.5) as usize;
    assert_eq!(pixel(&buf, 200, cx, cy), blue, "cell interior must be blue");
    let gx = (first.x + first.w + gap * 0.5) as usize;
    assert_eq!(
        pixel(&buf, 200, gx, cy),
        [0, 0, 0, 0],
        "gap stays transparent"
    );

    // The caption is laid out above the grid: its glyphs must land in the band
    // before the first cell background.
    let has_caption_ink = (0..first.y as usize).any(|y| {
        (0..200usize).any(|x| {
            let i = (y * 200 + x) * 4;
            buf[i + 3] > 0 && [buf[i], buf[i + 1], buf[i + 2]] != [0, 0, 255]
        })
    });
    assert!(has_caption_ink, "caption must rasterize above the cells");
}

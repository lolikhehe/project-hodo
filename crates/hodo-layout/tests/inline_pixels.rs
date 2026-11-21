//! Pixel-level check of per-fragment inline backgrounds through hodo-render.
//! An integration test keeps the crate identity the same for layout and render
//! (a unit test would link two copies through the dev-dependency cycle).

use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
use hodo_dom::{Dom, NodeKind};
use hodo_layout::layout;
use hodo_style::compute_styles;
use hodo_types::{Color, Size};

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

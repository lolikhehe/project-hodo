//! Pixel-level check of flexbox placement through hodo-render.
//! An integration test keeps the crate identity the same for layout and render
//! (a unit test would link two copies through the dev-dependency cycle).

use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
use hodo_dom::{Dom, NodeKind};
use hodo_layout::layout;
use hodo_style::compute_styles;
use hodo_types::{Color, Size};

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

#[test]
fn space_between_boxes_rasterize_at_three_distinct_bands() {
    let mut dom = Dom::new();
    let root = dom.root();
    let flex = dom.create(NodeKind::Element {
        tag: "flex".into(),
        attrs: Vec::new(),
    });
    dom.append(root, flex);
    for (cls, label) in [("red", "A"), ("green", "B"), ("blue", "C")] {
        let box_node = dom.create(NodeKind::Element {
            tag: cls.into(),
            attrs: Vec::new(),
        });
        dom.append(flex, box_node);
        let t = dom.create(NodeKind::Text(label.into()));
        dom.append(box_node, t);
    }

    let styles = compute_styles(
        &dom,
        &[Stylesheet {
            rules: vec![
                rule(
                    "flex",
                    &[("display", "flex"), ("justify-content", "space-between")],
                ),
                rule(
                    "red",
                    &[
                        ("display", "block"),
                        ("width", "40px"),
                        ("height", "40px"),
                        ("background", "#ff0000"),
                    ],
                ),
                rule(
                    "green",
                    &[
                        ("display", "block"),
                        ("width", "40px"),
                        ("height", "40px"),
                        ("background", "#00ff00"),
                    ],
                ),
                rule(
                    "blue",
                    &[
                        ("display", "block"),
                        ("width", "40px"),
                        ("height", "40px"),
                        ("background", "#0000ff"),
                    ],
                ),
            ],
        }],
    );

    let size = Size { w: 300.0, h: 40.0 };
    let list = layout(&dom, &styles, size);
    let frame = hodo_render::render(&list, size);
    let pixel = |x: usize, y: usize| -> [u8; 4] {
        let i = (y * 300 + x) * 4;
        [frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]
    };

    let red = Color::rgb(255, 0, 0);
    let green = Color::rgb(0, 255, 0);
    let blue = Color::rgb(0, 0, 255);
    // space-between of three 40px boxes in 300px: x = 0, 130, 260.
    assert_eq!(pixel(20, 20), [red.r, red.g, red.b, 255]);
    assert_eq!(pixel(150, 20), [green.r, green.g, green.b, 255]);
    assert_eq!(pixel(280, 20), [blue.r, blue.g, blue.b, 255]);
    // The 90px gaps stay transparent.
    assert_eq!(pixel(100, 20), [0, 0, 0, 0]);
    assert_eq!(pixel(230, 20), [0, 0, 0, 0]);
}

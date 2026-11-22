//! Pixel-level check that grid columns land at the expected x positions.
//! An integration test keeps the crate identity the same for layout and
//! render (a unit test would link two copies through the dev-dependency cycle).

use hodo_css::parse_stylesheet;
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_layout::layout;
use hodo_style::compute_styles;
use hodo_types::Size;

fn elem(dom: &mut Dom, parent: NodeId, tag: &str, class: &str, style: &str) -> NodeId {
    let mut attrs = Vec::new();
    if !class.is_empty() {
        attrs.push(("class".to_string(), class.to_string()));
    }
    attrs.push(("style".to_string(), style.to_string()));
    let id = dom.create(NodeKind::Element {
        tag: tag.into(),
        attrs,
    });
    dom.append(parent, id);
    id
}

#[test]
fn grid_fr_columns_rasterize_in_three_bands() {
    let mut dom = Dom::new();
    let root = dom.root();
    let body = elem(&mut dom, root, "body", "", "");
    let grid = elem(
        &mut dom,
        body,
        "div",
        "grid-fr",
        "display: grid; grid-template-columns: 100px 1fr 2fr",
    );
    // Backgrounds from examples/grid-site/index.html, in DOM order.
    for bg in ["#8fb3e0", "#e0a98f", "#a9e08f"] {
        let cell = elem(&mut dom, grid, "div", "cell", &format!("background: {bg}"));
        let t = dom.create(NodeKind::Text("x".into()));
        dom.append(cell, t);
    }

    let sheet = parse_stylesheet(include_str!("../../../examples/grid-site/style.css"));
    let styles = compute_styles(&dom, &[sheet]);
    let size = Size { w: 340.0, h: 120.0 };
    let list = layout(&dom, &styles, size);
    let frame = hodo_render::render(&list, size);

    let px = |x: usize, y: usize| -> [u8; 3] {
        let i = (y * 340 + x) * 4;
        [frame[i], frame[i + 1], frame[i + 2]]
    };
    // Body margin 20px leaves 300px: tracks are 100px, ~66px and ~133px.
    assert_eq!(px(110, 25), [143, 179, 224], "first column background");
    assert_eq!(px(175, 25), [224, 169, 143], "second column background");
    assert_eq!(px(280, 25), [169, 224, 143], "third column background");
}

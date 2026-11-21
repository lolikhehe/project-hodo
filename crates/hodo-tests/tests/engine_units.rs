//! Cross-crate seam tests: each assertion spans two or more engine crates.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_layout::DisplayItem;
use hodo_types::{Color, Size};

fn demo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo-site")
}

fn find_tag(dom: &Dom, tag: &str) -> NodeId {
    let mut found = None;
    dom.walk(|id, node| {
        if found.is_none()
            && matches!(&node.kind, NodeKind::Element { tag: name, .. } if name == tag)
        {
            found = Some(id);
        }
    });
    found.unwrap_or_else(|| panic!("element <{tag}> not found"))
}

#[test]
fn html_to_dom_builds_implied_structure_and_keeps_attributes() {
    let dom = hodo_html::parse_document(r#"<p id="greet" class="lead note">hello</p>"#);

    let root_children = dom.children(dom.root());
    assert_eq!(root_children.len(), 1);
    let html = root_children[0];
    assert_eq!(dom.tag(html), Some("html"));

    let html_children: Vec<&str> = dom
        .children(html)
        .iter()
        .filter_map(|&id| dom.tag(id))
        .collect();
    assert_eq!(html_children, ["head", "body"]);

    let p = find_tag(&dom, "p");
    assert_eq!(dom.parent(p), Some(dom.children(html)[1]));
    assert_eq!(dom.attr(p, "id"), Some("greet"));
    assert_eq!(dom.attr(p, "class"), Some("lead note"));
    let text = dom.children(p)[0];
    assert_eq!(dom.text(text), Some("hello"));
}

#[test]
fn css_color_inherits_from_body_to_paragraph() {
    let dom = hodo_html::parse_document("<html><body><p>child</p></body></html>");
    let sheets = [hodo_css::parse_stylesheet("body { color: red }")];
    let styles = hodo_style::compute_styles(&dom, &sheets);

    let p = find_tag(&dom, "p");
    assert_eq!(
        styles.get(p).expect("computed style for <p>").color,
        Color::rgb(255, 0, 0),
        "color must inherit from body to its paragraph child"
    );
}

#[test]
fn display_none_produces_zero_items_for_its_subtree() {
    let size = Size { w: 800.0, h: 600.0 };
    let sheets = [hodo_css::parse_stylesheet(".hidden { display: none }")];

    let dom =
        hodo_html::parse_document(r#"<div>visible</div><div class="hidden"><p>secret</p></div>"#);
    let styles = hodo_style::compute_styles(&dom, &sheets);
    let list = hodo_layout::layout(&dom, &styles, size);
    let leaked = list.items.iter().any(|item| match item {
        DisplayItem::Text { text, .. } => text.contains("secret"),
        _ => false,
    });
    assert!(!leaked, "hidden subtree text leaked into the display list");

    // The visible-only document should have exactly the same item count.
    let plain_dom = hodo_html::parse_document("<div>visible</div>");
    let plain_styles = hodo_style::compute_styles(&plain_dom, &sheets);
    let plain_list = hodo_layout::layout(&plain_dom, &plain_styles, size);
    assert_eq!(
        list.items.len(),
        plain_list.items.len(),
        "the display:none subtree must contribute zero display items"
    );
}

#[test]
fn file_url_loads_the_demo_html() {
    let path = demo_dir().join("index.html");
    assert!(path.exists(), "missing fixture: {}", path.display());
    let url = format!("file://{}", path.to_string_lossy());
    match hodo_net::load(&url).expect("load via file:// URL") {
        hodo_net::Resource::Html(html) => {
            assert!(html.contains("hodo"), "demo HTML should mention hodo")
        }
        other => panic!("expected Html resource, got {other:?}"),
    }
}

#[test]
fn oversized_local_file_is_rejected_as_too_large() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "hodo-e2e-too-large-{}-{nanos}.bin",
        std::process::id()
    ));
    fs::write(&path, vec![0u8; hodo_net::MAX_BYTES + 1]).expect("write oversized temp file");

    let result = hodo_net::load(&path.to_string_lossy());
    let _ = fs::remove_file(&path);

    assert!(
        matches!(result, Err(hodo_net::FetchError::TooLarge)),
        "expected TooLarge for a 2 MiB + 1 byte file, got {result:?}"
    );
}

#[test]
fn stylesheet_parsing_is_deterministic_across_cold_calls() {
    let css = fs::read_to_string(demo_dir().join("style.css")).expect("read demo style.css");
    let first = hodo_css::parse_stylesheet(&css);
    let second = hodo_css::parse_stylesheet(&css);
    assert!(
        !first.rules.is_empty(),
        "demo stylesheet must contain rules"
    );
    assert_eq!(
        first.rules.len(),
        second.rules.len(),
        "two independent parses must yield the same rule count"
    );
}

//! Accessibility page tree for hodo: a pure `Dom -> accesskit::TreeUpdate` map.
//!
//! No winit, no shell, no window handles. The host owns the AccessKit
//! platform adapter and pushes the returned [`accesskit::TreeUpdate`] to it.
//!
//! # Shape of the tree
//!
//! The DOM document root becomes the tree's `Window` node. Its children are
//! exactly the document's element children; text nodes are not emitted as
//! accessibility nodes, they only supply the accessible name of their
//! nearest element ancestor. Every reachable element becomes one node, so
//! a full snapshot is emitted on every call.
//!
//! Names come from the first non-empty text descendant (bottom-up, so this
//! stays linear even on a 512-deep chain) and are capped at [`MAX_NAME_LEN`].
//! AccessKit 0.25 dropped the old `Node::name` field; the accessible name is
//! now `Node::label`, which is what this crate sets.
//!
//! # Deferred to wave 2
//!
//! * **States** (`hidden` from `display:none`/`visibility`/`aria-hidden`,
//!   `disabled`, `checked`, `expanded`, `invalid`, ...). This crate sees only
//!   the [`Dom`] tree — computed style and form-control state live in
//!   `hodo-style`/`hodo-layout` and are not passed in. `NodeClassSet` no longer
//!   exists in AccessKit, so there is nothing to memoize here yet.
//! * **Actions** (click, focus, select, ...) and all shell/OS event wiring.
//!   That is X3 work in the shell crate; this crate stays side-effect free.

use std::collections::{HashMap, HashSet};

use accesskit::{Node, NodeId as AkNodeId, Role, TreeId, TreeInfo, TreeUpdate};
use hodo_dom::{Dom, NodeId, NodeKind};

/// Byte cap for an accessible name copied out of DOM text content.
pub const MAX_NAME_LEN: usize = 256;

/// Map a DOM handle to an AccessKit id. The whole 32-bit handle (slot +
/// generation) is used, so a recycled arena slot never aliases a live node.
#[inline]
fn ak_id(id: NodeId) -> AkNodeId {
    AkNodeId::from(u64::from(id.0))
}

#[inline]
fn is_text(dom: &Dom, id: NodeId) -> bool {
    matches!(dom.get(id).kind, NodeKind::Text(_))
}

/// `h1`..`h6` -> heading level. AccessKit levels are zero-based (ARIA's
/// `aria-level` is one-based), so `h1` is level 0.
fn heading_level(tag: &str) -> Option<usize> {
    let bytes = tag.as_bytes();
    if bytes.len() == 2 && bytes[0] == b'h' {
        if let b'1'..=b'6' = bytes[1] {
            return Some((bytes[1] - b'1') as usize);
        }
    }
    None
}

fn role_for(tag: Option<&str>, dom: &Dom, id: NodeId) -> Role {
    let Some(tag) = tag else {
        return Role::Window; // only the document root has no tag
    };
    match tag {
        "a" | "link" => Role::Link,
        "button" => Role::Button,
        "input" => match dom
            .attr(id, "type")
            .unwrap_or_default()
            .to_ascii_lowercase()
        {
            s if s == "checkbox" => Role::CheckBox,
            s if s == "radio" => Role::RadioButton,
            // Missing/empty/"text"/"password"/other input types default here,
            // per the frozen brief. `password` masking is a wave-2 concern.
            _ => Role::TextInput,
        },
        "textarea" => Role::MultilineTextInput,
        "select" => Role::ComboBox,
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Role::Heading,
        "ul" | "ol" => Role::List,
        "li" => Role::ListItem,
        "table" => Role::Table,
        "tr" => Role::Row,
        "td" | "th" => Role::Cell,
        "img" => Role::Image,
        // AccessKit has no single `Landmark` role; these are the four ARIA
        // landmark roles browsers map nav/main/header/footer onto.
        "nav" => Role::Navigation,
        "main" => Role::Main,
        "header" => Role::Banner,
        "footer" => Role::ContentInfo,
        // Any unlisted tag (div, span, p, ...) is a filterable container.
        _ => Role::GenericContainer,
    }
}

/// First non-empty text descendant, truncated on a UTF-8 boundary.
fn first_text_name(dom: &Dom, id: NodeId, names: &HashMap<u32, String>) -> Option<String> {
    for &child in dom.children(id) {
        if let Some(text) = dom.text(child) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(truncate(trimmed));
            }
        } else if let Some(name) = names.get(&child.0) {
            if !name.is_empty() {
                return Some(name.clone());
            }
        }
    }
    None
}

fn truncate(s: &str) -> String {
    let mut end = MAX_NAME_LEN.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}

/// Build one complete, self-contained accessibility tree from the DOM.
///
/// `focused` is a DOM handle; if it is invalid, detached, a text node or an
/// orphan, focus falls back to the window (as AccessKit requires when no
/// specific node has focus).
pub fn build_tree_update(dom: &Dom, focused: NodeId) -> TreeUpdate {
    let root = dom.root();
    let window_id = ak_id(root);

    // Names are computed bottom-up: a parent's first text descendant is either
    // a direct text child or the already-computed name of an element child.
    // One visit per node, so no recursion and no O(n^2) descendant scans.
    let mut names: HashMap<u32, String> = HashMap::new();
    let mut emitted: HashSet<u32> = HashSet::new();
    let mut nodes: Vec<(AkNodeId, Node)> = Vec::with_capacity(dom.len());

    enum Step {
        Enter(NodeId),
        Exit(NodeId),
    }
    let mut stack = vec![Step::Enter(root)];
    while let Some(step) = stack.pop() {
        match step {
            Step::Enter(id) => {
                if is_text(dom, id) {
                    continue;
                }
                stack.push(Step::Exit(id));
                // Reverse push order keeps DOM order in the output list.
                for &child in dom.children(id).iter().rev() {
                    if !is_text(dom, child) {
                        stack.push(Step::Enter(child));
                    }
                }
            }
            Step::Exit(id) => {
                let tag = dom.tag(id).map(str::to_ascii_lowercase);
                let mut node = Node::new(role_for(tag.as_deref(), dom, id));

                if let Some(level) = tag.as_deref().and_then(heading_level) {
                    node.set_level(level);
                }

                let children: Vec<AkNodeId> = dom
                    .children(id)
                    .iter()
                    .copied()
                    .filter(|&child| !is_text(dom, child))
                    .map(ak_id)
                    .collect();
                node.set_children(children);

                // An image's accessible name comes from `alt`, not from any
                // text descendants (which are not rendered for <img> anyway).
                let name = if tag.as_deref() == Some("img") {
                    dom.attr(id, "alt")
                        .map(str::trim)
                        .filter(|alt| !alt.is_empty())
                        .map(truncate)
                } else {
                    first_text_name(dom, id, &names)
                };
                if let Some(name) = name {
                    node.set_label(&name);
                    names.insert(id.0, name);
                }

                emitted.insert(id.0);
                nodes.push((ak_id(id), node));
            }
        }
    }

    let focus = if emitted.contains(&focused.0) {
        ak_id(focused)
    } else {
        window_id
    };

    TreeUpdate {
        nodes,
        tree: Some(TreeInfo::new(window_id)),
        tree_id: TreeId::ROOT,
        focus,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn elem(dom: &mut Dom, tag: &str) -> NodeId {
        elem_with(dom, tag, &[])
    }

    fn elem_with(dom: &mut Dom, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: attrs
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        })
    }

    fn txt(dom: &mut Dom, s: &str) -> NodeId {
        dom.create(NodeKind::Text(s.into()))
    }

    fn attach(dom: &mut Dom, parent: NodeId, tag: &str) -> NodeId {
        let id = elem(dom, tag);
        dom.append(parent, id);
        id
    }

    fn attach_text(dom: &mut Dom, parent: NodeId, s: &str) {
        let id = txt(dom, s);
        dom.append(parent, id);
    }

    /// TreeUpdate has no serde and Node's fields are private; snapshot the
    /// public (id, role, accessible name) triplet instead.
    type Snap = Vec<(u64, Role, Option<String>)>;

    fn snap(upd: &TreeUpdate) -> Snap {
        let mut out: Snap = upd
            .nodes
            .iter()
            .map(|(id, node)| (id.0, node.role(), node.label().map(str::to_owned)))
            .collect();
        out.sort_by_key(|triplet| triplet.0);
        out
    }

    fn node_with(upd: &TreeUpdate, id: AkNodeId) -> &Node {
        &upd.nodes
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .expect("node must be present")
            .1
    }

    struct Sample {
        dom: Dom,
        h1: NodeId,
        link: NodeId,
        ul: NodeId,
        table: NodeId,
    }

    fn sample_dom() -> Sample {
        let mut dom = Dom::new();

        let h1 = elem(&mut dom, "h1");
        dom.append(dom.root(), h1);
        attach_text(&mut dom, h1, "Title");

        let link = elem_with(&mut dom, "a", &[("href", "/x")]);
        dom.append(dom.root(), link);
        attach_text(&mut dom, link, "Link text");

        let ul = elem(&mut dom, "ul");
        dom.append(dom.root(), ul);
        for word in ["one", "two"] {
            let li = attach(&mut dom, ul, "li");
            attach_text(&mut dom, li, word);
        }

        let table = elem(&mut dom, "table");
        dom.append(dom.root(), table);
        let tr = attach(&mut dom, table, "tr");
        for (tag, word) in [("th", "H"), ("td", "D")] {
            let cell = attach(&mut dom, tr, tag);
            attach_text(&mut dom, cell, word);
        }

        Sample {
            dom,
            h1,
            link,
            ul,
            table,
        }
    }

    #[test]
    fn full_tree_snapshot() {
        let s = sample_dom();
        let upd = build_tree_update(&s.dom, s.link);

        assert_eq!(
            snap(&upd),
            vec![
                (0, Role::Window, Some("Title".into())),
                (1, Role::Heading, Some("Title".into())),
                (3, Role::Link, Some("Link text".into())),
                (5, Role::List, Some("one".into())),
                (6, Role::ListItem, Some("one".into())),
                (8, Role::ListItem, Some("two".into())),
                (10, Role::Table, Some("H".into())),
                (11, Role::Row, Some("H".into())),
                (12, Role::Cell, Some("H".into())),
                (14, Role::Cell, Some("D".into())),
            ]
        );
        assert_eq!(upd.tree.as_ref().unwrap().root, ak_id(s.dom.root()));
        assert_eq!(upd.tree_id, TreeId::ROOT);
        assert_eq!(upd.focus, ak_id(s.link));
        assert_eq!(node_with(&upd, ak_id(s.h1)).level(), Some(0));
        assert_eq!(
            node_with(&upd, ak_id(s.dom.root())).children(),
            &[ak_id(s.h1), ak_id(s.link), ak_id(s.ul), ak_id(s.table),]
        );
    }

    #[test]
    fn focus_changes_only_the_focus_field() {
        let s = sample_dom();
        let first = build_tree_update(&s.dom, s.h1);
        let second = build_tree_update(&s.dom, s.link);

        assert_eq!(snap(&first), snap(&second));
        assert_eq!(first.tree, second.tree);
        assert_eq!(first.tree_id, second.tree_id);
        assert_ne!(first.focus, second.focus);
        assert_eq!(first.focus, ak_id(s.h1));
        assert_eq!(second.focus, ak_id(s.link));
    }

    #[test]
    fn bad_focus_falls_back_to_window() {
        let mut dom = Dom::new();
        let p = elem(&mut dom, "p");
        dom.append(dom.root(), p);
        let text = txt(&mut dom, "hello");
        dom.append(p, text);
        let orphan = elem(&mut dom, "div"); // never appended

        let via_text = build_tree_update(&dom, text);
        assert_eq!(via_text.focus, ak_id(dom.root()));
        let via_orphan = build_tree_update(&dom, orphan);
        assert_eq!(via_orphan.focus, ak_id(dom.root()));
        let via_forged = build_tree_update(&dom, NodeId(u32::MAX));
        assert_eq!(via_forged.focus, ak_id(dom.root()));
        assert_eq!(snap(&via_text), snap(&via_orphan));
    }

    #[test]
    fn detached_subtree_is_excluded() {
        let mut dom = Dom::new();
        let keep = elem(&mut dom, "div");
        dom.append(dom.root(), keep);
        let gone = elem(&mut dom, "section");
        dom.append(dom.root(), gone);
        let child = elem(&mut dom, "a");
        dom.append(gone, child);
        attach_text(&mut dom, child, "hidden");
        dom.detach_subtree(gone);

        let upd = build_tree_update(&dom, gone);
        let ids: Vec<u64> = upd.nodes.iter().map(|(id, _)| id.0).collect();
        assert!(!ids.contains(&u64::from(gone.0)));
        assert!(!ids.contains(&u64::from(child.0)));
        assert_eq!(upd.focus, ak_id(dom.root()));
        assert_eq!(
            node_with(&upd, ak_id(dom.root())).children(),
            &[ak_id(keep)]
        );
    }

    #[test]
    fn deep_chain_512_is_iterative_and_complete() {
        let mut dom = Dom::new();
        let mut parent = dom.root();
        for _ in 0..512 {
            let node = elem(&mut dom, "div");
            dom.append(parent, node);
            parent = node;
        }

        let upd = build_tree_update(&dom, parent);
        assert_eq!(upd.nodes.len(), 513);
        assert_eq!(upd.focus, ak_id(parent));
    }

    #[test]
    fn hundred_k_nodes_under_one_second() {
        let mut dom = Dom::new();
        for _ in 0..1_000 {
            let group = elem(&mut dom, "div");
            dom.append(dom.root(), group);
            for _ in 0..100 {
                attach(&mut dom, group, "span");
            }
        }
        assert_eq!(dom.len(), 1 + 1_000 + 100_000);

        let started = Instant::now();
        let upd = build_tree_update(&dom, dom.root());
        let elapsed = started.elapsed();

        assert_eq!(upd.nodes.len(), dom.len());
        assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
    }

    #[test]
    fn img_name_comes_from_alt_only() {
        let mut dom = Dom::new();
        let with_alt = elem_with(&mut dom, "img", &[("alt", "A cat")]);
        dom.append(dom.root(), with_alt);
        attach_text(&mut dom, with_alt, "ignored text child");

        let without_alt = elem(&mut dom, "img");
        dom.append(dom.root(), without_alt);
        attach_text(&mut dom, without_alt, "also ignored");

        let upd = build_tree_update(&dom, dom.root());
        assert_eq!(
            node_with(&upd, ak_id(with_alt)).label(),
            Some("A cat"),
            "img with alt must use alt as its accessible name"
        );
        assert_eq!(
            node_with(&upd, ak_id(without_alt)).label(),
            None,
            "img without alt must have no accessible name"
        );
    }

    #[test]
    fn name_is_capped_on_a_utf8_boundary() {
        let mut dom = Dom::new();
        let button = elem(&mut dom, "button");
        dom.append(dom.root(), button);
        attach_text(&mut dom, button, &"x".repeat(1_000));
        let ascii = build_tree_update(&dom, button);
        assert_eq!(
            node_with(&ascii, ak_id(button)).label().unwrap().len(),
            MAX_NAME_LEN
        );

        let mut dom = Dom::new();
        let button = elem(&mut dom, "button");
        dom.append(dom.root(), button);
        attach_text(&mut dom, button, &"€".repeat(100));
        let unicode = build_tree_update(&dom, button);
        let name = node_with(&unicode, ak_id(button)).label().unwrap();
        assert!(name.len() <= MAX_NAME_LEN);
        assert_eq!(name.len() % '€'.len_utf8(), 0);
    }

    #[test]
    fn role_mapping_table() {
        let tags = [
            ("a", Role::Link),
            ("link", Role::Link),
            ("button", Role::Button),
            ("textarea", Role::MultilineTextInput),
            ("select", Role::ComboBox),
            ("h1", Role::Heading),
            ("h2", Role::Heading),
            ("h3", Role::Heading),
            ("h4", Role::Heading),
            ("h5", Role::Heading),
            ("h6", Role::Heading),
            ("ul", Role::List),
            ("ol", Role::List),
            ("li", Role::ListItem),
            ("table", Role::Table),
            ("tr", Role::Row),
            ("td", Role::Cell),
            ("th", Role::Cell),
            ("img", Role::Image),
            ("nav", Role::Navigation),
            ("main", Role::Main),
            ("header", Role::Banner),
            ("footer", Role::ContentInfo),
            ("div", Role::GenericContainer),
            ("span", Role::GenericContainer),
        ];
        for (tag, expected) in tags {
            let mut dom = Dom::new();
            let id = elem(&mut dom, tag);
            dom.append(dom.root(), id);
            let role = build_tree_update(&dom, id)
                .nodes
                .into_iter()
                .find(|(node_id, _)| *node_id == ak_id(id))
                .unwrap()
                .1
                .role();
            assert_eq!(role, expected, "tag {tag}");
        }

        let inputs = [
            ("text", Role::TextInput),
            ("password", Role::TextInput),
            ("checkbox", Role::CheckBox),
            ("radio", Role::RadioButton),
        ];
        for (kind, expected) in inputs {
            let mut dom = Dom::new();
            let id = elem_with(&mut dom, "input", &[("type", kind)]);
            dom.append(dom.root(), id);
            let role = build_tree_update(&dom, id)
                .nodes
                .into_iter()
                .find(|(node_id, _)| *node_id == ak_id(id))
                .unwrap()
                .1
                .role();
            assert_eq!(role, expected, "input[type={kind}]");
        }

        for level in 1..=6 {
            let mut dom = Dom::new();
            let id = elem(&mut dom, &format!("h{level}"));
            dom.append(dom.root(), id);
            let node = build_tree_update(&dom, id)
                .nodes
                .into_iter()
                .find(|(node_id, _)| *node_id == ak_id(id))
                .unwrap()
                .1;
            assert_eq!(node.role(), Role::Heading);
            assert_eq!(node.level(), Some(level - 1)); // AccessKit is zero-based
        }
    }
}

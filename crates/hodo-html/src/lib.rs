//! HTML5 document parsing into the arena DOM.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use hodo_dom::{Dom, NodeId, NodeKind};
use html5ever::tendril::{fmt::UTF8, Tendril, TendrilSink};
use html5ever::tree_builder::{ElemName, ElementFlags, NodeOrText, TreeSink};
use html5ever::{Attribute, LocalNameStaticSet, NamespaceStaticSet, QualName};
use string_cache::Atom;

/// Owned qualified name used to satisfy `TreeSink::elem_name`.
#[derive(Debug, Clone)]
struct OwnedElemName(QualName);

impl ElemName for OwnedElemName {
    fn ns(&self) -> &Atom<NamespaceStaticSet> {
        &self.0.ns
    }

    fn local_name(&self) -> &Atom<LocalNameStaticSet> {
        &self.0.local
    }
}

/// `html5ever` tree sink that builds a `hodo_dom::Dom` directly.
struct Sink {
    dom: RefCell<Dom>,
    names: RefCell<HashMap<NodeId, QualName>>,
    templates: RefCell<HashMap<NodeId, NodeId>>,
    stack: RefCell<Vec<NodeId>>,
    /// Detached node used to take children out of their parent (Dom has no detach API).
    trash: NodeId,
}

impl Default for Sink {
    fn default() -> Self {
        let mut dom = Dom::new();
        let trash = dom.create(NodeKind::Element {
            tag: "#detached".to_string(),
            attrs: Vec::new(),
        });
        Self {
            dom: RefCell::new(dom),
            names: RefCell::new(HashMap::new()),
            templates: RefCell::new(HashMap::new()),
            stack: RefCell::new(Vec::new()),
            trash,
        }
    }
}

impl Sink {
    fn detach(&self, id: NodeId) {
        self.dom.borrow_mut().append(self.trash, id);
    }

    /// Re-append the given children in order, which reorders a parent's child list.
    fn reorder(&self, parent: NodeId, children: Vec<NodeId>) {
        let mut dom = self.dom.borrow_mut();
        for child in children {
            dom.append(parent, child);
        }
    }

    fn append_text(&self, parent: NodeId, text: Tendril<UTF8>) {
        let value = text.to_string();
        let mut dom = self.dom.borrow_mut();
        if let Some(last) = dom.children(parent).last().copied() {
            if let Some(previous) = dom.text(last) {
                let mut merged = String::with_capacity(previous.len() + value.len());
                merged.push_str(previous);
                merged.push_str(&value);
                let new_id = dom.create(NodeKind::Text(merged));
                dom.append(self.trash, last);
                dom.append(parent, new_id);
                return;
            }
        }
        let id = dom.create(NodeKind::Text(value));
        dom.append(parent, id);
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Dom;
    type ElemName<'a> = OwnedElemName;

    fn finish(self) -> Self::Output {
        self.dom.into_inner()
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> Self::Handle {
        self.dom.borrow().root()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        let names = self.names.borrow();
        names
            .get(target)
            .cloned()
            .map(OwnedElemName)
            .unwrap_or_else(|| {
                OwnedElemName(QualName::new(
                    None,
                    html5ever::ns!(html),
                    html5ever::local_name!("div"),
                ))
            })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let tag = name.local.to_string();
        let attrs: Vec<(String, String)> = attrs
            .into_iter()
            .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
            .collect();
        let is_template =
            tag == "template" && name.ns.to_string() == "http://www.w3.org/1999/xhtml";
        let id = self
            .dom
            .borrow_mut()
            .create(NodeKind::Element { tag, attrs });
        self.names.borrow_mut().insert(id, name.clone());
        self.stack.borrow_mut().push(id);

        if is_template {
            let fragment = self.dom.borrow_mut().create(NodeKind::Element {
                tag: "#document-fragment".to_string(),
                attrs: Vec::new(),
            });
            self.templates.borrow_mut().insert(id, fragment);
        }
        id
    }

    fn create_comment(&self, text: Tendril<UTF8>) -> Self::Handle {
        self.dom.borrow_mut().create(NodeKind::Element {
            tag: "#comment".to_string(),
            attrs: vec![("text".to_string(), text.to_string())],
        })
    }

    fn create_pi(&self, target: Tendril<UTF8>, data: Tendril<UTF8>) -> Self::Handle {
        self.dom.borrow_mut().create(NodeKind::Element {
            tag: "#processing-instruction".to_string(),
            attrs: vec![
                ("target".to_string(), target.to_string()),
                ("data".to_string(), data.to_string()),
            ],
        })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent = self
            .templates
            .borrow()
            .get(parent)
            .copied()
            .unwrap_or(*parent);
        match child {
            NodeOrText::AppendNode(id) => self.dom.borrow_mut().append(parent, id),
            NodeOrText::AppendText(text) => self.append_text(parent, text),
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if self.dom.borrow().parent(*element).is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: Tendril<UTF8>,
        _public_id: Tendril<UTF8>,
        _system_id: Tendril<UTF8>,
    ) {
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.templates
            .borrow()
            .get(target)
            .copied()
            .unwrap_or(*target)
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _mode: html5ever::tree_builder::QuirksMode) {}

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let Some(parent) = self.dom.borrow().parent(*sibling) else {
            return;
        };
        let handle = match new_node {
            NodeOrText::AppendNode(id) => id,
            NodeOrText::AppendText(text) => self
                .dom
                .borrow_mut()
                .create(NodeKind::Text(text.to_string())),
        };
        let children: Vec<NodeId> = self
            .dom
            .borrow()
            .children(parent)
            .iter()
            .copied()
            .filter(|id| *id != handle)
            .collect();
        let insert_at = children
            .iter()
            .position(|id| id == sibling)
            .unwrap_or(children.len());
        let mut reordered = children;
        reordered.insert(insert_at, handle);
        self.reorder(parent, reordered);
    }

    fn add_attrs_if_missing(&self, _target: &Self::Handle, _attrs: Vec<Attribute>) {
        // The arena exposes no attribute mutation; the tree builder only uses this
        // for duplicate <html>/<body> tags, which is rare in real documents.
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.detach(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let children: Vec<NodeId> = self.dom.borrow().children(*node).to_vec();
        let mut dom = self.dom.borrow_mut();
        for child in children {
            dom.append(*new_parent, child);
        }
    }

    fn pop(&self, _node: &Self::Handle) {
        self.stack.borrow_mut().pop();
    }
}

/// Parse an HTML5 document into the arena DOM.
pub fn parse_document(html: &str) -> Dom {
    html5ever::parse_document(Sink::default(), Default::default())
        .from_utf8()
        .one(html.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_tag(dom: &Dom, tag: &str) -> NodeId {
        let mut found = None;
        dom.walk(|id, node| {
            if found.is_none() && matches!(&node.kind, NodeKind::Element { tag: t, .. } if t == tag)
            {
                found = Some(id);
            }
        });
        found.unwrap_or_else(|| panic!("element <{tag}> not found"))
    }

    fn all_tags(dom: &Dom, tag: &str) -> Vec<NodeId> {
        let mut found = Vec::new();
        dom.walk(|id, node| {
            if matches!(&node.kind, NodeKind::Element { tag: t, .. } if t == tag) {
                found.push(id);
            }
        });
        found
    }

    fn text_content(dom: &Dom, id: NodeId) -> String {
        fn collect(dom: &Dom, id: NodeId, out: &mut String) {
            let node = dom.get(id);
            if let NodeKind::Text(text) = &node.kind {
                out.push_str(text);
            }
            for &child in &node.children {
                collect(dom, child, out);
            }
        }
        let mut out = String::new();
        collect(dom, id, &mut out);
        out
    }

    fn assert_sane(dom: &Dom) {
        dom.walk(|id, node| {
            for &child in &node.children {
                assert_eq!(dom.parent(child), Some(id), "child parent link mismatch");
            }
        });
    }

    #[test]
    fn nested_divs_with_attributes() {
        let dom = parse_document(
            r#"<div id="main" class="a b" data-x="1"><span class="c">hi</span></div>"#,
        );
        let div = find_tag(&dom, "div");
        assert_eq!(dom.attr(div, "id"), Some("main"));
        assert_eq!(dom.attr(div, "class"), Some("a b"));
        assert_eq!(dom.attr(div, "data-x"), Some("1"));
        let span = find_tag(&dom, "span");
        assert_eq!(dom.attr(span, "class"), Some("c"));
        assert_eq!(dom.parent(span), Some(div));
        assert_eq!(text_content(&dom, div), "hi");
        assert_sane(&dom);
    }

    #[test]
    fn text_nodes() {
        let dom = parse_document("<p>Hello <b>world</b>!</p>");
        let p = find_tag(&dom, "p");
        assert_eq!(text_content(&dom, p), "Hello world!");
        assert_sane(&dom);
    }

    #[test]
    fn void_elements() {
        let dom = parse_document("<div><br><img src=\"pic.png\"></div>");
        let br = find_tag(&dom, "br");
        let img = find_tag(&dom, "img");
        assert!(dom.children(br).is_empty());
        assert!(dom.children(img).is_empty());
        assert_eq!(dom.attr(img, "src"), Some("pic.png"));
        assert_sane(&dom);
    }

    #[test]
    fn entities_decode() {
        let dom = parse_document("<p>a &amp; b &lt; c &gt; &quot;d&quot; &#65;</p>");
        assert_eq!(
            text_content(&dom, find_tag(&dom, "p")),
            "a & b < c > \"d\" A"
        );
    }

    #[test]
    fn implied_html_head_body() {
        let dom = parse_document("<p>x</p>");
        let root = dom.root();
        let children = dom.children(root);
        let tags: Vec<&str> = children.iter().filter_map(|id| dom.tag(*id)).collect();
        assert_eq!(tags, vec!["html"]);
        let html = children[0];
        let tags: Vec<&str> = dom
            .children(html)
            .iter()
            .filter_map(|id| dom.tag(*id))
            .collect();
        assert_eq!(tags, vec!["head", "body"]);
        assert_sane(&dom);
    }

    #[test]
    fn style_and_script_are_single_text_nodes() {
        let dom = parse_document("<style>.a{color:red}</style><script>let x=1;</script>");
        for (tag, expected) in [("style", ".a{color:red}"), ("script", "let x=1;")] {
            let element = find_tag(&dom, tag);
            let children = dom.children(element);
            assert_eq!(children.len(), 1, "{tag} should have exactly one child");
            assert_eq!(dom.text(children[0]), Some(expected));
        }
    }

    #[test]
    fn malformed_markup_is_sane() {
        let dom = parse_document("<div><p>one<p>two</div></span>trailing");
        assert!(!all_tags(&dom, "div").is_empty());
        assert!(!all_tags(&dom, "p").is_empty());
        assert!(find_tag(&dom, "body") != dom.root());
        assert_sane(&dom);
    }

    #[test]
    fn stray_close_tags_do_not_panic() {
        let dom = parse_document("</div></p><p>ok</p>");
        assert_eq!(text_content(&dom, find_tag(&dom, "p")), "ok");
        assert_sane(&dom);
    }

    #[test]
    fn utf8_input() {
        let dom = parse_document("<p>Привет 世界 🌍</p>");
        assert_eq!(text_content(&dom, find_tag(&dom, "p")), "Привет 世界 🌍");
    }
}

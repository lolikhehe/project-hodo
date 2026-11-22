//! HTML5 document parsing into the arena DOM.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_types::limits::MAX_NESTING_DEPTH;
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
    /// Detached node used to take children out of their parent (Dom has no detach API).
    trash: NodeId,
}

impl Default for Sink {
    fn default() -> Self {
        // Shell page parsing owns the hot DOM path; intern tag/attr strings so
        // repetitive documents pay per distinct string, not per element.
        let mut dom = Dom::with_interning();
        let trash = dom.create(NodeKind::Element {
            tag: "#detached".to_string(),
            attrs: Vec::new(),
        });
        Self {
            dom: RefCell::new(dom),
            names: RefCell::new(HashMap::new()),
            templates: RefCell::new(HashMap::new()),
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

    /// Depth of a node from the document root (the root itself is depth 0).
    fn depth(&self, id: NodeId) -> usize {
        let dom = self.dom.borrow();
        let mut depth = 0;
        let mut cursor = dom.parent(id);
        while let Some(parent) = cursor {
            depth += 1;
            cursor = dom.parent(parent);
        }
        depth
    }

    /// Beyond the central nesting cap, content is degraded into the detached
    /// trash node instead of growing the reachable tree: parsing continues, no
    /// node is reachable past `MAX_NESTING_DEPTH`, and no recursive pass sees a
    /// deeper tree (layout relies on the same 512-level bound).
    fn capped(&self, id: NodeId) -> NodeId {
        if self.depth(id) >= MAX_NESTING_DEPTH {
            self.trash
        } else {
            id
        }
    }

    fn append_text(&self, parent: NodeId, text: Tendril<UTF8>) {
        let value = text.to_string();
        let parent = self.capped(parent);
        let mut dom = self.dom.borrow_mut();
        // Adjacent chunks share one Text node: detach the old slot (so its
        // allocation is reused by `create`) instead of leaking it into trash.
        // This keeps chunked parses at one node with the full text and cannot
        // exhaust MAX_DOM_NODES through merge churn.
        let merged = dom
            .children(parent)
            .last()
            .copied()
            .and_then(|last| dom.text(last).map(|prev| (last, format!("{prev}{value}"))));
        if let Some((last, merged)) = merged {
            dom.detach_subtree(last);
            let id = dom.create(NodeKind::Text(merged));
            dom.append(parent, id);
            return;
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
        let is_html = name.ns == html5ever::ns!(html);
        let mut attrs: Vec<(String, String)> = attrs
            .into_iter()
            .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
            .collect();
        let is_template = is_html && tag == "template";
        // html5ever defaults to scripting_enabled=true, so <noscript> content
        // arrives as raw text that would otherwise paint. Hide the subtree the
        // same way hodo-shell hides <script>/<style>; an inline style outranks
        // the UA and author sheets and every layout engine skips display:none.
        if is_html && tag == "noscript" {
            if let Some((_, value)) = attrs.iter_mut().find(|(k, _)| k == "style") {
                value.push_str(";display:none");
            } else {
                attrs.push(("style".to_string(), "display:none".to_string()));
            }
        }
        let id = self
            .dom
            .borrow_mut()
            .create(NodeKind::Element { tag, attrs });
        self.names.borrow_mut().insert(id, name.clone());

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
        let parent = self.capped(parent);
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
        let parent = self.capped(parent);
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

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        // html5ever calls this for duplicate <html>/<body> tags: keep the first
        // element but adopt attributes that were not present on it.
        let mut dom = self.dom.borrow_mut();
        for attr in attrs {
            let name = attr.name.local.to_string();
            if dom.attr(*target, &name).is_none() {
                dom.set_attr(*target, &name, attr.value.to_string());
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.detach(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        // Never lift nodes degraded into the detached trash back into the tree.
        if *node == self.trash || *new_parent == self.trash {
            return;
        }
        let children: Vec<NodeId> = self.dom.borrow().children(*node).to_vec();
        let mut dom = self.dom.borrow_mut();
        for child in children {
            dom.append(*new_parent, child);
        }
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
        dom.walk(|id, _| {
            if found.is_none() && dom.tag(id) == Some(tag) {
                found = Some(id);
            }
        });
        found.unwrap_or_else(|| panic!("element <{tag}> not found"))
    }

    fn all_tags(dom: &Dom, tag: &str) -> Vec<NodeId> {
        let mut found = Vec::new();
        dom.walk(|id, _| {
            if dom.tag(id) == Some(tag) {
                found.push(id);
            }
        });
        found
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

    fn assert_sane(dom: &Dom) {
        dom.walk(|id, node| {
            for &child in &node.children {
                assert_eq!(dom.parent(child), Some(id), "child parent link mismatch");
            }
        });
    }

    #[test]
    fn helpers_read_interned_dom() {
        // Mirror of the helper reads with per-node strings stripped: the pool
        // must serve tag/attr/text reads identically to Dom::new() mode.
        let mut dom = Dom::with_interning();
        let root = dom.root();
        let html = dom.create(NodeKind::Element {
            tag: "html".into(),
            attrs: Vec::new(),
        });
        let body = dom.create(NodeKind::Element {
            tag: "body".into(),
            attrs: vec![("class".into(), "c".into())],
        });
        let p = dom.create(NodeKind::Element {
            tag: "p".into(),
            attrs: Vec::new(),
        });
        let text = dom.create(NodeKind::Text("hi".into()));
        dom.append(root, html);
        dom.append(html, body);
        dom.append(body, p);
        dom.append(p, text);

        assert_eq!(find_tag(&dom, "p"), p);
        assert_eq!(all_tags(&dom, "body"), vec![body]);
        assert_eq!(text_content(&dom, p), "hi");
        assert_eq!(dom.tag(body), Some("body"));
        assert_eq!(dom.attr(body, "class"), Some("c"));
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

    #[test]
    fn duplicate_body_attrs_are_kept_on_the_first_body() {
        let dom = parse_document("<body><body class=\"x\" id=\"y\">text</body>");
        let bodies = all_tags(&dom, "body");
        assert_eq!(bodies.len(), 1);
        assert_eq!(dom.attr(bodies[0], "class"), Some("x"));
        assert_eq!(dom.attr(bodies[0], "id"), Some("y"));
        assert_eq!(text_content(&dom, bodies[0]), "text");
        assert_sane(&dom);
    }

    #[test]
    fn chunked_text_merges_into_one_node() {
        let mut parser = html5ever::parse_document(Sink::default(), Default::default()).from_utf8();
        for chunk in ["<p>Hel", "lo ", "world</p>"] {
            parser.process(Tendril::from_slice(chunk.as_bytes()));
        }
        let dom = parser.finish();
        let p = find_tag(&dom, "p");
        let children = dom.children(p);
        assert_eq!(
            children.len(),
            1,
            "adjacent chunks must share one Text node"
        );
        assert_eq!(dom.text(children[0]), Some("Hello world"));
        assert_sane(&dom);
    }

    fn parse_chunks_text(chunks: &[&str]) -> String {
        let mut parser = html5ever::parse_document(Sink::default(), Default::default()).from_utf8();
        for chunk in chunks {
            parser.process(Tendril::from_slice(chunk.as_bytes()));
        }
        let dom = parser.finish();
        text_content(&dom, find_tag(&dom, "p"))
    }

    #[test]
    fn noscript_subtree_is_hidden_even_though_content_is_present() {
        let dom = parse_document("<noscript><h2>enable JS</h2></noscript><p>real</p>");
        let noscript = find_tag(&dom, "noscript");
        // Render-skip contract: an inline display:none makes layout skip the
        // whole subtree (all four engines filter it, layout_child returns early)
        // without touching hodo-shell/hodo-layout/hodo-render.
        assert_eq!(dom.attr(noscript, "style"), Some("display:none"));
        // html5ever defaults to scripting_enabled=true, so the notice is raw
        // text; it stays in the DOM (reachable), it just never paints.
        assert!(text_content(&dom, noscript).contains("enable JS"));
        assert_eq!(text_content(&dom, find_tag(&dom, "p")), "real");
        assert_sane(&dom);
    }

    #[test]
    fn existing_noscript_style_is_preserved_when_hiding() {
        let dom = parse_document("<noscript style=\"color:red\"><b>x</b></noscript>");
        let noscript = find_tag(&dom, "noscript");
        let style = dom.attr(noscript, "style").unwrap();
        assert!(
            style.contains("color:red"),
            "author style must survive: {style}"
        );
        assert!(
            style.contains("display:none"),
            "hide must be appended: {style}"
        );
        assert_sane(&dom);
    }

    #[test]
    fn template_content_is_outside_the_live_tree() {
        let dom = parse_document("<template><h2>not rendered</h2></template><p>real</p>");
        // html5ever puts template children in a detached fragment created by
        // create_element; dom.walk from the root never reaches them.
        let mut h2_in_live_tree = false;
        dom.walk(|id, _| {
            if dom.tag(id) == Some("h2") {
                h2_in_live_tree = true;
            }
        });
        assert!(
            !h2_in_live_tree,
            "template content must not be in the live tree"
        );
        assert_eq!(text_content(&dom, find_tag(&dom, "p")), "real");
        assert_sane(&dom);
    }

    #[test]
    fn chunked_entity_boundaries_decode_after_merge() {
        // html5ever decodes character references in the tokenizer; chunking the
        // input mid-entity must not leak literal fragments into the merged text.
        assert_eq!(parse_chunks_text(&["<p>&am", "p;</p>"]), "&");
        assert_eq!(parse_chunks_text(&["<p>&#x27", ";</p>"]), "'");
        assert_eq!(parse_chunks_text(&["<p>&lt", ";</p>"]), "<");
        assert_eq!(parse_chunks_text(&["<p>&gt", ";</p>"]), ">");
        assert_eq!(parse_chunks_text(&["<p>&quo", "t;</p>"]), "\"");
        assert_eq!(parse_chunks_text(&["<p>&", "amp;</p>"]), "&");
        assert_eq!(
            parse_chunks_text(&["<p>&am", "p; and &l", "t;</p>"]),
            "& and <"
        );

        // One byte per chunk around a mix of named and numeric references.
        let whole = "<p>Tom &amp; Jerry &lt;b&gt; &#x27; &#65; &quot;q&quot;</p>";
        let bytes: Vec<&str> = whole.split_inclusive(|_: char| true).collect();
        assert_eq!(parse_chunks_text(&bytes), "Tom & Jerry <b> ' A \"q\"");
    }

    #[test]
    fn parser_caps_reachable_nesting_at_central_limit() {
        let html = format!(
            "{}x{}",
            "<div>".repeat(MAX_NESTING_DEPTH + 88),
            "</div>".repeat(MAX_NESTING_DEPTH + 88)
        );
        let dom = parse_document(&html);
        let mut max_depth = 0;
        let mut stack = vec![(dom.root(), 0usize)];
        while let Some((id, depth)) = stack.pop() {
            max_depth = max_depth.max(depth);
            for &child in dom.children(id) {
                stack.push((child, depth + 1));
            }
        }
        assert!(
            max_depth <= MAX_NESTING_DEPTH,
            "reachable depth {max_depth} exceeds the cap of {MAX_NESTING_DEPTH}"
        );
        assert!(
            max_depth >= MAX_NESTING_DEPTH - 4,
            "the capped prefix must survive, got depth {max_depth}"
        );
        assert_sane(&dom);
    }
}

//! Arena-based DOM: u32 handles, zero Rc/RefCell, minimal RAM.

/// Stable handle into the node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// What a node represents.
#[derive(Debug, Clone)]
pub enum NodeKind {
    Document,
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
    },
    Text(String),
}

/// A single arena node.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

/// The node arena, rooted at a `Document` node.
#[derive(Debug, Clone)]
pub struct Dom {
    nodes: Vec<Node>,
    root_id: NodeId,
}

impl Dom {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                kind: NodeKind::Document,
                parent: None,
                children: Vec::new(),
            }],
            root_id: NodeId(0),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root_id
    }

    pub fn create(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node {
            kind,
            parent: None,
            children: Vec::new(),
        });
        id
    }

    /// Attach `child` to `parent`, detaching it from any previous parent.
    ///
    /// Appends that would break the forest invariant are ignored: the root stays
    /// parentless, and a node cannot become its own ancestor's descendant.
    pub fn append(&mut self, parent: NodeId, child: NodeId) {
        // The document root must stay parentless.
        if child == self.root_id {
            return;
        }
        // If `parent` is `child` or lies below it, linking them makes a cycle.
        let mut cursor = Some(parent);
        while let Some(id) = cursor {
            if id == child {
                return;
            }
            cursor = self.nodes[id.0 as usize].parent;
        }
        if let Some(old) = self.nodes[child.0 as usize].parent {
            self.nodes[old.0 as usize].children.retain(|c| *c != child);
        }
        self.nodes[child.0 as usize].parent = Some(parent);
        self.nodes[parent.0 as usize].children.push(child);
    }

    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.0 as usize].children
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0 as usize].parent
    }

    pub fn tag(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.0 as usize].kind {
            NodeKind::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        match &self.nodes[id.0 as usize].kind {
            NodeKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    pub fn text(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.0 as usize].kind {
            NodeKind::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Visit every node from the root in pre-order.
    pub fn walk(&self, mut f: impl FnMut(NodeId, &Node)) {
        fn visit(dom: &Dom, id: NodeId, f: &mut impl FnMut(NodeId, &Node)) {
            let node = dom.get(id);
            f(id, node);
            for &child in &node.children {
                visit(dom, child, f);
            }
        }
        visit(self, self.root_id, &mut f);
    }
}

impl Default for Dom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn elem(dom: &mut Dom, tag: &str) -> NodeId {
        dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: Vec::new(),
        })
    }

    fn next_rand(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn tree_build_and_preorder_walk() {
        let mut dom = Dom::new();
        let html = elem(&mut dom, "html");
        let head = elem(&mut dom, "head");
        let title = elem(&mut dom, "title");
        let body = elem(&mut dom, "body");
        let p = elem(&mut dom, "p");
        let text = dom.create(NodeKind::Text("hello".into()));

        dom.append(dom.root(), html);
        dom.append(html, head);
        dom.append(head, title);
        dom.append(html, body);
        dom.append(body, p);
        dom.append(body, text);

        let mut seen = Vec::new();
        dom.walk(|_, node| {
            let label = match &node.kind {
                NodeKind::Document => "document".to_string(),
                NodeKind::Element { tag, .. } => tag.clone(),
                NodeKind::Text(t) => t.clone(),
            };
            seen.push(label);
        });
        assert_eq!(
            seen,
            ["document", "html", "head", "title", "body", "p", "hello"]
        );
    }

    #[test]
    fn append_reparent_detaches_old_parent() {
        let mut dom = Dom::new();
        let a = elem(&mut dom, "a");
        let b = elem(&mut dom, "b");
        let child = elem(&mut dom, "child");
        dom.append(dom.root(), a);
        dom.append(dom.root(), b);
        dom.append(a, child);

        assert_eq!(dom.children(a), &[child]);
        assert_eq!(dom.parent(child), Some(a));

        dom.append(b, child);
        assert!(dom.children(a).is_empty(), "old parent must lose the child");
        assert_eq!(dom.children(b), &[child]);
        assert_eq!(dom.parent(child), Some(b));

        // Re-appending to the same parent must not duplicate the child.
        dom.append(b, child);
        assert_eq!(dom.children(b), &[child]);
    }

    #[test]
    fn accessors_tag_attr_text() {
        let mut dom = Dom::new();
        let el = dom.create(NodeKind::Element {
            tag: "a".into(),
            attrs: vec![("href".into(), "/x".into()), ("id".into(), "link".into())],
        });
        let txt = dom.create(NodeKind::Text("hi".into()));

        assert_eq!(dom.tag(el), Some("a"));
        assert_eq!(dom.attr(el, "href"), Some("/x"));
        assert_eq!(dom.attr(el, "id"), Some("link"));
        assert_eq!(dom.attr(el, "missing"), None);
        assert_eq!(dom.text(el), None);
        assert_eq!(dom.tag(txt), None);
        assert_eq!(dom.attr(txt, "href"), None);
        assert_eq!(dom.text(txt), Some("hi"));
        assert_eq!(dom.tag(dom.root()), None);
        assert_eq!(dom.text(dom.root()), None);
        assert_eq!(dom.attr(dom.root(), "x"), None);
    }

    #[test]
    fn len_counts_nodes_and_is_empty_tracks_len() {
        let mut dom = Dom::new();
        assert_eq!(dom.len(), 1);
        assert!(!dom.is_empty());

        let a = elem(&mut dom, "a");
        dom.append(dom.root(), a);
        assert_eq!(dom.len(), 2);
        assert!(!dom.is_empty());

        let b = elem(&mut dom, "b");
        dom.append(b, a); // reparenting must not create or destroy nodes
        assert_eq!(dom.len(), 3);
        assert!(!dom.is_empty());
    }

    #[test]
    fn walk_visits_every_node_exactly_once() {
        let mut dom = Dom::new();
        for _ in 0..50 {
            let id = elem(&mut dom, "n");
            dom.append(NodeId(id.0 / 2), id);
        }

        let mut seen = HashSet::new();
        let mut count = 0;
        dom.walk(|id, _| {
            assert!(seen.insert(id.0), "duplicate visit of node {}", id.0);
            count += 1;
        });
        assert_eq!(count, dom.len());
        assert_eq!(seen.len(), dom.len());
    }

    #[test]
    fn append_refuses_cycles_self_and_root_reparent() {
        let mut dom = Dom::new();
        let a = elem(&mut dom, "a");
        let b = elem(&mut dom, "b");
        let c = elem(&mut dom, "c");
        dom.append(dom.root(), a);
        dom.append(a, b);
        dom.append(b, c);

        dom.append(a, a); // self
        assert_eq!(dom.parent(a), Some(dom.root()));

        dom.append(c, a); // would place an ancestor below its descendant
        assert_eq!(dom.parent(a), Some(dom.root()));
        assert_eq!(dom.children(a), &[b]);
        assert!(dom.children(c).is_empty());

        let orphan = elem(&mut dom, "orphan");
        dom.append(orphan, dom.root());
        assert_eq!(dom.parent(dom.root()), None);

        // Attach the orphan so every node is reachable, then confirm the walk
        // terminates and covers the whole tree even after refused appends.
        dom.append(c, orphan);
        let mut n = 0;
        dom.walk(|_, _| n += 1);
        assert_eq!(n, dom.len());
    }

    #[test]
    fn three_level_random_tree_parent_child_invariants() {
        let mut dom = Dom::new();
        let mut rng = 0x9E37_79B9_7F4A_7C15_u64;
        let root = dom.root();

        let mut level1 = Vec::new();
        for _ in 0..8 {
            let n = elem(&mut dom, "l1");
            dom.append(root, n);
            level1.push(n);
        }
        let mut level2 = Vec::new();
        for _ in 0..24 {
            let p = level1[(next_rand(&mut rng) as usize) % level1.len()];
            let n = elem(&mut dom, "l2");
            dom.append(p, n);
            level2.push(n);
        }
        for _ in 0..64 {
            let p = level2[(next_rand(&mut rng) as usize) % level2.len()];
            let n = elem(&mut dom, "l3");
            dom.append(p, n);
        }

        assert_eq!(dom.len(), 1 + 8 + 24 + 64);
        assert_eq!(dom.parent(root), None);

        for i in 0..dom.len() {
            let id = NodeId(i as u32);
            let kids = dom.children(id);
            let unique: HashSet<_> = kids.iter().collect();
            assert_eq!(unique.len(), kids.len(), "duplicate child under node {i}");
            for &c in kids {
                assert_eq!(dom.parent(c), Some(id), "child must point back to parent");
            }
            if id != root {
                let p = dom.parent(id).expect("non-root node has a parent");
                assert!(
                    dom.children(p).contains(&id),
                    "parent must list the node as a child"
                );
            }
        }
    }

    #[test]
    fn document_root_has_no_parent() {
        let dom = Dom::new();
        assert_eq!(dom.parent(dom.root()), None);
        assert!(dom.children(dom.root()).is_empty());
        assert_eq!(dom.tag(dom.root()), None);
    }
}

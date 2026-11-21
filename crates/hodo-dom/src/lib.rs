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
    pub fn append(&mut self, parent: NodeId, child: NodeId) {
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

//! Arena-based DOM: u32 handles, zero Rc/RefCell, minimal RAM.
//!
//! Internal invariant: every `NodeId` used here was issued by this `Dom`.
//! All arena indexing goes through [`NodeId::slot`], and debug builds check the
//! handle's generation. Untrusted handles (from JS, IPC or extensions) must be
//! filtered through [`Dom::validate`] first; forged indices are developer
//! misuse, never parser-fed data.

use hodo_types::limits::MAX_DOM_NODES;

/// Opaque handle into the node arena: low 20 bits are the slot, high 12 bits
/// the generation. A reused slot therefore hands out a *different* handle, so
/// an old handle can be rejected instead of silently aliasing a new node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

const SLOT_BITS: u32 = 20;
const SLOT_MASK: u32 = (1 << SLOT_BITS) - 1;
const GEN_MOD: u32 = 1 << (32 - SLOT_BITS);
const GEN_MASK: u32 = GEN_MOD - 1;

// `MAX_DOM_NODES` must fit in the 20-bit slot encoding.
const _: () = assert!(MAX_DOM_NODES as u32 <= SLOT_MASK);

impl NodeId {
    /// Arena slot. Callers keeping parallel arrays by handle should use this
    /// rather than the raw `u32` once generation bits are in play.
    #[inline]
    pub const fn slot(self) -> u32 {
        self.0 & SLOT_MASK
    }

    #[inline]
    const fn generation(self) -> u32 {
        self.0 >> SLOT_BITS
    }

    #[inline]
    const fn encode(slot: u32, generation: u32) -> Self {
        Self(((generation & GEN_MASK) << SLOT_BITS) | (slot & SLOT_MASK))
    }

    #[inline]
    const fn bump(generation: u32) -> u32 {
        (generation + 1) & GEN_MASK
    }
}

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
    /// Current generation of each slot; bumped on detach and on reuse.
    generations: Vec<u32>,
    /// True while a slot belongs to the unreachable `#detached` trash.
    trashed: Vec<bool>,
    root_id: NodeId,
    /// Slots eligible for reuse, populated only by `detach_subtree`.
    free: Vec<u32>,
    /// Lazily created trash root: owned by the arena but unreachable from the root.
    detached: Option<NodeId>,
}

impl Dom {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                kind: NodeKind::Document,
                parent: None,
                children: Vec::new(),
            }],
            generations: vec![0],
            trashed: vec![false],
            root_id: NodeId(0),
            free: Vec::new(),
            detached: None,
        }
    }

    pub fn root(&self) -> NodeId {
        self.root_id
    }

    /// Create a node, reusing a detached slot when one is available.
    pub fn create(&mut self, kind: NodeKind) -> NodeId {
        let node = Node {
            kind,
            parent: None,
            children: Vec::new(),
        };
        let (slot, generation) = if let Some(slot) = self.free.pop() {
            let old = self.generations[slot as usize];
            let generation = NodeId::bump(old);
            // The whole point of the generation table: a recycled slot must
            // never hand out the previous occupant's handle.
            debug_assert_ne!(generation, old, "reused slot must bump generation");
            self.nodes[slot as usize] = node;
            self.generations[slot as usize] = generation;
            self.trashed[slot as usize] = false;
            (slot, generation)
        } else {
            if self.nodes.len() >= MAX_DOM_NODES {
                // Cap reached: degrade to the root handle instead of wrapping
                // the 20-bit slot encoding (no panic on oversized input).
                return self.root_id;
            }
            let slot = self.nodes.len() as u32;
            self.nodes.push(node);
            self.generations.push(0);
            self.trashed.push(false);
            (slot, 0)
        };
        let id = NodeId::encode(slot, generation);
        self.debug_assert_handle(id);
        id
    }

    /// Remove `root` and its whole subtree from the visible tree.
    ///
    /// The nodes are moved under an internal `#detached` node that is not
    /// reachable from the document root, so `walk` skips them. Their slots are
    /// put on the free list for a future [`Dom::create`]; every handle into the
    /// subtree is invalidated immediately. Invalid or already-detached handles
    /// are ignored rather than panicking.
    pub fn detach_subtree(&mut self, root: NodeId) {
        if root == self.root_id || !self.validate(root) {
            return;
        }
        let root_slot = root.slot() as usize;
        if let Some(parent) = self.nodes[root_slot].parent {
            let p = parent.slot() as usize;
            self.nodes[p].children.retain(|c| *c != root);
        }
        self.nodes[root_slot].parent = None;

        let trash = self.ensure_trash();
        let mut stack = vec![root];
        let mut detached = Vec::new();
        while let Some(id) = stack.pop() {
            let slot = id.slot() as usize;
            if slot >= self.nodes.len() || self.trashed[slot] {
                continue;
            }
            // A stale child handle may point at a slot that has since been
            // reused by a live node; never trash that new occupant.
            if self.generations[slot] != id.generation() {
                continue;
            }
            self.trashed[slot] = true;
            detached.push(slot as u32);
            for &child in &self.nodes[slot].children {
                stack.push(child);
            }
        }
        for slot in detached {
            self.generations[slot as usize] = NodeId::bump(self.generations[slot as usize]);
            self.free.push(slot);
        }
        let t = trash.slot() as usize;
        self.nodes[root_slot].parent = Some(trash);
        self.nodes[t].children.push(root);
    }

    /// True only for a handle that is in range, live and of the current generation.
    pub fn validate(&self, id: NodeId) -> bool {
        let slot = id.slot() as usize;
        slot < self.nodes.len() && !self.trashed[slot] && self.generations[slot] == id.generation()
    }

    /// Attach `child` to `parent`, detaching it from any previous parent.
    ///
    /// Appends that would break the forest invariant are ignored: the root stays
    /// parentless, and a node cannot become its own ancestor's descendant.
    pub fn append(&mut self, parent: NodeId, child: NodeId) {
        if child == self.root_id || !self.validate(parent) || !self.validate(child) {
            return;
        }
        // If `parent` is `child` or lies below it, linking them makes a cycle.
        let mut cursor = Some(parent);
        while let Some(id) = cursor {
            if id == child {
                return;
            }
            cursor = self.nodes[id.slot() as usize].parent;
        }
        let c = child.slot() as usize;
        if let Some(old) = self.nodes[c].parent {
            self.nodes[old.slot() as usize]
                .children
                .retain(|x| *x != child);
        }
        self.nodes[c].parent = Some(parent);
        self.nodes[parent.slot() as usize].children.push(child);
    }

    /// Indexing assumes `id` came from this arena (or passed [`Dom::validate`]):
    /// a forged out-of-range handle is developer misuse, not untrusted input.
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.slot() as usize]
    }

    pub fn children(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.slot() as usize].children
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.slot() as usize].parent
    }

    pub fn tag(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.slot() as usize].kind {
            NodeKind::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        match &self.nodes[id.slot() as usize].kind {
            NodeKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Set an attribute on a live element, adding it when absent. No-op for
    /// invalid handles and non-elements. Lets a host resolve relative URLs
    /// (e.g. `img[src]`) against the page it loaded before layout runs.
    pub fn set_attr(&mut self, id: NodeId, name: &str, value: String) {
        if !self.validate(id) {
            return;
        }
        if let NodeKind::Element { attrs, .. } = &mut self.nodes[id.slot() as usize].kind {
            if let Some((_, v)) = attrs.iter_mut().find(|(k, _)| k == name) {
                *v = value;
            } else {
                attrs.push((name.to_string(), value));
            }
        }
    }

    pub fn text(&self, id: NodeId) -> Option<&str> {
        match &self.nodes[id.slot() as usize].kind {
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

    /// Visit every node reachable from the root in pre-order. Trash is unreachable.
    pub fn walk(&self, mut f: impl FnMut(NodeId, &Node)) {
        fn visit(dom: &Dom, id: NodeId, f: &mut impl FnMut(NodeId, &Node)) {
            dom.debug_assert_handle(id);
            let node = dom.get(id);
            f(id, node);
            for &child in &node.children {
                visit(dom, child, f);
            }
        }
        visit(self, self.root_id, &mut f);
    }

    fn ensure_trash(&mut self) -> NodeId {
        if let Some(id) = self.detached {
            return id;
        }
        let slot = self.nodes.len() as u32;
        self.nodes.push(Node {
            kind: NodeKind::Element {
                tag: "#detached".to_string(),
                attrs: Vec::new(),
            },
            parent: None,
            children: Vec::new(),
        });
        self.generations.push(0);
        // The trash root is an internal node, never a valid external handle.
        self.trashed.push(true);
        let id = NodeId::encode(slot, 0);
        self.detached = Some(id);
        id
    }

    /// Debug-only arena invariants; called from `create` and `walk`.
    #[inline]
    pub(crate) fn debug_assert_handle(&self, id: NodeId) {
        let slot = id.slot() as usize;
        debug_assert!(slot < self.nodes.len(), "NodeId slot out of range");
        debug_assert!(
            slot < self.generations.len(),
            "generation table out of sync"
        );
        debug_assert!(!self.trashed[slot], "trash node reached from the live tree");
        debug_assert_eq!(
            self.generations[slot],
            id.generation(),
            "stale NodeId used after slot reuse"
        );
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
    use hodo_types::limits::MAX_NESTING_DEPTH;
    use std::collections::HashSet;
    use std::mem::size_of;

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
                // Test-only invariant of this hand-built tree: every non-root
                // node has a parent (not parser-fed data).
                let p = dom.parent(id);
                assert!(p.is_some(), "non-root node has a parent");
                if let Some(p) = p {
                    assert!(
                        dom.children(p).contains(&id),
                        "parent must list the node as a child"
                    );
                }
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

    fn count_walk(dom: &Dom) -> usize {
        let mut n = 0;
        dom.walk(|_, _| n += 1);
        n
    }

    fn collect_tags(dom: &Dom) -> Vec<String> {
        let mut tags = Vec::new();
        dom.walk(|_, node| {
            if let NodeKind::Element { tag, .. } = &node.kind {
                tags.push(tag.clone());
            }
        });
        tags
    }

    #[test]
    fn detach_subtree_removes_it_from_parent_and_walk() {
        let mut dom = Dom::new();
        let shell = elem(&mut dom, "shell");
        let branch = elem(&mut dom, "branch");
        let leaf = elem(&mut dom, "leaf");
        dom.append(dom.root(), shell);
        dom.append(shell, branch);
        dom.append(branch, leaf);

        assert_eq!(count_walk(&dom), 4);
        dom.detach_subtree(branch);

        assert!(dom.children(shell).is_empty());
        assert_eq!(collect_tags(&dom), ["shell"]);
        assert_eq!(count_walk(&dom), 2); // document + shell
        assert!(!dom.validate(branch));
        assert!(!dom.validate(leaf));
        assert!(dom.validate(shell));
        assert!(dom.validate(dom.root()));
    }

    #[test]
    fn reused_slot_bumps_generation_and_invalidates_old_handle() {
        let mut dom = Dom::new();
        let old = elem(&mut dom, "old");
        dom.append(dom.root(), old);
        dom.detach_subtree(old);
        assert!(!dom.validate(old));

        let new = elem(&mut dom, "new");
        assert_eq!(old.slot(), new.slot(), "create must reuse the free slot");
        assert_ne!(old, new, "a reused slot must produce a different handle");
        assert_ne!(old.generation(), new.generation());
        assert!(!dom.validate(old));
        assert!(dom.validate(new));
        assert_eq!(dom.tag(new), Some("new"));
        assert_eq!(dom.children(dom.root()), &[]);

        // A second reuse keeps invalidating every older generation.
        dom.detach_subtree(new);
        let newest = elem(&mut dom, "newest");
        assert!(!dom.validate(new));
        assert!(!dom.validate(old));
        assert!(dom.validate(newest));
    }

    #[test]
    fn validate_rejects_forged_and_internal_handles() {
        let mut dom = Dom::new();
        assert!(!dom.validate(NodeId(u32::MAX)));
        assert!(!dom.validate(NodeId(SLOT_MASK)));
        let a = elem(&mut dom, "a");
        dom.detach_subtree(a);
        let internal_trash = NodeId::encode((dom.len() - 1) as u32, 0);
        assert!(!dom.validate(internal_trash));
        dom.append(dom.root(), internal_trash); // forged append is ignored
        assert!(dom.children(dom.root()).is_empty());
    }

    #[test]
    fn parent_child_invariants_hold_after_detach_and_reuse() {
        let mut dom = Dom::new();
        let keep = elem(&mut dom, "keep");
        let gone = elem(&mut dom, "gone");
        let child = elem(&mut dom, "child");
        dom.append(dom.root(), keep);
        dom.append(dom.root(), gone);
        dom.append(gone, child);
        dom.detach_subtree(gone);
        let replacement = elem(&mut dom, "replacement");
        dom.append(keep, replacement);

        assert_eq!(dom.children(dom.root()), &[keep]);
        assert_eq!(dom.parent(keep), Some(dom.root()));
        assert_eq!(dom.children(keep), &[replacement]);
        assert_eq!(dom.parent(replacement), Some(keep));
        assert_eq!(collect_tags(&dom), ["keep", "replacement"]);
        dom.walk(|id, node| {
            assert!(dom.validate(id));
            for &c in &node.children {
                assert_eq!(dom.parent(c), Some(id));
            }
        });
    }

    #[test]
    fn node_size_budget_matches_nesting_cap() {
        // ТЗ §1: a stack overflow aborts instead of panicking, so the depth cap
        // must keep a full recursion frame chain far below 64 MiB.
        assert!(
            MAX_NESTING_DEPTH * size_of::<Node>() < 64 * 1024 * 1024,
            "depth cap and node size exceed the stack budget"
        );
        assert!(MAX_DOM_NODES as u32 <= SLOT_MASK);
    }
}

//! Arena-based DOM: u32 handles, zero Rc/RefCell, minimal RAM.
//!
//! Internal invariant: every `NodeId` used here was issued by this `Dom`.
//! All arena indexing goes through [`NodeId::slot`], and debug builds check the
//! handle's generation. Untrusted handles (from JS, IPC or extensions) must be
//! filtered through [`Dom::validate`] first; forged indices are developer
//! misuse, never parser-fed data.
//!
//! # String interning (opt-in)
//!
//! [`Dom::new`] keeps the original storage behaviour: every tag/attribute
//! `String` lives in its `Node`. [`Dom::with_interning`] instead moves those
//! strings into a per-`Dom` [`StringInterner`]; each `Node` then holds only
//! [`Interned`] ids in a parallel side table, so a document with repeated
//! tags/classes pays for each distinct string once instead of once per element.
//! The public `tag`/`attr`/`set_attr`/`text` signatures are identical in both
//! modes. In interned mode, callers must use those accessors: [`NodeKind::Element`]'s
//! `tag` is emptied and its `attrs` Vec is cleared (the pool is the single
//! source of truth). This is a RAM win on repetitive documents (the dedup map
//! plus one copy per unique string replaces thousands of copies and their Vec
//! allocations) and a small loss on all-unique documents (the map and side
//! table are extra bookkeeping).

use hodo_types::limits::MAX_DOM_NODES;
use std::collections::HashMap;

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

/// A u32 index into a [`StringInterner`] table. Equal indices mean equal
/// strings, so consumers (e.g. selector matching) can compare ids instead of
/// walking bytes. Ids are meaningful only with the `Dom` that produced them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Interned(u32);

impl Interned {
    /// Raw table index. Stable for the lifetime of the owning `Dom`.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A per-`Dom` string pool: `Vec<Box<str>>` holds one copy of each unique
/// string, `HashMap` deduplicates on insertion. The map is kept for the `Dom`'s
/// lifetime (dropping it would lose O(1) dedup and reintroduce duplicates).
#[derive(Debug, Clone, Default)]
pub struct StringInterner {
    strings: Vec<Box<str>>,
    map: HashMap<String, u32>,
}

impl StringInterner {
    /// Return the id for `s`, inserting it once if unseen.
    pub fn intern(&mut self, s: &str) -> Interned {
        if let Some(&index) = self.map.get(s) {
            return Interned(index);
        }
        let index = self.strings.len() as u32;
        self.strings.push(Box::from(s));
        self.map.insert(s.to_string(), index);
        Interned(index)
    }

    /// Borrow the string behind `id`. The borrow is tied to `&self`.
    #[inline]
    pub fn get(&self, id: Interned) -> &str {
        &self.strings[id.0 as usize]
    }

    /// Look up an existing id without inserting.
    #[inline]
    pub fn lookup(&self, s: &str) -> Option<Interned> {
        self.map.get(s).copied().map(Interned)
    }

    /// Number of distinct strings in the pool.
    #[inline]
    pub fn unique_count(&self) -> usize {
        self.strings.len()
    }

    /// Bytes of string data owned by the pool (table + map keys), excluding
    /// allocator overhead and the side table that points into it.
    pub fn heap_bytes(&self) -> usize {
        self.strings.iter().map(|s| s.len()).sum::<usize>()
            + self.map.keys().map(String::len).sum::<usize>()
    }
}

/// What a node represents. Public shape is unchanged: external callers build
/// elements with `String` fields and read `Node::kind` directly.
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

/// Interned ids for one arena slot; parallel to `nodes`, indexed by slot.
#[derive(Debug, Clone, Default)]
struct StrMeta {
    tag: Option<Interned>,
    /// (name, value) ids, in the same order as `NodeKind::Element.attrs`.
    attrs: Vec<(Interned, Interned)>,
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
    /// String pool. Empty but present in the default (non-interned) mode.
    interner: StringInterner,
    /// When true, tag/attr strings live only in `interner`, not in `Node`.
    interning: bool,
    /// Interned ids parallel to `nodes` (same slot indices).
    str_meta: Vec<StrMeta>,
}

impl Dom {
    /// Original behaviour: every tag/attr string is stored in its `Node`.
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
            interner: StringInterner::default(),
            interning: false,
            str_meta: vec![StrMeta::default()],
        }
    }

    /// Opt in to string interning. `tag`/`attr`/`set_attr` keep their signatures
    /// and return `&str` borrowed from the pool, but element nodes lose their
    /// `tag` bytes and `attrs` allocation: use the accessors, not the raw fields.
    pub fn with_interning() -> Self {
        let mut dom = Self::new();
        dom.interning = true;
        dom
    }

    pub fn root(&self) -> NodeId {
        self.root_id
    }

    /// Create a node, reusing a detached slot when one is available.
    pub fn create(&mut self, kind: NodeKind) -> NodeId {
        // Check the cap before interning so a rejected node cannot pollute the pool.
        if self.free.is_empty() && self.nodes.len() >= MAX_DOM_NODES {
            return self.root_id;
        }

        let (kind, meta) = if self.interning {
            match kind {
                NodeKind::Element { tag, attrs } => {
                    let tag_id = self.interner.intern(&tag);
                    let ids = attrs
                        .iter()
                        .map(|(k, v)| (self.interner.intern(k), self.interner.intern(v)))
                        .collect();
                    // Drop the per-node strings and the attrs Vec allocation:
                    // the pool is now the single source of truth.
                    let stripped = NodeKind::Element {
                        tag: String::new(),
                        attrs: Vec::new(),
                    };
                    (
                        stripped,
                        StrMeta {
                            tag: Some(tag_id),
                            attrs: ids,
                        },
                    )
                }
                other => (other, StrMeta::default()),
            }
        } else {
            (kind, StrMeta::default())
        };

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
            self.str_meta[slot as usize] = meta;
            (slot, generation)
        } else {
            let slot = self.nodes.len() as u32;
            self.nodes.push(node);
            self.generations.push(0);
            self.trashed.push(false);
            self.str_meta.push(meta);
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
        let slot = id.slot() as usize;
        if self.interning {
            return self.str_meta.get(slot)?.tag.map(|i| self.interner.get(i));
        }
        match &self.nodes[slot].kind {
            NodeKind::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        let slot = id.slot() as usize;
        if self.interning {
            let wanted = self.interner.lookup(name)?;
            return self
                .str_meta
                .get(slot)?
                .attrs
                .iter()
                .find(|(k, _)| *k == wanted)
                .map(|(_, v)| self.interner.get(*v));
        }
        match &self.nodes[slot].kind {
            NodeKind::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Interned id of `name`'s value on `id`, for u32-based comparison in
    /// selector matching (e.g. `a.attr_interned(n, "class") == b.attr_interned(n, "class")`).
    /// `None` if interning is off, the name was never interned, or the attribute
    /// is absent. Ids are only comparable within the same `Dom`.
    pub fn attr_interned(&self, id: NodeId, name: &str) -> Option<u32> {
        if !self.interning {
            return None;
        }
        let wanted = self.interner.lookup(name)?;
        self.str_meta
            .get(id.slot() as usize)?
            .attrs
            .iter()
            .find(|(k, _)| *k == wanted)
            .map(|(_, v)| v.index())
    }

    /// Set an attribute on a live element, adding it when absent. No-op for
    /// invalid handles and non-elements. Lets a host resolve relative URLs
    /// (e.g. `img[src]`) against the page it loaded before layout runs.
    pub fn set_attr(&mut self, id: NodeId, name: &str, value: String) {
        if !self.validate(id) {
            return;
        }
        let slot = id.slot() as usize;
        if self.interning {
            // Node.kind's strings are emptied in this mode, so match by id.
            let key = self.interner.intern(name);
            let val = self.interner.intern(&value);
            let meta = &mut self.str_meta[slot];
            if let Some((_, v)) = meta.attrs.iter_mut().find(|(k, _)| *k == key) {
                *v = val;
                return;
            }
            meta.attrs.push((key, val));
            return;
        }
        if let NodeKind::Element { attrs, .. } = &mut self.nodes[slot].kind {
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

    /// Number of distinct strings in this `Dom`'s pool (0 in default mode).
    #[inline]
    pub fn interner_len(&self) -> usize {
        self.interner.unique_count()
    }

    /// Bytes of string data held by the pool (table + dedup map keys).
    #[inline]
    pub fn interner_heap_bytes(&self) -> usize {
        self.interner.heap_bytes()
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
        self.str_meta.push(StrMeta::default());
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
        debug_assert!(
            slot < self.str_meta.len(),
            "interner side table out of sync"
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
    use std::time::Instant;

    fn elem(dom: &mut Dom, tag: &str) -> NodeId {
        dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: Vec::new(),
        })
    }

    /// Every structural test runs against both storage modes. Assertions are
    /// written against the frozen accessors so they are mode-independent.
    fn run_both(f: impl Fn(Dom)) {
        f(Dom::new());
        f(Dom::with_interning());
    }

    fn next_rand(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn tree_build_and_preorder_walk() {
        run_both(|mut dom| {
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
            dom.walk(|id, node| {
                let label = match &node.kind {
                    NodeKind::Document => "document".to_string(),
                    NodeKind::Element { .. } => dom.tag(id).unwrap_or("").to_string(),
                    NodeKind::Text(_) => dom.text(id).unwrap_or("").to_string(),
                };
                seen.push(label);
            });
            assert_eq!(
                seen,
                ["document", "html", "head", "title", "body", "p", "hello"]
            );
        });
    }

    #[test]
    fn append_reparent_detaches_old_parent() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn accessors_tag_attr_text() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn len_counts_nodes_and_is_empty_tracks_len() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn walk_visits_every_node_exactly_once() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn append_refuses_cycles_self_and_root_reparent() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn three_level_random_tree_parent_child_invariants() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn document_root_has_no_parent() {
        run_both(|dom| {
            assert_eq!(dom.parent(dom.root()), None);
            assert!(dom.children(dom.root()).is_empty());
            assert_eq!(dom.tag(dom.root()), None);
        });
    }

    fn count_walk(dom: &Dom) -> usize {
        let mut n = 0;
        dom.walk(|_, _| n += 1);
        n
    }

    fn collect_tags(dom: &Dom) -> Vec<String> {
        let mut tags = Vec::new();
        dom.walk(|id, node| {
            if matches!(node.kind, NodeKind::Element { .. }) {
                tags.push(dom.tag(id).unwrap_or("").to_string());
            }
        });
        tags
    }

    #[test]
    fn detach_subtree_removes_it_from_parent_and_walk() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn reused_slot_bumps_generation_and_invalidates_old_handle() {
        run_both(|mut dom| {
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
        });
    }

    #[test]
    fn validate_rejects_forged_and_internal_handles() {
        run_both(|mut dom| {
            assert!(!dom.validate(NodeId(u32::MAX)));
            assert!(!dom.validate(NodeId(SLOT_MASK)));
            let a = elem(&mut dom, "a");
            dom.detach_subtree(a);
            let internal_trash = NodeId::encode((dom.len() - 1) as u32, 0);
            assert!(!dom.validate(internal_trash));
            dom.append(dom.root(), internal_trash); // forged append is ignored
            assert!(dom.children(dom.root()).is_empty());
        });
    }

    #[test]
    fn parent_child_invariants_hold_after_detach_and_reuse() {
        run_both(|mut dom| {
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

    // ---- Interning-specific contracts -------------------------------------

    #[test]
    fn interner_roundtrip_same_string_same_index() {
        let mut pool = StringInterner::default();
        let a = pool.intern("class");
        let b = pool.intern("class");
        let c = pool.intern("id");
        assert_eq!(a, b, "interning the same string must return the same id");
        assert_ne!(a, c);
        assert_eq!(pool.get(a), "class");
        assert_eq!(pool.get(c), "id");
        assert_eq!(pool.unique_count(), 2);
        assert_eq!(pool.lookup("class"), Some(a));
        assert_eq!(pool.lookup("missing"), None);
    }

    #[test]
    fn same_class_doc_stores_far_fewer_unique_strings_than_elements() {
        let mut dom = Dom::with_interning();
        for i in 0..500 {
            let class = if i % 2 == 0 { "card" } else { "card wide" };
            let id = dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![("class".into(), class.into())],
            });
            dom.append(dom.root(), id);
        }
        assert_eq!(dom.len(), 501);
        assert!(
            dom.interner_len() < 10,
            "unique strings should be far below the 500 element count, got {}",
            dom.interner_len()
        );
        for i in 0..500 {
            let id = NodeId((i + 1) as u32); // slot 0 is the document root
            assert_eq!(dom.tag(id), Some("div"));
            assert_eq!(
                dom.attr(id, "class"),
                if i % 2 == 0 {
                    Some("card")
                } else {
                    Some("card wide")
                }
            );
        }
    }

    #[test]
    fn detach_and_slot_reuse_keeps_interner_consistent() {
        run_both(|mut dom| {
            let old = dom.create(NodeKind::Element {
                tag: "old".into(),
                attrs: vec![("class".into(), "gone".into())],
            });
            dom.append(dom.root(), old);
            dom.detach_subtree(old);

            let new = dom.create(NodeKind::Element {
                tag: "new".into(),
                attrs: vec![("id".into(), "fresh".into())],
            });
            assert_eq!(old.slot(), new.slot(), "slot must be reused");
            assert_eq!(dom.tag(new), Some("new"));
            assert_eq!(dom.attr(new, "id"), Some("fresh"));
            assert_eq!(
                dom.attr(new, "class"),
                None,
                "no stale attr from old occupant"
            );
            assert_eq!(dom.attr_interned(new, "class"), None);
        });
    }

    #[test]
    fn set_attr_keeps_both_modes_in_sync() {
        run_both(|mut dom| {
            let el = dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![("class".into(), "a".into())],
            });
            dom.set_attr(el, "class", "b".into());
            assert_eq!(dom.attr(el, "class"), Some("b"));
            dom.set_attr(el, "id", "x".into());
            assert_eq!(dom.attr(el, "id"), Some("x"));
            assert_eq!(dom.attr(el, "class"), Some("b"));

            let other = dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![("class".into(), "b".into())],
            });
            if dom.interning {
                assert_eq!(
                    dom.attr_interned(el, "class"),
                    dom.attr_interned(other, "class"),
                    "equal values must share an interned id"
                );
            }
        });
    }

    #[test]
    fn attr_interned_ids_are_stable_and_comparable() {
        let mut dom = Dom::with_interning();
        let a = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("class".into(), "x".into())],
        });
        let b = dom.create(NodeKind::Element {
            tag: "span".into(),
            attrs: vec![("class".into(), "x".into())],
        });
        let c = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("class".into(), "y".into())],
        });
        assert_eq!(dom.attr_interned(a, "class"), dom.attr_interned(b, "class"));
        assert_ne!(dom.attr_interned(a, "class"), dom.attr_interned(c, "class"));
        assert_eq!(dom.attr_interned(a, "missing"), None);

        let plain = Dom::new();
        assert_eq!(plain.interner_len(), 0, "default mode must not intern");
    }

    #[test]
    fn default_mode_keeps_node_string_fields() {
        // Byte-for-byte compatibility: `Dom::new()` still stores the strings in
        // the node, so direct `Node::kind` readers keep working.
        let mut dom = Dom::new();
        let id = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("class".into(), "card".into())],
        });
        match &dom.get(id).kind {
            NodeKind::Element { tag, attrs } => {
                assert_eq!(tag, "div");
                assert_eq!(attrs, &[("class".to_string(), "card".to_string())]);
            }
            _ => panic!("expected element"),
        }
        assert_eq!(dom.interner_len(), 0);
    }

    // ---- Micro A/B ---------------------------------------------------------

    #[test]
    fn perf_ab_interned_vs_plain() {
        fn run(intern: bool) -> (u128, u128, usize, usize) {
            let mut dom = if intern {
                Dom::with_interning()
            } else {
                Dom::new()
            };
            let build = Instant::now();
            for i in 0..2000 {
                let id = dom.create(NodeKind::Element {
                    tag: "div".into(),
                    attrs: vec![("class".into(), format!("s{}", i % 4))],
                });
                dom.append(dom.root(), id);
            }
            let build_us = build.elapsed().as_micros();

            let first = NodeId(1);
            let read = Instant::now();
            let mut total = 0usize;
            for _ in 0..10_000 {
                total += dom.attr(first, "class").map_or(0, str::len);
            }
            let read_us = read.elapsed().as_micros();
            assert_eq!(total, 10_000 * 2, "A/B must read a real value");

            // Heap attributable to strings/attrs. Plain pays one string buffer
            // per node plus a Vec<(String, String)> allocation per element.
            // Interned pays the pool plus one compact id per interned slot.
            let (string_bytes, side_bytes) = if intern {
                (
                    dom.interner_heap_bytes(),
                    dom.str_meta
                        .iter()
                        .map(|m| size_of::<Interned>() + m.attrs.len() * 8)
                        .sum::<usize>(),
                )
            } else {
                let mut payload = 0usize;
                let mut vec_alloc = 0usize;
                for n in &dom.nodes {
                    match &n.kind {
                        NodeKind::Element { tag, attrs } => {
                            payload += tag.len()
                                + attrs.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>();
                            vec_alloc += attrs.len() * size_of::<(String, String)>();
                        }
                        NodeKind::Text(t) => payload += t.len(),
                        NodeKind::Document => {}
                    }
                }
                (payload, vec_alloc)
            };
            (
                build_us,
                read_us,
                string_bytes + side_bytes,
                dom.interner_len(),
            )
        }

        let (plain_build, plain_read, plain_bytes, plain_unique) = run(false);
        let (intern_build, intern_read, intern_bytes, intern_unique) = run(true);
        println!(
            "A/B 2000 elements (class=s0..s3) + 10k reads (microseconds):\n\
             plain:    build {plain_build}us, reads {plain_read}us, heap ~{plain_bytes} bytes (string buffers + attrs Vecs), unique {plain_unique}\n\
             interned: build {intern_build}us, reads {intern_read}us, heap ~{intern_bytes} bytes (pool + compact ids), unique {intern_unique}"
        );
    }
}

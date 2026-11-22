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

/// Reserved tag for the node created by [`Dom::attach_shadow`].
///
/// Light-DOM views must filter this tag: JS `children`/`childNodes` and any
/// DOM-API view hide `#shadow-root` nodes. Rendering/layout instead reads the
/// flattened view through [`Dom::render_children`].
pub const SHADOW_ROOT_TAG: &str = "#shadow-root";

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
    /// Shadow root of a host node, parallel to `nodes` (same slot indices).
    /// `Some(root)` marks this node as a shadow host.
    shadow_roots: Vec<Option<NodeId>>,
    // ---- Cache-local relation core (parallel to `nodes`, indexed by slot) ---
    // Node.parent / Node.children stay the source of truth (public, frozen:
    // `children()` returns `&[NodeId]`, `get()` returns `&Node`). These three
    // packed arrays are a read-cache rebuilt at every mutation site; `walk`
    // uses them so a traversal never dereferences a per-node children Vec.
    // `parents` uses `root_id` as the "no parent" sentinel (the root is never
    // a child, so that value never denotes a real parent edge).
    //
    // INVARIANT: this is only authoritative for live slots. `detach_subtree`
    // rewrites the parent of the detached root but leaves its descendants'
    // parent links (packed *and* source) stale, pointing into the trash. Every
    // consumer must `validate()` the handle (generation + trashed check) before
    // trusting `parents[slot]`; `create()` resets the entry to `root_id` when it
    // recycles a trash slot. Never walk an unvalidated slot through this array.
    parents: Vec<NodeId>,
    first_child: Vec<Option<NodeId>>,
    next_sibling: Vec<Option<NodeId>>,
    /// Debug-only mutation count driving the sampled chain check below.
    /// Compiled out in release so wide-tree builds pay no sampling cost there.
    #[cfg(debug_assertions)]
    mutations: u64,
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
            shadow_roots: vec![None],
            parents: vec![NodeId(0)],
            first_child: vec![None],
            next_sibling: vec![None],
            #[cfg(debug_assertions)]
            mutations: 0,
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
            self.shadow_roots[slot as usize] = None;
            self.parents[slot as usize] = self.root_id;
            self.first_child[slot as usize] = None;
            self.next_sibling[slot as usize] = None;
            (slot, generation)
        } else {
            let slot = self.nodes.len() as u32;
            self.nodes.push(node);
            self.generations.push(0);
            self.trashed.push(false);
            self.str_meta.push(meta);
            self.shadow_roots.push(None);
            self.parents.push(self.root_id);
            self.first_child.push(None);
            self.next_sibling.push(None);
            (slot, 0)
        };
        let id = NodeId::encode(slot, generation);
        self.debug_assert_handle(id);
        id
    }

    /// Append `child` to the tail of `parent`'s child list, keeping the packed
    /// relation arrays in lock-step with the source-of-truth `children` Vec.
    /// The old tail is read from the Vec (append order), so no `last_child`
    /// array is needed and appends stay O(1).
    fn link_child(&mut self, parent: NodeId, child: NodeId) {
        let p = parent.slot() as usize;
        let c = child.slot() as usize;
        match self.nodes[p].children.last().copied() {
            None => self.first_child[p] = Some(child),
            Some(last) => self.next_sibling[last.slot() as usize] = Some(child),
        }
        self.next_sibling[c] = None;
        self.parents[c] = parent;
        self.nodes[p].children.push(child);
        self.nodes[c].parent = Some(parent);
        #[cfg(debug_assertions)]
        self.check_chain_sampled(parent);
    }

    /// Remove `child` from `parent`'s child list (both the Vec and the packed
    /// chain). The chain walk is O(#children), the same order as `Vec::retain`.
    fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        let p = parent.slot() as usize;
        let c = child.slot() as usize;
        let mut cur = self.first_child[p];
        let mut prev: Option<NodeId> = None;
        while let Some(x) = cur {
            if x == child {
                let next = self.next_sibling[x.slot() as usize];
                match prev {
                    None => self.first_child[p] = next,
                    Some(pr) => self.next_sibling[pr.slot() as usize] = next,
                }
                self.next_sibling[c] = None;
                break;
            }
            prev = Some(x);
            cur = self.next_sibling[x.slot() as usize];
        }
        self.nodes[p].children.retain(|x| *x != child);
        #[cfg(debug_assertions)]
        self.check_chain_sampled(parent);
    }

    /// Debug-only: number of nodes in `parent`'s packed sibling chain.
    #[cfg(debug_assertions)]
    fn chain_len(&self, parent: NodeId) -> usize {
        let mut n = 0;
        let mut cur = self.first_child[parent.slot() as usize];
        while let Some(x) = cur {
            n += 1;
            cur = self.next_sibling[x.slot() as usize];
        }
        n
    }

    /// Sampling rule: every mutation for the first 1024, then only when
    /// `mutations` is a power of two. Total chain-walk work stays O(n) even for
    /// a 100k-sibling build, while small trees still check on every mutation.
    #[cfg(debug_assertions)]
    fn check_chain_sampled(&mut self, parent: NodeId) {
        self.mutations = self.mutations.wrapping_add(1);
        if self.mutations < 1024 || self.mutations.is_power_of_two() {
            let expected = self.nodes[parent.slot() as usize].children.len();
            debug_assert_eq!(self.chain_len(parent), expected);
        }
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
            self.remove_child(parent, root);
        }
        self.nodes[root_slot].parent = None;
        self.parents[root_slot] = self.root_id;

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
        // The trash is unreachable from `walk`, so only the source-of-truth
        // fields are updated here. Its packed chain is deliberately left alone:
        // trashed slots are recycled, and stale chain links into a reused slot
        // would be harmless but would trip the debug chain-length assertion.
        let t = trash.slot() as usize;
        self.nodes[root_slot].parent = Some(trash);
        self.parents[root_slot] = trash;
        self.nodes[t].children.push(root);
    }

    /// Remove `id` from its parent without re-attaching it anywhere.
    ///
    /// The node stays live (its slot is neither trashed nor recycled), so this
    /// is the "drop" path for callers whose capacity budget forbids growing a
    /// trash tree: the subtree simply becomes unreachable from the root.
    pub fn detach_orphan(&mut self, id: NodeId) {
        if !self.validate(id) {
            return;
        }
        let slot = id.slot() as usize;
        if let Some(parent) = self.nodes[slot].parent {
            self.remove_child(parent, id);
        }
        self.nodes[slot].parent = None;
        self.parents[slot] = self.root_id;
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
        // Walk the packed parent array: compact, no per-node children Vec touched.
        // INVARIANT: trashed descendants keep stale packed parents, so a packed
        // parent is trustworthy only after `validate()` (done above for both
        // endpoints). A live node never has a trashed ancestor, so this climb
        // stays on live slots and cannot follow a stale link into the trash;
        // `create()` resets each recycled slot's parent on reuse. Do not extend
        // this walk to unvalidated handles.
        let mut cursor = parent;
        loop {
            if cursor == child {
                return;
            }
            if cursor == self.root_id {
                break;
            }
            cursor = self.parents[cursor.slot() as usize];
        }
        let c = child.slot() as usize;
        if let Some(old) = self.nodes[c].parent {
            self.remove_child(old, child);
        }
        self.link_child(parent, child);
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

    /// Give `host` a shadow root, returning the new root node.
    ///
    /// The root is parented *under* `host`, so [`Dom::walk`] reaches shadow
    /// content for free. A host can only be attached once: further calls return
    /// `None`. Invalid hosts and arena-cap exhaustion also return `None`.
    pub fn attach_shadow(&mut self, host: NodeId) -> Option<NodeId> {
        if !self.validate(host) || self.shadow_root(&host).is_some() {
            return None;
        }
        let root = self.create(NodeKind::Element {
            tag: SHADOW_ROOT_TAG.to_string(),
            attrs: Vec::new(),
        });
        // `create` hands back the document root when the arena cap is exhausted.
        if root == self.root_id {
            return None;
        }
        self.shadow_roots[host.slot() as usize] = Some(root);
        self.append(host, root);
        Some(root)
    }

    /// The shadow root attached to `id`, if any.
    pub fn shadow_root(&self, id: &NodeId) -> Option<NodeId> {
        self.shadow_roots.get(id.slot() as usize).copied().flatten()
    }

    /// The host of a shadow root, if `root` is one.
    pub fn host(&self, root: &NodeId) -> Option<NodeId> {
        if !self.validate(*root) || self.tag(*root) != Some(SHADOW_ROOT_TAG) {
            return None;
        }
        self.parent(*root)
    }

    /// Children in render order: the shadow subtree's children when `id` is a
    /// shadow host, otherwise its plain children.
    ///
    /// The host's light children — including the `#shadow-root` node itself —
    /// are skipped for hosts; a shadow root's own children render normally.
    /// Light-DOM views (JS `children`/`childNodes`, DOM APIs) instead hide
    /// `#shadow-root`; layout uses this method to see shadow content.
    pub fn render_children(&self, id: NodeId) -> &[NodeId] {
        match self.shadow_root(&id) {
            Some(root) => &self.nodes[root.slot() as usize].children,
            None => &self.nodes[id.slot() as usize].children,
        }
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
    ///
    /// Pre-order DFS over the packed sibling links with a small explicit stack.
    /// Only one continuation per depth is held, so the stack never grows with
    /// sibling count — a 10k-child flat tree keeps it at a single entry.
    pub fn walk(&self, mut f: impl FnMut(NodeId, &Node)) {
        self.debug_assert_handle(self.root_id);
        f(self.root_id, self.get(self.root_id));
        let mut stack = Vec::with_capacity(64);
        if let Some(first) = self.first_child[self.root_id.slot() as usize] {
            stack.push(first);
        }
        while let Some(id) = stack.pop() {
            self.debug_assert_handle(id);
            let node = self.get(id);
            f(id, node);
            let slot = id.slot() as usize;
            // LIFO: push the sibling first so the child (and its subtree) is
            // visited before it, preserving the frozen append-order walk.
            if let Some(sib) = self.next_sibling[slot] {
                stack.push(sib);
            }
            if let Some(child) = self.first_child[slot] {
                stack.push(child);
            }
        }
    }

    /// Original recursive traversal, kept only so the micro-bench can A/B the
    /// two walk strategies in one process (no cross-run machine drift).
    #[cfg(test)]
    fn walk_baseline(&self, mut f: impl FnMut(NodeId, &Node)) {
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
        self.shadow_roots.push(None);
        self.parents.push(self.root_id);
        self.first_child.push(None);
        self.next_sibling.push(None);
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
        debug_assert!(
            slot < self.shadow_roots.len(),
            "shadow side table out of sync"
        );
        debug_assert!(slot < self.parents.len(), "parent array out of sync");
        debug_assert!(
            slot < self.first_child.len(),
            "first_child array out of sync"
        );
        debug_assert!(
            slot < self.next_sibling.len(),
            "next_sibling array out of sync"
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

    // ---- Shadow DOM --------------------------------------------------------

    #[test]
    fn attach_shadow_creates_parented_root_and_is_idempotent() {
        run_both(|mut dom| {
            let host = elem(&mut dom, "my-widget");
            dom.append(dom.root(), host);
            assert_eq!(dom.shadow_root(&host), None);

            let root = dom.attach_shadow(host).expect("first attach succeeds");
            assert!(dom.validate(root));
            assert_eq!(dom.tag(root), Some(SHADOW_ROOT_TAG));
            assert_eq!(dom.parent(root), Some(host), "root parented under its host");
            assert!(
                dom.children(host).contains(&root),
                "host light children include the shadow-root node"
            );
            assert_eq!(dom.shadow_root(&host), Some(root));
            assert_eq!(dom.host(&root), Some(host));
            assert_eq!(dom.host(&host), None, "a host is not itself a shadow root");

            assert_eq!(dom.attach_shadow(host), None, "second attach returns None");
            assert_eq!(dom.shadow_root(&host), Some(root), "root is unchanged");
        });
    }

    #[test]
    fn shadow_host_root_roundtrip_and_invalid_inputs() {
        run_both(|mut dom| {
            let host = elem(&mut dom, "host");
            dom.append(dom.root(), host);
            let root = dom.attach_shadow(host).unwrap();
            assert_eq!(dom.shadow_root(&host), Some(root));
            assert_eq!(dom.host(&root), Some(host));
            assert_eq!(
                dom.shadow_root(&root),
                None,
                "a shadow root has no shadow root"
            );
            assert_eq!(dom.host(&host), None);
            assert_eq!(dom.shadow_root(&dom.root()), None);
            assert_eq!(dom.attach_shadow(NodeId(u32::MAX)), None, "invalid host");
            assert_eq!(dom.host(&NodeId(u32::MAX)), None, "invalid root handle");
        });
    }

    #[test]
    fn render_children_prefers_shadow_content_and_walk_reaches_it() {
        run_both(|mut dom| {
            let host = elem(&mut dom, "my-widget");
            let light = dom.create(NodeKind::Text("light".into()));
            dom.append(dom.root(), host);
            dom.append(host, light);

            let root = dom.attach_shadow(host).unwrap();
            let shadow_child = elem(&mut dom, "shadow-child");
            let shadow_text = dom.create(NodeKind::Text("shadow text".into()));
            dom.append(root, shadow_child);
            dom.append(root, shadow_text);

            // Host renders the shadow subtree's children: light children and the
            // #shadow-root node itself are skipped.
            assert_eq!(dom.render_children(host), &[shadow_child, shadow_text]);
            assert_eq!(
                dom.children(host),
                &[light, root],
                "plain children unchanged"
            );
            // Non-hosts, including the root and its children, render plainly.
            assert_eq!(dom.render_children(root), &[shadow_child, shadow_text]);
            assert_eq!(
                dom.render_children(shadow_child),
                dom.children(shadow_child)
            );
            assert_eq!(dom.render_children(light), dom.children(light));

            // The trick's payoff: walk() sees shadow content because the root is
            // parented under host.
            let mut tags = Vec::new();
            let mut texts = Vec::new();
            dom.walk(|id, _| {
                if let Some(t) = dom.tag(id) {
                    tags.push(t.to_string());
                }
                if let Some(t) = dom.text(id) {
                    texts.push(t.to_string());
                }
            });
            assert!(tags.contains(&SHADOW_ROOT_TAG.to_string()));
            assert!(tags.contains(&"shadow-child".to_string()));
            assert!(texts.contains(&"shadow text".to_string()));
            assert!(texts.contains(&"light".to_string()));
        });
    }

    #[test]
    fn shadow_slot_reuse_after_trash_keeps_side_table_consistent() {
        run_both(|mut dom| {
            let host = elem(&mut dom, "host");
            dom.append(dom.root(), host);
            let root = dom.attach_shadow(host).unwrap();
            assert_eq!(dom.shadow_root(&host), Some(root));

            dom.detach_subtree(root);
            assert!(!dom.validate(root));

            let reused = elem(&mut dom, "reused");
            assert_eq!(reused.slot(), root.slot(), "create reuses the trashed slot");
            assert_eq!(
                dom.shadow_root(&reused),
                None,
                "reused slot must not inherit a stale shadow root"
            );
            assert_eq!(dom.host(&reused), None);
            assert_eq!(dom.tag(reused), Some("reused"));
        });
    }

    #[test]
    fn shadow_fixture_is_identical_in_both_storage_modes() {
        fn snapshot(
            dom: &Dom,
            host: NodeId,
            root: NodeId,
            child: NodeId,
            text: NodeId,
        ) -> Vec<String> {
            let mut out = vec![
                dom.tag(host).unwrap_or("").to_string(),
                dom.tag(root).unwrap_or("").to_string(),
                dom.tag(child).unwrap_or("").to_string(),
                dom.attr(child, "class").unwrap_or("").to_string(),
                dom.text(text).unwrap_or("").to_string(),
            ];
            out.extend(
                dom.render_children(host)
                    .iter()
                    .map(|&id| dom.tag(id).unwrap_or("").to_string()),
            );
            out
        }
        fn build(intern: bool) -> (Dom, NodeId, NodeId, NodeId, NodeId) {
            let mut dom = if intern {
                Dom::with_interning()
            } else {
                Dom::new()
            };
            let host = elem(&mut dom, "host");
            dom.append(dom.root(), host);
            let root = dom.attach_shadow(host).unwrap();
            let child = elem(&mut dom, "item");
            dom.set_attr(child, "class", "x".into());
            let text = dom.create(NodeKind::Text("shadow".into()));
            dom.append(root, child);
            dom.append(child, text);
            (dom, host, root, child, text)
        }
        let (plain, h1, r1, c1, t1) = build(false);
        let (interned, h2, r2, c2, t2) = build(true);
        assert_eq!(
            snapshot(&plain, h1, r1, c1, t1),
            snapshot(&interned, h2, r2, c2, t2)
        );
    }

    // ---- Packed-relation cache invariants ----------------------------------

    /// Walk the packed sibling chain directly; must always equal `children()`.
    fn cached_chain(dom: &Dom, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = dom.first_child[id.slot() as usize];
        while let Some(c) = cur {
            out.push(c);
            cur = dom.next_sibling[c.slot() as usize];
        }
        out
    }

    fn assert_cache_matches_children(dom: &Dom) {
        for slot in 0..dom.len() {
            if dom.trashed[slot] {
                continue; // trash is unreachable and intentionally not chained
            }
            let id = NodeId::encode(slot as u32, dom.generations[slot]);
            assert_eq!(
                cached_chain(dom, id),
                dom.children(id),
                "packed sibling chain diverged from children() at slot {slot}"
            );
        }
    }

    /// Release-real packed-cache check. The `debug_assert_eq!` in `link_child`
    /// / `remove_child` is compiled out in release; this uses `assert_eq!` so it
    /// still fails fast under `cargo test --release`. Trashed slots are skipped
    /// (their packed links are stale by design, see the `parents` invariant).
    fn assert_packed_consistent(dom: &Dom) {
        for slot in 0..dom.len() {
            if dom.trashed[slot] {
                continue;
            }
            let id = NodeId::encode(slot as u32, dom.generations[slot]);
            assert_eq!(
                cached_chain(dom, id),
                dom.children(id),
                "packed sibling chain diverged from children() at slot {slot}"
            );
            let expected = dom.nodes[slot].parent.unwrap_or(dom.root_id);
            assert_eq!(
                dom.parents[slot], expected,
                "packed parent diverged from source parent at slot {slot}"
            );
        }
    }

    #[test]
    fn soa_cache_tracks_children_through_reparent_detach_and_reuse() {
        run_both(|mut dom| {
            let a = elem(&mut dom, "a");
            let b = elem(&mut dom, "b");
            let c = elem(&mut dom, "c");
            let d = elem(&mut dom, "d");
            dom.append(dom.root(), a);
            dom.append(dom.root(), b);
            dom.append(a, c);
            assert_cache_matches_children(&dom);

            dom.append(b, c); // reparent: unlink from a, append under b
            assert_cache_matches_children(&dom);
            dom.append(b, c); // no-op re-append must not duplicate
            assert_cache_matches_children(&dom);
            dom.append(b, d);
            assert_cache_matches_children(&dom);

            dom.detach_subtree(b); // unlink b (and its subtree) from the root
            assert_cache_matches_children(&dom);

            let reused = elem(&mut dom, "reused"); // reuses a trashed slot
            dom.append(a, reused);
            assert_cache_matches_children(&dom);
            assert_eq!(dom.children(a), &[reused]);
        });
    }

    // ---- Release-real packed-consistency regression coverage ---------------

    #[test]
    fn packed_consistency_interleaved_append_remove_release() {
        run_both(|mut dom| {
            let a = elem(&mut dom, "a");
            let b = elem(&mut dom, "b");
            let c = elem(&mut dom, "c");
            dom.append(dom.root(), a);
            dom.append(dom.root(), b);
            dom.append(dom.root(), c);
            assert_eq!(dom.children(dom.root()), &[a, b, c]);

            dom.detach_subtree(b); // remove the middle child
            assert_eq!(dom.children(dom.root()), &[a, c]);
            assert_packed_consistent(&dom);
            assert_cache_matches_children(&dom);

            let d = elem(&mut dom, "d"); // reuses b's slot
            dom.append(dom.root(), d);
            assert_eq!(dom.children(dom.root()), &[a, c, d]);
            assert_packed_consistent(&dom);
            assert_cache_matches_children(&dom);
        });
    }

    #[test]
    fn packed_consistency_detach_reuse_attach_shadow_release() {
        run_both(|mut dom| {
            let host = elem(&mut dom, "host");
            dom.append(dom.root(), host);
            dom.detach_subtree(host);

            let reused = elem(&mut dom, "reused");
            assert_eq!(reused.slot(), host.slot(), "create reuses the trashed slot");
            dom.append(dom.root(), reused);

            let shadow = dom
                .attach_shadow(reused)
                .expect("shadow attaches on a reused slot");
            assert!(dom.validate(shadow));
            assert_eq!(dom.parent(shadow), Some(reused));
            assert_eq!(dom.children(reused), &[shadow]);
            assert_packed_consistent(&dom);
            assert_cache_matches_children(&dom);

            let mut visited = Vec::new();
            dom.walk(|id, _| visited.push(id));
            assert_eq!(visited, vec![dom.root(), reused, shadow]);
            assert_packed_consistent(&dom);
        });
    }

    #[test]
    fn packed_consistency_deep_chain_three_splices_release() {
        run_both(|mut dom| {
            const DEPTH: usize = 100;
            let chain: Vec<NodeId> = (0..DEPTH)
                .map(|i| elem(&mut dom, &format!("n{i}")))
                .collect();
            let mut parent = dom.root();
            for &id in &chain {
                dom.append(parent, id);
                parent = id;
            }
            assert_packed_consistent(&dom);

            // Splice three middle levels out at different depths: re-parent the
            // child under the grandparent, then re-home the removed level under
            // the root so every node stays live and generation-checked.
            for &depth in &[30usize, 60, 90] {
                let grandparent = chain[depth - 2];
                let removed = chain[depth - 1];
                let child = chain[depth];
                dom.append(grandparent, child);
                dom.append(dom.root(), removed);
                assert_packed_consistent(&dom);
                assert_cache_matches_children(&dom);
            }

            let spliced = [chain[29], chain[59], chain[89]];
            let expected: Vec<NodeId> = chain
                .iter()
                .copied()
                .filter(|id| !spliced.contains(id))
                .collect();
            let mut actual = Vec::new();
            let mut cursor = Some(chain[0]);
            while let Some(id) = cursor {
                actual.push(id);
                cursor = dom.children(id).first().copied();
            }
            assert_eq!(actual, expected, "main chain must skip the spliced levels");
            assert_eq!(
                dom.children(chain[28]),
                &[chain[30]],
                "packed order must skip the removed depth-30 level"
            );
            assert_packed_consistent(&dom);
            assert_cache_matches_children(&dom);
        });
    }

    // ---- Micro bench: 10k-node walk + child iteration ----------------------

    /// Deterministic parser-shaped tree: 10k elements, each node (bar the
    /// first few levels) with up to `branch` sequentially-created children.
    fn build_bench_tree(n: usize, branch: usize) -> (Dom, Vec<NodeId>) {
        let mut dom = Dom::new();
        let ids: Vec<NodeId> = (0..n).map(|_| elem(&mut dom, "n")).collect();
        for i in 0..n {
            let parent = if i == 0 {
                dom.root()
            } else {
                ids[(i - 1) / branch]
            };
            dom.append(parent, ids[i]);
        }
        (dom, ids)
    }

    /// Build `n` siblings under one parent and time only the append phase.
    /// Node creation is excluded so the measurement isolates mutation cost.
    /// Best of 5 keeps the ~1ms release timings from being scheduler noise.
    fn build_flat_tree(n: usize) -> u128 {
        let mut best = u128::MAX;
        for _ in 0..5 {
            let mut dom = Dom::new();
            let root = dom.root();
            let ids: Vec<NodeId> = (0..n).map(|_| elem(&mut dom, "n")).collect();
            let t = Instant::now();
            for id in ids {
                dom.append(root, id);
            }
            best = best.min(t.elapsed().as_nanos());
            assert_eq!(dom.children(root).len(), n);
            assert_eq!(dom.len(), n + 1);
        }
        best
    }

    /// Regression guard for the sampled debug chain check: a wide-tree build
    /// must stay near-linear. The old per-op `chain_len` walk was O(n²), so the
    /// 100k/50k append-time ratio was ~4x; with sampling it must be ~2x.
    #[test]
    fn perf_wide_tree_build_is_linear() {
        let _ = build_flat_tree(1_000); // warm up allocator and code paths
        let t50 = build_flat_tree(50_000);
        let t100 = build_flat_tree(100_000);
        let ratio = t100 as f64 / t50.max(1) as f64;
        println!(
            "wide build: 50k {t50} ns, 100k {t100} ns, ratio {ratio:.2}x (linear ~2x, O(n²) ~4x)"
        );
        // Generous slack for noise; the O(n²) baseline measured 4.01x in debug.
        assert!(
            ratio < 3.0,
            "100k/50k build ratio {ratio:.2}x suggests superlinear chain checks"
        );
        // Absolute budget also catches the old 31s debug build directly.
        assert!(
            t100 < 5_000_000_000,
            "100k build took {} ms",
            t100 / 1_000_000
        );
    }

    /// One op = one full `walk` + one full pass over every node's `children()`.
    /// Both the original recursive walk and the current walk are timed in one
    /// process (best of 7 release-mode runs each), so the ratio is honest.
    #[test]
    fn perf_dom_walk_and_children() {
        const N: usize = 10_000;
        const BRANCH: usize = 8;
        const REPS: u32 = 200;
        let (dom, ids) = build_bench_tree(N, BRANCH);
        assert_eq!(dom.len(), N + 1);

        fn bench(dom: &Dom, ids: &[NodeId], baseline: bool, sink: &mut usize) -> u128 {
            let mut best = u128::MAX;
            for _ in 0..7 {
                let t = Instant::now();
                let mut local = 0usize;
                for _ in 0..REPS {
                    if baseline {
                        dom.walk_baseline(|id, _| local += id.slot() as usize);
                    } else {
                        dom.walk(|id, _| local += id.slot() as usize);
                    }
                    for &id in ids {
                        local += dom.children(id).len();
                    }
                }
                best = best.min(t.elapsed().as_nanos());
                *sink += local;
            }
            best / REPS as u128
        }

        let mut sink = 0usize;
        // Interleave warmup and two A/B rounds to cancel drift.
        for _ in 0..10 {
            dom.walk_baseline(|id, _| sink += id.slot() as usize);
            dom.walk(|id, _| sink += id.slot() as usize);
        }
        let base_a = bench(&dom, &ids, true, &mut sink);
        let new_a = bench(&dom, &ids, false, &mut sink);
        let new_b = bench(&dom, &ids, false, &mut sink);
        let base_b = bench(&dom, &ids, true, &mut sink);
        let base = base_a.min(base_b);
        let new = new_a.min(new_b);
        println!(
            "DOM bench: {} nodes (branch {}), {} ops: baseline {} ns/op, current {} ns/op, {:.1}% change",
            N,
            BRANCH,
            REPS,
            base,
            new,
            (new as f64 / base as f64 - 1.0) * 100.0
        );
        assert!(sink > 0, "bench must do real work");
        // The two walks must visit exactly the same set of nodes.
        let mut a = Vec::new();
        let mut b = Vec::new();
        dom.walk_baseline(|id, _| a.push(id));
        dom.walk(|id, _| b.push(id));
        assert_eq!(a, b, "walk strategies must agree on order and membership");
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

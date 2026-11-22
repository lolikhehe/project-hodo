//! Bounded measurement cache shared by the recursive layout walk.
//!
//! Reruns are the hot path on real pages: nav menus, table cells and repeated
//! cards shape the same text at the same width many times per layout. This
//! module memoises the *result* of shaping one self-contained text run so a
//! rerun can replay the finished line boxes instead of re-scanning words.
//!
//! RAM budget (documented honestly): at most [`MEASURE_CACHE_MAX_ENTRIES`]
//! entries and [`MEASURE_CACHE_MAX_BYTES`] of estimated payload. A key is 56
//! bytes; the average entry (a handful of line strings) is a few hundred
//! bytes. The byte cap is the binding constraint and keeps the whole cache
//! under 256 KiB even in the pathological case. Oversized single runs are
//! simply not cached; on overflow the cache is flushed wholesale (O(1)) instead
//! of scanning every entry on each insert.

use std::collections::HashMap;

use hodo_dom::Dom;

/// Entry-count ceiling. The byte budget below is the real bound.
pub const MEASURE_CACHE_MAX_ENTRIES: usize = 4096;
/// 192 KiB payload ceiling; with HashMap/Vec overhead the cache stays below
/// the 256 KiB target.
pub const MEASURE_CACHE_MAX_BYTES: usize = 192 * 1024;
/// Estimated per-entry cost of the key plus its HashMap slot, added to the
/// plan's own estimate so the published byte cap covers both.
const ENTRY_OVERHEAD: usize = 128;

/// 64-bit FNV-1a step, used for style/href fingerprints and the cheap
/// document-generation fold. Text uses [`hash_text`] (two independent lanes).
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Two independent FNV-1a lanes over the exact bytes of a text run.
/// Colliding runs would only share a cached shape, but 128 bits makes that
/// effectively impossible for browser-sized documents.
pub(crate) fn hash_text(text: &str) -> u128 {
    let mut a = 0xcbf2_9ce4_8422_2325u64;
    let mut b = 0x9e37_79b9_7f4a_7c15u64;
    for &byte in text.as_bytes() {
        a = (a ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
        b = (b ^ u64::from(byte))
            .wrapping_mul(FNV_PRIME)
            .rotate_left(17);
    }
    (u128::from(b) << 64) | u128::from(a)
}

fn fold(mut h: u64, bytes: impl Iterator<Item = u64>) -> u64 {
    for x in bytes {
        h = (h ^ x).wrapping_mul(FNV_PRIME);
    }
    h
}

/// The shaping inputs that change a run's line breaks or emitted geometry.
/// Everything else (owner, colour, weight, href, inline boxes) is applied at
/// replay time from the caller's current template and is folded into
/// `style_bits` only so a stale run can never be replayed with different paint.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct MeasureKey {
    text_hash: u128,
    text_len: u32,
    font_size: u32,
    char_w: u32,
    space_w: u32,
    line_h: u32,
    avail_w: u32,
    white_space: u8,
    text_align: u8,
    style_bits: u64,
    generation: u64,
}

impl MeasureKey {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        text: &str,
        font_size: f32,
        char_w: f32,
        space_w: f32,
        line_h: f32,
        avail_w: f32,
        white_space: u8,
        text_align: u8,
        style_bits: u64,
        generation: u64,
    ) -> Self {
        Self {
            text_hash: hash_text(text),
            text_len: text.len() as u32,
            font_size: font_size.to_bits(),
            char_w: char_w.to_bits(),
            space_w: space_w.to_bits(),
            line_h: line_h.to_bits(),
            avail_w: avail_w.to_bits(),
            white_space,
            text_align,
            style_bits,
            generation,
        }
    }
}

/// One already-shaped line, ready to be pushed as a `DisplayItem::Text`.
#[derive(Clone, Debug)]
pub(crate) struct CachedLine {
    pub(crate) text: String,
    pub(crate) w: f32,
    pub(crate) h: f32,
    /// x offset from the content edge (text-align shift included).
    pub(crate) shift: f32,
}

/// One atom of the still-open line left after a run. Kept as atoms (not a
/// pre-built item) so a following inline sibling can join the same line box.
#[derive(Clone, Debug)]
pub(crate) struct PendingAtom {
    pub(crate) text: String,
    pub(crate) w: f32,
    pub(crate) spacer: bool,
}

/// Complete result of shaping a text run: line count, total height, line
/// width, plus the ready-made lines needed to replay it byte-identically.
#[derive(Clone, Debug)]
pub(crate) struct TextRunPlan {
    pub(crate) lines: Vec<CachedLine>,
    pub(crate) pending: Vec<PendingAtom>,
    pub(crate) pending_line_w: f32,
    pub(crate) pending_space: Option<f32>,
    pub(crate) line_count: usize,
    pub(crate) height: f32,
    pub(crate) width: f32,
    /// Estimated heap cost for the byte budget.
    bytes: usize,
}

impl TextRunPlan {
    pub(crate) fn new(
        lines: Vec<CachedLine>,
        pending: Vec<PendingAtom>,
        pending_line_w: f32,
        pending_space: Option<f32>,
    ) -> Self {
        let line_count = lines.len() + usize::from(!pending.is_empty());
        let height = lines.iter().map(|l| l.h).sum();
        let width = lines.iter().map(|l| l.w).fold(pending_line_w, f32::max);
        let bytes = std::mem::size_of::<Self>()
            + lines.iter().map(|l| l.text.len() + 64).sum::<usize>()
            + pending.iter().map(|a| a.text.len() + 64).sum::<usize>();
        Self {
            lines,
            pending,
            pending_line_w,
            pending_space,
            line_count,
            height,
            width,
            bytes,
        }
    }

    #[cfg(test)]
    pub(crate) fn line_count(&self) -> usize {
        self.line_count
    }
    #[cfg(test)]
    pub(crate) fn height(&self) -> f32 {
        self.height
    }
    #[cfg(test)]
    pub(crate) fn width(&self) -> f32 {
        self.width
    }
}

/// Bounded LRU of text-run shapes. `generation` is an epoch bumped by the
/// integrator (or derived from hodo-dom's per-slot generation bits) whenever
/// the DOM or stylesheet changes; a bump logically invalidates every entry.
pub struct MeasureCache {
    entries: HashMap<MeasureKey, (TextRunPlan, u64)>,
    stamp: u64,
    generation: u64,
    bytes: usize,
    hits: u64,
    misses: u64,
    /// `false` for the benchmark's cold baseline: all lookups miss and no
    /// shape is recorded, so the only difference is the replay itself.
    enabled: bool,
}

impl Default for MeasureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MeasureCache {
    pub fn new() -> Self {
        Self::with_enabled(true)
    }

    /// A cache-shaped no-op used to measure the uncached baseline fairly.
    pub fn new_cold() -> Self {
        Self::with_enabled(false)
    }

    fn with_enabled(enabled: bool) -> Self {
        Self {
            entries: HashMap::new(),
            stamp: 0,
            generation: 0,
            bytes: 0,
            hits: 0,
            misses: 0,
            enabled,
        }
    }

    /// Current epoch. Entries are only valid for this generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Bump the epoch and drop every stored shape. Call once per DOM/style
    /// mutation transaction when a cache is shared across layout calls.
    pub fn set_generation(&mut self, generation: u64) {
        if generation != self.generation {
            self.generation = generation;
            self.clear();
        }
    }

    /// Convenience: advance the epoch by one.
    pub fn bump_generation(&mut self) {
        self.set_generation(self.generation.wrapping_add(1));
    }

    /// Fold hodo-dom's per-slot generation bits (bumped on detach/reuse) and
    /// each node's child count into a document-generation stamp. Structural
    /// mutations change the stamp; in-place text/attribute edits are already
    /// covered by each run's content fingerprint. O(DOM nodes) — call once per
    /// mutation, not per layout.
    pub fn observe_dom(&mut self, dom: &Dom) {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        dom.walk(|id, node| {
            h = fold(h, [u64::from(id.0), node.children.len() as u64].into_iter());
        });
        self.set_generation(h);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Estimated payload bytes (always <= [`MEASURE_CACHE_MAX_BYTES`]).
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub(crate) fn get(&mut self, key: &MeasureKey) -> Option<&TextRunPlan> {
        if !self.enabled {
            return None;
        }
        match self.entries.get_mut(key) {
            Some((plan, stamp)) => {
                *stamp = self.stamp;
                self.stamp = self.stamp.wrapping_add(1);
                self.hits += 1;
                Some(plan)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    pub(crate) fn insert(&mut self, key: MeasureKey, plan: TextRunPlan) {
        let entry_bytes = plan.bytes.saturating_add(ENTRY_OVERHEAD);
        if !self.enabled || entry_bytes > MEASURE_CACHE_MAX_BYTES {
            return;
        }
        // Bulk eviction on overflow: O(1) instead of scanning every entry on
        // each insert (a 10k-cell table otherwise spends seconds in eviction).
        // The hot path is reruns of identical runs, which this preserves.
        if self.entries.len() >= MEASURE_CACHE_MAX_ENTRIES
            || self.bytes.saturating_add(entry_bytes) > MEASURE_CACHE_MAX_BYTES
        {
            self.clear();
        }
        self.bytes += entry_bytes;
        self.entries.insert(key, (plan, self.stamp));
        self.stamp = self.stamp.wrapping_add(1);
    }

    /// Fold a colour, weight, href and flags into the key's paint tag.
    pub(crate) fn style_bits(
        color: hodo_types::Color,
        weight: u16,
        href: Option<&str>,
        underline: bool,
        line_through: bool,
    ) -> u64 {
        let mut h = fold(
            0,
            [
                u64::from(color.r)
                    | (u64::from(color.g) << 8)
                    | (u64::from(color.b) << 16)
                    | (u64::from(color.a) << 24),
                u64::from(weight),
                u64::from(underline),
                u64::from(line_through),
            ]
            .into_iter(),
        );
        if let Some(href) = href {
            h = fold(h, href.as_bytes().iter().map(|&b| u64::from(b)));
            h = fold(h, [href.len() as u64].into_iter());
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_records_line_count_height_and_width() {
        use crate::cache::CachedLine;

        let plan = TextRunPlan::new(
            vec![
                CachedLine {
                    text: "aa".to_string(),
                    w: 20.0,
                    h: 12.0,
                    shift: 0.0,
                },
                CachedLine {
                    text: "bb".to_string(),
                    w: 12.0,
                    h: 12.0,
                    shift: 0.0,
                },
            ],
            vec![],
            0.0,
            None,
        );
        assert_eq!(plan.line_count(), 2);
        assert_eq!(plan.height(), 24.0);
        assert_eq!(plan.width(), 20.0);
    }

    #[test]
    fn cache_hit_miss_and_bounded_bytes() {
        let mut cache = MeasureCache::new();
        let key = MeasureKey::new("hello world", 16.0, 9.6, 9.6, 19.2, 100.0, 0, 0, 0, 0);
        let plan = TextRunPlan::new(vec![], vec![], 0.0, None);
        assert!(cache.get(&key).is_none(), "cold run must miss");
        assert_eq!(cache.misses(), 1);
        cache.insert(key, plan);
        assert!(cache.get(&key).is_some(), "same shape must hit");
        assert_eq!(cache.hits(), 1);
        let other = MeasureKey::new("hello world", 16.0, 9.6, 9.6, 19.2, 101.0, 0, 0, 0, 0);
        assert!(cache.get(&other).is_none(), "different width must miss");
        assert!(cache.bytes() <= MEASURE_CACHE_MAX_BYTES);
        assert!(cache.len() <= MEASURE_CACHE_MAX_ENTRIES);
    }

    #[test]
    fn generation_bump_invalidates_all_entries() {
        let mut cache = MeasureCache::new();
        let key = MeasureKey::new("x", 16.0, 9.6, 9.6, 19.2, 100.0, 0, 0, 0, 0);
        cache.insert(key, TextRunPlan::new(vec![], vec![], 0.0, None));
        assert!(cache.get(&key).is_some());
        cache.bump_generation();
        assert!(cache.is_empty(), "a new generation drops all shapes");
        assert!(cache.get(&key).is_none(), "old generation cannot hit");
    }

    /// 200 identical-shaped runs in one page: the second layout shares the
    /// cache and should replay their line boxes instead of re-scanning words.
    /// Timings are printed with `--nocapture`.
    #[test]
    fn measure_cache_warm_layout_is_faster() {
        use std::time::Instant;

        use hodo_dom::NodeKind;
        use hodo_style::compute_styles;
        use hodo_types::Size;

        const RUNS: usize = 200;
        const LEN: usize = 600;

        let word = "lorem ipsum dolor sit amet consectetur ";
        let mut text = String::new();
        while text.len() < LEN {
            text.push_str(word);
        }
        text.truncate(LEN);

        let mut dom = Dom::new();
        let root = dom.root();
        for _ in 0..RUNS {
            let div = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![(
                    "style".to_string(),
                    "width:120px;font-size:10px".to_string(),
                )],
            });
            dom.append(root, div);
            let t = dom.create(NodeKind::Text(text.clone()));
            dom.append(div, t);
        }
        let styles = compute_styles(&dom, &[]);
        let viewport = Size {
            w: 1200.0,
            h: 800.0,
        };
        // This bench measures the measure cache, not the parallel sibling gate
        // (which uses per-thread caches and would bypass `cache` entirely).
        crate::parallel::set_force_sequential(true);

        let time = |cache: &mut MeasureCache| {
            let started = Instant::now();
            let list = crate::layout_with_cache(&dom, &styles, viewport, cache);
            (started.elapsed(), list.items.len())
        };

        let mut cold = f64::MAX;
        for _ in 0..5 {
            // Same DOM, same key computation, no replay: the honest baseline.
            let mut cache = MeasureCache::new_cold();
            let (d, _) = time(&mut cache);
            cold = cold.min(d.as_secs_f64() * 1000.0);
        }

        let mut warm_cache = MeasureCache::new();
        time(&mut warm_cache); // prime; ignore this first, cold-ish run
        let mut warm = f64::MAX;
        for _ in 0..5 {
            let (d, _) = time(&mut warm_cache);
            warm = warm.min(d.as_secs_f64() * 1000.0);
        }

        eprintln!(
            "measure-cache bench ({RUNS} identical runs, {LEN}-byte text): cold {cold:.3} ms, warm {warm:.3} ms, speedup {:.2}x, entries {}, bytes {}",
            cold / warm,
            warm_cache.len(),
            warm_cache.bytes()
        );
        crate::parallel::set_force_sequential(false);
        assert!(
            warm < cold / 1.5,
            "warm layout must be >=1.5x faster: cold {cold:.3} ms vs warm {warm:.3} ms"
        );
    }

    #[test]
    fn observe_dom_tracks_structural_mutation() {
        use hodo_dom::NodeKind;

        let mut dom = Dom::new();
        let parent = dom.create(NodeKind::Element {
            tag: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append(dom.root(), parent);
        let mut cache = MeasureCache::new();
        cache.observe_dom(&dom);
        let first = cache.generation();
        cache.observe_dom(&dom);
        assert_eq!(cache.generation(), first, "no mutation keeps the epoch");
        let child = dom.create(NodeKind::Text("x".to_string()));
        dom.append(parent, child);
        cache.observe_dom(&dom);
        assert_ne!(cache.generation(), first, "a new node bumps the epoch");
    }
}

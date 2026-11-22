//! Incremental layout: replay the display items of clean top-level subtrees.
//!
//! The recursive engines stream items into one flat display list, so a clean
//! subtree is skipped by copying its last-emitted items (translated by the new
//! y origin) instead of re-walking its boxes and re-shaping its text. The
//! integrator owns the decision of which nodes are dirty; see
//! [`layout_from`] for the exact seam.

use std::collections::HashMap;

use hodo_dom::{Dom, NodeId};
use hodo_style::{Clear, Display, Float, Position, StyleMap};
use hodo_types::Size;

use crate::{cache::MeasureCache, fallback_style, flush_line, Ctx, DisplayItem, DisplayList, Flow};

fn subtree_has_escape(dom: &Dom, styles: &StyleMap, id: NodeId) -> bool {
    let style = styles.get(id);
    let positioned = style.is_some_and(|s| {
        s.position != Position::Static || s.float != Float::None || s.clear != Clear::None
    });
    positioned
        || dom.get(id).children.iter().any(|&child| {
            subtree_has_escape(dom, styles, child)
                || styles
                    .get(child)
                    .is_some_and(|s| s.position == Position::Fixed)
        })
}

fn is_reusable_block(dom: &Dom, styles: &StyleMap, id: NodeId) -> bool {
    let Some(s) = styles.get(id) else {
        return false;
    };
    matches!(
        s.display,
        Display::Block | Display::Flex | Display::Grid | Display::Table
    ) && s.position == Position::Static
        && s.float == Float::None
        && s.clear == Clear::None
        && !subtree_has_escape(dom, styles, id)
}

fn shift_item_y(item: &mut DisplayItem, dy: f32) {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => rect.y += dy,
    }
}

/// Last-emitted geometry of one clean top-level subtree.
#[derive(Clone)]
struct CachedSubtree {
    /// Raw handle including hodo-dom's per-slot generation bits; catches a
    /// recycled slot even when the integrator forgot to report it.
    node_bits: u32,
    items: Vec<DisplayItem>,
    owners: Vec<NodeId>,
    consumed: f32,
    y: f32,
}

/// Per-run counters so callers (and the parity tests) can prove that a clean
/// subtree's items were reused rather than re-emitted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LayoutStats {
    pub fresh_items: usize,
    pub reused_items: usize,
    pub fresh_subtrees: usize,
    pub reused_subtrees: usize,
}

/// Persistent state for [`layout_from`]: the measurement cache plus the last
/// display items emitted by each clean top-level subtree.
pub struct LayoutContext {
    cache: MeasureCache,
    subtrees: HashMap<NodeId, CachedSubtree>,
    viewport: Option<Size>,
    stats: LayoutStats,
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutContext {
    pub fn new() -> Self {
        Self {
            cache: MeasureCache::new(),
            subtrees: HashMap::new(),
            viewport: None,
            stats: LayoutStats::default(),
        }
    }

    /// Counters from the most recent [`layout_from`] call.
    pub fn stats(&self) -> LayoutStats {
        self.stats
    }

    /// The shared measurement cache (generation bumps belong here).
    pub fn measure_cache(&mut self) -> &mut MeasureCache {
        &mut self.cache
    }

    /// Drop every cached subtree and measurement. Call on a stylesheet swap,
    /// viewport resize or any mutation not described by `dirty_roots`.
    pub fn reset(&mut self) {
        self.cache.clear();
        self.subtrees.clear();
        self.viewport = None;
    }
}

/// Incremental layout entry point.
///
/// `dirty_roots` are the `NodeId`s whose text, attributes or style changed
/// since the previous call. They are authoritative: every changed node must be
/// included. A recycled slot is additionally caught by comparing hodo-dom's
/// generation bits in `NodeId`. Only the top-level block subtrees containing a
/// dirty root are relaid out; every other top-level subtree replays its
/// last-emitted items at the new y origin.
///
/// Integrator seam: build one [`LayoutContext`] per document and keep it next
/// to the `Dom` and `StyleMap`. After each mutation transaction (and after
/// recomputing styles), call
/// `layout_from(&dom, &styles, viewport, &mut ctx, &changed_node_ids)` and
/// draw the returned `DisplayList`. Pass an empty slice when nothing changed.
/// On a stylesheet swap, viewport resize, or a change that crosses top-level
/// inline/float boundaries, call `ctx.reset()` first. A plain full relayout is
/// still `hodo_layout::layout(&dom, &styles, viewport)`.
pub fn layout_from(
    dom: &Dom,
    styles: &StyleMap,
    viewport: Size,
    ctx: &mut LayoutContext,
    dirty_roots: &[NodeId],
) -> DisplayList {
    // Geometry is viewport-specific; a resize invalidates every replayed rect.
    if ctx.viewport != Some(viewport) {
        ctx.reset();
        ctx.viewport = Some(viewport);
    }

    let live: Vec<NodeId> = dom
        .render_children(dom.root())
        .iter()
        .copied()
        .filter(|&id| !styles.get(id).is_some_and(|s| s.display == Display::None))
        .collect();
    let all_reusable =
        !live.is_empty() && live.iter().all(|&id| is_reusable_block(dom, styles, id));

    if !all_reusable {
        // Mixed inline/float top-level content needs the anonymous-block
        // splitter; replaying individual siblings would change paint order.
        let list = crate::layout_with_cache(dom, styles, viewport, &mut ctx.cache);
        ctx.subtrees.clear();
        ctx.stats = LayoutStats {
            fresh_items: list.items.len(),
            reused_items: 0,
            fresh_subtrees: live.len(),
            reused_subtrees: 0,
        };
        return list;
    }

    let dirty_top: Vec<NodeId> = dirty_roots
        .iter()
        .filter_map(|&dirty| {
            let mut cursor = Some(dirty);
            let mut top = None;
            while let Some(id) = cursor {
                if live.contains(&id) {
                    top = Some(id);
                }
                cursor = dom.parent(id);
            }
            top
        })
        .collect();

    let fallback = fallback_style();
    let mut stats = LayoutStats::default();
    let mut list = DisplayList::default();
    {
        let mut flow = Flow::new(0.0, viewport.w, 0.0);
        let mut kept = HashMap::new();
        for &child in &live {
            // The integrator reports every mutation in `dirty_roots`, so a
            // subtree not named there keeps its cached items. Hashing every
            // clean subtree here would cost more than the replay itself.
            let cached = ctx.subtrees.get(&child);
            let can_reuse =
                !dirty_top.contains(&child) && cached.is_some_and(|c| c.node_bits == child.0);
            if can_reuse {
                let c = cached.expect("checked above");
                let dy = flow.y - c.y;
                for (item, &owner) in c.items.iter().zip(&c.owners) {
                    let mut item = item.clone();
                    shift_item_y(&mut item, dy);
                    list.push(owner, item);
                }
                // Fixed children are excluded from caching, so the fixed
                // channel needs no translation here.
                flow.y += c.consumed;
                stats.reused_items += c.items.len();
                stats.reused_subtrees += 1;
                kept.insert(child, c.clone());
                continue;
            }

            let start = list.items.len();
            let y = flow.y;
            {
                let mut cx = Ctx {
                    dom,
                    styles,
                    list: &mut list,
                    cache: &mut ctx.cache,
                };
                crate::block::layout_child(&mut cx, &mut flow, &fallback, child, None);
            }
            let items = list.items[start..].to_vec();
            let owners = list.node_map[start..].to_vec();
            stats.fresh_items += items.len();
            stats.fresh_subtrees += 1;
            kept.insert(
                child,
                CachedSubtree {
                    node_bits: child.0,
                    items,
                    owners,
                    consumed: flow.y - y,
                    y,
                },
            );
        }
        flush_line(&mut flow, &mut list);
        ctx.subtrees = kept;
    }
    debug_assert_eq!(list.items.len(), list.node_map.len());
    ctx.stats = stats;
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_dom::NodeKind;
    use hodo_style::compute_styles;

    fn decl(p: &str, v: &str) -> Declaration {
        Declaration {
            property: p.to_string(),
            value: v.to_string(),
        }
    }

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn tag_rule(tag: &str, decls: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector: Selector {
                tag: Some(tag.to_string()),
                id: None,
                classes: Vec::new(),
                universal: false,
            },
            declarations: decls.iter().map(|(p, v)| decl(p, v)).collect(),
        }
    }

    fn page(n: usize) -> (Dom, StyleMap, Vec<NodeId>) {
        let mut dom = Dom::new();
        let root = dom.root();
        let mut blocks = Vec::new();
        for i in 0..n {
            let block = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![("style".to_string(), format!("background:#{:06x}", i + 1))],
            });
            dom.append(root, block);
            let text = dom.create(NodeKind::Text(format!("line {}", i)));
            dom.append(block, text);
            blocks.push(block);
        }
        let styles = styled(
            &dom,
            vec![tag_rule("div", &[("font-size", "10px"), ("margin", "1px")])],
        );
        (dom, styles, blocks)
    }

    fn debug_list(list: &DisplayList) -> Vec<String> {
        list.items.iter().map(|it| format!("{it:?}")).collect()
    }

    #[test]
    fn one_node_text_change_reemits_only_that_subtree() {
        let (mut dom, styles, blocks) = page(8);
        let viewport = Size { w: 800.0, h: 600.0 };
        let mut ctx = LayoutContext::new();
        let first = layout_from(&dom, &styles, viewport, &mut ctx, &[]);
        assert_eq!(ctx.stats().reused_subtrees, 0, "first call is full");
        let per_block = ctx.stats().fresh_items / 8;
        assert!(per_block > 0);

        // Replace one text node (hodo-dom has no set_text: detach + create).
        let old_text = dom.children(blocks[3])[0];
        dom.detach_subtree(old_text);
        let new_text = dom.create(NodeKind::Text("changed".to_string()));
        dom.append(blocks[3], new_text);

        let second = layout_from(&dom, &styles, viewport, &mut ctx, &[new_text]);
        assert_eq!(ctx.stats().fresh_subtrees, 1, "only one subtree is fresh");
        assert_eq!(ctx.stats().reused_subtrees, 7);
        assert_eq!(ctx.stats().fresh_items, per_block);
        assert_eq!(ctx.stats().reused_items, per_block * 7);

        // Replay must be byte-identical to a full relayout of the same DOM.
        let mut fresh_ctx = LayoutContext::new();
        let full = layout_from(&dom, &styles, viewport, &mut fresh_ctx, &[]);
        assert_eq!(debug_list(&second), debug_list(&full));
        assert_eq!(first.items.len(), full.items.len());
    }

    #[test]
    fn style_only_change_elsewhere_does_not_reemit_clean_subtrees() {
        let (mut dom, styles, blocks) = page(8);
        let viewport = Size { w: 800.0, h: 600.0 };
        let mut ctx = LayoutContext::new();
        layout_from(&dom, &styles, viewport, &mut ctx, &[]);

        dom.set_attr(blocks[6], "style", "background:#ff0000".to_string());
        let list = layout_from(&dom, &styles, viewport, &mut ctx, &[blocks[6]]);
        assert_eq!(ctx.stats().fresh_subtrees, 1);
        assert_eq!(ctx.stats().reused_subtrees, 7);
        assert_eq!(list.node_map.len(), list.items.len());
    }

    /// One-node damage path against a full relayout of the same 24-block page.
    /// Timings print with `--nocapture`.
    #[test]
    fn one_node_edit_is_faster_than_full_relayout() {
        use std::time::Instant;

        const BLOCKS: usize = 24;
        const BOXES: usize = 60;

        // A card-shaped page: many nested painted boxes + one text node per
        // block. Full layout pays the box walk; the damage path only clones
        // the clean blocks' already-emitted rects.
        let build = |alternate: bool| {
            let mut dom = Dom::new();
            let root = dom.root();
            let mut blocks = Vec::new();
            for i in 0..BLOCKS {
                let block = dom.create(NodeKind::Element {
                    tag: "div".to_string(),
                    attrs: vec![(
                        "style".to_string(),
                        "width:120px;font-size:10px;background:#0000ff".to_string(),
                    )],
                });
                dom.append(root, block);
                let mut parent = block;
                for _ in 0..BOXES {
                    let box_node = dom.create(NodeKind::Element {
                        tag: "div".to_string(),
                        attrs: vec![(
                            "style".to_string(),
                            "height:1px;background:#00ff00".to_string(),
                        )],
                    });
                    dom.append(parent, box_node);
                    parent = box_node;
                }
                let text = if alternate && i == 0 {
                    "changed text".to_string()
                } else {
                    "card text".to_string()
                };
                let t = dom.create(NodeKind::Text(text));
                dom.append(parent, t);
                blocks.push((block, t));
            }
            let styles = compute_styles(&dom, &[]);
            (dom, styles, blocks)
        };

        let viewport = Size {
            w: 1200.0,
            h: 800.0,
        };
        // Compare against the sequential full path; the parallel sibling gate
        // is a separate rung and would change what this bench measures.
        crate::parallel::set_force_sequential(true);
        let (dom, styles, _) = build(false);
        let mut full = f64::MAX;
        for _ in 0..5 {
            let started = Instant::now();
            let _ = crate::layout(&dom, &styles, viewport);
            full = full.min(started.elapsed().as_secs_f64() * 1000.0);
        }

        // Prime the incremental context with the first document. NodeIds are
        // deterministic across rebuilds, so cached handles keep matching.
        let (prime_dom, prime_styles, _) = build(false);
        let mut ctx = LayoutContext::new();
        let _ = layout_from(&prime_dom, &prime_styles, viewport, &mut ctx, &[]);

        let mut incremental = f64::MAX;
        for round in 0..5 {
            let alternate = round % 2 == 0;
            let (edit_dom, edit_styles, blocks) = build(alternate);
            // The integrator passes the node whose text changed this round.
            let dirty = blocks[0].1;
            let started = Instant::now();
            let _ = layout_from(&edit_dom, &edit_styles, viewport, &mut ctx, &[dirty]);
            incremental = incremental.min(started.elapsed().as_secs_f64() * 1000.0);
        }

        crate::parallel::set_force_sequential(false);
        eprintln!(
            "damage bench ({BLOCKS} blocks x {BOXES} boxes, one-node edit): full {full:.3} ms, incremental {incremental:.3} ms, speedup {:.2}x",
            full / incremental
        );
        assert!(
            incremental < full / 2.0,
            "one-node edit must be >=2x faster: full {full:.3} ms vs incremental {incremental:.3} ms"
        );
    }

    #[test]
    fn mixed_inline_top_level_falls_back_to_full_layout() {
        let mut dom = Dom::new();
        let root = dom.root();
        let block = dom.create(NodeKind::Element {
            tag: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append(root, block);
        let text = dom.create(NodeKind::Text("bare".to_string()));
        dom.append(root, text);
        let styles = compute_styles(&dom, &[]);
        let mut ctx = LayoutContext::new();
        let list = layout_from(&dom, &styles, Size { w: 800.0, h: 600.0 }, &mut ctx, &[]);
        assert_eq!(ctx.stats().reused_items, 0);
        assert_eq!(list.node_map.len(), list.items.len());
    }
}

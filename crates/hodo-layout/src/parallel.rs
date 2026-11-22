//! Risk-gated parallel layout of disjoint top-level block siblings.
//!
//! Only used at the document root when every live child is a plain block and
//! no descendant floats or is fixed. Floats are the hard stop: a float raised
//! by one sibling narrows the line boxes of *later* siblings, so their shapes
//! are not disjoint. Fixed items would need a second ordered merge channel.
//! Results are joined by child index, so paint order is deterministic.

use std::thread;

use hodo_dom::{Dom, NodeId};
use hodo_style::{ComputedStyle, Display, Float, Position, StyleMap};

use crate::{
    block::layout_child, cache::MeasureCache, flush_line, Ctx, DisplayItem, DisplayList, Flow,
};

#[cfg(test)]
thread_local! {
    static FORCE_SEQUENTIAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn force_sequential() -> bool {
    FORCE_SEQUENTIAL.with(|cell| cell.get())
}

/// Test-only switch so cache and incremental benches can measure the
/// sequential path without the parallel sibling gate interfering.
#[cfg(test)]
pub(crate) fn set_force_sequential(value: bool) {
    FORCE_SEQUENTIAL.with(|cell| cell.set(value));
}

/// True when a subtree can be laid out without affecting any later sibling.
pub(crate) fn subtree_is_disjoint(dom: &Dom, styles: &StyleMap, id: NodeId) -> bool {
    let style = styles.get(id);
    if style.is_some_and(|s| s.float != Float::None || s.position == Position::Fixed) {
        return false;
    }
    dom.get(id)
        .children
        .iter()
        .all(|&child| subtree_is_disjoint(dom, styles, child))
}

fn shift_item_y(item: &mut DisplayItem, dy: f32) {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => rect.y += dy,
    }
}

/// Try the parallel root path. Returns `false` when the caller must use the
/// sequential walk (not enough siblings, floats/fixed present, or <2 cores).
pub(crate) fn try_parallel_root_children(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    parent: NodeId,
    link: &Option<String>,
    width: f32,
) -> bool {
    #[cfg(test)]
    if force_sequential() {
        return false;
    }
    if parent != ctx.dom.root() || flow.y != 0.0 || flow.left != 0.0 {
        return false;
    }
    let children: Vec<NodeId> = ctx
        .dom
        .render_children(parent)
        .iter()
        .copied()
        .filter(|&id| {
            !ctx.styles
                .get(id)
                .is_some_and(|s| s.display == Display::None)
        })
        .collect();
    if children.len() < 8 {
        return false;
    }
    if !children.iter().all(|&id| {
        ctx.styles.get(id).is_some_and(|s| {
            matches!(
                s.display,
                Display::Block | Display::Flex | Display::Grid | Display::Table
            ) && s.float == Float::None
                && s.position == Position::Static
        }) && subtree_is_disjoint(ctx.dom, ctx.styles, id)
    }) {
        return false;
    }
    let threads = thread::available_parallelism().map_or(1, |n| n.get());
    if threads < 2 {
        return false;
    }
    let chunk_size = children.len().div_ceil(threads).max(1);
    let dom = ctx.dom;
    let styles = ctx.styles;
    let mut y = flow.y;

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in children.chunks(chunk_size) {
            let fallback = fallback.clone();
            let link = link.clone();
            handles.push(scope.spawn(move || {
                let mut list = DisplayList::default();
                let mut cache = MeasureCache::new();
                let mut local_flow = Flow::new(0.0, width, 0.0);
                {
                    let mut local_ctx = Ctx {
                        dom,
                        styles,
                        list: &mut list,
                        cache: &mut cache,
                    };
                    for &child in chunk {
                        layout_child(
                            &mut local_ctx,
                            &mut local_flow,
                            &fallback,
                            child,
                            link.clone(),
                        );
                    }
                    flush_line(&mut local_flow, local_ctx.list);
                }
                (list, local_flow.y)
            }));
        }
        // Index-join: the nth chunk's items precede the (n+1)th's in paint
        // order regardless of which thread finished first.
        for handle in handles {
            let (mut list, consumed) = handle.join().expect("parallel layout thread panicked");
            for item in &mut list.items {
                shift_item_y(item, y);
            }
            ctx.list.append(list);
            y += consumed;
        }
    });
    flow.y = y;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_dom::NodeKind;
    use hodo_style::compute_styles;
    use hodo_types::Size;

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn debug_list(list: &DisplayList) -> Vec<String> {
        list.items.iter().map(|it| format!("{it:?}")).collect()
    }

    fn page(with_float: bool) -> (Dom, StyleMap) {
        let mut dom = Dom::new();
        let root = dom.root();
        for i in 0..24 {
            let block = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![(
                    "style".to_string(),
                    "width:120px;font-size:10px;background:#0000ff".to_string(),
                )],
            });
            dom.append(root, block);
            if with_float && i == 5 {
                dom.set_attr(
                    block,
                    "style",
                    "float:left;width:40px;background:#00ff00".to_string(),
                );
            }
            let text = dom.create(NodeKind::Text(format!("card {i}")));
            dom.append(block, text);
        }
        let styles = styled(
            &dom,
            vec![StyleRule {
                selector: Selector {
                    tag: Some("div".to_string()),
                    id: None,
                    classes: Vec::new(),
                    universal: false,
                },
                declarations: vec![Declaration {
                    property: "font-size".to_string(),
                    value: "10px".to_string(),
                }],
            }],
        );
        (dom, styles)
    }

    #[test]
    fn float_subtree_is_not_disjoint() {
        let (dom, styles) = page(true);
        let root = dom.root();
        let children = dom.children(root);
        assert!(subtree_is_disjoint(&dom, &styles, children[0]));
        assert!(!subtree_is_disjoint(&dom, &styles, children[5]));
    }

    /// 24 disjoint blocks, distinct text so the sequential measure cache has
    /// nothing to reuse. Timings print with `--nocapture`.
    #[test]
    fn parallel_siblings_are_faster_than_sequential() {
        use std::cell::Cell;
        use std::time::Instant;

        let mut dom = Dom::new();
        let root = dom.root();
        for i in 0..24 {
            let block = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: vec![(
                    "style".to_string(),
                    "width:120px;font-size:10px;background:#0000ff".to_string(),
                )],
            });
            dom.append(root, block);
            let text = format!("block {i} lorem ipsum dolor sit amet ").repeat(20);
            let t = dom.create(NodeKind::Text(text));
            dom.append(block, t);
        }
        let styles = compute_styles(&dom, &[]);
        let viewport = Size {
            w: 1200.0,
            h: 800.0,
        };

        let time = || {
            let started = Instant::now();
            let _ = crate::layout(&dom, &styles, viewport);
            started.elapsed().as_secs_f64() * 1000.0
        };

        FORCE_SEQUENTIAL.with(|c| c.set(true));
        let mut sequential = f64::MAX;
        for _ in 0..5 {
            sequential = sequential.min(time());
        }
        FORCE_SEQUENTIAL.with(|c: &Cell<bool>| c.set(false));
        let mut parallel = f64::MAX;
        for _ in 0..5 {
            parallel = parallel.min(time());
        }

        eprintln!(
            "parallel bench (24 blocks, distinct text): sequential {sequential:.3} ms, parallel {parallel:.3} ms, speedup {:.2}x",
            sequential / parallel
        );
        assert!(
            parallel < sequential / 1.5,
            "parallel must be >=1.5x faster: {sequential:.3} vs {parallel:.3} ms"
        );
    }

    #[test]
    fn parallel_layout_matches_sequential_item_order() {
        let (dom, styles) = page(false);
        let viewport = Size { w: 800.0, h: 600.0 };
        FORCE_SEQUENTIAL.with(|c| c.set(true));
        let expected = crate::layout(&dom, &styles, viewport);
        FORCE_SEQUENTIAL.with(|c| c.set(false));
        let mut actual = DisplayList::default();
        let mut cache = MeasureCache::new();
        let used = {
            let mut flow = Flow::new(0.0, viewport.w, 0.0);
            let mut ctx = Ctx {
                dom: &dom,
                styles: &styles,
                list: &mut actual,
                cache: &mut cache,
            };
            let fallback = crate::fallback_style();
            let used = try_parallel_root_children(
                &mut ctx,
                &mut flow,
                &fallback,
                dom.root(),
                &None,
                viewport.w,
            );
            flush_line(&mut flow, ctx.list);
            used
        };
        assert!(used, "24 clean root blocks must take the parallel path");
        assert_eq!(debug_list(&actual), debug_list(&expected));
        assert_eq!(actual.node_map, expected.node_map);
    }
}

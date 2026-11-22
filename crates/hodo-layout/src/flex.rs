//! Flexbox layout for `Display::Flex` / `Display::InlineFlex`.
//!
//! Single-line (nowrap) wave: collect items in order-modified document order,
//! measure each with a throw-away Ctx, distribute main space (grow / shrink /
//! justify), align on the cross axis, then render every item into its own
//! scratch buffer and concatenate in *paint* order. Per-item buffers are what
//! make `order` and `row-reverse` paint in flex order rather than x order.

use super::*;
use crate::block::{layout_block, layout_child};

fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

fn nonneg(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

/// Bounding box of every emitted rect/text/image, in local coordinates.
fn bbox(items: &[DisplayItem]) -> Rect {
    let (mut x0, mut y0) = (f32::INFINITY, f32::INFINITY);
    let (mut x1, mut y1) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for item in items {
        let r = match item {
            DisplayItem::Rect { rect, .. }
            | DisplayItem::Text { rect, .. }
            | DisplayItem::Image { rect, .. }
            | DisplayItem::Control { rect, .. } => rect,
        };
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.w);
        y1 = y1.max(r.y + r.h);
    }
    if x0.is_infinite() {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    } else {
        Rect {
            x: x0,
            y: y0,
            w: (x1 - x0).max(0.0),
            h: (y1 - y0).max(0.0),
        }
    }
}

/// Approximate baseline: bottom of the first text line of an item.
fn first_baseline(items: &[DisplayItem]) -> Option<f32> {
    items.iter().find_map(|it| match it {
        DisplayItem::Text { rect, .. } => Some(rect.y + rect.h),
        _ => None,
    })
}

/// Indices of the box rects `layout_block` emits, in the order it emits them.
#[derive(Default, Clone, Copy)]
struct BoxRects {
    bg: Option<usize>,
    top: Option<usize>,
    bottom: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

fn box_rects(style: &ComputedStyle) -> BoxRects {
    let mut out = BoxRects::default();
    let mut i = 0usize;
    if style.background != Color::TRANSPARENT {
        out.bg = Some(i);
        i += 1;
    }
    let bw = style.border_width.max(0.0);
    if bw > 0.0 && style.border_color.a > 0 {
        out.top = Some(i);
        out.bottom = Some(i + 1);
        out.left = Some(i + 2);
        out.right = Some(i + 3);
    }
    out
}

/// Grow an item's painted box to the resolved cross size, mirroring the
/// height patch `layout_block` does for its own box.
fn patch_height(list: &mut DisplayList, r: BoxRects, new_h: f32) {
    let Some(reference) = r.bg.or(r.top) else {
        return;
    };
    // Fixed items live in the fixed channel, so their box rects are absent here.
    let y0 = match list.items.get(reference) {
        Some(DisplayItem::Rect { rect, .. }) => rect.y,
        _ => return,
    };
    if let Some(i) = r.bg {
        patch_rect(list, i, |q| q.h = new_h);
    }
    if let Some(i) = r.left {
        patch_rect(list, i, |q| {
            q.y = y0;
            q.h = new_h;
        });
    }
    if let Some(i) = r.right {
        patch_rect(list, i, |q| {
            q.y = y0;
            q.h = new_h;
        });
    }
    if let Some(i) = r.bottom {
        patch_rect(list, i, |q| q.y = y0 + new_h - q.h);
    }
}

fn resolve_align(ai: AlignItems, aself: AlignSelf) -> AlignItems {
    match aself {
        AlignSelf::Auto => ai,
        AlignSelf::Stretch => AlignItems::Stretch,
        AlignSelf::Center => AlignItems::Center,
        AlignSelf::FlexStart => AlignItems::FlexStart,
        AlignSelf::FlexEnd => AlignItems::FlexEnd,
        AlignSelf::Baseline => AlignItems::Baseline,
    }
}

struct Item {
    node: NodeId,
    style: ComputedStyle,
    grow: f32,
    shrink: f32,
    main: f32,
    cross: f32,
    min_main: f32,
    baseline: f32,
    cross_margin: f32,
    align: AlignItems,
    patchable: bool,
    definite_cross: bool,
}

pub(crate) fn layout_flex(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    let m = &style.margin;
    let p = &style.padding;
    let bw = style.border_width.max(0.0);
    let box_x = avail.x + m.left;
    let box_y = avail.y + m.top;
    let box_w = (avail.width - m.left - m.right).max(0.0);
    let inner_left = box_x + bw + p.left;
    let inner_right = box_x + box_w - bw - p.right;
    let top = box_y + bw + p.top;
    let inner_w = (inner_right - inner_left).max(0.0);

    // order-modified document order: (order, dom index).
    let mut children: Vec<(usize, NodeId)> = ctx
        .dom
        .children(id)
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, c)| {
            !ctx.styles
                .get(c)
                .is_some_and(|s| s.display == Display::None)
        })
        .collect();
    children.sort_by_key(|&(idx, c)| {
        let ord = ctx.styles.get(c).map(|s| s.order).unwrap_or(0);
        (ord, idx)
    });

    // Fixed children live in the fixed channel, outside the scrolled flex
    // formatting context. They must not be measured (an empty scratch list
    // would give them zero size) or take any share of grow/shrink/justify.
    // shortcut: their static-position box is the container's content box; they
    // are still rendered below so their items reach the root fixed channel.
    let fixed_children: Vec<NodeId> = children
        .iter()
        .filter(|(_, c)| {
            ctx.styles
                .get(*c)
                .is_some_and(|s| s.position == Position::Fixed)
        })
        .map(|(_, c)| *c)
        .collect();
    children.retain(|(_, c)| {
        !ctx.styles
            .get(*c)
            .is_some_and(|s| s.position == Position::Fixed)
    });

    // Fast path: an empty flex container is just a block with no content.
    if children.is_empty() {
        return layout_block(ctx, style, id, avail, link);
    }

    // Reserve the container's background and border rects; heights are patched
    // after the items have been placed (same convention as `layout_block`).
    let mut bg_idx = None;
    if style.background != Color::TRANSPARENT {
        bg_idx = Some(push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: box_w,
                h: 0.0,
            },
            style.background,
        ));
    }
    let (mut bottom_idx, mut left_idx, mut right_idx) = (None, None, None);
    if bw > 0.0 && style.border_color.a > 0 {
        push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: box_w,
                h: bw,
            },
            style.border_color,
        ); // top edge: complete already
        bottom_idx = Some(push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: box_w,
                h: bw,
            },
            style.border_color,
        ));
        left_idx = Some(push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: bw,
                h: 0.0,
            },
            style.border_color,
        ));
        right_idx = Some(push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x + box_w - bw,
                y: box_y,
                w: bw,
                h: 0.0,
            },
            style.border_color,
        ));
    }

    let column = matches!(
        style.flex_direction,
        FlexDirection::Column | FlexDirection::ColumnReverse
    );
    let reverse = matches!(
        style.flex_direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );
    // shortcut: flex-wrap is treated as nowrap this wave; wrapped lines land with
    // a real line-breaking pass. Percentage gap/basis never resolve here.
    let gap = nonneg(finite(style.gap));
    let fallback = fallback_style();
    let n = children.len();

    // Pass 1: measure every item in isolation.
    let mut items: Vec<Item> = Vec::with_capacity(n);
    for (_, node) in children {
        let s = ctx.styles.get(node).cloned().unwrap_or_else(fallback_style);
        let explicit_main = if column { s.height } else { s.width };
        let explicit_cross = if column { s.width } else { s.height };
        let explicit_main = explicit_main.map(nonneg);
        let explicit_cross = explicit_cross.map(nonneg);

        let main_margin = if column {
            nonneg(s.margin.top) + nonneg(s.margin.bottom)
        } else {
            nonneg(s.margin.left) + nonneg(s.margin.right)
        };
        let cross_margin = if column {
            nonneg(s.margin.left) + nonneg(s.margin.right)
        } else {
            nonneg(s.margin.top) + nonneg(s.margin.bottom)
        };

        let mut scratch = DisplayList::default();
        let measure_w = if column {
            inner_w
        } else {
            explicit_main.unwrap_or(inner_w)
        };
        let mut measure_flow = Flow::new(0.0, measure_w, 0.0);
        {
            let mut mctx = Ctx {
                dom: ctx.dom,
                styles: ctx.styles,
                list: &mut scratch,
            };
            layout_child(&mut mctx, &mut measure_flow, &fallback, node, link.clone());
        }
        flush_line(&mut measure_flow, &mut scratch);
        let bb = bbox(&scratch.items);
        let measured_main = if column { bb.h } else { bb.w };
        let measured_cross = if column { bb.w } else { bb.h };
        let baseline = first_baseline(&scratch.items).unwrap_or(measured_cross + cross_margin);

        // flex-basis wins; `auto` falls back to the main size property, then to
        // the measured content size. Margins are part of the outer main size.
        let base = match s.flex_basis {
            FlexBasis::Length(v) => nonneg(v),
            FlexBasis::Auto => explicit_main.unwrap_or(measured_main),
        } + main_margin;
        let cross = explicit_cross.unwrap_or(measured_cross) + cross_margin;
        // A definite basis/size can shrink freely; an auto-sized text item is
        // clamped to its measured content at most once during shrink.
        let min_main = if explicit_main.is_some() || matches!(s.flex_basis, FlexBasis::Length(_)) {
            0.0
        } else {
            measured_main + main_margin
        };

        items.push(Item {
            node,
            style: s.clone(),
            grow: nonneg(finite(s.flex_grow)),
            shrink: nonneg(finite(s.flex_shrink)),
            main: base,
            cross,
            min_main,
            baseline,
            cross_margin,
            align: resolve_align(style.align_items, s.align_self),
            // Only block element boxes emit a background/border rect whose
            // height can be patched to the resolved cross size.
            patchable: matches!(ctx.dom.get(node).kind, NodeKind::Element { .. })
                && s.display == Display::Block,
            definite_cross: explicit_cross.is_some(),
        });
    }

    // Cross size of the single line: definite container size wins, else items.
    let max_cross = items.iter().fold(0.0_f32, |a, it| a.max(it.cross));
    let line_cross = if column {
        inner_w
    } else {
        style.height.map(nonneg).unwrap_or(max_cross)
    };
    let line_baseline = items.iter().fold(0.0_f32, |a, it| a.max(it.baseline));

    // Pass 2: main-axis sizing. For a column without an explicit height there
    // is no main-size constraint, so grow/shrink/justify are no-ops.
    let constrained = !column || style.height.is_some();
    let main_extent = if column {
        style.height.map(nonneg).unwrap_or(0.0)
    } else {
        inner_w
    };
    let total_gap = gap * (n - 1) as f32;
    let used = items.iter().fold(0.0_f32, |a, it| a + it.main) + total_gap;
    let mut free = if constrained { main_extent - used } else { 0.0 };

    let grow_sum: f32 = items.iter().fold(0.0_f32, |a, it| a + it.grow);
    let mut frozen = vec![false; n];
    if free > 0.0 && grow_sum > 0.0 && grow_sum.is_finite() {
        for item in &mut items {
            if item.grow > 0.0 {
                item.main += free * item.grow / grow_sum;
            }
        }
        free = 0.0;
    } else if free < 0.0 {
        // Shrink; freeze an item at its floor once and redistribute the rest.
        // Bounded by the item count so a pathological set cannot spin.
        let mut weight: f32 = items
            .iter()
            .zip(&frozen)
            .filter(|(_, f)| !**f)
            .map(|(it, _)| it.shrink * it.main)
            .sum();
        for _ in 0..=n {
            if free >= 0.0 || weight <= 0.0 || !weight.is_finite() {
                break;
            }
            let mut hit = false;
            for i in 0..n {
                if frozen[i] {
                    continue;
                }
                let w = items[i].shrink * items[i].main;
                if w <= 0.0 {
                    continue;
                }
                let target = items[i].main + free * w / weight;
                let next = target.max(items[i].min_main).max(0.0);
                if next > target {
                    free += items[i].main - next;
                    items[i].main = next;
                    frozen[i] = true;
                    weight -= w;
                    hit = true;
                } else {
                    items[i].main = next;
                }
            }
            if !hit {
                break;
            }
        }
        free = main_extent - (items.iter().fold(0.0_f32, |a, it| a + it.main) + total_gap);
    }

    // Justify distributes only the free space left after flex sizing.
    let free = free.max(0.0);
    let (lead, extra) = match style.justify_content {
        JustifyContent::FlexStart => (0.0, 0.0),
        JustifyContent::Center => (free * 0.5, 0.0),
        JustifyContent::FlexEnd => (free, 0.0),
        JustifyContent::SpaceBetween => (0.0, if n > 1 { free / (n - 1) as f32 } else { 0.0 }),
        JustifyContent::SpaceAround => (free / (2 * n) as f32, free / n as f32),
        JustifyContent::SpaceEvenly => (free / (n + 1) as f32, free / (n + 1) as f32),
    };

    let cross_pos = |it: &Item| match it.align {
        AlignItems::Stretch | AlignItems::FlexStart => top,
        AlignItems::FlexEnd => top + line_cross - it.cross,
        AlignItems::Center => top + (line_cross - it.cross) * 0.5,
        AlignItems::Baseline => top + line_baseline - it.baseline,
    };

    // Pass 3: render each item into its own buffer, then append in *paint*
    // order (the order-modified sequence), not visual x order.
    let main_start = if reverse { inner_right } else { inner_left };
    let mut logical = lead;
    for item in &items {
        let (lo, hi) = if reverse {
            (main_start - logical - item.main, main_start - logical)
        } else {
            (main_start + logical, main_start + logical + item.main)
        };
        let cpos = cross_pos(item);
        let stretch = item.align == AlignItems::Stretch && !item.definite_cross;
        let cross_extent = if column && stretch {
            line_cross
        } else {
            item.cross
        };
        let (fl, fr, fy) = if column {
            (cpos, cpos + cross_extent, lo)
        } else {
            (lo, hi, cpos)
        };

        let mut scratch = DisplayList::default();
        let mut flow = Flow::new(fl, fr, fy);
        {
            let mut rctx = Ctx {
                dom: ctx.dom,
                styles: ctx.styles,
                list: &mut scratch,
            };
            layout_child(&mut rctx, &mut flow, &fallback, item.node, link.clone());
        }
        flush_line(&mut flow, &mut scratch);
        if item.patchable && !column {
            // Force the painted box to the resolved cross size: stretch fills
            // the line, a definite cross size keeps its declared height.
            let resolved = if stretch { line_cross } else { item.cross };
            let target = (resolved - item.cross_margin).max(0.0);
            patch_height(&mut scratch, box_rects(&item.style), target);
        }
        ctx.list.append(scratch);
        logical += item.main + gap + extra;
    }

    // Render the excluded fixed children at the container's static-position
    // box; layout_child routes their items into the fixed channel for us.
    for node in fixed_children {
        let mut scratch = DisplayList::default();
        let mut flow = Flow::new(inner_left, inner_right, top);
        {
            let mut rctx = Ctx {
                dom: ctx.dom,
                styles: ctx.styles,
                list: &mut scratch,
            };
            layout_child(&mut rctx, &mut flow, &fallback, node, link.clone());
        }
        flush_line(&mut flow, &mut scratch);
        ctx.list.append(scratch);
    }

    // Patch the container box to its final height.
    let content_h = if column {
        if constrained {
            main_extent
        } else {
            items.iter().fold(0.0_f32, |a, it| a + it.main) + total_gap + lead
        }
    } else {
        line_cross
    };
    let box_h = content_h + p.top + p.bottom + 2.0 * bw;
    if let Some(i) = bg_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }
    if let Some(i) = bottom_idx {
        patch_rect(ctx.list, i, |r| r.y = box_y + box_h - bw);
    }
    if let Some(i) = left_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }
    if let Some(i) = right_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }

    m.top + box_h + m.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_style::compute_styles;
    use hodo_types::Size;

    fn decl(p: &str, v: &str) -> Declaration {
        Declaration {
            property: p.into(),
            value: v.into(),
        }
    }

    fn rule(tag: &str, props: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector: Selector {
                tag: Some(tag.into()),
                id: None,
                classes: Vec::new(),
                universal: false,
            },
            declarations: props.iter().map(|&(p, v)| decl(p, v)).collect(),
        }
    }

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: Vec::new(),
        });
        dom.append(parent, id);
        id
    }

    fn text(dom: &mut Dom, parent: NodeId, s: &str) {
        let t = dom.create(NodeKind::Text(s.into()));
        dom.append(parent, t);
    }

    fn colored(list: &DisplayList, color: Color) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Rect { rect, color: c, .. } if *c == color => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn texts(list: &DisplayList) -> Vec<&str> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    fn three_boxes(extra: &[(&str, &str)]) -> (Dom, StyleMap) {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        for name in ["A", "B", "C"] {
            let c = elem(&mut dom, f, "item");
            text(&mut dom, c, name);
        }
        let mut container = vec![("display", "flex")];
        container.extend_from_slice(extra);
        let mut item = vec![
            ("display", "block"),
            ("background", "#ff0000"),
            ("width", "40px"),
        ];
        item.extend_from_slice(
            &extra
                .iter()
                .filter(|(p, _)| *p == "align-items")
                .copied()
                .collect::<Vec<_>>(),
        );
        let styles = styled(&dom, vec![rule("flex", &container), rule("item", &item)]);
        (dom, styles)
    }

    const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    #[test]
    fn grow_one_splits_space_equally() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        for name in ["A", "B", "C"] {
            let c = elem(&mut dom, f, "item");
            text(&mut dom, c, name);
        }
        let styles = styled(
            &dom,
            vec![
                rule("flex", &[("display", "flex")]),
                rule(
                    "item",
                    &[
                        ("display", "block"),
                        ("flex", "1"),
                        ("background", "#ff0000"),
                    ],
                ),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        let widths: Vec<f32> = colored(&list, RED).iter().map(|r| r.w).collect();
        assert_eq!(widths.len(), 3);
        assert!(
            widths.iter().all(|w| (w - 100.0).abs() < 0.01),
            "grow:1 must split 300px into three equal 100px items: {widths:?}"
        );
    }

    #[test]
    fn justify_content_positions() {
        let cases: &[(&str, [f32; 3])] = &[
            ("space-between", [0.0, 130.0, 260.0]),
            ("center", [90.0, 130.0, 170.0]),
            ("flex-end", [180.0, 220.0, 260.0]),
        ];
        for &(justify, expected) in cases {
            let (dom, styles) = three_boxes(&[("justify-content", justify)]);
            let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
            let xs: Vec<f32> = colored(&list, RED).iter().map(|r| r.x).collect();
            for (got, want) in xs.iter().zip(expected.iter()) {
                assert!(
                    (got - want).abs() < 0.01,
                    "justify {justify}: expected {expected:?}, got {xs:?}"
                );
            }
        }
    }

    #[test]
    fn gap_separates_items() {
        let (dom, styles) = three_boxes(&[("gap", "10px")]);
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        let rs = colored(&list, RED);
        assert_eq!(rs.len(), 3);
        assert!((rs[1].x - (rs[0].x + rs[0].w) - 10.0).abs() < 0.01);
        assert!((rs[2].x - (rs[1].x + rs[1].w) - 10.0).abs() < 0.01);
    }

    #[test]
    fn stretch_patches_item_heights() {
        let (dom, styles) = three_boxes(&[("height", "100px")]);
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        let hs: Vec<f32> = colored(&list, RED).iter().map(|r| r.h).collect();
        assert_eq!(hs.len(), 3);
        assert!(
            hs.iter().all(|h| (h - 100.0).abs() < 0.01),
            "stretch must patch each item's box to 100px: {hs:?}"
        );
    }

    #[test]
    fn align_self_overrides_container_alignment() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        for (name, cls) in [("A", "item"), ("B", "tail"), ("C", "item")] {
            let c = elem(&mut dom, f, cls);
            text(&mut dom, c, name);
        }
        let styles = styled(
            &dom,
            vec![
                rule("flex", &[("display", "flex"), ("height", "100px")]),
                rule(
                    "item",
                    &[
                        ("display", "block"),
                        ("background", "#ff0000"),
                        ("width", "40px"),
                    ],
                ),
                rule(
                    "tail",
                    &[
                        ("display", "block"),
                        ("background", "#00ff00"),
                        ("width", "40px"),
                        ("height", "20px"),
                        ("align-self", "flex-end"),
                    ],
                ),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        let tail = colored(&list, Color::rgb(0, 255, 0));
        assert_eq!(tail.len(), 1);
        assert!(
            (tail[0].y - 80.0).abs() < 0.01,
            "align-self:flex-end must sit at bottom: {:?}",
            tail[0]
        );
        let rest = colored(&list, RED);
        assert!(rest.iter().all(|r| (r.h - 100.0).abs() < 0.01));
    }

    #[test]
    fn order_reorders_paint_and_position() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        for (name, cls) in [("A", "oa"), ("B", "ob"), ("C", "oc")] {
            let c = elem(&mut dom, f, cls);
            text(&mut dom, c, name);
        }
        let styles = styled(
            &dom,
            vec![
                rule("flex", &[("display", "flex")]),
                rule("item", &[("display", "block"), ("width", "40px")]),
                rule("oa", &[("order", "3")]),
                rule("ob", &[("order", "1")]),
                rule("oc", &[("order", "2")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        assert_eq!(texts(&list), ["B", "C", "A"], "paint follows order");
        let tr: Vec<f32> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { rect, .. } => Some(rect.x),
                _ => None,
            })
            .collect();
        assert!(
            tr.windows(2).all(|w| w[0] < w[1]),
            "x ascends in order: {tr:?}"
        );
    }

    #[test]
    fn row_reverse_descends_x_but_paints_in_order() {
        let (dom, styles) = three_boxes(&[("flex-direction", "row-reverse")]);
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        assert_eq!(texts(&list), ["A", "B", "C"], "paint order unchanged");
        let xs: Vec<f32> = colored(&list, RED).iter().map(|r| r.x).collect();
        assert!(xs.windows(2).all(|w| w[0] > w[1]), "x descends: {xs:?}");
    }

    #[test]
    fn basis_auto_uses_measured_content() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        text(&mut dom, f, "hello");
        let styles = styled(
            &dom,
            vec![rule("flex", &[("display", "flex"), ("font-size", "10px")])],
        );
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });
        let t = list.items.iter().find_map(|it| match it {
            DisplayItem::Text { rect, .. } => Some(rect),
            _ => None,
        });
        assert!(
            t.is_some_and(|r| (r.w - 30.0).abs() < 0.01),
            "5 chars × 6px = 30px: {t:?}"
        );
    }

    #[test]
    fn adversarial_values_do_not_panic_or_produce_nan() {
        let cases: &[(&str, &str)] = &[
            ("flex-basis", "0px"),
            ("flex-grow", "1e308"),
            ("flex-grow", "NaN"),
            ("flex-basis", "1000px"),
        ];
        for (prop, val) in cases {
            let mut dom = Dom::new();
            let root = dom.root();
            let f = elem(&mut dom, root, "flex");
            let c = elem(&mut dom, f, "item");
            text(&mut dom, c, "x");
            let styles = styled(
                &dom,
                vec![
                    rule("flex", &[("display", "flex")]),
                    rule(
                        "item",
                        &[("display", "block"), ("background", "#ff0000"), (prop, val)],
                    ),
                ],
            );
            let list = layout(&dom, &styles, Size { w: 200.0, h: 600.0 });
            assert!(
                colored(&list, RED)
                    .iter()
                    .all(|r| r.w.is_finite() && r.w <= 200.0),
                "{prop}:{val} produced a bad rect"
            );
        }
    }

    #[test]
    fn fixed_child_does_not_take_flex_space_and_reaches_fixed_channel() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        let fixed = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![(
                "style".into(),
                "position:fixed; left:10px; width:100px; flex:1; background:#0000ff".into(),
            )],
        });
        dom.append(f, fixed);
        let stat = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("style".into(), "flex:1; background:#ff0000".into())],
        });
        dom.append(f, stat);
        text(&mut dom, stat, "s");

        let styles = styled(&dom, vec![rule("flex", &[("display", "flex")])]);
        let list = layout(&dom, &styles, Size { w: 300.0, h: 600.0 });

        assert!(
            list.fixed_items.iter().any(|it| matches!(
                it,
                DisplayItem::Rect { color, .. } if *color == Color::rgb(0, 0, 255)
            )),
            "fixed child must render into the fixed channel"
        );
        assert!(
            colored(&list, Color::rgb(0, 0, 255)).is_empty(),
            "fixed child must not appear in the scrolled channel"
        );
        let red = colored(&list, RED);
        assert_eq!(red.len(), 1);
        assert!(
            (red[0].w - 300.0).abs() < 0.01,
            "the remaining grow:1 item must take all 300px: {red:?}"
        );
        assert_eq!(list.items.len(), list.node_map.len());
        assert_eq!(list.fixed_items.len(), list.fixed_node_map.len());
    }

    #[test]
    fn min_content_long_word_and_thousand_items_are_bounded() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flex");
        let long = "x".repeat(500);
        for i in 0..1000 {
            let c = elem(&mut dom, f, "item");
            text(&mut dom, c, if i == 0 { &long } else { "a" });
        }
        let styles = styled(
            &dom,
            vec![
                rule("flex", &[("display", "flex")]),
                rule("item", &[("display", "block"), ("width", "2px")]),
            ],
        );
        let started = std::time::Instant::now();
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let elapsed = started.elapsed();
        assert!(!list.items.is_empty());
        assert!(
            elapsed.as_secs_f64() < 2.0,
            "1000 items must lay out in <2s, took {elapsed:?}"
        );
    }
}

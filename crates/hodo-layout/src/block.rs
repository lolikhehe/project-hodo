//! Block flow: the recursive child walk, anonymous inline runs and block boxes.

use super::*;

/// Lay out `dom` into a flat display list for a viewport of `size`.
pub fn layout(dom: &Dom, styles: &StyleMap, viewport: Size) -> DisplayList {
    let mut list = DisplayList::default();
    {
        let mut ctx = Ctx {
            dom,
            styles,
            list: &mut list,
        };
        let fallback = fallback_style();
        let mut flow = Flow::new(0.0, viewport.w, 0.0);
        layout_children(&mut ctx, &mut flow, &fallback, dom.root(), None);
        flush_line(&mut flow, ctx.list);
    }
    debug_assert_eq!(
        list.items.len(),
        list.node_map.len(),
        "every display item must have an owning NodeId"
    );
    debug_assert_eq!(
        list.fixed_items.len(),
        list.fixed_node_map.len(),
        "every fixed display item must have an owning NodeId"
    );
    list
}

/// Walk a node's children in order. Mixed inline/block child lists are split
/// into anonymous block boxes: each consecutive inline run gets its own flow,
/// so it cannot share a line box across a block sibling. Pure lists keep the
/// original single-flow behaviour.
pub(crate) fn layout_children(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    parent: NodeId,
    link: Option<String>,
) {
    let live: Vec<NodeId> = ctx
        .dom
        .children(parent)
        .iter()
        .copied()
        .filter(|&child| {
            !ctx.styles
                .get(child)
                .is_some_and(|s| s.display == Display::None)
        })
        .collect();

    let has_block = live.iter().any(|&c| is_block_level(ctx, c));
    let has_inline = live.iter().any(|&c| !is_block_level(ctx, c));

    if has_block && has_inline {
        // Anonymous block boxes: wrap consecutive inline runs.
        let mut run: Vec<NodeId> = Vec::new();
        let mut run_y = flow.y;
        for &child in &live {
            if is_block_level(ctx, child) {
                if !run.is_empty() {
                    layout_inline_run(ctx, flow, fallback, &run, link.clone(), run_y);
                    run.clear();
                }
                layout_child(ctx, flow, fallback, child, link.clone());
                run_y = flow.y;
            } else {
                run.push(child);
            }
        }
        if !run.is_empty() {
            layout_inline_run(ctx, flow, fallback, &run, link, run_y);
        }
    } else {
        for child in live {
            layout_child(ctx, flow, fallback, child, link.clone());
        }
    }
}

/// Lay out one anonymous block box made from a consecutive inline run.
pub(crate) fn layout_inline_run(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    run: &[NodeId],
    link: Option<String>,
    y: f32,
) {
    let mut sub = Flow::new(flow.left, flow.right, y);
    if flow.bullet {
        sub.bullet = true;
        flow.bullet = false;
    }
    // An anonymous inline run is a fresh Flow: carry the parent's float state
    // over so its line boxes wrap around the same floats.
    sub.float_l = flow.float_l;
    sub.float_r = flow.float_r;
    sub.float_bottoms = flow.float_bottoms;
    for &child in run {
        layout_child(ctx, &mut sub, fallback, child, link.clone());
    }
    flush_line(&mut sub, ctx.list);
    flow.y = sub.y;
}

/// Shift display items emitted by one element for `position: relative`.
/// `from` is captured after any preceding line is flushed, so earlier
/// siblings keep their positions and the parent's flow cursor is untouched.
// shortcut: tolerate new display item kinds; the allow only matters on builds
// where these three painted kinds are the whole enum.
#[allow(irrefutable_let_patterns)]
fn shift_items(list: &mut DisplayList, from: usize, dx: f32, dy: f32) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }
    for item in &mut list.items[from..] {
        // Form controls are replaced elements too: relative offsets move them.
        if let DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } = item
        {
            rect.x += dx;
            rect.y += dy;
        }
    }
}

/// Per-element `overflow: hidden|clip` post-pass over a descendant range.
/// Rects and images are intersected with the padding box; text runs are
/// dropped whole when they do not fit because the display list has no
/// per-glyph runs. The element's own background/border items live before
/// `from` and therefore survive untouched.
fn clip_items(list: &mut DisplayList, from: usize, clip: Rect) {
    let (right, bottom) = (clip.x + clip.w, clip.y + clip.h);
    let mut i = from;
    while i < list.items.len() {
        let mut remove = false;
        if let DisplayItem::Rect { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } = &mut list.items[i]
        {
            let (x0, y0) = (rect.x.max(clip.x), rect.y.max(clip.y));
            let (x1, y1) = ((rect.x + rect.w).min(right), (rect.y + rect.h).min(bottom));
            if x1 <= x0 || y1 <= y0 {
                remove = true;
            } else {
                *rect = Rect {
                    x: x0,
                    y: y0,
                    w: x1 - x0,
                    h: y1 - y0,
                };
            }
        } else if let DisplayItem::Text { rect, .. } = &mut list.items[i] {
            remove = rect.x < clip.x
                || rect.y < clip.y
                || rect.x + rect.w > right
                || rect.y + rect.h > bottom;
        }
        if remove {
            list.items.remove(i);
            list.node_map.remove(i);
        } else {
            i += 1;
        }
    }
}

/// Paint one inline element's insets and recurse into its children. Kept
/// separate so an absolutely positioned inline element can paint into its own
/// scratch flow without advancing the parent's line box.
fn layout_inline_element(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    child: NodeId,
    link: Option<String>,
    st: &ComputedStyle,
) {
    let paintable = st.background.a > 0 || (st.border_color.a > 0 && st.border_width > 0.0);
    if paintable {
        let bi = flow.boxes.len();
        flow.boxes.push(InlineBox {
            bg: st.background,
            border_color: st.border_color,
            border: st.border_widths,
            margin: st.margin,
            padding: st.padding,
            seen: false,
            active: true,
            owner: child,
        });
        let mut boxes = flow.stack.clone();
        boxes.push(bi);
        let left = left_inset(&flow.boxes[bi]);
        if left > 0.0 {
            push_atom(
                flow,
                Atom {
                    text: String::new(),
                    font_size: st.font_size,
                    color: st.color,
                    weight: st.font_weight,
                    href: None,
                    underline: false,
                    line_through: false,
                    x: crate::inline::float_bounds(flow).0 + flow.line_w,
                    w: left,
                    boxes: boxes.clone(),
                    spacer: true,
                    owner: child,
                },
            );
            flow.line_w += left;
        }
        flow.stack.push(bi);
    }

    layout_children(ctx, flow, fallback, child, link);

    if paintable {
        // The right spacer still belongs to the box being closed.
        let boxes = flow.stack.clone();
        let bi = flow
            .stack
            .pop()
            .expect("paintable inline box must be on the stack");
        flow.boxes[bi].active = false;
        let right = right_inset(&flow.boxes[bi]);
        if right > 0.0 {
            push_atom(
                flow,
                Atom {
                    text: String::new(),
                    font_size: st.font_size,
                    color: st.color,
                    weight: st.font_weight,
                    href: None,
                    underline: false,
                    line_through: false,
                    x: crate::inline::float_bounds(flow).0 + flow.line_w,
                    w: right,
                    boxes,
                    spacer: true,
                    owner: child,
                },
            );
            flow.line_w += right;
        }
    }
}

/// Union of two margin boxes, used when two floats share a side.
fn union_rect(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    let right = (a.x + a.w).max(b.x + b.w);
    let bottom = (a.y + a.h).max(b.y + b.h);
    Rect {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}

/// Record a floated child's margin box on the containing flow. Its consumed
/// height is deliberately not added to `flow.y`: following line boxes wrap
/// around it instead. Later floats on the same side union with the first.
/// shortcut: one rect per side; two side-by-side floats over-exclude the gap
/// between them. Upgrade to a list when paged float stacks need exact slots.
fn record_float(flow: &mut Flow, kind: Float, rect: Rect) {
    let bottom = rect.y + rect.h;
    match kind {
        Float::Left => {
            flow.float_l = Some(match flow.float_l.take() {
                Some(old) => union_rect(old, rect),
                None => rect,
            });
            flow.float_bottoms.0 = flow.float_bottoms.0.max(bottom);
        }
        Float::Right => {
            flow.float_r = Some(match flow.float_r.take() {
                Some(old) => union_rect(old, rect),
                None => rect,
            });
            flow.float_bottoms.1 = flow.float_bottoms.1.max(bottom);
        }
        Float::None => {}
    }
}

/// Lay out a single child into `flow`.
///
/// `position: fixed` children are laid out into a temporary list whose emitted
/// items are moved into the separate fixed channel, so the shell can draw them
/// unshifted by scroll. Static pages never take this path (no-fixed guard).
pub(crate) fn layout_child(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    child: NodeId,
    link: Option<String>,
) {
    let s = ctx.styles.get(child).cloned();
    if s.as_ref().is_some_and(|st| st.display == Display::None) {
        return;
    }
    if s.as_ref().is_some_and(|st| st.position == Position::Fixed) {
        let mut sub = DisplayList::default();
        std::mem::swap(&mut *ctx.list, &mut sub);
        layout_child_inner(ctx, flow, fallback, child, link, s);
        std::mem::swap(&mut *ctx.list, &mut sub);
        for (item, owner) in sub.items.into_iter().zip(sub.node_map) {
            ctx.list.push_fixed(owner, item);
        }
        // Nested fixed elements already emitted into the subtree's fixed channel.
        ctx.list.fixed_items.append(&mut sub.fixed_items);
        ctx.list.fixed_node_map.append(&mut sub.fixed_node_map);
        return;
    }
    layout_child_inner(ctx, flow, fallback, child, link, s);
}

/// Lay out a non-fixed child into `flow`. `s` is the pre-fetched computed style.
fn layout_child_inner(
    ctx: &mut Ctx<'_>,
    flow: &mut Flow,
    fallback: &ComputedStyle,
    child: NodeId,
    link: Option<String>,
    s: Option<ComputedStyle>,
) {
    let tag = ctx.dom.tag(child).unwrap_or("");
    // An <a> without href does not replace an outer link already in scope.
    let link = if tag.eq_ignore_ascii_case("a") {
        ctx.dom
            .attr(child, "href")
            .map(str::to_string)
            .or_else(|| link.clone())
    } else {
        link
    };

    match &ctx.dom.get(child).kind {
        NodeKind::Text(t) => {
            let st = s.as_ref().unwrap_or(fallback);
            let boxes = flow.stack.clone();
            // Events target elements, not text nodes: the owner is the text
            // node's parent element (the document root for bare top-level text).
            let owner = ctx.dom.parent(child).unwrap_or_else(|| ctx.dom.root());
            feed_text(flow, ctx.list, t, st, link, &boxes, owner);
        }
        NodeKind::Element { .. } => {
            let st = s.as_ref().unwrap_or(fallback);
            // Fixed shares absolute's static-position box and never advances
            // the parent flow; `layout_child` routes its items to the fixed
            // channel instead of the scrolled page items.
            let absolute = st.position == Position::Absolute || st.position == Position::Fixed;
            let float_kind = st.float;
            let is_float = !absolute && float_kind != Float::None;
            // clear works on the containing flow's float bottoms; floats and
            // absolutely positioned boxes ignore it.
            let clear_y = if is_float || absolute {
                flow.y
            } else {
                match st.clear {
                    Clear::Left => flow.float_bottoms.0.max(flow.y),
                    Clear::Right => flow.float_bottoms.1.max(flow.y),
                    Clear::Both => flow.float_bottoms.0.max(flow.float_bottoms.1).max(flow.y),
                    Clear::None => flow.y,
                }
            };
            let fm = &st.margin;
            // shortcut: floats without an explicit width take the full content
            // width (no shrink-to-fit pass yet); upgrade with intrinsic sizing.
            let float_w = if is_float {
                st.width
                    .unwrap_or((flow.right - flow.left - fm.left - fm.right).max(0.0))
                    .max(st.min_size.w.max(0.0))
                    .min(st.max_size.w.max(0.0))
                    .max(0.0)
            } else {
                0.0
            };
            // Static + visible takes the original path: the offset pair is
            // zero and the clip/relocation passes are skipped entirely.
            let (dx, dy) = if st.position == Position::Relative {
                (
                    st.offsets.left.unwrap_or(0.0),
                    st.offsets.top.unwrap_or(0.0),
                )
            } else {
                (0.0, 0.0)
            };
            // shortcut: wave 1 containing block is the parent's content box;
            // right/bottom offsets are ignored for both absolute and fixed.
            // Upgrade when an ancestor padding box becomes the containing block.
            let left = st.offsets.left.unwrap_or(0.0);
            // Inline-level atomics take a slot in the line box, so they start
            // at the float-free edge (and after any atoms already on the line)
            // instead of the bare content edge.
            let inline_atomic = matches!(
                st.display,
                Display::InlineBlock
                    | Display::InlineFlex
                    | Display::InlineGrid
                    | Display::InlineTable
            );
            let float_band = crate::inline::float_bounds(flow);
            let avail = Avail {
                x: if is_float && float_kind == Float::Right {
                    // Right floats hug the content edge minus their margins.
                    flow.right - float_w - fm.left - fm.right
                } else if absolute {
                    flow.left + left
                } else if inline_atomic {
                    float_band.0 + flow.line_w
                } else {
                    flow.left
                },
                y: clear_y,
                width: if absolute {
                    (flow.right - flow.left - left).max(0.0)
                } else if inline_atomic {
                    (float_band.1 - float_band.0 - flow.line_w).max(0.0)
                } else {
                    flow.right - flow.left
                },
            };

            if crate::forms::is_form_control(tag) {
                // Form controls are atomic replaced elements (M7.3): one
                // Control item per element, never fragmented by inline flow.
                flush_line(flow, ctx.list);
                let (rec_x, rec_y) = (avail.x, avail.y);
                let start = ctx.list.items.len();
                let consumed = crate::forms::layout_control_atom(ctx, st, child, avail, link);
                shift_items(ctx.list, start, dx, dy);
                if is_float {
                    record_float(
                        flow,
                        float_kind,
                        Rect {
                            x: rec_x,
                            y: rec_y,
                            w: float_w + fm.left + fm.right,
                            h: consumed,
                        },
                    );
                } else if !absolute {
                    flow.y = clear_y + consumed;
                }
            } else if tag.eq_ignore_ascii_case("img") {
                flush_line(flow, ctx.list);
                let start = ctx.list.items.len();
                // shortcut: relative src resolution needs a base URL; the shell passes
                // absolute paths/URLs until layout grows a base-url parameter.
                if let Some(src) = ctx.dom.attr(child, "src") {
                    let w = ctx
                        .dom
                        .attr(child, "width")
                        .and_then(|v| v.parse::<f32>().ok())
                        .unwrap_or(64.0);
                    let h = ctx
                        .dom
                        .attr(child, "height")
                        .and_then(|v| v.parse::<f32>().ok())
                        .unwrap_or(64.0);
                    let img_x = if is_float {
                        match float_kind {
                            Float::Right => flow.right - fm.right - w,
                            _ => avail.x + fm.left,
                        }
                    } else {
                        avail.x
                    };
                    let img_y = if is_float { avail.y + fm.top } else { avail.y };
                    ctx.list.push(
                        child,
                        DisplayItem::Image {
                            rect: Rect {
                                x: img_x,
                                y: img_y,
                                w,
                                h,
                            },
                            path: src.to_string(),
                        },
                    );
                    if is_float {
                        record_float(
                            flow,
                            float_kind,
                            Rect {
                                x: img_x - fm.left,
                                y: avail.y,
                                w: w + fm.left + fm.right,
                                h: h + fm.top + fm.bottom,
                            },
                        );
                    } else if !absolute {
                        flow.y += h;
                    }
                    shift_items(ctx.list, start, dx, dy);
                }
            } else if tag.eq_ignore_ascii_case("br") {
                // Hard line break. An empty line still occupies one line box.
                if !absolute {
                    if has_content(flow) {
                        flush_line(flow, ctx.list);
                    } else {
                        flow.y += 1.2 * st.font_size;
                    }
                }
            } else if matches!(
                st.display,
                Display::Block
                    | Display::Flex
                    | Display::InlineFlex
                    | Display::Grid
                    | Display::InlineGrid
                    | Display::Table
                    | Display::InlineTable
                    | Display::InlineBlock
            ) {
                flush_line(flow, ctx.list);
                let start = ctx.list.items.len();
                // `avail` is consumed by the layout functions below.
                let (rec_x, rec_y, avail_w) = (avail.x, avail.y, avail.width);
                let consumed = match st.display {
                    Display::Flex | Display::InlineFlex => layout_flex(ctx, st, child, avail, link),
                    Display::Grid | Display::InlineGrid => layout_grid(ctx, st, child, avail, link),
                    Display::Table | Display::InlineTable => {
                        layout_table(ctx, st, child, avail, link)
                    }
                    Display::InlineBlock => inline_block_atom(ctx, st, child, avail, link),
                    _ => layout_block_inner(ctx, st, child, avail, link, flow),
                };
                shift_items(ctx.list, start, dx, dy);
                if inline_atomic && !is_float && !absolute {
                    // Reserve the atomic's horizontal slot in the line so the
                    // next atom advances past the float edge. The spacer carries
                    // no paint, so flushing it only clears the line state; the
                    // flow cursor is set from the atomic's consumed height below.
                    // shortcut: box width mirrors inline_block_atom; flex/grid/table
                    // atomics use the same declared-or-available width here.
                    let mut box_w = st.width.unwrap_or((avail_w - fm.left - fm.right).max(0.0));
                    box_w = box_w
                        .max(st.min_size.w.max(0.0))
                        .min(st.max_size.w.max(0.0))
                        .max(0.0);
                    push_atom(
                        flow,
                        Atom {
                            text: String::new(),
                            font_size: st.font_size,
                            color: st.color,
                            weight: st.font_weight,
                            href: None,
                            underline: false,
                            line_through: false,
                            x: float_band.0 + flow.line_w,
                            w: fm.left + box_w + fm.right,
                            boxes: Vec::new(),
                            spacer: true,
                            owner: child,
                        },
                    );
                    flow.line_w += fm.left + box_w + fm.right;
                    flush_line(flow, ctx.list);
                }
                // shortcut: vertical margins sum, no collapsing (MVP); upgrade when
                // CSS margin collapsing is required for spec fidelity.
                // Floats and absolute boxes do not advance the parent's y.
                if is_float {
                    record_float(
                        flow,
                        float_kind,
                        Rect {
                            x: rec_x,
                            y: rec_y,
                            w: float_w + fm.left + fm.right,
                            h: consumed,
                        },
                    );
                } else if !absolute {
                    flow.y = clear_y + consumed;
                }
            } else if absolute {
                // An inline absolute box paints into a scratch flow so it
                // cannot advance the parent's line box.
                let mut sub = Flow::new(avail.x, avail.x + avail.width, avail.y);
                let start = ctx.list.items.len();
                layout_inline_element(ctx, &mut sub, fallback, child, link, st);
                flush_line(&mut sub, ctx.list);
                shift_items(ctx.list, start, dx, dy);
            } else {
                // shortcut: relative offsets on in-flow inline elements are
                // not shifted: their fragments paint on a later parent line
                // break, outside the element's display-list range. Upgrade
                // when fragments carry an owning element id.
                layout_inline_element(ctx, flow, fallback, child, link, st);
            }
        }
        NodeKind::Document => layout_children(ctx, flow, fallback, child, link),
    }
}

/// Lay out one block box and return the total vertical space it consumes,
/// including its own top and bottom margins. Callers outside block flow (flex,
/// grid, table) have no float state to inherit.
pub(crate) fn layout_block(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    // An empty Flow holds only Vec::new() fields (no heap allocation).
    let empty = Flow::new(0.0, 0.0, 0.0);
    layout_block_inner(ctx, style, id, avail, link, &empty)
}

/// Block layout with the containing flow's float state threaded through: the
/// child's own line boxes wrap around the same floats and its clear can see
/// their bottoms (handled by `layout_child` when the box is positioned).
fn layout_block_inner(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
    inherited: &Flow,
) -> f32 {
    let m = &style.margin;
    let p = &style.padding;
    let bw = style.border_width.max(0.0);

    let box_x = avail.x + m.left;
    let box_y = avail.y + m.top;
    // shortcut: explicit width/min/max clamp the border box (no box-sizing
    // support); max-size wins when the two constraints cross over.
    let mut box_w = style
        .width
        .unwrap_or(avail.width - m.left - m.right)
        .max(0.0);
    box_w = box_w.max(style.min_size.w.max(0.0));
    box_w = box_w.min(style.max_size.w.max(0.0));
    let inner_left = box_x + bw + p.left;
    let inner_right = box_x + box_w - bw - p.right;
    let top = box_y + bw + p.top;

    // Reserve the background and border rects now so they paint behind the
    // children; their heights are patched once the content height is known.
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

    let mut bottom_idx = None;
    let mut left_idx = None;
    let mut right_idx = None;
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
        ); // top edge: already complete
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

    // Only an overflowing element sets aside a range start; static/visible
    // blocks skip the post-pass entirely and keep the original path.
    let clip_enabled = style.overflow != Overflow::Visible;
    let children_start = if clip_enabled {
        ctx.list.items.len()
    } else {
        0
    };

    let mut flow = Flow::new(inner_left, inner_right, top);
    flow.float_l = inherited.float_l;
    flow.float_r = inherited.float_r;
    flow.float_bottoms = inherited.float_bottoms;
    if ctx
        .dom
        .tag(id)
        .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
    {
        flow.bullet = true;
    }
    layout_children(ctx, &mut flow, style, id, link);
    flush_line(&mut flow, ctx.list);

    // Cyclic clamp: width is resolved before children lay out, height after.
    let mut content_h = (flow.y - top).max(0.0);
    content_h = content_h
        .max(style.height.unwrap_or(0.0).max(0.0))
        .max(style.min_size.h.max(0.0));
    content_h = content_h.min(style.max_size.h.max(0.0));
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

    if clip_enabled {
        // The element's own bg/border rects are before `children_start`, so
        // they are never clipped by their own overflow setting.
        clip_items(
            ctx.list,
            children_start,
            Rect {
                x: box_x + bw,
                y: box_y + bw,
                w: (box_w - 2.0 * bw).max(0.0),
                h: (box_h - 2.0 * bw).max(0.0),
            },
        );
    }

    m.top + box_h + m.bottom
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

    fn id_sel(v: &str) -> Selector {
        Selector {
            tag: None,
            id: Some(v.to_string()),
            classes: Vec::new(),
            universal: false,
        }
    }

    fn rule(sel: Selector, decls: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector: sel,
            declarations: decls.iter().map(|&(p, v)| decl(p, v)).collect(),
        }
    }

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.to_string(),
            attrs: attrs
                .iter()
                .map(|&(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        dom.append(parent, id);
        id
    }

    fn text_node(dom: &mut Dom, parent: NodeId, s: &str) {
        let id = dom.create(NodeKind::Text(s.to_string()));
        dom.append(parent, id);
    }

    fn texts(list: &DisplayList) -> Vec<(&str, Rect)> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { rect, text, .. } => Some((text.as_str(), *rect)),
                _ => None,
            })
            .collect()
    }

    fn colored_rects(list: &DisplayList, color: Color) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Rect { rect, color: c, .. } if *c == color => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn images(list: &DisplayList) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Image { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn viewport(w: f32) -> Size {
        Size { w, h: 600.0 }
    }

    #[test]
    fn relative_shifts_glyphs_but_not_flow() {
        let shifted = {
            let mut dom = Dom::new();
            let root = dom.root();
            let a = elem(&mut dom, root, "div", &[("id", "a")]);
            text_node(&mut dom, a, "shifted");
            let b = elem(&mut dom, root, "div", &[("id", "b")]);
            text_node(&mut dom, b, "after");
            let styles = styled(
                &dom,
                vec![rule(
                    id_sel("a"),
                    &[("position", "relative"), ("left", "30px"), ("top", "20px")],
                )],
            );
            layout(&dom, &styles, viewport(800.0))
        };
        let plain = {
            let mut dom = Dom::new();
            let root = dom.root();
            let a = elem(&mut dom, root, "div", &[("id", "a")]);
            text_node(&mut dom, a, "shifted");
            let b = elem(&mut dom, root, "div", &[("id", "b")]);
            text_node(&mut dom, b, "after");
            layout(&dom, &compute_styles(&dom, &[]), viewport(800.0))
        };
        let s = texts(&shifted);
        let p = texts(&plain);
        assert_eq!(s.len(), 2, "both text runs survive: {s:?}");
        assert!(
            (s[0].1.x - p[0].1.x - 30.0).abs() < 0.01 && (s[0].1.y - p[0].1.y - 20.0).abs() < 0.01,
            "relative offsets must move the glyph rect by (30, 20): {:?} vs {:?}",
            s[0],
            p[0]
        );
        assert_eq!(
            s[1].1, p[1].1,
            "relative keeps its space: next sibling unmoved"
        );
    }

    #[test]
    fn absolute_child_escapes_parent_flow() {
        let mut dom = Dom::new();
        let root = dom.root();
        let abs = elem(&mut dom, root, "div", &[("id", "abs")]);
        text_node(&mut dom, abs, "floating");
        let after = elem(&mut dom, root, "div", &[("id", "after")]);
        text_node(&mut dom, after, "after");
        let styles = styled(
            &dom,
            vec![rule(
                id_sel("abs"),
                &[
                    ("position", "absolute"),
                    ("left", "40px"),
                    ("top", "0"),
                    ("background", "#ff0000"),
                ],
            )],
        );
        let list = layout(&dom, &styles, viewport(800.0));
        let ts = texts(&list);
        assert_eq!(ts.len(), 2, "absolute text still paints: {ts:?}");
        let abs_rect = ts.iter().find(|t| t.0 == "floating").unwrap().1;
        let after_rect = ts.iter().find(|t| t.0 == "after").unwrap().1;
        assert!(
            (abs_rect.x - 40.0).abs() < 0.01 && abs_rect.y.abs() < 0.01,
            "absolute child starts at its left offset: {abs_rect:?}"
        );
        assert!(
            after_rect.x.abs() < 0.01 && after_rect.y.abs() < 0.01,
            "next sibling must not be pushed down by an absolute child: {after_rect:?}"
        );
        let red = colored_rects(&list, Color::rgb(255, 0, 0));
        assert_eq!(red.len(), 1, "absolute box paints its own background");
        assert!(
            (red[0].x - 40.0).abs() < 0.01,
            "background starts at the absolute x"
        );
    }

    #[test]
    fn min_max_width_clamp_box_and_max_wins() {
        let one = |decls: &[(&str, &str)], w: f32| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div", &[("id", "d")]);
            text_node(&mut dom, d, "x");
            let styles = styled(&dom, vec![rule(id_sel("d"), decls)]);
            colored_rects(&layout(&dom, &styles, viewport(w)), Color::rgb(0, 0, 255))[0]
        };
        let min = one(&[("min-width", "200px"), ("background", "#0000ff")], 100.0);
        assert!((min.w - 200.0).abs() < 0.01, "min-width: {min:?}");
        let max = one(&[("max-width", "50px"), ("background", "#0000ff")], 800.0);
        assert!((max.w - 50.0).abs() < 0.01, "max-width: {max:?}");
        let crossed = one(
            &[
                ("min-width", "200px"),
                ("max-width", "60px"),
                ("background", "#0000ff"),
            ],
            100.0,
        );
        assert!(
            (crossed.w - 60.0).abs() < 0.01,
            "max-width wins when constraints cross: {crossed:?}"
        );
    }

    #[test]
    fn height_min_max_clamp_box() {
        let one = |decls: &[(&str, &str)]| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div", &[("id", "d")]);
            text_node(&mut dom, d, "one short line");
            let styles = styled(&dom, vec![rule(id_sel("d"), decls)]);
            colored_rects(
                &layout(&dom, &styles, viewport(800.0)),
                Color::rgb(0, 0, 255),
            )[0]
        };
        let min = one(&[
            ("height", "10px"),
            ("min-height", "60px"),
            ("background", "#0000ff"),
        ]);
        assert!((min.h - 60.0).abs() < 0.01, "min-height: {min:?}");
        let max = one(&[
            ("height", "100px"),
            ("max-height", "30px"),
            ("background", "#0000ff"),
        ]);
        assert!((max.h - 30.0).abs() < 0.01, "max-height: {max:?}");
    }

    #[test]
    fn overflow_hidden_clips_descendants_but_keeps_own_border() {
        let page = |overflow: bool| {
            let mut dom = Dom::new();
            let root = dom.root();
            let style = if overflow {
                "width:100px; height:40px; max-height:40px; overflow:hidden; border:2px solid #ff0000; background:#eeeeee"
            } else {
                "width:100px; height:40px; max-height:40px; border:2px solid #ff0000; background:#eeeeee"
            };
            let clip = elem(&mut dom, root, "div", &[("id", "clip"), ("style", style)]);
            elem_with_src(&mut dom, clip, "/wide.png", 300.0, 100.0);
            text_node(
                &mut dom,
                clip,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            );
            let styles = compute_styles(&dom, &[]);
            layout(&dom, &styles, viewport(800.0))
        };
        let hidden = page(true);
        let visible = page(false);
        let red = Color::rgb(255, 0, 0);
        assert_eq!(
            colored_rects(&hidden, red).len(),
            4,
            "the element's own border survives clipping"
        );
        let his = images(&hidden);
        assert_eq!(his.len(), 1, "clipped image item is still emitted");
        assert!(
            his[0].w <= 100.0 && his[0].h <= 40.0,
            "image rect must be intersected with the padding box: {:?}",
            his[0]
        );
        assert!(
            images(&visible)[0].w > his[0].w,
            "without overflow the image keeps its full width"
        );
        let ht = texts(&hidden);
        let vt = texts(&visible);
        assert!(
            ht.len() < vt.len(),
            "text runs outside the clip box are dropped: {} vs {}",
            ht.len(),
            vt.len()
        );
        assert!(
            ht.iter().all(|(_, r)| r.y + r.h <= 42.01),
            "no surviving text may extend past the padding box: {ht:?}"
        );
    }

    #[test]
    fn overflow_hidden_clips_control_items() {
        let mut dom = Dom::new();
        let root = dom.root();
        let clip = elem(
            &mut dom,
            root,
            "div",
            &[(
                "style",
                "width:10px; height:10px; max-height:10px; overflow:hidden",
            )],
        );
        elem(
            &mut dom,
            clip,
            "input",
            &[("style", "width:100px; height:50px")],
        );
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, viewport(800.0));
        let controls: Vec<Rect> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Control { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect();
        assert_eq!(controls.len(), 1, "the control is still emitted");
        assert!(
            controls[0].w <= 10.01 && controls[0].h <= 10.01,
            "control rect must be intersected with the padding box: {:?}",
            controls[0]
        );
    }

    #[test]
    fn relative_inside_absolute_accumulates_offsets() {
        let mut dom = Dom::new();
        let root = dom.root();
        let abs = elem(
            &mut dom,
            root,
            "div",
            &[("style", "position:absolute; left:50px; top:10px")],
        );
        let rel = elem(
            &mut dom,
            abs,
            "div",
            &[("style", "position:relative; left:7px; top:3px")],
        );
        text_node(&mut dom, rel, "nested");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, viewport(800.0));
        let t = texts(&list);
        assert_eq!(t.len(), 1);
        // shortcut: absolute `top` is ignored in wave 1 (only `left` shifts
        // the static-position box), so only the relative top offset applies.
        assert!(
            (t[0].1.x - 57.0).abs() < 0.01 && (t[0].1.y - 3.0).abs() < 0.01,
            "relative offsets inside an absolute parent compose: {:?}",
            t[0]
        );
    }

    #[test]
    fn deep_absolute_chain_does_not_overflow() {
        let mut dom = Dom::new();
        let root = dom.root();
        let mut cur = root;
        for _ in 0..200 {
            cur = elem(
                &mut dom,
                cur,
                "div",
                &[("style", "position:absolute; left:1px")],
            );
        }
        text_node(&mut dom, cur, "deep");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, viewport(800.0));
        let t = texts(&list);
        assert_eq!(t.len(), 1, "innermost text survives a 200-deep chain");
        assert!(
            t[0].1.x.is_finite() && (t[0].1.x - 200.0).abs() < 1.0,
            "each level shifts by left:1px: {:?}",
            t[0]
        );
    }

    #[test]
    fn left_float_narrows_text_lines() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(
            &mut dom,
            root,
            "div",
            &[("style", "width:120px; font-size:10px; line-height:12px")],
        );
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:left; width:60px; height:40px; background:#99ccff",
            )],
        );
        text_node(&mut dom, c, &"a".repeat(20));
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(120.0));
        let ts = texts(&list);
        assert_eq!(ts.len(), 2, "60px gutter halves the line width: {ts:?}");
        assert!(
            (ts[0].1.x - 60.0).abs() < 0.01,
            "first line starts past the float: {:?}",
            ts[0]
        );
        assert!(
            (ts[0].1.w - 60.0).abs() < 0.01,
            "first line is 10 chars: {:?}",
            ts[0]
        );
        assert!((ts[1].1.x - 60.0).abs() < 0.01, "second line still wraps");
    }

    #[test]
    fn right_float_mirrors_the_gutter() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(
            &mut dom,
            root,
            "div",
            &[("style", "width:120px; font-size:10px; line-height:12px")],
        );
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:right; width:60px; height:40px; margin-left:0; background:#ffcc99",
            )],
        );
        text_node(&mut dom, c, &"b".repeat(20));
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(120.0));
        let ts = texts(&list);
        assert_eq!(ts.len(), 2, "right float narrows the line too: {ts:?}");
        assert!(
            (ts[0].1.x).abs() < 0.01,
            "text starts at the content edge: {:?}",
            ts[0]
        );
        assert!(
            (ts[0].1.w - 60.0).abs() < 0.01,
            "text stops before the float: {:?}",
            ts[0]
        );
        let orange = colored_rects(&list, Color::rgb(0xff, 0xcc, 0x99));
        assert_eq!(orange.len(), 1);
        assert!(
            (orange[0].x + orange[0].w - 120.0).abs() < 0.01,
            "right float hugs the content edge: {:?}",
            orange[0]
        );
    }

    #[test]
    fn clear_both_drops_block_below_float_bottoms() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(&mut dom, root, "div", &[("style", "width:200px")]);
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:left; width:60px; height:40px; background:#99ccff",
            )],
        );
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:right; width:60px; height:25px; background:#ffcc99",
            )],
        );
        let cleared = elem(
            &mut dom,
            c,
            "div",
            &[("style", "clear:both; height:10px; background:#ddd0e8")],
        );
        text_node(&mut dom, cleared, "below");
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(200.0));
        let purple = colored_rects(&list, Color::rgb(0xdd, 0xd0, 0xe8));
        assert_eq!(purple.len(), 1);
        assert!(
            (purple[0].y - 40.0).abs() < 0.01,
            "clear:both must use the taller (left) float bottom: {:?}",
            purple[0]
        );
    }

    #[test]
    fn float_bottom_passing_mid_block_only_narrows_affected_lines() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(
            &mut dom,
            root,
            "div",
            &[("style", "width:120px; font-size:10px; line-height:12px")],
        );
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:left; width:60px; height:30px; background:#99ccff",
            )],
        );
        text_node(&mut dom, c, &"a".repeat(50));
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(120.0));
        let ts = texts(&list);
        assert_eq!(ts.len(), 4, "3 narrowed lines then a full line: {ts:?}");
        for (i, (_, r)) in ts.iter().enumerate().take(3) {
            assert!(
                (r.x - 60.0).abs() < 0.01 && (r.w - 60.0).abs() < 0.01,
                "line {i} wraps: {r:?}"
            );
        }
        assert!(
            (ts[3].1.x).abs() < 0.01 && (ts[3].1.w - 120.0).abs() < 0.01,
            "line 3 is full width: {:?}",
            ts[3]
        );
    }

    #[test]
    fn block_taller_than_float_resumes_full_width() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(
            &mut dom,
            root,
            "div",
            &[("style", "width:120px; font-size:10px; line-height:12px")],
        );
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:right; width:60px; height:15px; background:#ffcc99",
            )],
        );
        text_node(&mut dom, c, &"x".repeat(36));
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(120.0));
        let ts = texts(&list);
        // First line (y=0) is narrowed to 10 chars; the float ends at y=15 so
        // the second line (y=12) is still narrowed but the third is full width.
        assert!(
            (ts[2].1.x).abs() < 0.01 && (ts[2].1.w - 96.0).abs() < 0.01,
            "after the float bottom all 16 remaining chars fit on one line: {:?}",
            ts[2]
        );
    }

    #[test]
    fn floated_image_wraps_text_without_pushing_it_down() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(
            &mut dom,
            root,
            "div",
            &[("style", "width:200px; font-size:10px; line-height:12px")],
        );
        let img = elem_with_src(&mut dom, c, "/icon.png", 40.0, 40.0);
        dom.set_attr(img, "style", "float:left; margin:0 10px 10px 0".to_string());
        text_node(&mut dom, c, &"z".repeat(30));
        let list = layout(&dom, &compute_styles(&dom, &[]), viewport(200.0));
        let imgs = images(&list);
        assert_eq!(imgs.len(), 1);
        assert!(
            (imgs[0].x).abs() < 0.01 && (imgs[0].y).abs() < 0.01,
            "float image at the content origin: {:?}",
            imgs[0]
        );
        let ts = texts(&list);
        assert!(
            (ts[0].1.x - 50.0).abs() < 0.01 && (ts[0].1.y).abs() < 0.01,
            "text starts past the image margin box on the same y: {:?}",
            ts[0]
        );
    }

    #[test]
    fn inline_block_atom_respects_a_left_float_band() {
        let mut dom = Dom::new();
        let root = dom.root();
        let c = elem(&mut dom, root, "div", &[("style", "width:200px")]);
        elem(
            &mut dom,
            c,
            "div",
            &[(
                "style",
                "float:left; width:100px; height:40px; background:#99ccff",
            )],
        );
        elem(
            &mut dom,
            c,
            "span",
            &[(
                "style",
                "display:inline-block; width:80px; height:20px; background:#ff0000",
            )],
        );
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, viewport(200.0));
        let red = colored_rects(&list, Color::rgb(255, 0, 0));
        assert_eq!(red.len(), 1, "the inline-block paints one box: {red:?}");
        assert!(
            red[0].x >= 100.0,
            "inline-block must start past the 100px left float: {:?}",
            red[0]
        );
    }

    #[test]
    fn clear_on_empty_float_state_is_noop() {
        let build = |clear: bool| {
            let mut dom = Dom::new();
            let root = dom.root();
            let a = elem(&mut dom, root, "div", &[]);
            text_node(&mut dom, a, "first");
            let b = elem(
                &mut dom,
                root,
                "div",
                if clear {
                    &[("style", "clear:both")]
                } else {
                    &[]
                },
            );
            text_node(&mut dom, b, "second");
            layout(&dom, &compute_styles(&dom, &[]), viewport(200.0))
        };
        let plain_list = build(false);
        let cleared_list = build(true);
        let plain = texts(&plain_list);
        let cleared = texts(&cleared_list);
        assert_eq!(plain.len(), 2);
        assert_eq!(
            plain[1].1, cleared[1].1,
            "clear with no floats must not move the block"
        );
    }

    fn elem_with_src(dom: &mut Dom, parent: NodeId, src: &str, w: f32, h: f32) -> NodeId {
        let ws = w.to_string();
        let hs = h.to_string();
        elem(
            dom,
            parent,
            "img",
            &[("src", src), ("width", &ws), ("height", &hs)],
        )
    }
}

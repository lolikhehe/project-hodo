//! Block-flow + inline formatting layout producing a flat display list.
//!
//! Scope and documented shortcuts:
//! - Geometry uses approximate metrics: char width = 0.6 * font_size, line
//!   height = 1.2 * font_size. Real fontdue ascent/descent metrics in layout
//!   are a follow-up (the renderer already advances glyphs with fontdue).
//! - Inline formatting contexts produce real line boxes. An inline element
//!   paints its background and border once per line fragment. Horizontal
//!   margin/padding/border shift text within the line; vertical ones do not.
//! - `white-space: normal` only; `<br>` forces a hard break and long words are
//!   hard-broken in linear work (one bounded scan per emitted chunk).
//! - One Text item per line (first-atom color/font/href wins) so link clicks
//!   map to the line fragment that was clicked, not a whole-element box.
//! - Deferred to the property-plumbing sweep (ComputedStyle is owned by the
//!   hodo-style/selectors wave): `vertical-align`, `line-height`, `text-align`,
//!   `text-indent`, `letter-spacing`, `word-spacing`, `overflow-wrap`,
//!   non-normal `white-space`, `display: inline-block`, font fallback/synthesis,
//!   text-decoration rasterization and bidi/complex scripts.
//! - Vertical margins sum, never collapse. Unknown nodes are transparent
//!   containers.

use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_style::{ComputedStyle, Display, StyleMap};
use hodo_types::{Color, Edges, Rect, Size};

#[derive(Debug, Clone)]
pub enum DisplayItem {
    Rect {
        rect: Rect,
        color: Color,
        border_color: Option<Color>,
        radius: f32,
    },
    Text {
        rect: Rect,
        text: String,
        color: Color,
        font_size: f32,
        weight: u16,
        href: Option<String>,
    },
    Image {
        rect: Rect,
        path: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct DisplayList {
    pub items: Vec<DisplayItem>,
}

/// Lay out `dom` into a flat display list for a viewport of `size`.
pub fn layout(dom: &Dom, styles: &StyleMap, viewport: Size) -> DisplayList {
    let mut list = Vec::new();
    {
        let mut ctx = Ctx {
            dom,
            styles,
            list: &mut list,
        };
        let fallback = fallback_style();
        let mut flow = Flow::new(0.0, viewport.w, 0.0);
        layout_children(&mut ctx, &mut flow, &fallback, dom.root(), None);
        flush_line(&mut flow, &mut *ctx.list);
    }
    DisplayList { items: list }
}

/// Borrowed inputs shared by the recursive layout walk.
struct Ctx<'a> {
    dom: &'a Dom,
    styles: &'a StyleMap,
    list: &'a mut Vec<DisplayItem>,
}

/// The available box handed to a block: left edge, top edge and usable width.
struct Avail {
    x: f32,
    y: f32,
    width: f32,
}

/// One rendered word, collapsed space, or inline-box inset spacer waiting to
/// be placed on a line. `spacer` atoms carry geometry but no glyphs.
struct Atom {
    text: String,
    font_size: f32,
    color: Color,
    weight: u16,
    href: Option<String>,
    /// Absolute x where this atom starts.
    x: f32,
    /// Advance width contributed to the line.
    w: f32,
    /// Inline boxes containing this atom, outermost first (indices into
    /// `Flow::boxes`). Empty for ordinary text.
    boxes: Vec<usize>,
    spacer: bool,
}

/// Paint state of one inline element inside a block's flow.
struct InlineBox {
    bg: Color,
    border_color: Color,
    border: Edges<f32>,
    margin: Edges<f32>,
    padding: Edges<f32>,
    /// Already painted a fragment on an earlier line: suppress left border.
    seen: bool,
    /// Element is still open: suppress right border on this fragment.
    active: bool,
}

/// Mutable inline-flow state for a single block's content area.
struct Flow {
    left: f32,
    right: f32,
    y: f32,
    line: Vec<Atom>,
    line_w: f32,
    pending_space: Option<f32>,
    /// One bullet is emitted on this flow's first text line (list item).
    bullet: bool,
    /// Every paintable inline element encountered in this flow.
    boxes: Vec<InlineBox>,
    /// Indices of boxes currently open, outermost first.
    stack: Vec<usize>,
}

impl Flow {
    fn new(left: f32, right: f32, y: f32) -> Self {
        Self {
            left,
            right,
            y,
            line: Vec::new(),
            line_w: 0.0,
            pending_space: None,
            bullet: false,
            boxes: Vec::new(),
            stack: Vec::new(),
        }
    }
}

/// True when the current line has at least one glyph-bearing atom. Spacer-only
/// lines are not a wrap boundary and never produce a Text item.
fn has_content(flow: &Flow) -> bool {
    flow.line.iter().any(|a| !a.spacer)
}

/// Style used for text nodes that have no computed style of their own.
fn fallback_style() -> ComputedStyle {
    ComputedStyle {
        display: Display::Inline,
        color: Color::BLACK,
        background: Color::TRANSPARENT,
        font_size: 16.0,
        font_weight: 400,
        italic: false,
        underline: false,
        margin: Edges::zero(),
        padding: Edges::zero(),
        border_width: 0.0,
        border_widths: Edges::zero(),
        border_color: Color::BLACK,
    }
}

fn push_rect(list: &mut Vec<DisplayItem>, rect: Rect, color: Color) -> usize {
    let idx = list.len();
    list.push(DisplayItem::Rect {
        rect,
        color,
        border_color: None,
        radius: 0.0,
    });
    idx
}

fn patch_rect(list: &mut [DisplayItem], idx: usize, f: impl FnOnce(&mut Rect)) {
    if let DisplayItem::Rect { rect, .. } = &mut list[idx] {
        f(rect);
    }
}

fn push_atom(flow: &mut Flow, atom: Atom) {
    flow.line.push(atom);
}

/// Whether a child participates as a block-level sibling for anonymous-block
/// splitting. `<img>` is an atomic replaced element in this engine.
fn is_block_level(ctx: &Ctx<'_>, id: NodeId) -> bool {
    // Text nodes inherit their parent's computed `display`, so only elements
    // may be classified as block-level siblings here.
    matches!(ctx.dom.get(id).kind, NodeKind::Element { .. })
        && (ctx
            .styles
            .get(id)
            .is_some_and(|s| s.display == Display::Block)
            || ctx
                .dom
                .tag(id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("img")))
}

/// Walk a node's children in order. Mixed inline/block child lists are split
/// into anonymous block boxes: each consecutive inline run gets its own flow,
/// so it cannot share a line box across a block sibling. Pure lists keep the
/// original single-flow behaviour.
fn layout_children(
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
fn layout_inline_run(
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
    for &child in run {
        layout_child(ctx, &mut sub, fallback, child, link.clone());
    }
    flush_line(&mut sub, ctx.list);
    flow.y = sub.y;
}

/// Lay out a single child into `flow`.
fn layout_child(
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
            feed_text(flow, ctx.list, t, st, link, &boxes);
        }
        NodeKind::Element { .. } => {
            if tag.eq_ignore_ascii_case("img") {
                flush_line(flow, ctx.list);
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
                    ctx.list.push(DisplayItem::Image {
                        rect: Rect {
                            x: flow.left,
                            y: flow.y,
                            w,
                            h,
                        },
                        path: src.to_string(),
                    });
                    flow.y += h;
                }
            } else if tag.eq_ignore_ascii_case("br") {
                // Hard line break. An empty line still occupies one line box.
                if has_content(flow) {
                    flush_line(flow, ctx.list);
                } else {
                    flow.y += 1.2 * s.as_ref().unwrap_or(fallback).font_size;
                }
            } else if s.as_ref().is_some_and(|st| st.display == Display::Block) {
                flush_line(flow, ctx.list);
                let avail = Avail {
                    x: flow.left,
                    y: flow.y,
                    width: flow.right - flow.left,
                };
                let consumed = layout_block(ctx, s.as_ref().unwrap(), child, avail, link);
                // shortcut: vertical margins sum, no collapsing (MVP); upgrade when
                // CSS margin collapsing is required for spec fidelity.
                flow.y += consumed;
            } else {
                let st = s.as_ref().unwrap_or(fallback);
                let paintable =
                    st.background.a > 0 || (st.border_color.a > 0 && st.border_width > 0.0);
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
                                x: flow.left + flow.line_w,
                                w: left,
                                boxes: boxes.clone(),
                                spacer: true,
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
                                x: flow.left + flow.line_w,
                                w: right,
                                boxes,
                                spacer: true,
                            },
                        );
                        flow.line_w += right;
                    }
                }
            }
        }
        NodeKind::Document => layout_children(ctx, flow, fallback, child, link),
    }
}

fn left_inset(b: &InlineBox) -> f32 {
    b.margin.left + b.border.left + b.padding.left
}

fn right_inset(b: &InlineBox) -> f32 {
    b.padding.right + b.border.right + b.margin.right
}

/// Lay out one block box and return the total vertical space it consumes,
/// including its own top and bottom margins.
fn layout_block(
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

    // Reserve the background and border rects now so they paint behind the
    // children; their heights are patched once the content height is known.
    let mut bg_idx = None;
    if style.background != Color::TRANSPARENT {
        bg_idx = Some(push_rect(
            ctx.list,
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
            Rect {
                x: box_x + box_w - bw,
                y: box_y,
                w: bw,
                h: 0.0,
            },
            style.border_color,
        ));
    }

    let mut flow = Flow::new(inner_left, inner_right, top);
    if ctx
        .dom
        .tag(id)
        .is_some_and(|tag| tag.eq_ignore_ascii_case("li"))
    {
        flow.bullet = true;
    }
    layout_children(ctx, &mut flow, style, id, link);
    flush_line(&mut flow, ctx.list);

    let content_h = (flow.y - top).max(0.0);
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

/// Paint the per-line fragments of every inline box that contributed to the
/// current line. A fragment's extent is the union of all atoms carrying that
/// box (including its inset spacers), so nested boxes compose correctly.
fn emit_fragments(flow: &mut Flow, list: &mut Vec<DisplayItem>, h: f32) {
    let mut extents: Vec<(usize, f32, f32)> = Vec::new();
    for atom in &flow.line {
        for &bi in &atom.boxes {
            if let Some(e) = extents.iter_mut().find(|e| e.0 == bi) {
                e.1 = e.1.min(atom.x);
                e.2 = e.2.max(atom.x + atom.w);
            } else {
                extents.push((bi, atom.x, atom.x + atom.w));
            }
        }
    }

    for (bi, x0, x1) in extents {
        let (bg, border_color, border, seen, active) = {
            let b = &flow.boxes[bi];
            (b.bg, b.border_color, b.border, b.seen, b.active)
        };
        let first = !seen;
        let last = !active;
        let w = (x1 - x0).max(0.0);
        let y = flow.y;
        if bg.a > 0 {
            push_rect(list, Rect { x: x0, y, w, h }, bg);
        }
        if border_color.a > 0 {
            let bt = border.top;
            let bb = border.bottom;
            if bt > 0.0 {
                push_rect(list, Rect { x: x0, y, w, h: bt }, border_color);
            }
            if bb > 0.0 {
                push_rect(
                    list,
                    Rect {
                        x: x0,
                        y: y + h - bb,
                        w,
                        h: bb,
                    },
                    border_color,
                );
            }
            // Left/right borders belong only to the first/last fragment.
            if first && border.left > 0.0 {
                push_rect(
                    list,
                    Rect {
                        x: x0,
                        y,
                        w: border.left,
                        h,
                    },
                    border_color,
                );
            }
            if last && border.right > 0.0 {
                push_rect(
                    list,
                    Rect {
                        x: x0 + w - border.right,
                        y,
                        w: border.right,
                        h,
                    },
                    border_color,
                );
            }
        }
        flow.boxes[bi].seen = true;
    }
}

/// Commit the current visual line as Text (fragments first) and advance y.
fn flush_line(flow: &mut Flow, list: &mut Vec<DisplayItem>) {
    if flow.line.is_empty() {
        return;
    }
    let line_h = flow
        .line
        .iter()
        .map(|a| 1.2 * a.font_size)
        .fold(0.0_f32, f32::max);
    emit_fragments(flow, list, line_h);

    let mut text = String::new();
    let mut first: Option<&Atom> = None;
    for atom in &flow.line {
        if atom.spacer {
            continue;
        }
        if first.is_none() {
            first = Some(atom);
        }
        text.push_str(&atom.text);
    }
    // shortcut: a line mixing linked and plain atoms takes its link from the first
    // atom, matching the first-atom-wins style rule; upgrade for per-run hit boxes.
    if let Some(f) = first {
        if flow.bullet {
            // shortcut: UA ::marker emulation, real markers when list-style engine lands
            text.insert_str(0, "• ");
            flow.bullet = false;
        }
        list.push(DisplayItem::Text {
            rect: Rect {
                // Start at the first glyph, past any leading inline-box inset.
                x: f.x,
                y: flow.y,
                w: flow.line_w - (f.x - flow.left),
                h: line_h,
            },
            text,
            color: f.color,
            font_size: f.font_size,
            weight: f.weight,
            href: f.href.clone(),
        });
    }
    flow.y += line_h;
    flow.line.clear();
    flow.line_w = 0.0;
    flow.pending_space = None;
}

/// Byte index just after the `n`-th character of `s` (or `s.len()` if shorter).
fn char_boundary(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}

/// Place a word into the flow, wrapping at the content width. Words longer
/// than a whole line are hard-broken. Each chunk scan is bounded by the chars
/// actually emitted on that line, so total work is linear in the text length.
fn place_word(flow: &mut Flow, list: &mut Vec<DisplayItem>, atom: Atom) {
    let cw = 0.6 * atom.font_size;
    let max_w = (flow.right - flow.left).max(0.0);
    let total = atom.text.chars().count();
    let mut remaining = total;
    let mut word = atom.text.as_str();
    let mut guard = 0usize;
    while remaining > 0 {
        guard += 1;
        debug_assert!(
            guard <= total.saturating_add(1),
            "hard-break loop must stay linear in text length"
        );
        let word_w = remaining as f32 * cw;
        let space_w = flow.pending_space.unwrap_or(0.0);
        if has_content(flow) && flow.line_w + space_w + word_w > max_w {
            flush_line(flow, list);
            flow.pending_space = None;
            continue;
        }
        if word_w > max_w {
            // n >= 1 guarantees progress even when max_w < one character.
            let n = ((max_w / cw).floor() as usize).clamp(1, remaining);
            let end = char_boundary(word, n);
            let (chunk, rest) = word.split_at(end);
            let chunk_w = n as f32 * cw;
            push_atom(
                flow,
                Atom {
                    text: chunk.to_string(),
                    font_size: atom.font_size,
                    color: atom.color,
                    weight: atom.weight,
                    href: atom.href.clone(),
                    x: flow.left + flow.line_w,
                    w: chunk_w,
                    boxes: atom.boxes.clone(),
                    spacer: false,
                },
            );
            flow.line_w += chunk_w;
            flush_line(flow, list);
            flow.pending_space = None;
            remaining -= n;
            word = rest;
            continue;
        }
        if let Some(sw) = flow.pending_space {
            if has_content(flow) {
                push_atom(
                    flow,
                    Atom {
                        text: " ".to_string(),
                        font_size: atom.font_size,
                        color: atom.color,
                        weight: atom.weight,
                        href: atom.href.clone(),
                        x: flow.left + flow.line_w,
                        w: sw,
                        boxes: atom.boxes.clone(),
                        spacer: false,
                    },
                );
                flow.line_w += sw;
            }
        }
        flow.pending_space = None;
        push_atom(
            flow,
            Atom {
                text: word.to_string(),
                font_size: atom.font_size,
                color: atom.color,
                weight: atom.weight,
                href: atom.href,
                x: flow.left + flow.line_w,
                w: word_w,
                boxes: atom.boxes.clone(),
                spacer: false,
            },
        );
        flow.line_w += word_w;
        break;
    }
}

/// Feed a text node into the inline flow: whitespace runs collapse to a
/// single space, pure-whitespace nodes are skipped.
fn feed_text(
    flow: &mut Flow,
    list: &mut Vec<DisplayItem>,
    text: &str,
    style: &ComputedStyle,
    href: Option<String>,
    boxes: &[usize],
) {
    if text.chars().all(char::is_whitespace) {
        return;
    }
    let fs = style.font_size;
    let color = style.color;
    let weight = style.font_weight;
    if has_content(flow) && text.chars().next().is_some_and(char::is_whitespace) {
        flow.pending_space = Some(0.6 * fs);
    }
    let mut first = true;
    for word in text.split_whitespace() {
        if !first && has_content(flow) {
            flow.pending_space = Some(0.6 * fs);
        }
        place_word(
            flow,
            list,
            Atom {
                text: word.to_string(),
                font_size: fs,
                color,
                weight,
                href: href.clone(),
                x: 0.0,
                w: 0.0,
                boxes: boxes.to_vec(),
                spacer: false,
            },
        );
        first = false;
    }
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

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str) -> NodeId {
        elem_with_attrs(dom, parent, tag, &[])
    }

    fn elem_with_attrs(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.to_string(),
            attrs: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        dom.append(parent, id);
        id
    }

    fn text(dom: &mut Dom, parent: NodeId, value: &str) {
        let id = dom.create(NodeKind::Text(value.to_string()));
        dom.append(parent, id);
    }

    fn rects(list: &DisplayList) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Rect { rect, .. } => Some(*rect),
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

    fn text_rects(list: &DisplayList) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { rect, .. } => Some(*rect),
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

    #[test]
    fn sibling_blocks_with_margins_stack_in_order() {
        let mut dom = Dom::new();
        let root = dom.root();
        let a = elem(&mut dom, root, "div");
        text(&mut dom, a, "A");
        let b = elem(&mut dom, root, "div");
        text(&mut dom, b, "B");

        // Both shorthand and longhands are supplied so the cascade picks up
        // margins regardless of which form the style crate supports.
        let styles = styled(
            &dom,
            vec![tag_rule(
                "div",
                &[
                    ("margin", "10px"),
                    ("margin-top", "10px"),
                    ("margin-right", "10px"),
                    ("margin-bottom", "10px"),
                    ("margin-left", "10px"),
                    ("background", "#ff0000"),
                ],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let rs = rects(&list);
        assert_eq!(rs.len(), 2, "expected one background rect per div");
        assert!(
            rs[0].y < rs[1].y,
            "first div must be painted above the second: {:?}",
            rs
        );
        assert!(
            rs[1].y > rs[0].y + 10.0,
            "bottom margin of the first div must separate the boxes"
        );
    }

    #[test]
    fn text_wraps_at_viewport_width() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(
            &mut dom,
            d,
            "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu",
        );

        // Empty author sheet: rely on the UA defaults (div is a block, 16px).
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 90.0, h: 600.0 });
        let lines = texts(&list);
        assert!(
            lines.len() >= 2,
            "expected the paragraph to wrap into at least two lines, got {:?}",
            lines
        );
    }

    #[test]
    fn padding_shifts_text_x() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "hello");
        let styles = styled(
            &dom,
            vec![tag_rule(
                "div",
                &[("padding-left", "20px"), ("padding", "0 0 0 20px")],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let t = list
            .items
            .iter()
            .find_map(|it| match it {
                DisplayItem::Text { rect, .. } => Some(rect),
                _ => None,
            })
            .expect("expected a Text item");
        assert!(
            (t.x - 20.0).abs() < 1.0,
            "left padding should shift text to x=20, got {}",
            t.x
        );
    }

    #[test]
    fn display_none_skips_subtree() {
        let mut dom = Dom::new();
        let root = dom.root();
        let hidden = elem(&mut dom, root, "div");
        let inner = elem(&mut dom, hidden, "div");
        text(&mut dom, inner, "invisible");
        text(&mut dom, hidden, "also invisible");
        let styles = styled(&dom, vec![tag_rule("div", &[("display", "none")])]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert!(
            list.items.is_empty(),
            "a display:none subtree must produce zero items, got {:?}",
            list.items
        );
    }

    #[test]
    fn border_emits_four_edges_around_background() {
        let mut dom = Dom::new();
        let root = dom.root();
        let _d = elem(&mut dom, root, "div");
        let styles = styled(
            &dom,
            vec![tag_rule(
                "div",
                &[
                    ("background", "#00ff00"),
                    ("border-width", "2px"),
                    ("border-color", "#ff0000"),
                    ("border", "2px solid #ff0000"),
                ],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let rect_items: Vec<_> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Rect { rect, color, .. } => Some((*rect, *color)),
                _ => None,
            })
            .collect();
        assert_eq!(
            rect_items.len(),
            5,
            "expected one background rect plus four border edges"
        );
        let red = Color::rgb(255, 0, 0);
        let edges = rect_items.iter().filter(|(_, c)| *c == red).count();
        assert_eq!(edges, 4, "expected exactly four red border edges");
        assert!(
            rect_items[0].0.w > 0.0 && rect_items[0].0.h >= 0.0,
            "background rect must span the box"
        );
    }

    #[test]
    fn empty_sheet_uses_ua_defaults() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "hi");
        let styles = compute_styles(&dom, &[]);
        let cs = styles.get(d).expect("UA style for div");
        assert_eq!(cs.display, Display::Block);
        assert!((cs.font_size - 16.0).abs() < f32::EPSILON);
        assert_eq!(cs.color, Color::BLACK);
        assert_eq!(cs.background, Color::TRANSPARENT);

        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert_eq!(texts(&list).len(), 1);
    }

    #[test]
    fn li_text_gets_one_bullet_prefix() {
        let mut dom = Dom::new();
        let root = dom.root();
        let ul = elem(&mut dom, root, "ul");
        let li = elem(&mut dom, ul, "li");
        text(&mut dom, li, "first item");
        let div = elem(&mut dom, root, "div");
        text(&mut dom, div, "plain");

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let lines = texts(&list);
        assert!(
            lines.contains(&"• first item"),
            "li text must carry the bullet marker: {lines:?}"
        );
        assert!(
            lines.contains(&"plain"),
            "non-list text must stay unprefixed: {lines:?}"
        );
        assert_eq!(
            lines.iter().filter(|s| s.starts_with('•')).count(),
            1,
            "only the list item's first line gets a bullet"
        );
    }

    #[test]
    fn text_in_anchor_carries_href() {
        let mut dom = Dom::new();
        let root = dom.root();
        let a = elem_with_attrs(&mut dom, root, "a", &[("href", "/target")]);
        text(&mut dom, a, "linked");
        let p = elem(&mut dom, root, "p");
        text(&mut dom, p, "plain");

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let links: Vec<_> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { text, href, .. } => Some((text.as_str(), href.clone())),
                _ => None,
            })
            .collect();
        assert!(
            links.contains(&("linked", Some("/target".to_string()))),
            "anchor text must carry its href: {links:?}"
        );
        assert!(
            links.contains(&("plain", None)),
            "text outside an anchor must have no href: {links:?}"
        );
    }

    #[test]
    fn img_emits_image_item_in_flow() {
        let mut dom = Dom::new();
        let root = dom.root();
        elem_with_attrs(
            &mut dom,
            root,
            "img",
            &[("src", "/p.png"), ("width", "30"), ("height", "40")],
        );
        elem_with_attrs(&mut dom, root, "img", &[("src", "/q.png")]);
        elem(&mut dom, root, "img"); // no src -> skipped

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let images: Vec<_> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Image { rect, path } => Some((*rect, path.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(images.len(), 2, "only images with a src are emitted");
        assert_eq!(images[0].1, "/p.png");
        assert!((images[0].0.w - 30.0).abs() < f32::EPSILON);
        assert!((images[0].0.h - 40.0).abs() < f32::EPSILON);
        assert_eq!(images[1].1, "/q.png");
        assert!((images[1].0.w - 64.0).abs() < f32::EPSILON);
        assert!((images[1].0.h - 64.0).abs() < f32::EPSILON);
        assert!(
            images[1].0.y >= images[0].0.y + images[0].0.h,
            "images stack in flow"
        );
    }

    // ---- M7.2: inline formatting context ---------------------------------

    fn span_page(
        word: &str,
        viewport_w: f32,
        decls: &[(&str, &str)],
    ) -> (Dom, StyleMap, DisplayList) {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let span = elem(&mut dom, d, "span");
        text(&mut dom, span, word);
        let styles = styled(
            &dom,
            vec![
                tag_rule("div", &[("font-size", "10px")]),
                tag_rule("span", decls),
            ],
        );
        let list = layout(
            &dom,
            &styles,
            Size {
                w: viewport_w,
                h: 600.0,
            },
        );
        (dom, styles, list)
    }

    #[test]
    fn br_forces_a_new_line() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "before");
        elem(&mut dom, d, "br");
        text(&mut dom, d, "after");

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert_eq!(
            texts(&list),
            ["before", "after"],
            "br must split the two text runs into separate lines"
        );
        let r = text_rects(&list);
        assert_eq!(r.len(), 2);
        assert!(
            r[0].y + r[0].h <= r[1].y,
            "br must push 'after' below 'before': {r:?}"
        );
    }

    #[test]
    fn long_word_hard_breaks_in_linear_time() {
        // 10 chars per line at 10px font in a 60px column; 5k chars => 500 lines.
        let word = "x".repeat(5_000);
        let started = std::time::Instant::now();
        let (_, _, list) = span_page(&word, 60.0, &[]);
        let elapsed = started.elapsed();
        let lines = texts(&list);
        assert_eq!(
            lines.len(),
            500,
            "each hard-broken line holds exactly 10 characters"
        );
        assert!(lines.iter().all(|l| l.len() == 10));
        assert!(
            elapsed.as_secs_f64() < 2.0,
            "hard break must not be quadratic: took {elapsed:?}"
        );
    }

    #[test]
    fn inline_span_background_fragments_across_two_lines() {
        let word = "abcdefghijklmnopqrst"; // 20 chars -> 2 lines of 10
        let (_, _, list) = span_page(word, 60.0, &[("background", "#0000ff")]);
        let bg = Color::rgb(0, 0, 255);
        let rects = colored_rects(&list, bg);
        assert_eq!(
            rects.len(),
            2,
            "one background rect per line fragment, got {rects:?}"
        );
        assert!(
            rects[0].y + rects[0].h <= rects[1].y,
            "fragments must occupy disjoint y ranges: {rects:?}"
        );
        assert!(rects
            .iter()
            .all(|r| (r.x - 0.0).abs() < 0.01 && (r.w - 60.0).abs() < 0.01));
        assert_eq!(texts(&list).len(), 2, "per-line Text items survive");
    }

    #[test]
    fn inline_fragment_borders_only_at_outer_edges() {
        let word = "abcdefghijklmnopqrst";
        let (_, _, list) = span_page(
            word,
            60.0,
            &[("background", "#0000ff"), ("border", "2px solid #ff0000")],
        );
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        let bg = colored_rects(&list, blue);
        let edges = colored_rects(&list, red);
        assert_eq!(bg.len(), 2);
        assert_eq!(
            edges.len(),
            6,
            "top+bottom on both fragments, left on first, right on last: {edges:?}"
        );

        // First fragment: left edge present, right edge absent.
        let first = bg[0];
        let has_left = edges
            .iter()
            .any(|e| (e.x - first.x).abs() < 0.01 && (e.w - 2.0).abs() < 0.01 && e.y == first.y);
        let has_right = edges.iter().any(|e| {
            ((e.x + e.w) - (first.x + first.w)).abs() < 0.01
                && (e.w - 2.0).abs() < 0.01
                && e.y == first.y
        });
        assert!(has_left, "first fragment must paint its left border");
        assert!(!has_right, "first fragment must not paint a right border");

        // Last fragment: right edge present, left edge absent.
        let last = bg[1];
        let has_left = edges
            .iter()
            .any(|e| (e.x - last.x).abs() < 0.01 && (e.w - 2.0).abs() < 0.01 && e.y == last.y);
        let has_right = edges.iter().any(|e| {
            ((e.x + e.w) - (last.x + last.w)).abs() < 0.01
                && (e.w - 2.0).abs() < 0.01
                && e.y == last.y
        });
        assert!(!has_left, "last fragment must not paint a left border");
        assert!(has_right, "last fragment must paint its right border");
    }

    #[test]
    fn mixed_inline_and_block_split_into_anonymous_blocks() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "before");
        let p = elem(&mut dom, d, "p");
        text(&mut dom, p, "block");
        text(&mut dom, d, "after");

        let styles = styled(&dom, vec![tag_rule("p", &[("background", "#00ff00")])]);
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        assert_eq!(
            texts(&list),
            ["before", "block", "after"],
            "inline runs must be separate from the block child"
        );
        let tr = text_rects(&list);
        let pbg = colored_rects(&list, Color::rgb(0, 255, 0));
        assert_eq!(pbg.len(), 1, "the block child paints one background");
        assert!(tr[0].y + tr[0].h <= pbg[0].y, "leading run above the block");
        assert!(pbg[0].y <= tr[2].y, "trailing run below the block");
    }

    #[test]
    fn inline_horizontal_insets_apply_vertical_do_not() {
        let mut dom = Dom::new();
        let root = dom.root();
        let span = elem(&mut dom, root, "span");
        text(&mut dom, span, "hi");
        let styles = styled(
            &dom,
            vec![tag_rule(
                "span",
                &[
                    ("background", "#ff0000"),
                    ("margin", "20px 5px"),
                    ("padding", "10px 5px"),
                ],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        let tr = text_rects(&list);
        assert_eq!(tr.len(), 1);
        assert!(
            (tr[0].x - 10.0).abs() < 0.01,
            "5px margin + 5px padding shift text right: {:?}",
            tr[0]
        );
        assert!(
            (tr[0].y - 0.0).abs() < 0.01,
            "vertical margin/padding must not move the line: {:?}",
            tr[0]
        );
        assert!(
            (tr[0].h - 19.2).abs() < 0.01,
            "line height stays 1.2 * font-size: {:?}",
            tr[0]
        );
        let bg = colored_rects(&list, Color::rgb(255, 0, 0));
        assert_eq!(bg.len(), 1);
        assert!(
            bg[0].x <= tr[0].x && bg[0].y == tr[0].y && bg[0].h == tr[0].h,
            "fragment spans the text plus horizontal insets: {:?}",
            bg[0]
        );
    }

    #[test]
    fn wrapped_link_keeps_href_on_every_line_fragment() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let a = elem_with_attrs(&mut dom, d, "a", &[("href", "/go")]);
        text(&mut dom, a, "abcdefghijklmnopqrst");
        let styles = styled(
            &dom,
            vec![
                tag_rule("div", &[("font-size", "10px")]),
                tag_rule("a", &[("background", "#0000ff")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 60.0, h: 600.0 });
        let hits: Vec<_> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { href, .. } => Some(href.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            hits,
            [Some("/go".to_string()), Some("/go".to_string())],
            "both line fragments of a wrapped link must stay clickable"
        );
    }

    #[test]
    fn deeply_nested_inline_elements_do_not_panic() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let mut cur = d;
        for _ in 0..300 {
            cur = elem(&mut dom, cur, "span");
        }
        text(&mut dom, cur, "deep");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert!(
            texts(&list).contains(&"deep"),
            "deeply nested transparent inline elements must still flow their text"
        );
    }

    #[test]
    fn nested_inline_boxes_each_paint_a_fragment() {
        let word = "abcdefghijklmnopqrst";
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let outer = elem(&mut dom, d, "span");
        let inner = elem(&mut dom, outer, "em");
        text(&mut dom, inner, word);
        let styles = styled(
            &dom,
            vec![
                tag_rule("div", &[("font-size", "10px")]),
                tag_rule("span", &[("background", "#0000ff"), ("padding", "0 2px")]),
                tag_rule("em", &[("background", "#00ff00")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 60.0, h: 600.0 });
        assert_eq!(colored_rects(&list, Color::rgb(0, 0, 255)).len(), 2);
        assert_eq!(colored_rects(&list, Color::rgb(0, 255, 0)).len(), 2);
    }
}

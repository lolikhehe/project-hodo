//! Inline formatting context: atoms, line boxes and their painted fragments.

use super::*;
use crate::block::layout_children;
use hodo_style::{LineHeight, TextAlign, VerticalAlign, WhiteSpace};

/// One rendered word, collapsed space, or inline-box inset spacer waiting to
/// be placed on a line. `spacer` atoms carry geometry but no glyphs.
///
/// Kept field-for-field stable: `block` constructs insets with a struct literal.
/// Per-atom formatting properties live in the parallel `Flow::meta` table.
#[derive(Clone)]
pub(crate) struct Atom {
    pub(crate) text: String,
    pub(crate) font_size: f32,
    pub(crate) color: Color,
    pub(crate) weight: u16,
    pub(crate) href: Option<String>,
    pub(crate) underline: bool,
    pub(crate) line_through: bool,
    /// Absolute x where this atom starts.
    pub(crate) x: f32,
    /// Advance width contributed to the line.
    pub(crate) w: f32,
    /// Inline boxes containing this atom, outermost first (indices into
    /// `Flow::boxes`). Empty for ordinary text.
    pub(crate) boxes: Vec<usize>,
    pub(crate) spacer: bool,
    /// Owning element of this atom's glyphs: the text node's parent element
    /// for text atoms, the painted element for inset spacers.
    pub(crate) owner: NodeId,
}

/// Formatting properties of one atom, indexed in lock-step with `Flow::line`.
#[derive(Clone, Copy)]
struct AtomMeta {
    /// Resolved line-height contribution (already clamped to a nonzero floor).
    line_h: f32,
    /// Per-character advance including letter-spacing.
    char_w: f32,
    font_size: f32,
    va: VerticalAlign,
    /// Baseline shift in CSS convention (positive = up); screen y uses `-dy`.
    dy: f32,
}

/// Paint state of one inline element inside a block's flow.
pub(crate) struct InlineBox {
    pub(crate) bg: Color,
    pub(crate) border_color: Color,
    pub(crate) border: Edges<f32>,
    pub(crate) margin: Edges<f32>,
    pub(crate) padding: Edges<f32>,
    /// Already painted a fragment on an earlier line: suppress left border.
    pub(crate) seen: bool,
    /// Element is still open: suppress right border on this fragment.
    pub(crate) active: bool,
    /// Owning element of this box's painted fragments.
    pub(crate) owner: NodeId,
}

/// Mutable inline-flow state for a single block's content area.
pub(crate) struct Flow {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) y: f32,
    pub(crate) line: Vec<Atom>,
    /// Parallel to `line`: formatting properties used at flush time.
    meta: Vec<AtomMeta>,
    pub(crate) line_w: f32,
    pub(crate) pending_space: Option<f32>,
    /// One bullet is emitted on this flow's first text line (list item).
    pub(crate) bullet: bool,
    /// Every paintable inline element encountered in this flow.
    pub(crate) boxes: Vec<InlineBox>,
    /// Indices of boxes currently open, outermost first.
    pub(crate) stack: Vec<usize>,
    // ---- M7.2 property state ---------------------------------------------
    strut_fs: f32,
    strut_line_h: f32,
    last_line_h: f32,
    text_align: TextAlign,
    /// `false` for `white-space: pre`/`nowrap`: no soft line breaking.
    wrap: bool,
    /// Outstanding first-line indent, subtracted from `left` after first flush.
    indent: f32,
    indent_applied: bool,
    // ---- M8 floats -------------------------------------------------------
    /// Margin boxes of floats in this flow's containing block, in absolute
    /// coordinates. Inherited by descendant block flows so their line boxes
    /// wrap around the same floats.
    pub(crate) float_l: Option<Rect>,
    pub(crate) float_r: Option<Rect>,
    /// Left/right float bottom edges (margin-box bottoms).
    pub(crate) float_bottoms: (f32, f32),
}

impl Flow {
    pub(crate) fn new(left: f32, right: f32, y: f32) -> Self {
        Self {
            left,
            right,
            y,
            line: Vec::new(),
            meta: Vec::new(),
            line_w: 0.0,
            pending_space: None,
            bullet: false,
            boxes: Vec::new(),
            stack: Vec::new(),
            strut_fs: 16.0,
            strut_line_h: 0.0,
            last_line_h: 0.0,
            text_align: TextAlign::Left,
            wrap: true,
            indent: 0.0,
            indent_applied: false,
            float_l: None,
            float_r: None,
            float_bottoms: (0.0, 0.0),
        }
    }
}

/// x range open to line boxes at this flow's current y. Float rects are
/// absolute, so nested flows only need to clamp them to their own content
/// edges; no coordinate translation is required.
pub(crate) fn float_bounds(flow: &Flow) -> (f32, f32) {
    if flow.float_l.is_none() && flow.float_r.is_none() {
        return (flow.left, flow.right);
    }
    let y = flow.y;
    let left = flow.float_l.filter(|r| y < r.y + r.h);
    let right = flow.float_r.filter(|r| y < r.y + r.h);
    (
        left.map_or(flow.left, |r| flow.left.max(r.x + r.w)),
        right.map_or(flow.right, |r| flow.right.min(r.x)),
    )
}

/// Defensive clamp: layout geometry must never see NaN or infinity.
// TODO(style): reject non-finite computed font-size/letter-spacing/word-spacing
// in hodo-style so these layout-side guards are only a last resort.
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// Resolve a line-height against the atom's own font size. A non-finite or
/// non-positive result has no room for glyphs, so clamp to the font height
/// (approximated by `font-size`).
fn resolve_line_height(lh: LineHeight, fs: f32) -> f32 {
    let h = match lh {
        LineHeight::Normal => 1.2 * fs,
        LineHeight::Number(n) => fs * n,
        LineHeight::Length(px) => px,
    };
    if h.is_finite() && h > 0.0 {
        h
    } else {
        fs.max(1.0)
    }
}

/// True when the current line has at least one glyph-bearing atom. Spacer-only
/// lines are not a wrap boundary and never produce a Text item.
pub(crate) fn has_content(flow: &Flow) -> bool {
    flow.line.iter().any(|a| !a.spacer)
}

/// Push an atom with default metrics. `block` uses this for inline-box inset
/// spacers; text atoms carry explicit metadata through `push_atom_meta`.
pub(crate) fn push_atom(flow: &mut Flow, atom: Atom) {
    let fs = finite_or(atom.font_size, 16.0);
    let meta = AtomMeta {
        line_h: 1.2 * fs,
        char_w: 0.6 * fs,
        font_size: fs,
        va: VerticalAlign::Baseline,
        dy: 0.0,
    };
    push_atom_meta(flow, atom, meta);
}

fn push_atom_meta(flow: &mut Flow, atom: Atom, meta: AtomMeta) {
    flow.meta.push(meta);
    flow.line.push(atom);
}

pub(crate) fn left_inset(b: &InlineBox) -> f32 {
    b.margin.left + b.border.left + b.padding.left
}

pub(crate) fn right_inset(b: &InlineBox) -> f32 {
    b.padding.right + b.border.right + b.margin.right
}

/// Paint the per-line fragments of every inline box that contributed to the
/// current line. A fragment's extent is the union of all atoms carrying that
/// box (including its inset spacers), so nested boxes compose correctly.
pub(crate) fn emit_fragments(flow: &mut Flow, list: &mut DisplayList, h: f32) {
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
        let (bg, border_color, border, seen, active, owner) = {
            let b = &flow.boxes[bi];
            (b.bg, b.border_color, b.border, b.seen, b.active, b.owner)
        };
        let first = !seen;
        let last = !active;
        let w = (x1 - x0).max(0.0);
        let y = flow.y;
        if bg.a > 0 {
            push_rect(list, owner, Rect { x: x0, y, w, h }, bg);
        }
        if border_color.a > 0 {
            let bt = border.top;
            let bb = border.bottom;
            if bt > 0.0 {
                push_rect(list, owner, Rect { x: x0, y, w, h: bt }, border_color);
            }
            if bb > 0.0 {
                push_rect(
                    list,
                    owner,
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
                    owner,
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
                    owner,
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

/// Baseline shift for one atom relative to the strut baseline (CSS sign:
/// positive raises). Top/Middle/Bottom need the resolved line height, hence
/// the two-pass computation in `flush_line`.
fn atom_dy(va: VerticalAlign, m: &AtomMeta, strut_fs: f32, line_h: f32) -> f32 {
    let fs = m.font_size;
    match va {
        VerticalAlign::Baseline => 0.0,
        VerticalAlign::Sub => -0.25 * fs,
        VerticalAlign::Super => 0.33 * fs,
        VerticalAlign::Length(px) => px,
        VerticalAlign::Top => 0.8 * (strut_fs - fs),
        VerticalAlign::Middle => 0.8 * strut_fs - 0.8 * fs + m.line_h / 2.0 - line_h / 2.0,
        VerticalAlign::Bottom => 0.8 * strut_fs - 0.8 * fs + m.line_h - line_h,
    }
}

/// Commit the current visual line as Text (fragments first) and advance y.
pub(crate) fn flush_line(flow: &mut Flow, list: &mut DisplayList) {
    if flow.line.is_empty() {
        flow.last_line_h = flow.strut_line_h;
        return;
    }

    // Pass 1: strut + tallest atom decide the line box height. `Number` was
    // resolved against each atom's own font size at feed time, so it inherits
    // as a number while `Length` stays absolute.
    let mut line_h = flow.strut_line_h.max(0.0);
    for m in &flow.meta {
        line_h = line_h.max(m.line_h);
    }
    if line_h <= 0.0 || !line_h.is_finite() {
        line_h = 1.2 * 16.0;
    }
    flow.last_line_h = line_h;

    // Pass 2: vertical-align shifts against the strut baseline (0.8 * fs).
    let strut_fs = flow.strut_fs;
    for m in flow.meta.iter_mut() {
        m.dy = atom_dy(m.va, m, strut_fs, line_h);
    }
    emit_fragments(flow, list, line_h);

    let mut text = String::new();
    let mut first: Option<usize> = None;
    for (i, atom) in flow.line.iter().enumerate() {
        if atom.spacer {
            continue;
        }
        if first.is_none() {
            first = Some(i);
        }
        text.push_str(&atom.text);
    }
    // shortcut: a line mixing linked and plain atoms takes its link from the first
    // atom, matching the first-atom-wins style rule; upgrade for per-run hit boxes.
    if let Some(fi) = first {
        let f = &flow.line[fi];
        let fm = flow.meta[fi];
        if flow.bullet {
            // shortcut: UA ::marker emulation, real markers when list-style engine lands
            text.insert_str(0, "• ");
            flow.bullet = false;
        }
        // text-align shifts the single Text rect of the line (fragments stay).
        // With floats the available band is narrower than the content box.
        let (content_left, content_right) = if flow.wrap {
            float_bounds(flow)
        } else {
            (flow.left, flow.right)
        };
        let avail_w = finite_or((content_right - content_left).max(0.0), 0.0);
        let text_w = finite_or((flow.line_w - (f.x - content_left)).max(0.0), 0.0);
        let shift = match flow.text_align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (avail_w - text_w) / 2.0,
            TextAlign::Right => avail_w - text_w,
        };
        list.push(
            f.owner,
            DisplayItem::Text {
                rect: Rect {
                    // Start at the first glyph, past any leading inline-box inset.
                    x: finite_or(f.x + shift, 0.0),
                    y: finite_or(flow.y - fm.dy, 0.0),
                    w: text_w,
                    h: line_h,
                },
                text,
                color: f.color,
                font_size: finite_or(f.font_size, 16.0),
                weight: f.weight,
                href: f.href.clone(),
                underline: f.underline,
                line_through: f.line_through,
            },
        );
    }
    flow.y += line_h;
    flow.line.clear();
    flow.meta.clear();
    flow.line_w = 0.0;
    flow.pending_space = None;
    // shortcut: first-line indent is applied to this flow's first formatted
    // line only; anonymous block runs made in `block` start a fresh Flow, so
    // an indented paragraph split by a block sibling re-indents each run.
    if flow.indent != 0.0 {
        flow.left -= flow.indent;
        flow.indent = 0.0;
    }
}

/// Byte index just after the `n`-th character of `s` (or `s.len()` if shorter).
pub(crate) fn char_boundary(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(i, _)| i).unwrap_or(s.len())
}

/// Place a word (or a preserved whitespace run) into the flow, wrapping at the
/// content width unless `white-space` disables it. Words longer than a whole
/// line are hard-broken. Each chunk scan is bounded by the chars actually
/// emitted on that line, so total work is linear in the text length.
fn place_word(flow: &mut Flow, list: &mut DisplayList, atom: Atom, meta: AtomMeta) {
    let cw = finite_or(meta.char_w, 0.0);
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
        // If the two floats leave no gap at this y, drop the empty line to the
        // shallower float bottom and retry. Bounded: each hop removes at least
        // one float from the active set, so at most two hops are possible.
        if flow.wrap && (flow.float_l.is_some() || flow.float_r.is_some()) {
            let mut hops = 0usize;
            loop {
                let (x0, x1) = float_bounds(flow);
                if x1 > x0 {
                    break;
                }
                if has_content(flow) {
                    flush_line(flow, list);
                }
                flow.pending_space = None;
                let y = flow.y;
                let lb = flow.float_l.filter(|r| y < r.y + r.h).map(|r| r.y + r.h);
                let rb = flow.float_r.filter(|r| y < r.y + r.h).map(|r| r.y + r.h);
                let next = match (lb, rb) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                };
                match next {
                    Some(bottom) => flow.y = bottom.max(y + 1.0),
                    None => break,
                }
                hops += 1;
                if hops > 4 {
                    break;
                }
            }
        }
        let (line_left, max_w) = if flow.wrap {
            let (l, r) = float_bounds(flow);
            (l, (r - l).max(0.0))
        } else {
            // `nowrap`/`pre` still emit hard newlines but never wrap for width.
            (flow.left, f32::INFINITY)
        };
        let word_w = finite_or(remaining as f32 * cw, 0.0);
        let space_w = finite_or(flow.pending_space.unwrap_or(0.0), 0.0);
        if has_content(flow) && flow.line_w + space_w + word_w > max_w {
            flush_line(flow, list);
            flow.pending_space = None;
            continue;
        }
        if word_w > max_w {
            // shortcut: unconditional hard-break is a superset of
            // overflow-wrap: break-word/anywhere; upgrade only if a future
            // rule needs to keep intact words on their own line.
            // n >= 1 guarantees progress even when max_w < one character.
            let n = ((max_w / cw).floor() as usize).clamp(1, remaining);
            let end = char_boundary(word, n);
            let (chunk, rest) = word.split_at(end);
            let chunk_w = n as f32 * cw;
            push_atom_meta(
                flow,
                Atom {
                    text: chunk.to_string(),
                    font_size: atom.font_size,
                    color: atom.color,
                    weight: atom.weight,
                    href: atom.href.clone(),
                    underline: atom.underline,
                    line_through: atom.line_through,
                    x: line_left + flow.line_w,
                    w: chunk_w,
                    boxes: atom.boxes.clone(),
                    spacer: false,
                    owner: atom.owner,
                },
                meta,
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
                push_atom_meta(
                    flow,
                    Atom {
                        text: " ".to_string(),
                        font_size: atom.font_size,
                        color: atom.color,
                        weight: atom.weight,
                        href: atom.href.clone(),
                        underline: atom.underline,
                        line_through: atom.line_through,
                        x: line_left + flow.line_w,
                        w: sw,
                        boxes: atom.boxes.clone(),
                        spacer: false,
                        owner: atom.owner,
                    },
                    meta,
                );
                flow.line_w += sw;
            }
        }
        flow.pending_space = None;
        push_atom_meta(
            flow,
            Atom {
                text: word.to_string(),
                font_size: atom.font_size,
                color: atom.color,
                weight: atom.weight,
                href: atom.href,
                underline: atom.underline,
                line_through: atom.line_through,
                x: line_left + flow.line_w,
                w: word_w,
                boxes: atom.boxes.clone(),
                spacer: false,
                owner: atom.owner,
            },
            meta,
        );
        flow.line_w += word_w;
        break;
    }
}

/// Collapsing `white-space` modes: runs of whitespace become one pending space.
fn emit_collapsed(
    flow: &mut Flow,
    list: &mut DisplayList,
    seg: &str,
    template: &Atom,
    meta: AtomMeta,
    space_w: f32,
) {
    if seg.chars().all(char::is_whitespace) {
        if has_content(flow) {
            flow.pending_space = Some(space_w);
        }
        return;
    }
    if has_content(flow) && seg.chars().next().is_some_and(char::is_whitespace) {
        flow.pending_space = Some(space_w);
    }
    let mut first = true;
    for word in seg.split_whitespace() {
        if !first && has_content(flow) {
            flow.pending_space = Some(space_w);
        }
        let mut atom = template.clone();
        atom.text = word.to_string();
        place_word(flow, list, atom, meta);
        first = false;
    }
}

/// Preserving `white-space` modes: emit whitespace runs verbatim as atoms.
fn emit_preserved(
    flow: &mut Flow,
    list: &mut DisplayList,
    seg: &str,
    template: &Atom,
    meta: AtomMeta,
) {
    if seg.is_empty() {
        return;
    }
    let mut start = 0usize;
    let mut space_run: Option<bool> = None;
    for (i, ch) in seg.char_indices() {
        let is_space = ch.is_whitespace();
        if space_run.is_some_and(|s| s != is_space) {
            let run = &seg[start..i];
            flow.pending_space = None;
            let mut atom = template.clone();
            atom.text = run.to_string();
            place_word(flow, list, atom, meta);
            start = i;
        }
        space_run = Some(is_space);
    }
    if space_run.is_some() {
        flow.pending_space = None;
        let mut atom = template.clone();
        atom.text = seg[start..].to_string();
        place_word(flow, list, atom, meta);
    }
}

/// Force a hard line box. Consecutive breaks still consume a strut line.
fn hard_break(flow: &mut Flow, list: &mut DisplayList) {
    if !flow.line.is_empty() {
        flush_line(flow, list);
    } else {
        flow.y += flow.strut_line_h.max(1.2 * flow.strut_fs);
    }
    flow.pending_space = None;
}

/// Feed a text node into the inline flow. Collapsing modes fold whitespace
/// runs; preserving modes keep them; all modes honour hard `\n` breaks when
/// `white-space` requests them.
pub(crate) fn feed_text(
    flow: &mut Flow,
    list: &mut DisplayList,
    text: &str,
    style: &ComputedStyle,
    href: Option<String>,
    boxes: &[usize],
    owner: NodeId,
) {
    if text.is_empty() {
        return;
    }
    // TODO(style): the root fix for non-finite computed values belongs in
    // hodo-style; layout clamps defensively so no rect ever carries NaN/inf.
    let fs = finite_or(style.font_size, 16.0);
    let line_h = resolve_line_height(style.line_height, fs);
    let meta = AtomMeta {
        line_h,
        char_w: finite_or(0.6 * fs + style.letter_spacing, 0.6 * fs),
        font_size: fs,
        va: style.vertical_align,
        dy: 0.0,
    };
    let space_w = finite_or(0.6 * fs + style.word_spacing, 0.6 * fs);
    let ws = style.white_space;
    let preserve = matches!(ws, WhiteSpace::Pre | WhiteSpace::PreWrap);
    let hard_nl = matches!(
        ws,
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine
    );

    // Line-level state comes from the text node's computed style (text
    // inherits these properties, so a span can change them for its line).
    flow.strut_fs = fs;
    flow.strut_line_h = line_h;
    flow.text_align = style.text_align;
    flow.wrap = matches!(
        ws,
        WhiteSpace::Normal | WhiteSpace::PreLine | WhiteSpace::PreWrap
    );
    if !flow.indent_applied && style.text_indent != 0.0 {
        flow.left += style.text_indent;
        flow.indent = style.text_indent;
        flow.indent_applied = true;
    }

    let template = Atom {
        text: String::new(),
        font_size: fs,
        color: style.color,
        weight: style.font_weight,
        href,
        underline: style.underline,
        line_through: style.line_through,
        x: 0.0,
        w: 0.0,
        boxes: boxes.to_vec(),
        spacer: false,
        owner,
    };

    let mut rest = text;
    loop {
        let seg = match rest.find('\n') {
            Some(i) => &rest[..i],
            None => rest,
        };
        if preserve {
            emit_preserved(flow, list, seg, &template, meta);
        } else {
            emit_collapsed(flow, list, seg, &template, meta, space_w);
        }
        if seg.len() == rest.len() {
            break;
        }
        if hard_nl {
            hard_break(flow, list);
        } else if has_content(flow) {
            // A newline is just whitespace in collapsing modes: fold it into
            // the pending space instead of concatenating the two segments.
            flow.pending_space = Some(space_w);
        }
        rest = &rest[seg.len() + 1..];
    }
}

fn shift_item_y(item: &mut DisplayItem, dy: f32) {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => rect.y += dy,
    }
}

/// Lay out an atomic inline-block: one slot in the parent line, never
/// fragmented. The caller has already flushed the current line, so the box
/// occupies exactly one line box; a narrowed `Avail` (explicit width clamped
/// by min/max) keeps it whole even when it overflows the containing line.
///
/// Returns the vertical space consumed by that one slot.
pub(crate) fn inline_block_atom(
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
    // shortcut: explicit width clamps the atomic slot; without one the box
    // takes the remaining line width (no shrink-to-fit pass yet).
    let mut box_w = style
        .width
        .unwrap_or((avail.width - m.left - m.right).max(0.0));
    box_w = box_w
        .max(style.min_size.w.max(0.0))
        .min(style.max_size.w.max(0.0))
        .max(0.0);
    let inner_left = box_x + bw + p.left;
    let inner_right = box_x + box_w - bw - p.right;
    let top = box_y + bw + p.top;

    let start = ctx.list.items.len();
    let mut bg_idx = None;
    if style.background.a > 0 {
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

    // Own block context: lay children out in the narrowed slot, no parent
    // line fragmentation.
    let mut sub = Flow::new(inner_left, inner_right, top);
    sub.strut_fs = style.font_size;
    sub.strut_line_h = resolve_line_height(style.line_height, style.font_size);
    sub.wrap = !matches!(style.white_space, WhiteSpace::Pre | WhiteSpace::NoWrap);
    layout_children(ctx, &mut sub, style, id, link);
    flush_line(&mut sub, ctx.list);

    let mut content_h = (sub.y - top).max(0.0);
    content_h = content_h
        .max(style.height.unwrap_or(0.0).max(0.0))
        .max(style.min_size.h.max(0.0));
    content_h = content_h.min(style.max_size.h.max(0.0));
    let box_h = content_h + p.top + p.bottom + 2.0 * bw;

    // Atomic baseline: the last content line's baseline, or the bottom margin
    // edge when there is no text. Align it to the containing line's strut
    // baseline (approximated by this element's inherited font metrics).
    let baseline = if sub.last_line_h > 0.0 {
        sub.y - sub.last_line_h + 0.8 * style.font_size
    } else {
        box_y + box_h
    };
    let dy = (avail.y + 0.8 * style.font_size) - baseline;
    if dy != 0.0 {
        for item in &mut ctx.list.items[start..] {
            shift_item_y(item, dy);
        }
    }

    if let Some(i) = bg_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }
    if let Some(i) = bottom_idx {
        patch_rect(ctx.list, i, |r| r.y = box_y + box_h - bw + dy);
    }
    if let Some(i) = left_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }
    if let Some(i) = right_idx {
        patch_rect(ctx.list, i, |r| r.h = box_h);
    }

    // Vertical margins sum, no collapsing (MVP).
    m.top + box_h + m.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_dom::{Dom, NodeKind};
    use hodo_style::compute_styles;
    use hodo_types::Size;

    fn decl(p: &str, v: &str) -> Declaration {
        Declaration {
            property: p.to_string(),
            value: v.to_string(),
        }
    }

    fn rule(tag: &str, decls: &[(&str, &str)]) -> StyleRule {
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

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.to_string(),
            attrs: Vec::new(),
        });
        dom.append(parent, id);
        id
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

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> hodo_style::StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
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
    fn line_height_number_inherits_as_number_px_stays_absolute() {
        // Parent line-height 1.5 with a larger child font: number scales, px
        // does not. Strut (10px * 1.5 = 15) is smaller than either child box.
        let build = |lh: &str| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div");
            let span = elem(&mut dom, d, "span");
            text(&mut dom, span, "X");
            let styles = styled(
                &dom,
                vec![
                    rule("div", &[("font-size", "10px"), ("line-height", lh)]),
                    rule("span", &[("font-size", "20px")]),
                ],
            );
            layout(&dom, &styles, Size { w: 800.0, h: 600.0 })
        };
        let num = build("1.5");
        let px = build("15px");
        let nr = text_rects(&num);
        let pr = text_rects(&px);
        assert_eq!(nr.len(), 1);
        assert!((nr[0].h - 30.0).abs() < 0.01, "number scales: {:?}", nr[0]);
        assert!((pr[0].h - 15.0).abs() < 0.01, "px is absolute: {:?}", pr[0]);
    }

    #[test]
    fn line_height_zero_clamps_to_font_height() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "X");
        let styles = styled(
            &dom,
            vec![rule("div", &[("font-size", "20px"), ("line-height", "0")])],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let r = text_rects(&list);
        assert!(
            (r[0].h - 20.0).abs() < 0.01,
            "line-height:0 must clamp to the font height: {:?}",
            r[0]
        );
    }

    #[test]
    fn text_indent_only_first_formatted_line() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        // 6px chars; 30px column -> 5 chars/line. 10 chars -> two lines.
        text(&mut dom, d, "abcdefghij");
        let styles = styled(
            &dom,
            vec![rule(
                "div",
                &[("font-size", "10px"), ("text-indent", "12px")],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 30.0, h: 600.0 });
        let r = text_rects(&list);
        assert!(r.len() >= 2, "expected wrapped lines: {r:?}");
        assert!(
            (r[0].x - 12.0).abs() < 0.01,
            "first line indented: {:?}",
            r[0]
        );
        assert!(
            r[1..].iter().all(|rect| rect.x.abs() < 0.01),
            "later lines back at the left edge: {r:?}"
        );
    }

    #[test]
    fn letter_and_word_spacing_widen_advances() {
        let build = |spacing: &[(&str, &str)]| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div");
            text(&mut dom, d, "aa bb");
            let mut decls = vec![("font-size", "10px")];
            decls.extend_from_slice(spacing);
            let styles = styled(&dom, vec![rule("div", &decls)]);
            layout(&dom, &styles, Size { w: 800.0, h: 600.0 })
        };
        let plain = build(&[]);
        let spaced = build(&[("letter-spacing", "3px"), ("word-spacing", "5px")]);
        let pw = text_rects(&plain)[0].w;
        let sw = text_rects(&spaced)[0].w;
        // 4 word chars * 3px letter + 1 pending space * 5px word = 17px.
        assert!(
            (sw - pw - 17.0).abs() < 0.01,
            "letter+word spacing must widen the line: {pw} -> {sw}"
        );
    }

    #[test]
    fn vertical_align_shifts_first_atom_baseline() {
        let build = |va: &str| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div");
            let span = elem(&mut dom, d, "span");
            text(&mut dom, span, "X");
            let styles = styled(
                &dom,
                vec![
                    rule("div", &[("font-size", "10px")]),
                    rule("span", &[("vertical-align", va)]),
                ],
            );
            layout(&dom, &styles, Size { w: 800.0, h: 600.0 })
        };
        let base = text_rects(&build("baseline"))[0].y;
        let sub = text_rects(&build("sub"))[0].y;
        let sup = text_rects(&build("super"))[0].y;
        let up = text_rects(&build("8px"))[0].y;
        assert!((sub - base - 2.5).abs() < 0.01, "sub lowers by 0.25em");
        assert!((sup - base + 3.3).abs() < 0.01, "super raises by 0.33em");
        assert!((up - base + 8.0).abs() < 0.01, "8px raises by 8px");
    }

    #[test]
    fn inline_block_is_single_fragment_with_baseline() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let ib = elem_with_attrs(
            &mut dom,
            d,
            "span",
            &[("style", "display:inline-block;width:80px;height:30px")],
        );
        text(&mut dom, ib, "inside");
        let styles = styled(
            &dom,
            vec![
                rule("div", &[("font-size", "10px")]),
                rule("span", &[("background", "#0000ff")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        let bg = colored_rects(&list, Color::rgb(0, 0, 255));
        assert_eq!(bg.len(), 1, "inline-block must be one atomic fragment");
        assert!((bg[0].w - 80.0).abs() < 0.01, "slot width from style");
        assert!((bg[0].h - 30.0).abs() < 0.01, "height from consumed run");
        let tr = text_rects(&list);
        assert_eq!(tr.len(), 1, "inner text paints exactly one line");
    }

    #[test]
    fn pre_renders_verbatim_with_hard_newlines() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "a b\nc  d");
        let styles = styled(
            &dom,
            vec![rule(
                "div",
                &[("font-size", "10px"), ("white-space", "pre")],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        assert_eq!(texts(&list), ["a b", "c  d"], "pre keeps spaces + newlines");
    }

    #[test]
    fn pre_wrap_wraps_preserved_text() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "abc def\nghi");
        let styles = styled(
            &dom,
            vec![rule(
                "div",
                &[("font-size", "10px"), ("white-space", "pre-wrap")],
            )],
        );
        // 6px chars in a 30px box: "abc " fills the first line, the preserved
        // space survives, "def" wraps, and the hard newline starts a third.
        let list = layout(&dom, &styles, Size { w: 30.0, h: 600.0 });
        assert_eq!(texts(&list), ["abc ", "def", "ghi"]);
    }

    #[test]
    fn nowrap_never_breaks_and_preline_keeps_breaks() {
        let build = |ws: &str| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div");
            text(&mut dom, d, "abcdefghij");
            let styles = styled(
                &dom,
                vec![rule("div", &[("font-size", "10px"), ("white-space", ws)])],
            );
            layout(&dom, &styles, Size { w: 54.0, h: 600.0 })
        };
        assert_eq!(
            texts(&build("nowrap")),
            ["abcdefghij"],
            "nowrap ignores width"
        );
        assert_eq!(
            texts(&build("pre-line")),
            ["abcdefghi", "j"],
            "pre-line wraps at width"
        );
    }

    #[test]
    fn text_align_shifts_text_rect_only() {
        let build = |ta: &str| {
            let mut dom = Dom::new();
            let root = dom.root();
            let d = elem(&mut dom, root, "div");
            text(&mut dom, d, "abcde");
            let styles = styled(
                &dom,
                vec![rule("div", &[("font-size", "10px"), ("text-align", ta)])],
            );
            layout(&dom, &styles, Size { w: 100.0, h: 600.0 })
        };
        // 5 chars * 6px = 30px in a 100px box.
        let left = text_rects(&build("left"))[0].x;
        let center = text_rects(&build("center"))[0].x;
        let right = text_rects(&build("right"))[0].x;
        assert!((left - 0.0).abs() < 0.01);
        assert!((center - 35.0).abs() < 0.01, "center => (100-30)/2");
        assert!((right - 70.0).abs() < 0.01, "right => 100-30");
    }

    #[test]
    fn align_on_empty_line_does_not_panic() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        elem(&mut dom, d, "br");
        text(&mut dom, d, "after");
        let styles = styled(
            &dom,
            vec![rule(
                "div",
                &[("font-size", "10px"), ("text-align", "center")],
            )],
        );
        let list = layout(&dom, &styles, Size { w: 100.0, h: 600.0 });
        let r = text_rects(&list);
        assert_eq!(r.len(), 1, "empty line emits no Text");
        assert!(
            r[0].y >= 12.0,
            "empty line still consumed a strut: {:?}",
            r[0]
        );
    }

    #[test]
    fn deeply_nested_spans_still_flow() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let mut cur = d;
        for _ in 0..100 {
            cur = elem(&mut dom, cur, "span");
        }
        text(&mut dom, cur, "deep");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert!(texts(&list).contains(&"deep"));
    }

    #[test]
    fn float_bounds_narrow_both_edges_and_release_at_bottom() {
        let mut flow = Flow::new(10.0, 110.0, 0.0);
        assert_eq!(float_bounds(&flow), (10.0, 110.0));
        flow.float_l = Some(Rect {
            x: 10.0,
            y: 0.0,
            w: 40.0,
            h: 24.0,
        });
        assert_eq!(float_bounds(&flow), (50.0, 110.0), "left float narrows x");
        flow.float_r = Some(Rect {
            x: 90.0,
            y: 0.0,
            w: 20.0,
            h: 36.0,
        });
        assert_eq!(
            float_bounds(&flow),
            (50.0, 90.0),
            "both floats leave the middle gutter"
        );
        flow.y = 24.0;
        assert_eq!(
            float_bounds(&flow),
            (10.0, 90.0),
            "left bottom passed mid-block"
        );
        flow.y = 36.0;
        assert_eq!(float_bounds(&flow), (10.0, 110.0), "both floats ended");
    }

    #[test]
    fn float_bounds_closed_gutter_signals_hop() {
        let mut flow = Flow::new(0.0, 100.0, 0.0);
        flow.float_l = Some(Rect {
            x: 0.0,
            y: 0.0,
            w: 80.0,
            h: 20.0,
        });
        flow.float_r = Some(Rect {
            x: 20.0,
            y: 0.0,
            w: 80.0,
            h: 20.0,
        });
        let (l, r) = float_bounds(&flow);
        assert!(
            r <= l,
            "overlapping floats leave no float-free width: {l}..{r}"
        );
    }

    #[test]
    fn infinite_font_size_yields_finite_text_rect() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem_with_attrs(&mut dom, root, "div", &[("style", "font-size:1e308%")]);
        text(&mut dom, d, "x");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let r = text_rects(&list);
        assert_eq!(r.len(), 1, "text still paints: {r:?}");
        assert!(
            r[0].x.is_finite() && r[0].y.is_finite() && r[0].w.is_finite() && r[0].h.is_finite(),
            "text rect must be finite despite an infinite computed font size: {:?}",
            r[0]
        );

        // hodo-style already sanitises the CSS spelling today, so drive the
        // layout seam directly: a non-finite computed value must not reach
        // geometry even if an upstream fix regresses.
        let mut flow = Flow::new(0.0, 800.0, 0.0);
        let mut direct = crate::DisplayList::default();
        let mut st = crate::fallback_style();
        st.font_size = f32::INFINITY;
        feed_text(&mut flow, &mut direct, "x", &st, None, &[], d);
        flush_line(&mut flow, &mut direct);
        let dr = direct.items.iter().find_map(|it| match it {
            crate::DisplayItem::Text { rect, .. } => Some(*rect),
            _ => None,
        });
        assert!(
            dr.is_some_and(|q| q.x.is_finite()
                && q.y.is_finite()
                && q.w.is_finite()
                && q.h.is_finite()),
            "layout must clamp a non-finite computed font size: {dr:?}"
        );
    }

    #[test]
    fn hundred_k_char_word_budget_is_linear() {
        let word = "x".repeat(100_000);
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, &word);
        let styles = styled(&dom, vec![rule("div", &[("font-size", "10px")])]);
        let started = std::time::Instant::now();
        let list = layout(&dom, &styles, Size { w: 60.0, h: 600.0 });
        let elapsed = started.elapsed();
        assert_eq!(texts(&list).len(), 10_000, "10 chars per 60px line");
        assert!(
            elapsed.as_secs_f64() < 5.0,
            "100k hard-break must stay linear: {elapsed:?}"
        );
    }
}

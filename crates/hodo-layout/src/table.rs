//! CSS table layout with the fixed column algorithm and separate borders.
//!
//! The table's direct children are walked into an occupancy grid: `<tr>` rows
//! (anonymous rows wrap stray cells/text), `<td>/<th>` cells and an optional
//! first `<caption>`. Cells are measured once into a scratch display list, then
//! emitted at their final track positions once all row heights are known.
//!
//! Shortcuts (MVP):
//! - `table-layout: auto` uses the fixed algorithm; author column widths win
//!   and the remainder is split equally.
//! - `border-collapse: collapse` is parsed by hodo-style but laid out as
//!   `separate` here (no shared borders); upgrade when collapsed borders are
//!   specified.
//! - Vertical align shifts the whole cell box; `Baseline` and inline-only
//!   values are treated as `Top`.

use super::*;
use crate::block::{layout_block, layout_child};
use crate::{flush_line, Flow};

/// Hard cap on grid cells; beyond this, extra cells degrade to plain blocks.
const MAX_CELLS: usize = 100_000;
const MAX_COLSPAN: usize = 1_000;
const MAX_ROWSPAN: usize = 65_534;
/// Slot budget for the occupancy grid. 1M slots ~= 8 MB of `Option<NodeId>`
/// indices plus row-vector overhead, which keeps adversarial spans (1000 x
/// 65534 = 65M slots/cell) from allocating gigabytes. A cell whose span would
/// exceed this budget gets its span clamped, or degrades to a plain block.
const MAX_GRID_SLOTS: usize = 1_000_000;
/// Extra rows a rowspan may reserve beyond the rows already placed. Without
/// this, a single rowspan=65534 on a small table would allocate 65k rows; 64
/// keeps ordinary row groups intact while bounding worst-case row growth.
const ROW_LOOKAHEAD: usize = 64;

/// One occupied track: either a real cell element or an anonymous cell made
/// from bare inline nodes (sloppy HTML).
struct Cell {
    id: Option<NodeId>,
    inline_nodes: Vec<NodeId>,
    row: usize,
    col: usize,
    colspan: usize,
    rowspan: usize,
    width: Option<f32>,
    valign: VerticalAlign,
}

fn is_tag(ctx: &Ctx<'_>, id: NodeId, tag: &str) -> bool {
    ctx.dom.tag(id).is_some_and(|t| t.eq_ignore_ascii_case(tag))
}

fn display(ctx: &Ctx<'_>, id: NodeId) -> Option<Display> {
    ctx.styles.get(id).map(|s| s.display)
}

fn is_cell(ctx: &Ctx<'_>, id: NodeId) -> bool {
    is_tag(ctx, id, "td") || is_tag(ctx, id, "th") || display(ctx, id) == Some(Display::TableCell)
}

fn is_row(ctx: &Ctx<'_>, id: NodeId) -> bool {
    is_tag(ctx, id, "tr") || display(ctx, id) == Some(Display::TableRow)
}

fn is_row_group(ctx: &Ctx<'_>, id: NodeId) -> bool {
    is_tag(ctx, id, "thead")
        || is_tag(ctx, id, "tbody")
        || is_tag(ctx, id, "tfoot")
        || display(ctx, id) == Some(Display::TableRowGroup)
}

fn visible_children(ctx: &Ctx<'_>, id: NodeId) -> Vec<NodeId> {
    ctx.dom
        .render_children(id)
        .iter()
        .copied()
        .filter(|&c| {
            !ctx.styles
                .get(c)
                .is_some_and(|s| s.display == Display::None)
        })
        .collect()
}

fn parse_positive(s: &str) -> Option<f32> {
    let s = s.trim().strip_suffix("px").unwrap_or(s.trim());
    s.parse::<f32>().ok().filter(|v| v.is_finite() && *v > 0.0)
}

/// HTML `width` attributes accept pixels or percentages (HN's `85%` table).
fn parse_width(s: &str, basis: f32) -> Option<f32> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        return pct
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
            .map(|v| v * basis / 100.0);
    }
    parse_positive(s)
}

/// Right edge of an item that contributes intrinsic content width. Background
/// rects are excluded: they span their whole (possibly wrong) provisional cell.
fn content_right(item: &DisplayItem) -> Option<f32> {
    match item {
        DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => Some(rect.x + rect.w),
        DisplayItem::Rect { .. } => None,
    }
}

fn parse_span(s: Option<&str>, max: usize) -> usize {
    s.and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, max)
}

/// Accumulated occupancy state while walking the table's children.
struct TableBuilder<'a> {
    cells: &'a mut Vec<Cell>,
    deferred: &'a mut Vec<NodeId>,
    fallback_valign: VerticalAlign,
    /// Basis for percentage `width` attributes on cells.
    avail_width: f32,
    slots: usize,
}

/// A cell that cannot fit the grid budget still renders, as a plain block.
fn degrade(deferred: &mut Vec<NodeId>, inline_nodes: Vec<NodeId>, id: Option<NodeId>) {
    deferred.extend(inline_nodes);
    deferred.extend(id);
}

/// New slots needed to widen the covered rows to `end`.
fn added_slots(grid: &[Vec<Option<usize>>], row: usize, rows: usize, end: usize) -> usize {
    grid.iter()
        .skip(row)
        .take(rows)
        .map(|slots| end.saturating_sub(slots.len()))
        .sum()
}

/// Place a cell into the first free slot that fits its span, growing the grid
/// as needed. Spans overflowing the grid edge are clamped to that edge.
fn push_cell(
    ctx: &Ctx<'_>,
    b: &mut TableBuilder<'_>,
    grid: &mut Vec<Vec<Option<usize>>>,
    row: usize,
    id: Option<NodeId>,
    inline_nodes: Vec<NodeId>,
) {
    let TableBuilder {
        cells,
        deferred,
        fallback_valign,
        avail_width,
        slots,
    } = &mut *b;
    if cells.len() >= MAX_CELLS {
        // shortcut: degrade extra cells to plain blocks; upgrade when tables
        // with >100k cells need real track positioning.
        degrade(deferred, inline_nodes, id);
        return;
    }
    let colspan = id
        .and_then(|n| ctx.dom.attr(n, "colspan"))
        .map(|v| parse_span(Some(v), MAX_COLSPAN))
        .unwrap_or(1);
    let rowspan = id
        .and_then(|n| ctx.dom.attr(n, "rowspan"))
        .map(|v| parse_span(Some(v), MAX_ROWSPAN))
        .unwrap_or(1);
    let width = id
        .and_then(|n| ctx.dom.attr(n, "width"))
        .and_then(|v| parse_width(v, *avail_width))
        .or_else(|| {
            id.and_then(|n| ctx.styles.get(n))
                .and_then(|s| s.width)
                .filter(|w| w.is_finite() && *w > 0.0)
        });
    let valign = id
        .and_then(|n| ctx.styles.get(n))
        .map(|s| s.vertical_align)
        .unwrap_or(*fallback_valign);

    // Clamp the EFFECTIVE span (never the caller's attribute): a rowspan only
    // reserves a lookahead worth of rows, and a colspan is limited to the
    // columns the slot budget can still afford at the resulting row count.
    let placed_rows = grid.len().saturating_sub(row);
    let rowspan = rowspan.min(placed_rows + ROW_LOOKAHEAD).max(1);
    let target_rows = grid.len().max(row + rowspan);
    let budget_cols = MAX_GRID_SLOTS / target_rows.max(1);
    let mut col = grid[row]
        .iter()
        .position(Option::is_none)
        .unwrap_or(grid[row].len());
    if col >= budget_cols {
        degrade(deferred, inline_nodes, id);
        return;
    }
    let mut colspan = colspan.min(budget_cols - col).max(1);

    while grid.len() < target_rows {
        grid.push(Vec::new());
    }
    let widest = grid[row..target_rows]
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or(0);
    while col < widest
        && (row..target_rows).any(|r| {
            (col..col + colspan).any(|c| grid[r].get(c).is_some_and(|slot| slot.is_some()))
        })
    {
        col += 1;
        if col >= budget_cols {
            degrade(deferred, inline_nodes, id);
            return;
        }
    }
    // Rows placed earlier may already be wider than this cell's column budget,
    // so cap the width by the *remaining slot count*, not just by column index.
    let remaining = MAX_GRID_SLOTS.saturating_sub(*slots);
    let mut lo = col + 1;
    let mut hi = col + colspan;
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if added_slots(grid, row, rowspan, mid) <= remaining {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let end = lo;
    let added = added_slots(grid, row, rowspan, end);
    if added > remaining {
        degrade(deferred, inline_nodes, id);
        return;
    }
    colspan = end - col;
    for row_slots in grid.iter_mut().skip(row).take(rowspan) {
        row_slots.resize(end, None);
        for slot in &mut row_slots[col..end] {
            *slot = Some(cells.len());
        }
    }
    debug_assert!(
        *slots + added <= MAX_GRID_SLOTS,
        "occupancy grid exceeded its slot budget"
    );
    *slots += added;
    cells.push(Cell {
        id,
        inline_nodes,
        row,
        col,
        colspan,
        rowspan,
        width,
        valign,
    });
}

/// Lay out one cell. Element cells reuse the existing block path so nested
/// inline content (and nested tables) work unchanged; anonymous cells stream
/// their bare nodes through the same per-child path.
fn layout_cell(
    ctx: &mut Ctx<'_>,
    cell: &Cell,
    parent: &ComputedStyle,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    if let Some(id) = cell.id {
        let s = ctx.styles.get(id).cloned().unwrap_or_else(fallback_style);
        dispatch_child(ctx, &s, id, avail, link)
    } else {
        let mut st = fallback_style();
        st.font_size = parent.font_size;
        st.color = parent.color;
        let mut flow = Flow::new(avail.x, avail.x + avail.width, avail.y);
        for &n in &cell.inline_nodes {
            layout_child(ctx, &mut flow, &st, n, link.clone());
        }
        flush_line(&mut flow, ctx.list);
        (flow.y - avail.y).max(0.0)
    }
}

fn span_extent(positions: &[f32], sizes: &[f32], start: usize, span: usize) -> f32 {
    let end = (start + span).min(sizes.len());
    if end <= start {
        return 0.0;
    }
    positions[end - 1] + sizes[end - 1] - positions[start]
}

/// Track positions and widths passed to the measuring passes.
struct Tracks<'a> {
    x: &'a [f32],
    w: &'a [f32],
}

/// Lay out caption and cells into a scratch list. Returns caption height,
/// per-cell heights and the intrinsic (glyph) width each cell painted.
fn measure_pass(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    caption: Option<NodeId>,
    caption_avail: &Avail,
    cells: &[Cell],
    tracks: Tracks<'_>,
    link: Option<String>,
) -> (f32, Vec<f32>, Vec<f32>) {
    let mut scratch = DisplayList::default();
    std::mem::swap(&mut *ctx.list, &mut scratch);
    let caption_h = caption
        .map(|c| {
            let s = ctx.styles.get(c).cloned().unwrap_or_else(fallback_style);
            layout_block(
                ctx,
                &s,
                c,
                Avail {
                    x: caption_avail.x,
                    y: caption_avail.y,
                    width: caption_avail.width,
                },
                link.clone(),
            )
        })
        .unwrap_or(0.0);
    let mut heights = Vec::with_capacity(cells.len());
    let mut natural = vec![0.0_f32; cells.len()];
    for (i, cell) in cells.iter().enumerate() {
        let w = span_extent(tracks.x, tracks.w, cell.col, cell.colspan);
        let av = Avail {
            x: tracks.x[cell.col],
            y: 0.0,
            width: w,
        };
        let start = ctx.list.items.len();
        let cell_x = av.x;
        let h = layout_cell(ctx, cell, style, av, link.clone());
        natural[i] = ctx.list.items[start..]
            .iter()
            .filter_map(content_right)
            .map(|right| (right - cell_x).max(0.0))
            .fold(0.0_f32, f32::max);
        heights.push(h);
    }
    std::mem::swap(&mut *ctx.list, &mut scratch);
    (caption_h, heights, natural)
}

pub(crate) fn layout_table(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    // `style.table_layout`/`style.border_collapse` are intentionally read by
    // hodo-style only: both degrade to fixed/separate here (see module docs).
    let mut caption = None;
    let mut cells: Vec<Cell> = Vec::new();
    let mut grid: Vec<Vec<Option<usize>>> = Vec::new();
    let mut deferred: Vec<NodeId> = Vec::new();
    let mut builder = TableBuilder {
        cells: &mut cells,
        deferred: &mut deferred,
        fallback_valign: style.vertical_align,
        avail_width: avail.width,
        slots: 0,
    };

    for child in visible_children(ctx, id) {
        if is_tag(ctx, child, "caption") || display(ctx, child) == Some(Display::TableCaption) {
            if caption.is_none() {
                caption = Some(child);
            }
            continue;
        }
        let row_items = if is_row_group(ctx, child) {
            visible_children(ctx, child)
        } else {
            vec![child]
        };
        for item in row_items {
            let row = grid.len();
            grid.push(Vec::new());
            let kids = if is_row(ctx, item) {
                visible_children(ctx, item)
            } else {
                vec![item]
            };
            let mut pending: Vec<NodeId> = Vec::new();
            for kid in kids {
                if is_cell(ctx, kid) {
                    if !pending.is_empty() {
                        push_cell(
                            ctx,
                            &mut builder,
                            &mut grid,
                            row,
                            None,
                            std::mem::take(&mut pending),
                        );
                    }
                    push_cell(ctx, &mut builder, &mut grid, row, Some(kid), Vec::new());
                } else if is_tag(ctx, kid, "br") {
                    // A stray break inside a row does not make a cell.
                } else {
                    pending.push(kid);
                }
            }
            if !pending.is_empty() {
                push_cell(ctx, &mut builder, &mut grid, row, None, pending);
            }
        }
    }

    let rows = grid.len();
    let cols = grid.iter().map(Vec::len).max().unwrap_or(0);
    let m = &style.margin;
    let p = &style.padding;
    let bw = style.border_width.max(0.0);
    let nominal_w = ctx
        .dom
        .attr(id, "width")
        .and_then(|v| parse_width(v, avail.width))
        .or_else(|| style.width.filter(|w| w.is_finite() && *w > 0.0))
        .unwrap_or(avail.width);
    let box_w = nominal_w
        .min((avail.width - m.left - m.right).max(0.0))
        .max(0.0);
    let box_x = avail.x + m.left;
    let box_y = avail.y + m.top;
    let content_x = box_x + bw + p.left;
    let content_y = box_y + bw + p.top;
    let content_w = (box_w - bw - bw - p.left - p.right).max(0.0);
    let (h_space, v_space) = style.border_spacing;
    let h_space = h_space.max(0.0);
    let v_space = v_space.max(0.0);

    // Fixed columns: explicit widths first, remainder split equally, then
    // scaled down if the author asked for more width than the table has.
    let mut col_w = vec![0.0_f32; cols];
    let mut fixed = vec![false; cols];
    for cell in &cells {
        if let Some(w) = cell.width {
            if cell.col < cols && w > col_w[cell.col] {
                col_w[cell.col] = w;
                fixed[cell.col] = true;
            }
        }
    }
    let spacing_w = h_space * cols.saturating_sub(1) as f32;
    let fixed_sum: f32 = col_w.iter().sum();
    let flexible = cols - fixed.iter().filter(|&&f| f).count();
    let equal = if flexible > 0 {
        (content_w - spacing_w - fixed_sum).max(0.0) / flexible as f32
    } else {
        0.0
    };
    for w in &mut col_w {
        if *w <= 0.0 {
            *w = equal;
        }
    }
    let total = col_w.iter().sum::<f32>() + spacing_w;
    if total > content_w && total > 0.0 {
        let scale = (content_w - spacing_w).max(0.0) / col_w.iter().sum::<f32>().max(f32::EPSILON);
        for w in &mut col_w {
            *w *= scale;
        }
    }

    let mut col_x = vec![0.0_f32; cols];
    let mut cursor = content_x;
    for (c, w) in col_w.iter().enumerate() {
        col_x[c] = cursor;
        cursor += *w + h_space;
    }

    // First scratch pass measures intrinsic content widths. Author widths win;
    // unfixed columns start from what they actually painted, then share the
    // leftover proportionally. Without this, three equal tracks leave HN's
    // rank at x=0 and its story in a far-right column with a 700px void.
    let caption_avail = Avail {
        x: content_x,
        y: 0.0,
        width: content_w,
    };
    let (caption_h, _, natural) = measure_pass(
        ctx,
        style,
        caption,
        &caption_avail,
        &cells,
        Tracks {
            x: &col_x,
            w: &col_w,
        },
        link.clone(),
    );
    let mut pref = vec![0.0_f32; cols];
    for (i, cell) in cells.iter().enumerate() {
        if cell.colspan == 1 && cell.col < cols {
            pref[cell.col] = pref[cell.col].max(natural[i]);
        }
    }
    let usable = (content_w - spacing_w).max(0.0);
    for c in 0..cols {
        if !fixed[c] {
            col_w[c] = pref[c].min(usable);
        }
    }
    let flex_sum: f32 = (0..cols).filter(|&c| !fixed[c]).map(|c| col_w[c]).sum();
    let fixed_sum: f32 = (0..cols).filter(|&c| fixed[c]).map(|c| col_w[c]).sum();
    let flexible = cols - fixed.iter().filter(|&&f| f).count();
    if flexible > 0 {
        if flex_sum > 0.0 {
            let scale = (usable - fixed_sum).max(0.0) / flex_sum;
            for c in 0..cols {
                if !fixed[c] {
                    col_w[c] *= scale;
                }
            }
        } else {
            let equal = (usable - fixed_sum).max(0.0) / flexible as f32;
            for c in 0..cols {
                if !fixed[c] {
                    col_w[c] = equal;
                }
            }
        }
    }
    let total = col_w.iter().sum::<f32>() + spacing_w;
    if total > content_w && total > 0.0 {
        let scale = (content_w - spacing_w).max(0.0) / col_w.iter().sum::<f32>().max(f32::EPSILON);
        for w in &mut col_w {
            *w *= scale;
        }
    }
    let mut col_x = vec![0.0_f32; cols];
    let mut cursor = content_x;
    for (c, w) in col_w.iter().enumerate() {
        col_x[c] = cursor;
        cursor += *w + h_space;
    }

    // Second scratch pass: heights at the final track widths (wrapping changes
    // them), so rows cannot be too short for the text emitted in the paint pass.
    let (_, heights, _) = measure_pass(
        ctx,
        style,
        caption,
        &caption_avail,
        &cells,
        Tracks {
            x: &col_x,
            w: &col_w,
        },
        link.clone(),
    );

    // A spanning cell contributes its height to every row it covers (evenly),
    // so rows underneath a rowspan still get a sensible height.
    let mut row_h = vec![0.0_f32; rows];
    for (i, cell) in cells.iter().enumerate() {
        let share = heights[i] / cell.rowspan as f32;
        let covered = (cell.row + cell.rowspan).min(rows) - cell.row;
        for h in row_h.iter_mut().skip(cell.row).take(covered) {
            *h = (*h).max(share);
        }
    }
    let grid_y = content_y + caption_h + if caption.is_some() { v_space } else { 0.0 };
    let mut row_y = vec![0.0_f32; rows];
    let mut row_cursor = grid_y;
    for (r, h) in row_h.iter().enumerate() {
        row_y[r] = row_cursor;
        row_cursor += *h + v_space;
    }
    let grid_bottom = if rows > 0 {
        row_y[rows - 1] + row_h[rows - 1]
    } else {
        grid_y
    };
    let box_h = (grid_bottom - box_y + p.bottom + bw).max(0.0);

    // Paint order: table box first, then caption and cells on top.
    if style.background.a > 0 {
        push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: box_w,
                h: box_h,
            },
            style.background,
        );
    }
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
        );
        push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y + box_h - bw,
                w: box_w,
                h: bw,
            },
            style.border_color,
        );
        push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x,
                y: box_y,
                w: bw,
                h: box_h,
            },
            style.border_color,
        );
        push_rect(
            ctx.list,
            id,
            Rect {
                x: box_x + box_w - bw,
                y: box_y,
                w: bw,
                h: box_h,
            },
            style.border_color,
        );
    }

    if let Some(c) = caption {
        let s = ctx.styles.get(c).cloned().unwrap_or_else(fallback_style);
        layout_block(
            ctx,
            &s,
            c,
            Avail {
                x: content_x,
                y: content_y,
                width: content_w,
            },
            link.clone(),
        );
    }
    for (i, cell) in cells.iter().enumerate() {
        let w = span_extent(&col_x, &col_w, cell.col, cell.colspan);
        let h = span_extent(&row_y, &row_h, cell.row, cell.rowspan);
        let shift = match cell.valign {
            VerticalAlign::Middle => (h - heights[i]) * 0.5,
            VerticalAlign::Bottom => h - heights[i],
            _ => 0.0,
        };
        // shortcut: a spanning cell keeps its natural box height instead of
        // stretching to fill every covered row.
        layout_cell(
            ctx,
            cell,
            style,
            Avail {
                x: col_x[cell.col],
                y: row_y[cell.row] + shift.max(0.0),
                width: w,
            },
            link.clone(),
        );
    }

    // Over-cap cells degrade to ordinary blocks below the grid.
    let mut tail = grid_bottom;
    for n in deferred {
        let is_element = ctx.dom.tag(n).is_some();
        let loose = Cell {
            id: is_element.then_some(n),
            inline_nodes: if is_element { Vec::new() } else { vec![n] },
            row: 0,
            col: 0,
            colspan: 1,
            rowspan: 1,
            width: None,
            valign: style.vertical_align,
        };
        tail += layout_cell(
            ctx,
            &loose,
            style,
            Avail {
                x: content_x,
                y: tail,
                width: content_w,
            },
            link.clone(),
        );
    }
    m.top + box_h + (tail - grid_bottom) + m.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_dom::NodeKind;
    use hodo_style::compute_styles;

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
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

    fn rule(tag: &str, props: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector: Selector {
                tag: Some(tag.into()),
                id: None,
                classes: Vec::new(),
                universal: false,
            },
            declarations: props
                .iter()
                .map(|&(p, v)| Declaration {
                    property: p.into(),
                    value: v.into(),
                })
                .collect(),
        }
    }

    fn styled(dom: &Dom, rules: Vec<StyleRule>) -> StyleMap {
        compute_styles(dom, &[Stylesheet { rules }])
    }

    fn layout_table_dom(dom: &Dom, styles: &StyleMap, w: f32) -> DisplayList {
        layout(dom, styles, Size { w, h: 1000.0 })
    }

    fn rects(list: &DisplayList, color: Color) -> Vec<Rect> {
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

    fn text_rects(list: &DisplayList) -> Vec<Rect> {
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect()
    }

    fn grid_dom(cols: usize, rows: usize) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "202")]);
        for _ in 0..rows {
            let tr = elem(&mut dom, table, "tr", &[]);
            for _ in 0..cols {
                let td = elem(&mut dom, tr, "td", &[("width", "100")]);
                text(&mut dom, td, "x");
            }
        }
        (dom, table)
    }

    #[test]
    fn two_by_two_equal_columns() {
        let (dom, _) = grid_dom(2, 2);
        let styles = styled(&dom, vec![rule("td", &[("background", "#0000ff")])]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let mut bg = rects(&list, Color::rgb(0, 0, 255));
        bg.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap()
                .then(a.x.partial_cmp(&b.x).unwrap())
        });
        assert_eq!(bg.len(), 4, "one background per cell: {bg:?}");
        assert!((bg[0].x).abs() < 0.01 && (bg[1].x - 102.0).abs() < 0.01);
        assert!(bg[2].y > bg[0].y, "second row must sit below the first");
        assert!((bg[0].w - 100.0).abs() < 0.01);
    }

    #[test]
    fn hn_style_three_column_row_has_no_void() {
        // HN's item rows have no width attributes: rank, vote arrow and story.
        // The story column must take the leftover space instead of an equal
        // three-way split that parks it 700px from the rank.
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "1000")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        let rank = elem(&mut dom, tr, "td", &[("class", "title")]);
        text(&mut dom, rank, "1.");
        let vote = elem(&mut dom, tr, "td", &[("class", "votelinks")]);
        text(&mut dom, vote, "^");
        let story = elem(&mut dom, tr, "td", &[("class", "title")]);
        text(
            &mut dom,
            story,
            "Pill to treat pancreatic cancer hailed as biggest breakthrough in decades",
        );
        let styles = compute_styles(&dom, &[]);
        let list = layout_table_dom(&dom, &styles, 1000.0);
        let trs = text_rects(&list);
        let rank_rect = trs.iter().find(|r| r.x < 50.0).expect("rank column");
        let story_rect = trs.iter().find(|r| r.w > 100.0).expect("story column");
        assert!(
            story_rect.x - (rank_rect.x + rank_rect.w) < 200.0,
            "no empty middle column between rank and story: {trs:?}"
        );
        assert!(
            trs.iter().all(|r| r.x >= 0.0 && r.x + r.w <= 1001.0),
            "all columns stay inside the viewport: {trs:?}"
        );
    }

    #[test]
    fn percentage_table_width_is_honored() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "50%")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        let td = elem(&mut dom, tr, "td", &[("width", "20")]);
        text(&mut dom, td, "x");
        let styles = styled(&dom, vec![rule("table", &[("background", "#0000ff")])]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let bg = rects(&list, Color::rgb(0, 0, 255));
        assert_eq!(bg.len(), 1);
        assert!(
            (bg[0].w - 400.0).abs() < 0.01,
            "width=50% of an 800px viewport: {bg:?}"
        );
    }

    #[test]
    fn colspan_widens_and_rowspan_deepens() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "300")]);
        let r1 = elem(&mut dom, table, "tr", &[]);
        let wide = elem(&mut dom, r1, "td", &[("colspan", "2")]);
        text(&mut dom, wide, "wide");
        let r2 = elem(&mut dom, table, "tr", &[]);
        let deep = elem(&mut dom, r2, "td", &[("rowspan", "2")]);
        text(&mut dom, deep, "deep");
        let _fill = elem(&mut dom, r2, "td", &[]);
        let r3 = elem(&mut dom, table, "tr", &[]);
        let last = elem(&mut dom, r3, "td", &[]);
        text(&mut dom, last, "last");

        let styles = styled(&dom, vec![rule("td", &[("background", "#00ff00")])]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let mut bg = rects(&list, Color::rgb(0, 255, 0));
        bg.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap()
                .then(a.x.partial_cmp(&b.x).unwrap())
        });
        assert!(
            bg[0].w > 290.0,
            "a colspan=2 cell spans both columns: {bg:?}"
        );
        let ty = text_rects(&list);
        let deep_y = ty.iter().find(|r| r.y == bg[1].y + 1.0).map(|r| r.y);
        assert!(
            deep_y.is_some() || ty.len() >= 3,
            "rowspan cell text still renders"
        );
        assert!(bg.last().unwrap().y > bg[0].y, "rows stack");
    }

    #[test]
    fn spacing_gap_is_visible() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "208")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        for _ in 0..2 {
            let td = elem(&mut dom, tr, "td", &[("width", "100")]);
            text(&mut dom, td, "x");
        }
        let styles = styled(
            &dom,
            vec![
                rule("table", &[("border-spacing", "8px 8px")]),
                rule("td", &[("background", "#ff0000")]),
            ],
        );
        let list = layout_table_dom(&dom, &styles, 800.0);
        let mut bg = rects(&list, Color::rgb(255, 0, 0));
        bg.sort_by_key(|r| r.x as i32);
        assert_eq!(bg.len(), 2);
        assert!(
            ((bg[1].x - (bg[0].x + bg[0].w)) - 8.0).abs() < 0.01,
            "cells must be separated by border-spacing: {bg:?}"
        );
    }

    #[test]
    fn fixed_width_is_honored() {
        let (dom, _) = grid_dom(2, 1);
        let styles = styled(&dom, vec![rule("table", &[("background", "#0000ff")])]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let table_bg = rects(&list, Color::rgb(0, 0, 255));
        assert_eq!(table_bg.len(), 1);
        assert!((table_bg[0].w - 202.0).abs() < 0.01);
        let tr = texts(&list);
        assert_eq!(tr, ["x", "x"]);
    }

    #[test]
    fn spans_are_clamped() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "3000")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        let zero = elem(&mut dom, tr, "td", &[("colspan", "0")]);
        text(&mut dom, zero, "zero");
        let huge = elem(&mut dom, tr, "td", &[("colspan", "1001")]);
        text(&mut dom, huge, "huge");
        let deep = elem(&mut dom, tr, "td", &[("rowspan", "65535")]);
        text(&mut dom, deep, "deep");
        let styles = styled(&dom, vec![rule("td", &[("background", "#0000ff")])]);
        let started = std::time::Instant::now();
        let list = layout_table_dom(&dom, &styles, 3200.0);
        assert!(started.elapsed().as_secs_f64() < 2.0);
        assert_eq!(
            rects(&list, Color::rgb(0, 0, 255)).len(),
            3,
            "all three span-clamped cells must still be laid out"
        );
    }

    #[test]
    fn nested_inline_content_in_cells() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "300")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        let td = elem(&mut dom, tr, "td", &[]);
        let span = elem(&mut dom, td, "span", &[]);
        text(&mut dom, span, "hello");
        let em = elem(&mut dom, td, "em", &[]);
        text(&mut dom, em, "world");
        let styles = compute_styles(&dom, &[]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let t = texts(&list);
        assert!(t.iter().any(|s| s.contains("hello")), "{t:?}");
        assert!(t.iter().any(|s| s.contains("world")), "{t:?}");
    }

    #[test]
    fn caption_sits_above_the_grid() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "200")]);
        let cap = elem(&mut dom, table, "caption", &[]);
        text(&mut dom, cap, "Caption");
        let tr = elem(&mut dom, table, "tr", &[]);
        let td = elem(&mut dom, tr, "td", &[]);
        text(&mut dom, td, "Cell");
        let styles = compute_styles(&dom, &[]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let trs = text_rects(&list);
        let cap_y = trs.iter().find(|r| r.y <= 19.2).map(|r| r.y);
        let cell_y = trs.iter().map(|r| r.y).fold(f32::NEG_INFINITY, f32::max);
        assert!(cap_y.is_some(), "caption must render: {trs:?}");
        assert!(cell_y > cap_y.unwrap(), "{trs:?}");
    }

    #[test]
    fn ten_thousand_cells_render_under_two_seconds() {
        let (dom, _) = grid_dom(100, 100);
        let styles = styled(&dom, vec![rule("td", &[("background", "#0000ff")])]);
        let started = std::time::Instant::now();
        let list = layout_table_dom(&dom, &styles, 800.0);
        let elapsed = started.elapsed();
        assert!(elapsed.as_secs_f64() < 2.0, "10k cells took {elapsed:?}");
        assert_eq!(rects(&list, Color::rgb(0, 0, 255)).len(), 10_000);
    }

    #[test]
    fn adversarial_spans_stay_within_the_slot_budget() {
        // 100k cells each asking for colspan=1000/rowspan=65535 would need
        // ~100M grid slots with no budget. Both debug (debug_assert) and
        // release must stay under MAX_GRID_SLOTS while still painting content.
        assert_eq!(MAX_GRID_SLOTS, 1_000_000);
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "1000")]);
        for _ in 0..100_000 {
            let tr = elem(&mut dom, table, "tr", &[]);
            let td = elem(
                &mut dom,
                tr,
                "td",
                &[("colspan", "1000"), ("rowspan", "65535")],
            );
            text(&mut dom, td, "x");
        }
        let styles = compute_styles(&dom, &[]);
        let started = std::time::Instant::now();
        let list = layout_table_dom(&dom, &styles, 1200.0);
        assert!(
            started.elapsed().as_secs_f64() < 20.0,
            "adversarial table took {:?}",
            started.elapsed()
        );
        assert!(!list.items.is_empty(), "cells must still render");
        assert!(
            texts(&list).len() >= 100_000,
            "every cell's content must survive clamping"
        );
    }

    #[test]
    fn anonymous_rows_and_cells_from_sloppy_html() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "300")]);
        text(&mut dom, table, "loose");
        let tr = elem(&mut dom, table, "tr", &[]);
        let td = elem(&mut dom, tr, "td", &[]);
        text(&mut dom, td, "in");
        text(&mut dom, tr, "bare");
        text(&mut dom, table, "tail");
        let styles = compute_styles(&dom, &[]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let t = texts(&list);
        for word in ["loose", "in", "bare", "tail"] {
            assert!(t.iter().any(|s| s.contains(word)), "missing {word}: {t:?}");
        }
    }

    #[test]
    fn display_flex_cell_sizes_children_via_flex() {
        let mut dom = Dom::new();
        let root = dom.root();
        let table = elem(&mut dom, root, "table", &[("width", "300")]);
        let tr = elem(&mut dom, table, "tr", &[]);
        let td = elem(&mut dom, tr, "td", &[("style", "display:flex")]);
        for _ in 0..2 {
            elem(
                &mut dom,
                td,
                "div",
                &[(
                    "style",
                    "display:block; width:40px; height:20px; background:#ff0000",
                )],
            );
        }

        let styles = compute_styles(&dom, &[]);
        let list = layout_table_dom(&dom, &styles, 800.0);
        let r = rects(&list, Color::rgb(255, 0, 0));
        assert_eq!(r.len(), 2, "both cell children must paint: {r:?}");
        assert!(
            (r[1].x - r[0].x - 40.0).abs() < 0.01 && (r[0].y - r[1].y).abs() < 0.01,
            "a flex cell lays children out in a row, not stacked: {r:?}"
        );
    }
}

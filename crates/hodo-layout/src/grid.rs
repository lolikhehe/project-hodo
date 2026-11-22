//! CSS grid layout: explicit tracks, line placement and row-major auto-flow.
//!
//! Overlapping cells are not resolved: every item is laid out in DOM order and
//! simply paints over earlier ones (DOM order = paint order).

use super::*;
use hodo_style::GridTrack;

const MAX_TRACKS: usize = 1000;
const MAX_SPAN: usize = 1000;
const MAX_IMPLICIT_ROWS: usize = 100;

/// A template track after `MinMax`/`Repeat` expansion.
#[derive(Clone, Copy)]
enum Track {
    Fixed(f32),
    Percent(f32),
    Auto,
    Fr(f32),
}

fn expand(t: &GridTrack, out: &mut Vec<Track>) {
    if out.len() >= MAX_TRACKS {
        return;
    }
    match t {
        GridTrack::Fixed(px) => out.push(Track::Fixed(*px)),
        GridTrack::Percent(p) => out.push(Track::Percent(*p)),
        GridTrack::Fr(f) => out.push(Track::Fr(*f)),
        GridTrack::Auto => out.push(Track::Auto),
        GridTrack::MinMax(_, max) => expand(max, out), // keep the max track
        GridTrack::Repeat(n, tracks) => {
            if tracks.is_empty() {
                return;
            }
            for _ in 0..*n {
                for t in tracks {
                    if out.len() >= MAX_TRACKS {
                        return;
                    }
                    expand(t, out);
                }
            }
        }
    }
}

fn expand_all(templates: &[GridTrack]) -> Vec<Track> {
    let mut out = Vec::new();
    for t in templates {
        if out.len() >= MAX_TRACKS {
            break;
        }
        expand(t, &mut out);
    }
    if out.is_empty() {
        out.push(Track::Auto); // one implicit track when the template is empty
    }
    out
}

/// Max-content width via the engine's 0.6 * font-size glyph metric plus the
/// element's own horizontal insets. No display list is produced here.
fn metric_w(ctx: &Ctx<'_>, id: NodeId, fallback: &ComputedStyle) -> f32 {
    let st = ctx.styles.get(id).unwrap_or(fallback);
    match &ctx.dom.get(id).kind {
        NodeKind::Text(t) => {
            let words: Vec<&str> = t.split_whitespace().collect();
            let chars: usize = words.iter().map(|w| w.chars().count()).sum();
            let spaces = words.len().saturating_sub(1);
            (chars + spaces) as f32 * 0.6 * st.font_size
        }
        NodeKind::Element { .. } => {
            let inset = st.margin.left
                + st.margin.right
                + st.padding.left
                + st.padding.right
                + 2.0 * st.border_width.max(0.0);
            if ctx
                .dom
                .tag(id)
                .is_some_and(|t| t.eq_ignore_ascii_case("img"))
            {
                return ctx
                    .dom
                    .attr(id, "width")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(64.0)
                    + inset;
            }
            inset
                + ctx
                    .dom
                    .children(id)
                    .iter()
                    .map(|&c| metric_w(ctx, c, st))
                    .fold(0.0_f32, f32::max)
        }
        NodeKind::Document => ctx
            .dom
            .children(id)
            .iter()
            .map(|&c| metric_w(ctx, c, st))
            .sum(),
    }
}

/// Metric-only height for auto rows (one line per text run + vertical insets).
fn metric_h(ctx: &Ctx<'_>, id: NodeId, fallback: &ComputedStyle) -> f32 {
    let st = ctx.styles.get(id).unwrap_or(fallback);
    match &ctx.dom.get(id).kind {
        NodeKind::Text(_) => 1.2 * st.font_size,
        NodeKind::Element { .. } => {
            let inset = st.margin.top
                + st.margin.bottom
                + st.padding.top
                + st.padding.bottom
                + 2.0 * st.border_width.max(0.0);
            if ctx
                .dom
                .tag(id)
                .is_some_and(|t| t.eq_ignore_ascii_case("img"))
            {
                return ctx
                    .dom
                    .attr(id, "height")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(64.0)
                    + inset;
            }
            let child: f32 = ctx
                .dom
                .children(id)
                .iter()
                .map(|&c| metric_h(ctx, c, st))
                .sum();
            (child + inset).max(1.2 * st.font_size)
        }
        NodeKind::Document => ctx
            .dom
            .children(id)
            .iter()
            .map(|&c| metric_h(ctx, c, st))
            .sum(),
    }
}

#[derive(Clone, Copy)]
struct Cell {
    c: usize,
    r: usize,
    cs: usize,
    rs: usize,
}

/// Line numbers are 1-based; negatives count back from the last explicit line.
fn clamp_line(n: i32, tracks: usize) -> usize {
    let last = tracks as i32 + 1;
    let v = if n < 0 { last + n + 1 } else { n };
    v.clamp(1, last) as usize
}

/// Resolve one placement axis into an optional 0-based start and a track span.
fn axis(start: GridLine, end: GridLine, tracks: usize) -> (Option<usize>, usize) {
    match (start, end) {
        (GridLine::Line(a), GridLine::Line(b)) => {
            let (a, b) = (clamp_line(a, tracks), clamp_line(b, tracks));
            let (a, b) = if a <= b { (a, b) } else { (b, a) };
            (Some(a - 1), (b - a).max(1))
        }
        (GridLine::Line(a), GridLine::Span(k)) => (
            Some(clamp_line(a, tracks) - 1),
            (k as usize).clamp(1, MAX_SPAN),
        ),
        (GridLine::Line(a), GridLine::Auto) => (Some(clamp_line(a, tracks) - 1), 1),
        (GridLine::Span(k), GridLine::Line(b)) => {
            let span = (k as usize).clamp(1, MAX_SPAN);
            (Some(clamp_line(b, tracks).saturating_sub(span + 1)), span)
        }
        (GridLine::Span(k), GridLine::Auto) => (None, (k as usize).clamp(1, MAX_SPAN)),
        (GridLine::Auto, GridLine::Line(b)) => (Some(clamp_line(b, tracks).saturating_sub(2)), 1),
        (GridLine::Auto, GridLine::Span(k)) => (None, (k as usize).clamp(1, MAX_SPAN)),
        (GridLine::Span(k), GridLine::Span(_)) => (None, (k as usize).clamp(1, MAX_SPAN)),
        (GridLine::Auto, GridLine::Auto) => (None, 1),
    }
}

fn fits(
    occ: &[Vec<bool>],
    r: usize,
    c: usize,
    rs: usize,
    cs: usize,
    cols: usize,
    rows: usize,
) -> bool {
    if r >= rows || c >= cols || r + rs > rows || c + cs > cols {
        return false;
    }
    (r..r + rs).all(|rr| (c..c + cs).all(|cc| !occ[rr][cc]))
}

fn find_slot(
    occ: &[Vec<bool>],
    mut r: usize,
    mut c: usize,
    rs: usize,
    cs: usize,
    cols: usize,
    rows: usize,
) -> (usize, usize) {
    let cs = cs.clamp(1, cols);
    let rs = rs.min(rows);
    while r < rows {
        while c + cs <= cols {
            if fits(occ, r, c, rs, cs, cols, rows) {
                return (r, c);
            }
            c += 1;
        }
        c = 0;
        r += 1;
    }
    (rows.saturating_sub(1), cols.saturating_sub(cs))
}

fn find_row(
    occ: &[Vec<bool>],
    mut r: usize,
    c: usize,
    rs: usize,
    cs: usize,
    cols: usize,
    rows: usize,
) -> usize {
    let cs = cs.clamp(1, cols);
    let rs = rs.min(rows);
    while r < rows && !fits(occ, r, c, rs, cs, cols, rows) {
        r += 1;
    }
    r.min(rows.saturating_sub(1))
}

fn find_col(
    occ: &[Vec<bool>],
    r: usize,
    mut c: usize,
    rs: usize,
    cs: usize,
    cols: usize,
    rows: usize,
) -> usize {
    let cs = cs.clamp(1, cols);
    let rs = rs.min(rows);
    while c < cols && !fits(occ, r, c, rs, cs, cols, rows) {
        c += 1;
    }
    c.min(cols.saturating_sub(cs))
}

/// Fixed and percent tracks first, then auto tracks capped by max-content and
/// the remaining space, then `fr` tracks sharing what is left. Never NaN/neg.
fn resolve_cols(tracks: &[Track], content_w: f32, gap: f32, mc: &[f32]) -> Vec<f32> {
    let n = tracks.len();
    let gaps = gap * n.saturating_sub(1) as f32;
    let mut used = 0.0_f32;
    let mut fr_sum = 0.0_f32;
    let mut widths = vec![0.0_f32; n];
    for (i, t) in tracks.iter().enumerate() {
        match t {
            Track::Fixed(px) => {
                let w = (*px).max(0.0);
                widths[i] = w;
                used += w;
            }
            Track::Percent(pct) => {
                let w = (*pct / 100.0).max(0.0) * content_w;
                widths[i] = w;
                used += w;
            }
            Track::Auto => {
                let leftover = (content_w - gaps - used).max(0.0);
                let w = mc.get(i).copied().unwrap_or(0.0).min(leftover);
                widths[i] = w;
                used += w;
            }
            Track::Fr(f) => fr_sum += (*f).max(0.0),
        }
    }
    let free = content_w - gaps - used;
    if free > 0.0 && fr_sum > 0.0 {
        for (i, t) in tracks.iter().enumerate() {
            if let Track::Fr(f) = t {
                widths[i] = free * (*f).max(0.0) / fr_sum;
            }
        }
    }
    widths
}

pub(crate) fn layout_grid(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    let m = &style.margin;
    let p = &style.padding;
    let bw = style.border_width.max(0.0);
    let left = avail.x + m.left + bw + p.left;
    let top = avail.y + m.top + bw + p.top;
    let content_w = (avail.width - m.left - m.right - 2.0 * bw - p.left - p.right).max(0.0);

    // (1) expand templates; MinMax keeps its max track; cap at 1000 tracks.
    let col_tracks = expand_all(&style.grid_template_columns);
    let row_tracks = expand_all(&style.grid_template_rows);
    let cols = col_tracks.len();
    let row_limit = row_tracks.len() + MAX_IMPLICIT_ROWS;

    let fallback = fallback_style();
    let items: Vec<NodeId> = ctx
        .dom
        .children(id)
        .iter()
        .copied()
        .filter(|&c| {
            !ctx.styles
                .get(c)
                .is_some_and(|s| s.display == Display::None)
        })
        .collect();

    // (2) max-content metrics, no dry-run layout.
    let metrics: Vec<(f32, f32)> = items
        .iter()
        .map(|&c| (metric_w(ctx, c, &fallback), metric_h(ctx, c, &fallback)))
        .collect();

    // (5) placements: explicit lines first, then a row-major auto-flow cursor.
    let mut occ = vec![vec![false; cols]; row_limit];
    let mut cells: Vec<Cell> = Vec::with_capacity(items.len());
    let (mut cur_r, mut cur_c) = (0usize, 0usize);
    for (&child, _) in items.iter().zip(metrics.iter()) {
        let st = ctx.styles.get(child).unwrap_or(&fallback);
        let (ce, cs) = axis(st.grid_column.0, st.grid_column.1, cols);
        let (re, rs) = axis(st.grid_row.0, st.grid_row.1, row_limit);
        let (c, r) = match (ce, re) {
            (Some(c), Some(r)) => (c.min(cols - 1), r.min(row_limit - 1)),
            (Some(c), None) => {
                let c = c.min(cols - 1);
                let r = find_row(&occ, cur_r, c, rs, cs, cols, row_limit);
                cur_r = r + 1;
                (c, r)
            }
            (None, Some(r)) => {
                let r = r.min(row_limit - 1);
                let c = find_col(&occ, r, cur_c, rs, cs, cols, row_limit);
                cur_c = c + cs;
                (c, r)
            }
            (None, None) => {
                let (r, c) = find_slot(&occ, cur_r, cur_c, rs, cs, cols, row_limit);
                cur_r = r;
                cur_c = c + cs;
                (c, r)
            }
        };
        let cs = cs.clamp(1, cols.saturating_sub(c));
        let rs = rs.clamp(1, row_limit.saturating_sub(r));
        for row in occ.iter_mut().skip(r).take(rs) {
            for slot in row.iter_mut().skip(c).take(cs) {
                *slot = true;
            }
        }
        cells.push(Cell { c, r, cs, rs });
    }
    let rows = cells.iter().map(|c| c.r + c.rs).max().unwrap_or(0).max(1);

    let mut col_mc = vec![0.0_f32; cols];
    let mut row_mc = vec![0.0_f32; rows];
    for (cell, &(w, h)) in cells.iter().zip(metrics.iter()) {
        for v in col_mc.iter_mut().skip(cell.c).take(cell.cs) {
            *v = (*v).max(w);
        }
        for v in row_mc.iter_mut().skip(cell.r).take(cell.rs) {
            *v = (*v).max(h);
        }
    }

    // (3) resolve track sizes.
    let widths = resolve_cols(&col_tracks, content_w, style.column_gap, &col_mc);
    // shortcut: Avail carries no height, so percent/fr rows use metric height.
    let mut heights = vec![0.0_f32; rows];
    for r in 0..rows {
        heights[r] = match row_tracks.get(r) {
            Some(Track::Fixed(px)) => (*px).max(0.0),
            _ => row_mc[r],
        };
    }

    // (4) offsets along both axes.
    let mut xs = vec![0.0_f32; cols];
    let mut acc = left;
    for c in 0..cols {
        xs[c] = acc;
        acc += widths[c] + style.column_gap;
    }
    let mut ys = vec![0.0_f32; rows];
    let mut acc = top;
    for r in 0..rows {
        ys[r] = acc;
        acc += heights[r] + style.row_gap;
    }

    // (6) lay out every item into its cell via the shared block path.
    for (&child, cell) in items.iter().zip(cells.iter()) {
        let tag = ctx.dom.tag(child).unwrap_or("");
        let child_link = if tag.eq_ignore_ascii_case("a") {
            ctx.dom
                .attr(child, "href")
                .map(str::to_string)
                .or_else(|| link.clone())
        } else {
            link.clone()
        };
        let child_style = ctx
            .styles
            .get(child)
            .cloned()
            .unwrap_or_else(fallback_style);
        let end = (cell.c + cell.cs).min(cols);
        let span = end - cell.c;
        let mut w = 0.0_f32;
        for (i, cw) in widths.iter().skip(cell.c).take(span).enumerate() {
            w += *cw;
            if i + 1 < span {
                w += style.column_gap;
            }
        }
        let avail = Avail {
            x: xs[cell.c],
            y: ys[cell.r],
            width: w,
        };
        if let NodeKind::Text(t) = &ctx.dom.get(child).kind {
            // A bare text child is an anonymous grid item: feed it directly.
            let mut flow = Flow::new(avail.x, avail.x + avail.width, avail.y);
            let owner = ctx.dom.parent(child).unwrap_or(id);
            feed_text(&mut flow, ctx.list, t, &child_style, child_link, &[], owner);
            flush_line(&mut flow, ctx.list);
        } else {
            dispatch_child(ctx, &child_style, child, avail, child_link);
        }
    }

    let content_h = (ys[rows - 1] + heights[rows - 1] - top).max(0.0);
    let box_h = content_h + p.top + p.bottom + 2.0 * bw;
    m.top + box_h + m.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_style::compute_styles;

    type CellSpec<'a> = (&'a [(&'a str, &'a str)], Option<&'a str>);

    fn decls_str(decls: &[(&str, &str)]) -> String {
        decls
            .iter()
            .map(|(p, v)| format!("{p}: {v}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Build a grid container with one `<div>` per cell; each cell carries a
    /// red background so its box can be found in the display list.
    fn grid_doc(grid: &[(&str, &str)], cells: &[CellSpec<'_>]) -> Dom {
        let mut dom = Dom::new();
        let root = dom.root();
        let mut gd = vec![("display", "grid")];
        gd.extend_from_slice(grid);
        let g = dom.create(NodeKind::Element {
            tag: "div".into(),
            attrs: vec![("style".into(), decls_str(&gd))],
        });
        dom.append(root, g);
        for (extra, text) in cells {
            let mut cd = vec![("background", "#ff0000")];
            cd.extend_from_slice(extra);
            let c = dom.create(NodeKind::Element {
                tag: "div".into(),
                attrs: vec![("style".into(), decls_str(&cd))],
            });
            dom.append(g, c);
            if let Some(t) = text {
                let n = dom.create(NodeKind::Text((*t).into()));
                dom.append(c, n);
            }
        }
        dom
    }

    fn render(grid: &[(&str, &str)], cells: &[CellSpec<'_>], w: f32) -> DisplayList {
        let dom = grid_doc(grid, cells);
        let styles = compute_styles(&dom, &[]);
        layout(&dom, &styles, Size { w, h: 600.0 })
    }

    fn reds(list: &DisplayList) -> Vec<Rect> {
        let red = Color::rgb(255, 0, 0);
        list.items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Rect { rect, color, .. } if *color == red => Some(*rect),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn two_by_two_px_grid_with_gaps() {
        let list = render(
            &[
                ("grid-template-columns", "100px 100px"),
                ("grid-template-rows", "40px 40px"),
                ("gap", "10px"),
            ],
            &[(&[], None), (&[], None), (&[], None), (&[], None)],
            800.0,
        );
        let r = reds(&list);
        assert_eq!(r.len(), 4, "four cells must paint: {r:?}");
        assert_eq!((r[0].x, r[0].y), (0.0, 0.0));
        assert_eq!((r[1].x, r[1].y), (110.0, 0.0));
        assert_eq!((r[2].x, r[2].y), (0.0, 50.0));
        assert_eq!((r[3].x, r[3].y), (110.0, 50.0));
        assert!((r[0].w - 100.0).abs() < 0.01);
    }

    #[test]
    fn fr_tracks_split_free_space() {
        let list = render(
            &[("grid-template-columns", "1fr 2fr")],
            &[(&[], None), (&[], None)],
            300.0,
        );
        let r = reds(&list);
        assert!(
            (r[0].x - 0.0).abs() < 0.01 && (r[0].w - 100.0).abs() < 0.01,
            "{r:?}"
        );
        assert!(
            (r[1].x - 100.0).abs() < 0.01 && (r[1].w - 200.0).abs() < 0.01,
            "{r:?}"
        );
    }

    #[test]
    fn auto_track_is_at_least_max_content() {
        // 10 glyphs at 10px -> 60px max-content; viewport is wider.
        let list = render(
            &[("grid-template-columns", "auto"), ("font-size", "10px")],
            &[(&[], Some("abcdefghij"))],
            300.0,
        );
        let r = reds(&list);
        assert!(
            (r[0].w - 60.0).abs() < 0.01,
            "auto column must fit max-content: {r:?}"
        );
    }

    #[test]
    fn explicit_column_and_row_span() {
        let list = render(
            &[
                ("grid-template-columns", "repeat(3, 100px)"),
                ("grid-template-rows", "repeat(3, 40px)"),
            ],
            &[
                (&[("grid-column", "2/3"), ("grid-row", "span 2")], None),
                (&[], None),
            ],
            800.0,
        );
        let r = reds(&list);
        assert_eq!((r[0].x, r[0].y), (100.0, 0.0), "{r:?}");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn out_of_range_lines_clamp() {
        let list = render(
            &[("grid-template-columns", "100px 100px")],
            &[(&[("grid-column", "0 / 99")], None)],
            800.0,
        );
        let r = reds(&list);
        assert!((r[0].x - 0.0).abs() < 0.01, "{r:?}");
        assert!(
            (r[0].w - 200.0).abs() < 0.01,
            "0/99 must clamp to the two tracks: {r:?}"
        );
    }

    #[test]
    fn overlapping_cells_both_paint() {
        let list = render(
            &[
                ("grid-template-columns", "100px 100px"),
                ("grid-template-rows", "40px"),
            ],
            &[
                (&[("grid-column", "1/2"), ("grid-row", "1/2")], None),
                (&[("grid-column", "1/2"), ("grid-row", "1/2")], None),
            ],
            800.0,
        );
        let r = reds(&list);
        assert_eq!(r.len(), 2, "both overlapping cells must render: {r:?}");
        assert_eq!(r[0].x, r[1].x);
        assert_eq!(r[0].y, r[1].y);
    }

    #[test]
    fn fr_with_zero_free_space_is_zero_not_nan() {
        let list = render(
            &[("grid-template-columns", "100px 1fr")],
            &[(&[], None), (&[], None)],
            100.0,
        );
        let r = reds(&list);
        assert!(r[1].w.is_finite() && (r[1].w - 0.0).abs() < 0.01, "{r:?}");
    }

    #[test]
    fn huge_repeat_is_clipped() {
        let started = std::time::Instant::now();
        let list = render(
            &[("grid-template-columns", "repeat(99999, 1fr)")],
            &[(&[], None)],
            300.0,
        );
        assert!(
            started.elapsed().as_secs_f64() < 2.0,
            "repeat must be bounded"
        );
        let r = reds(&list);
        assert_eq!(r.len(), 1);
        assert!(r[0].w.is_finite() && r[0].w < 1.0, "{r:?}");
    }

    #[test]
    fn minmax_keeps_the_max_track() {
        let list = render(
            &[("grid-template-columns", "minmax(20px, 80px)")],
            &[(&[], None)],
            800.0,
        );
        let r = reds(&list);
        assert!(
            (r[0].w - 80.0).abs() < 0.01,
            "minmax must keep its max: {r:?}"
        );
    }

    #[test]
    fn percent_track_uses_content_width() {
        let list = render(
            &[("grid-template-columns", "50% 50%")],
            &[(&[], None), (&[], None)],
            300.0,
        );
        let r = reds(&list);
        assert!(
            (r[0].w - 150.0).abs() < 0.01 && (r[1].x - 150.0).abs() < 0.01,
            "{r:?}"
        );
    }

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str, style: &str) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: vec![("style".into(), style.into())],
        });
        dom.append(parent, id);
        id
    }

    #[test]
    fn grid_item_form_control_emits_control_item() {
        let mut dom = Dom::new();
        let root = dom.root();
        let grid = elem(
            &mut dom,
            root,
            "div",
            "display:grid; grid-template-columns:200px",
        );
        let input = dom.create(NodeKind::Element {
            tag: "input".into(),
            attrs: vec![("name".into(), "q".into())],
        });
        dom.append(grid, input);

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        let names: Vec<_> = list
            .items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Control { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            names,
            ["q"],
            "a control grid item must be dispatched to form layout"
        );
    }

    #[test]
    fn grid_item_with_display_flex_uses_flex_layout() {
        let mut dom = Dom::new();
        let root = dom.root();
        let grid = elem(
            &mut dom,
            root,
            "div",
            "display:grid; grid-template-columns:300px",
        );
        let flex = elem(&mut dom, grid, "div", "display:flex; width:300px");
        for _ in 0..2 {
            elem(
                &mut dom,
                flex,
                "div",
                "background:#ff0000; width:40px; height:20px",
            );
        }

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 400.0, h: 600.0 });
        let r = reds(&list);
        assert_eq!(r.len(), 2, "both flex children must paint: {r:?}");
        assert!(
            (r[1].x - r[0].x - 40.0).abs() < 0.01 && (r[0].y - r[1].y).abs() < 0.01,
            "flex children must sit side by side, not stack as blocks: {r:?}"
        );
    }
}

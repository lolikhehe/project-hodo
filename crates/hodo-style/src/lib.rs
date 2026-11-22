//! Style resolution: UA sheet + author sheets + inline `style`, cascaded by
//! origin, `!important`, specificity and source order.
//!
//! Limits inherited from `hodo-css`: at most 64 compounds per selector, `:not()`
//! depth 2, 10 000 rules per sheet. Each element tests only rules selected by
//! the rightmost-compound index; descendant rules are fast-rejected by a
//! 64-bit ancestor Bloom filter, sibling rules use a cached sibling scan.
//! Matching is iterative and a 1000-level DOM stays under the 1 s test budget.
//!
//! Style resolution has two phases: A matches every element against the read-
//! only DOM + sheets (parallel over disjoint arena-slot slices when there are
//! at least 512 elements), B cascades the precomputed matched-rule lists with
//! one sequential sharing LRU. Results are indexed by arena slot, so the output
//! is independent of thread scheduling.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};

use hodo_css::{
    cascade_layer_name, complex, declaration_value, is_compound, is_important, matches_compiled,
    matches_plain, matches_selector_ref, parse_style_attribute, parse_stylesheet, resolve_calc_now,
    resolve_calc_with_basis, resolve_custom_properties, substitute_vars, AncestorBloom, Compound,
    Declaration, MatchContext, PseudoClass, RuleIndex, StyleRule, Stylesheet, LAYER_MARKER,
    LAYER_NAME_MARKER,
};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_types::{Color, Edges, Size};

// (origin, source order, rule, layer rank; u32::MAX = unlayered)
type OrderedRule<'a> = (u8, usize, &'a StyleRule, u32);
// (origin rank, layer rank, specificity, source order, declaration references)
type Match<'a> = (u8, u32, (u32, u32, u32), usize, &'a [&'a Declaration]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    None,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Table,
    InlineTable,
    TableRowGroup,
    TableRow,
    TableCell,
    TableCaption,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    Stretch,
    Center,
    FlexStart,
    FlexEnd,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignSelf {
    Auto,
    Stretch,
    Center,
    FlexStart,
    FlexEnd,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexBasis {
    Auto,
    Length(f32),
}

// shortcut: GridTrack cannot derive Copy (Box/Vec are not Copy); Clone + PartialEq is the usable subset.
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    Fixed(f32),
    Fr(f32),
    /// Percentage of the grid container's content-box width, as `n` from `n%`.
    Percent(f32),
    Auto,
    MinMax(Box<GridTrack>, Box<GridTrack>),
    Repeat(u32, Vec<GridTrack>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridLine {
    Auto,
    Line(i32),
    Span(u32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableLayout {
    Auto,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderCollapse {
    Separate,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    Normal,
    Number(f32),
    Length(f32),
}

/// Cells use Baseline/Top/Middle/Bottom; inline content also uses Sub/Super/Length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Baseline,
    Sub,
    Super,
    Top,
    Middle,
    Bottom,
    Length(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhiteSpace {
    Normal,
    Pre,
    NoWrap,
    PreWrap,
    PreLine,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Float {
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Clip,
}

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Color,
    pub background: Color,
    pub font_size: f32,
    pub font_weight: u16,
    /// Parsed font stack (first family + generic fallback). Stored and inherited
    /// only; layout still uses its built-in font (TODO(fontstack) in the
    /// integrator's layout pass).
    pub font_family: String,
    pub italic: bool,
    pub underline: bool,
    pub margin: Edges<f32>,
    pub padding: Edges<f32>,
    pub border_width: f32,
    /// Per-side border widths from `border-width` / `border-*-width`.
    pub border_widths: Edges<f32>,
    pub border_color: Color,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub gap: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: FlexBasis,
    pub order: i32,
    pub grid_template_columns: Vec<GridTrack>,
    pub grid_template_rows: Vec<GridTrack>,
    pub grid_column: (GridLine, GridLine),
    pub grid_row: (GridLine, GridLine),
    pub column_gap: f32,
    pub row_gap: f32,
    pub table_layout: TableLayout,
    pub border_collapse: BorderCollapse,
    /// (horizontal, vertical) spacing between table cells.
    pub border_spacing: (f32, f32),
    pub line_height: LineHeight,
    pub vertical_align: VerticalAlign,
    pub text_align: TextAlign,
    pub text_indent: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub white_space: WhiteSpace,
    pub overflow_wrap: OverflowWrap,
    pub line_through: bool,
    pub position: Position,
    pub float: Float,
    pub clear: Clear,
    pub overflow: Overflow,
    /// Per-side offsets; `None` means `auto`.
    pub offsets: Edges<Option<f32>>,
    pub width: Option<f32>,
    /// Percentage width (`n` from `n%`), resolved against the containing
    /// block's content width in layout. `width` is `None` when this is `Some`.
    pub width_percent: Option<f32>,
    /// Percentage components of margin/padding (`n` from `n%`), resolved
    /// against the containing block's content width in layout.
    pub margin_percent: Edges<Option<f32>>,
    pub padding_percent: Edges<Option<f32>>,
    pub height: Option<f32>,
    pub min_size: Size,
    pub max_size: Size,
}

/// How a computed-style change propagates to the rendering pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleChangeClass {
    /// Only painted pixels change; geometry is untouched, so a repaint suffices.
    PaintOnly,
    /// Geometry, flow or text advance width changed; a relayout is required.
    LayoutAffecting,
}

/// Classify the difference between two computed styles across every field of
/// [`ComputedStyle`].
///
/// # Classification table (53 fields)
///
/// | Class | Fields |
/// |---|---|
/// | `PaintOnly` | `color`, `background`, `border_color`, `underline`, `line_through` |
/// | `LayoutAffecting` | all 48 remaining fields: `display`, `font_size`, `font_weight`, `font_family`, `italic`, `margin`, `padding`, `border_width`, `border_widths`, `flex_direction`, `flex_wrap`, `justify_content`, `align_items`, `align_self`, `gap`, `flex_grow`, `flex_shrink`, `flex_basis`, `order`, `grid_template_columns`, `grid_template_rows`, `grid_column`, `grid_row`, `column_gap`, `row_gap`, `table_layout`, `border_collapse`, `border_spacing`, `line_height`, `vertical_align`, `text_align`, `text_indent`, `letter_spacing`, `word_spacing`, `white_space`, `overflow_wrap`, `position`, `float`, `clear`, `overflow`, `offsets`, `width`, `width_percent`, `margin_percent`, `padding_percent`, `height`, `min_size`, `max_size` |
///
/// Font-family / weight / italic are deliberately `LayoutAffecting`: font
/// metrics decide glyph advance widths and baselines, so a change can reflow
/// text even though the built-in layout font currently ignores the stack.
/// No-change is reported as `PaintOnly` (a repaint is always safe, a relayout
/// is not needed). NaN values compare unequal and are conservatively treated
/// as a layout change.
///
/// # Integrator seam
/// `crates/hodo-shell/src/app.rs::recompute_if_changed` currently funnels every
/// DOM change through `relayout` (which calls `hodo_layout::layout`). Once the
/// shell retains the previous `StyleMap`, the single call site becomes:
/// `if style_change_class(&old_style, &new_style) == StyleChangeClass::LayoutAffecting
/// { relayout(viewport) } else { repaint() }`. This crate does not touch app.rs.
pub fn style_change_class(old: &ComputedStyle, new: &ComputedStyle) -> StyleChangeClass {
    macro_rules! any_changed {
        ($($field:ident),+ $(,)?) => {
            false $(|| old.$field != new.$field)+
        };
    }
    // `Edges<T>` has no PartialEq in hodo-types, so compare its four sides.
    macro_rules! edges_changed {
        ($field:ident) => {
            old.$field.top != new.$field.top
                || old.$field.right != new.$field.right
                || old.$field.bottom != new.$field.bottom
                || old.$field.left != new.$field.left
        };
    }
    let edges = edges_changed!(margin)
        || edges_changed!(padding)
        || edges_changed!(border_widths)
        || edges_changed!(offsets)
        || edges_changed!(margin_percent)
        || edges_changed!(padding_percent);
    if edges
        || any_changed!(
            display,
            font_size,
            font_weight,
            font_family,
            italic,
            border_width,
            flex_direction,
            flex_wrap,
            justify_content,
            align_items,
            align_self,
            gap,
            flex_grow,
            flex_shrink,
            flex_basis,
            order,
            grid_template_columns,
            grid_template_rows,
            grid_column,
            grid_row,
            column_gap,
            row_gap,
            table_layout,
            border_collapse,
            border_spacing,
            line_height,
            vertical_align,
            text_align,
            text_indent,
            letter_spacing,
            word_spacing,
            white_space,
            overflow_wrap,
            position,
            float,
            clear,
            overflow,
            width,
            width_percent,
            height,
            min_size,
            max_size,
        )
    {
        StyleChangeClass::LayoutAffecting
    } else {
        StyleChangeClass::PaintOnly
    }
}

#[derive(Debug, Clone, Default)]
pub struct StyleMap {
    styles: Vec<Option<ComputedStyle>>,
    /// Raw `calc()` text for `width` that contains `%`. Only layout knows the
    /// containing-block basis, so the expression is carried here verbatim and
    /// resolved by [`StyleMap::resolve_width_with_basis`].
    deferred_width: Vec<Option<String>>,
}

impl StyleMap {
    pub fn get(&self, id: NodeId) -> Option<&ComputedStyle> {
        self.styles.get(id.slot() as usize).and_then(Option::as_ref)
    }

    pub(crate) fn set(&mut self, id: NodeId, s: ComputedStyle) {
        let index = id.slot() as usize;
        if self.styles.len() <= index {
            self.styles.resize(index + 1, None);
        }
        self.styles[index] = Some(s);
    }

    /// The deferred percentage `calc()` text for `width`, if any.
    pub fn deferred_width(&self, id: NodeId) -> Option<&str> {
        self.deferred_width
            .get(id.slot() as usize)
            .and_then(Option::as_deref)
    }

    /// Resolve a deferred percentage `width` against a containing-block width.
    /// This is the layout-time seam for `calc(100% - 32px)`: `%` becomes
    /// `basis / 100`. Returns `None` when the width is not a deferred calc.
    pub fn resolve_width_with_basis(&self, id: NodeId, basis: f32) -> Option<f32> {
        self.deferred_width(id)
            .and_then(|text| resolve_calc_with_basis(text, basis))
    }

    pub(crate) fn set_deferred_width(&mut self, id: NodeId, text: Option<String>) {
        let index = id.slot() as usize;
        if self.deferred_width.len() <= index {
            self.deferred_width.resize(index + 1, None);
        }
        self.deferred_width[index] = text;
    }

    pub fn with_len(len: usize) -> Self {
        Self {
            styles: vec![None; len],
            deferred_width: vec![None; len],
        }
    }
}

/// Built-in UA stylesheet: block/inline defaults, 16px black text on transparent.
pub fn ua_stylesheet() -> &'static str {
    "html { display: block; color: black; background: transparent; font-size: 16px; }\n\
     body { display: block; }\n\
     div { display: block; }\n\
     p { display: block; }\n\
     h1 { display: block; }\n\
     h2 { display: block; }\n\
     h3 { display: block; }\n\
     h4 { display: block; }\n\
     h5 { display: block; }\n\
     h6 { display: block; }\n\
     ul { display: block; }\n\
     ol { display: block; }\n\
     li { display: block; }\n\
     span { display: inline; }\n\
     a { display: inline; }\n\
     b { display: inline; }\n\
     em { display: inline; }\n\
     strong { display: inline; }\n\
     i { display: inline; }\n\
     code { display: inline; }\n\
     label { display: inline; }\n\
     table { display: table; table-layout: auto; border-collapse: separate; border-spacing: 2px 2px; }\n\
     caption { display: table-caption; }\n\
     thead { display: table-row-group; }\n\
     tbody { display: table-row-group; }\n\
     tfoot { display: table-row-group; }\n\
     tr { display: table-row; }\n\
     td { display: table-cell; padding: 1px; vertical-align: middle; }\n\
     th { display: table-cell; padding: 1px; vertical-align: middle; }\n\
     input { display: inline-block; }\n\
     textarea { display: inline-block; }\n\
     select { display: inline-block; }\n\
     button { display: inline-block; }\n\
     form { display: block; }\n\
     fieldset { display: block; }\n"
}

/// Apply the UA sheet, then author sheets, then the element's inline `style`
/// attribute. Cascade order is UA normal < author normal < inline normal <
/// author `!important` < inline `!important` < UA `!important`, then
/// specificity, then source order. `color` and `font_size` inherit.
static UA_SHEET: std::sync::OnceLock<Stylesheet> = std::sync::OnceLock::new();

pub fn compute_styles(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    compute_styles_with_visited(dom, sheets, &HashSet::new())
}

/// Like [`compute_styles`], but seeds `:visited` matching with the given hrefs.
/// The shell should pass its redirect-visited-set so visited links stop being
/// styled as `:link`.
pub fn compute_styles_with_visited(
    dom: &Dom,
    sheets: &[Stylesheet],
    visited: &HashSet<String>,
) -> StyleMap {
    let ua = UA_SHEET.get_or_init(|| parse_stylesheet(ua_stylesheet()));
    compute_styles_parsed(dom, sheets, ua, true, visited)
}

/// The resolved GLOBAL cascade-layer order across all author sheets: first
/// occurrence of a dotted layer name wins, so `@layer a` in a later sheet merges
/// into the `a` established by an earlier sheet instead of restarting at rank 0.
/// Unlayered rules are not listed (they always outrank layered normal rules).
pub fn resolved_layer_order(sheets: &[Stylesheet]) -> Vec<String> {
    let mut order: Vec<String> = Vec::new();
    for sheet in sheets {
        // A sheet's per-sheet ranks establish statement-form order; sort by
        // them before folding names into the global list.
        let mut by_rank: BTreeMap<u32, &str> = BTreeMap::new();
        for rule in &sheet.rules {
            if let Some(name) = cascade_layer_name(rule) {
                // Per-sheet rank lives in the companion marker.
                let rank = rule
                    .declarations
                    .iter()
                    .find(|d| d.property == LAYER_MARKER)
                    .and_then(|d| d.value.parse().ok())
                    .unwrap_or(u32::MAX);
                by_rank.entry(rank).or_insert(name);
            }
        }
        for (_, name) in by_rank {
            if !order.iter().any(|existing| existing == name) {
                order.push(name.to_string());
            }
        }
    }
    order
}

#[cfg(test)]
fn compute_styles_inner(dom: &Dom, sheets: &[Stylesheet], ua_css: &str) -> StyleMap {
    let ua = parse_stylesheet(ua_css);
    compute_styles_parsed(dom, sheets, &ua, true, &HashSet::new())
}

/// Same resolution as `compute_styles`, but with style sharing turned off.
/// Used by the equality tests and the in-crate A/B harness.
#[cfg(test)]
fn compute_styles_uncached(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    let ua = UA_SHEET.get_or_init(|| parse_stylesheet(ua_stylesheet()));
    compute_styles_parsed(dom, sheets, ua, false, &HashSet::new())
}

fn compute_styles_parsed(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
    visited: &HashSet<String>,
) -> StyleMap {
    compute_styles_tracked(dom, sheets, ua, use_cache, visited).0
}

/// One matched-rule-id list per arena slot: candidate indices into the ordered
/// rule slice, ascending (the rule index already sorts + dedups candidates).
/// Results are indexed by slot, never by hash iteration order.
type MatchedLists = Vec<Vec<usize>>;

/// Upper bound on matching threads, and the element count below which matching
/// stays on the calling thread (the demo has ~33 elements).
const MATCH_MAX_THREADS: usize = 8;
const MATCH_MIN_ELEMENTS: usize = 512;

fn match_thread_count(element_count: usize) -> usize {
    if element_count < MATCH_MIN_ELEMENTS {
        return 1;
    }
    std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .clamp(1, MATCH_MAX_THREADS)
}

/// Match one element against all its candidate rules. Pure w.r.t. `dom`: each
/// caller owns its `MatchContext` (and hence its `SiblingCache`) and candidate
/// buffer. The sibling index behind `:nth-child` / `:first-child` and friends
/// is computed in `hodo_css::SiblingCache::entry` solely from `dom.parent(id)`
/// and `dom.children(parent)` — per-element-local reads of an immutable `Dom`,
/// with no cross-element state. The ancestor Bloom is passed in precomputed,
/// so every chunk is independent.
#[allow(clippy::too_many_arguments)]
fn match_one(
    dom: &Dom,
    ordered: &[OrderedRule<'_>],
    index: &RuleIndex,
    use_index: bool,
    bloom: AncestorBloom,
    id: NodeId,
    candidates: &mut Vec<usize>,
    ctx: &mut MatchContext,
) -> Vec<usize> {
    ctx.ancestors = bloom;
    if use_index {
        index.candidates(dom, id, candidates);
    } else {
        // `HODO_NO_RULE_INDEX=1`: every rule is a candidate, matched through
        // the same generic entry the A/B corpus uses as its brute-force oracle.
        candidates.clear();
        candidates.extend(0..ordered.len());
    }
    let mut matched = Vec::new();
    for &candidate in candidates.iter() {
        let Some(&(_, _, rule, _)) = ordered.get(candidate) else {
            continue;
        };
        // Fast path: plain compounds are plain tag/#id/.class tests with no
        // allocations and no ancestor-Bloom / sibling-cache consult. Non-plain
        // rules reuse the chain decoded once at index-build time instead of
        // re-decoding the CHAIN_PREFIX payload for every element.
        let is_match = if !use_index {
            matches_selector_ref(&rule.selector, dom, id, &mut *ctx)
        } else {
            match index.chain(candidate) {
                Some(chain) => matches_compiled(chain, dom, id, &mut *ctx),
                None => matches_plain(&rule.selector, dom, id),
            }
        };
        if is_match {
            matched.push(candidate);
        }
    }
    matched
}

// Test-only observation of the thread count chosen by the last `match_all`.
// Thread-local so concurrently running tests cannot overwrite each other.
#[cfg(test)]
thread_local! {
    static LAST_MATCH_THREADS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Phase A of style resolution: compute ancestor Blooms and the matched rule
/// set for every element.
///
/// `hodo_dom::Dom`'s accessors (`get`, `parent`, `children`, `tag`, `attr`,
/// `text`, `walk`) all take `&self`; the arena holds only `Vec`/`String`/
/// `HashMap` with no `Rc`/`RefCell` or other interior mutability. So `&Dom`,
/// `&RuleIndex` and `&[OrderedRule]` are `Sync` and are shared by the scoped
/// threads below without locks. Results are written by arena slot into
/// disjoint `Vec` slices: no shared mutable state, deterministic output
/// regardless of thread scheduling.
///
/// Blooms are built here in pre-order before the parallel phase (a parent's
/// Bloom depends only on the immutable tree, never on computed styles), which
/// removes the last sequential dependency from matching. `threads == 0` picks
/// the policy automatically; tests pin it to compare byte-for-byte.
fn match_all(
    dom: &Dom,
    ordered: &[OrderedRule<'_>],
    index: &RuleIndex,
    use_index: bool,
    needs_bloom: bool,
    threads: usize,
    visited: &HashSet<String>,
) -> MatchedLists {
    let mut blooms = vec![AncestorBloom::default(); dom.len()];
    let mut elements: Vec<NodeId> = Vec::new();
    dom.walk(|id, node| {
        if !matches!(node.kind, NodeKind::Element { .. }) {
            return;
        }
        if needs_bloom {
            if let Some(parent) = dom.parent(id) {
                blooms[id.slot() as usize] =
                    AncestorBloom::child_of(blooms[parent.slot() as usize], dom, parent);
            }
        }
        elements.push(id);
    });

    let threads = if threads == 0 {
        match_thread_count(elements.len())
    } else {
        threads
    };
    let mut matched: MatchedLists = vec![Vec::new(); dom.len()];
    #[cfg(test)]
    LAST_MATCH_THREADS.with(|count| count.set(threads));

    if threads <= 1 {
        let mut candidates = Vec::new();
        let mut ctx = MatchContext::with_visited(visited.iter().cloned());
        for &id in &elements {
            let slot = id.slot() as usize;
            matched[slot] = match_one(
                dom,
                ordered,
                index,
                use_index,
                blooms[slot],
                id,
                &mut candidates,
                &mut ctx,
            );
        }
        return matched;
    }

    let chunk_size = dom.len().div_ceil(threads);
    let mut by_thread: Vec<Vec<NodeId>> = (0..threads).map(|_| Vec::new()).collect();
    for id in elements {
        by_thread[id.slot() as usize / chunk_size].push(id);
    }
    let blooms = &blooms;
    std::thread::scope(|scope| {
        for (thread, slice) in matched.chunks_mut(chunk_size).enumerate() {
            let ids = &by_thread[thread];
            scope.spawn(move || {
                let base = thread * chunk_size;
                let mut candidates = Vec::new();
                let mut ctx = MatchContext::with_visited(visited.iter().cloned());
                for &id in ids {
                    let slot = id.slot() as usize;
                    slice[slot - base] = match_one(
                        dom,
                        ordered,
                        index,
                        use_index,
                        blooms[slot],
                        id,
                        &mut candidates,
                        &mut ctx,
                    );
                }
            });
        }
    });
    matched
}

fn compute_styles_tracked(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
    visited: &HashSet<String>,
) -> (StyleMap, SharingStats) {
    compute_styles_tracked_with(dom, sheets, ua, use_cache, 0, visited)
}

/// `threads == 0` selects the policy automatically; tests force a value to
/// compare single-threaded and parallel matching byte-for-byte.
fn compute_styles_tracked_with(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
    threads: usize,
    visited: &HashSet<String>,
) -> (StyleMap, SharingStats) {
    // Bench honesty switch: `HODO_NO_RULE_INDEX=1` makes the public entry walk
    // every rule, matching through the same brute-force path the A/B corpus
    // uses as its reference. Read once per pass, never per element.
    let use_index = !std::env::var_os("HODO_NO_RULE_INDEX").is_some_and(|v| v == "1");
    compute_styles_tracked_with_index(dom, sheets, ua, use_cache, threads, visited, use_index)
}

fn compute_styles_tracked_with_index(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
    threads: usize,
    visited: &HashSet<String>,
    use_index: bool,
) -> (StyleMap, SharingStats) {
    // (origin, source order, rule, layer); origin 0 = UA, 1 = author.
    let mut ordered: Vec<OrderedRule<'_>> = Vec::new();
    let mut order = 0;
    for rule in &ua.rules {
        ordered.push((0, order, rule, u32::MAX));
        order += 1;
    }
    // Resolve one GLOBAL layer order across all sheets before ranking rules.
    // The per-sheet rank only restores statement-form order inside a sheet;
    // the dotted name decides the global rank, so a repeated name merges.
    let global = resolved_layer_order(sheets);
    for sheet in sheets {
        for rule in &sheet.rules {
            let layer = cascade_layer_name(rule)
                .and_then(|name| global.iter().position(|n| n == name))
                .map_or(u32::MAX, |index| index as u32);
            ordered.push((1, order, rule, layer));
            order += 1;
        }
    }
    let index = RuleIndex::new(ordered.iter().map(|(_, _, rule, _)| *rule));
    let needs_bloom = ordered
        .iter()
        .any(|(_, _, rule, _)| !is_compound(&rule.selector));
    // Split declarations once per style pass, not once per element × rule.
    let prepared: Vec<(Vec<&Declaration>, Vec<&Declaration>)> = ordered
        .iter()
        .map(|(_, _, rule, _)| {
            let mut normal = Vec::new();
            let mut important = Vec::new();
            for declaration in &rule.declarations {
                if declaration.property == LAYER_MARKER || declaration.property == LAYER_NAME_MARKER
                {
                    continue;
                }
                if is_important(declaration) {
                    important.push(declaration);
                } else {
                    normal.push(declaration);
                }
            }
            (normal, important)
        })
        .collect();

    // Specificity comes from the chain decode already done while building the
    // index; the cascade only reads this slice, so no element re-decodes a
    // selector (the bench's brute-force path is the sole exception).
    let specificity: Vec<(u32, u32, u32)> = ordered
        .iter()
        .enumerate()
        .map(|(i, (_, _, rule, _))| index.specificity(i, &rule.selector))
        .collect();

    // Phase A — matching only: reads immutable DOM + sheets, writes one result
    // list per element slot. This is the part that runs in parallel.
    let mut matched = match_all(
        dom,
        &ordered,
        &index,
        use_index,
        needs_bloom,
        threads,
        visited,
    );

    // Phase B — cascade + style sharing stay sequential: they consume the
    // precomputed matched lists and share one mutable LRU / parent context.
    let mut map = StyleMap::with_len(dom.len());
    let mut rules = CascadeRules {
        ordered: &ordered,
        prepared: &prepared,
        specificity: &specificity,
    };
    // Conservative sheet-level bail-out: if any rule keys off sibling
    // position, two elements with the same class set can match different
    // rules, so sharing is disabled for the whole document.
    let sharing_allowed = use_cache && !sheet_has_positional_pseudo(&ordered);
    let mut cache = if sharing_allowed {
        Some(SharingCache::new())
    } else {
        None
    };
    let mut stats = SharingStats::default();
    // Resolved custom properties ride alongside computed styles: they inherit
    // and must not live in `ComputedStyle` (other crates construct it directly).
    let mut custom: Vec<BTreeMap<String, String>> = vec![BTreeMap::new(); dom.len()];
    // Pre-order guarantees a node's parent is already resolved when it is visited.
    dom.walk(|id, node| {
        let parent_custom = dom
            .parent(id)
            .map(|p| custom[p.slot() as usize].clone())
            .unwrap_or_default();
        let parent = dom
            .parent(id)
            .and_then(|p| map.get(p).cloned())
            .unwrap_or_else(initial_style);
        match node.kind {
            NodeKind::Text(_) => {
                custom[id.slot() as usize] = parent_custom;
                map.set(id, parent);
            }
            NodeKind::Element { .. } => {
                let (style, vars, deferred) = cascade(
                    dom,
                    id,
                    &mut rules,
                    &mut matched[id.slot() as usize],
                    &parent,
                    &parent_custom,
                    cache.as_mut(),
                    &mut stats,
                );
                custom[id.slot() as usize] = vars;
                map.set_deferred_width(id, deferred);
                map.set(id, style);
            }
            NodeKind::Document => {}
        }
    });
    (map, stats)
}

/// Cascade rank. Important author declarations outrank normal inline style,
/// while important UA declarations outrank important author declarations.
fn rank(origin: u8, important: bool) -> u8 {
    match (origin, important) {
        (0, false) => 0,
        (1, false) => 1,
        (2, false) => 2,
        (1, true) => 3,
        (2, true) => 4,
        (0, true) => 5,
        _ => 1,
    }
}

/// Layer rank inside one origin. Normal: unlayered wins, later layers beat
/// earlier ones. `!important`: the order is inverted, unlayered is weakest.
fn layer_key(layer: u32, important: bool) -> u32 {
    if layer == u32::MAX {
        if important {
            0
        } else {
            u32::MAX
        }
    } else if important {
        u32::MAX - layer
    } else {
        layer
    }
}

fn initial_style() -> ComputedStyle {
    ComputedStyle {
        display: Display::Inline,
        color: Color::BLACK,
        background: Color::TRANSPARENT,
        font_size: 16.0,
        font_weight: 400,
        font_family: String::new(),
        italic: false,
        underline: false,
        margin: Edges::zero(),
        padding: Edges::zero(),
        border_width: 0.0,
        border_widths: Edges::zero(),
        border_color: Color::BLACK,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        align_self: AlignSelf::Auto,
        gap: 0.0,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: FlexBasis::Auto,
        order: 0,
        grid_template_columns: vec![],
        grid_template_rows: vec![],
        grid_column: (GridLine::Auto, GridLine::Auto),
        grid_row: (GridLine::Auto, GridLine::Auto),
        column_gap: 0.0,
        row_gap: 0.0,
        table_layout: TableLayout::Auto,
        border_collapse: BorderCollapse::Separate,
        border_spacing: (2.0, 2.0),
        line_height: LineHeight::Normal,
        vertical_align: VerticalAlign::Baseline,
        text_align: TextAlign::Left,
        text_indent: 0.0,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        white_space: WhiteSpace::Normal,
        overflow_wrap: OverflowWrap::Normal,
        line_through: false,
        position: Position::Static,
        float: Float::None,
        clear: Clear::None,
        overflow: Overflow::Visible,
        offsets: Edges::zero(),
        width: None,
        width_percent: None,
        margin_percent: Edges::zero(),
        padding_percent: Edges::zero(),
        height: None,
        min_size: Size { w: 0.0, h: 0.0 },
        max_size: Size {
            w: f32::INFINITY,
            h: f32::INFINITY,
        },
    }
}

/// Per-pass rule data shared by every element's cascade (phase B only).
struct CascadeRules<'a> {
    ordered: &'a [OrderedRule<'a>],
    prepared: &'a [(Vec<&'a Declaration>, Vec<&'a Declaration>)],
    /// One specificity per rule, decoded once when the index was built.
    specificity: &'a [(u32, u32, u32)],
}

/// Maximum number of post-inheritance styles kept in one style pass.
const SHARING_CACHE_CAP: usize = 256;

/// Identity of a shareable cascade result. Components are independent 64-bit
/// SipHash digests, so a false hit requires all three to collide.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SharingKey {
    /// The element's tag (UA rules and tag selectors are part of the matched
    /// set, but carrying the tag keeps the key conservative and cheap).
    tag: u64,
    /// Sorted rule indices that actually matched this element.
    matched: u64,
    /// Hash of the parent's *inherited* computed values only.
    parent: u64,
}

#[derive(Debug, Default, Clone, Copy)]
struct SharingStats {
    hits: usize,
    misses: usize,
    inserted: usize,
    /// Elements that did not participate in sharing (inline `style`, or
    /// sharing disabled for the sheet / the A/B harness).
    excluded: usize,
}

fn hash_u64<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Hash only the properties the cascade copies from the parent. Two children
/// with the same inherited values and the same matched rule set produce a
/// byte-for-byte identical `ComputedStyle` (assuming no inline declarations).
fn hash_inherited(style: &ComputedStyle, custom: &BTreeMap<String, String>) -> u64 {
    let mut h = DefaultHasher::new();
    // Custom properties inherit too: omitting them would let two elements with
    // different --tokens reuse one computed style.
    custom.hash(&mut h);
    (style.color.r, style.color.g, style.color.b, style.color.a).hash(&mut h);
    style.font_size.to_bits().hash(&mut h);
    style.font_weight.hash(&mut h);
    style.font_family.hash(&mut h);
    style.italic.hash(&mut h);
    style.underline.hash(&mut h);
    std::mem::discriminant(&style.line_height).hash(&mut h);
    match style.line_height {
        LineHeight::Number(n) => n.to_bits().hash(&mut h),
        LineHeight::Length(v) => v.to_bits().hash(&mut h),
        LineHeight::Normal => {}
    }
    style.text_indent.to_bits().hash(&mut h);
    style.letter_spacing.to_bits().hash(&mut h);
    style.word_spacing.to_bits().hash(&mut h);
    std::mem::discriminant(&style.white_space).hash(&mut h);
    std::mem::discriminant(&style.vertical_align).hash(&mut h);
    if let VerticalAlign::Length(v) = style.vertical_align {
        v.to_bits().hash(&mut h);
    }
    std::mem::discriminant(&style.text_align).hash(&mut h);
    std::mem::discriminant(&style.border_collapse).hash(&mut h);
    style.border_spacing.0.to_bits().hash(&mut h);
    style.border_spacing.1.to_bits().hash(&mut h);
    h.finish()
}

type SharingEntry = (u64, ComputedStyle, BTreeMap<String, String>, Option<String>);

/// Pure-std LRU: O(1) hits, O(capacity) eviction. `clock` gives each entry a
/// monotonically increasing stamp; the smallest stamp is least recently used.
struct SharingCache {
    entries: HashMap<SharingKey, SharingEntry>,
    clock: u64,
}

impl SharingCache {
    fn new() -> Self {
        Self {
            entries: HashMap::with_capacity(SHARING_CACHE_CAP),
            clock: 0,
        }
    }

    fn get(
        &mut self,
        key: &SharingKey,
    ) -> Option<(ComputedStyle, BTreeMap<String, String>, Option<String>)> {
        self.clock += 1;
        let stamp = self.clock;
        let entry = self.entries.get_mut(key)?;
        entry.0 = stamp;
        Some((entry.1.clone(), entry.2.clone(), entry.3.clone()))
    }

    fn insert(
        &mut self,
        key: SharingKey,
        style: &ComputedStyle,
        custom: &BTreeMap<String, String>,
        deferred_width: &Option<String>,
    ) {
        self.clock += 1;
        let stamp = self.clock;
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.0 = stamp;
            return;
        }
        if self.entries.len() >= SHARING_CACHE_CAP {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, (s, _, _, _))| *s)
                .map(|(key, _)| *key);
            if let Some(oldest) = oldest {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            key,
            (stamp, style.clone(), custom.clone(), deferred_width.clone()),
        );
    }
}

fn compound_has_positional(compound: &Compound) -> bool {
    compound.pseudos.iter().any(|pseudo| match pseudo {
        PseudoClass::FirstChild
        | PseudoClass::LastChild
        | PseudoClass::OnlyChild
        | PseudoClass::NthChild(_, _) => true,
        PseudoClass::Not(inner) => compound_has_positional(inner),
        PseudoClass::Is(args) | PseudoClass::Where(args) => {
            args.iter().any(compound_has_positional)
        }
        _ => false,
    })
}

/// Conservative document-wide guard: sibling-position selectors make the set
/// of matched rules depend on where an element sits, not just on its tag and
/// class. With the matched-rule-set key this is strictly redundant, but the
/// guard keeps the cache sound if the key is ever coarsened toward Servo's
/// class-set-only both-hares trick.
fn sheet_has_positional_pseudo(ordered: &[OrderedRule<'_>]) -> bool {
    ordered.iter().any(|(_, _, rule, _)| {
        complex(&rule.selector)
            .compounds
            .iter()
            .any(compound_has_positional)
    })
}

#[allow(clippy::too_many_arguments)]
fn cascade(
    dom: &Dom,
    id: NodeId,
    rules: &mut CascadeRules<'_>,
    matched_ids: &mut Vec<usize>,
    parent: &ComputedStyle,
    parent_custom: &BTreeMap<String, String>,
    mut cache: Option<&mut SharingCache>,
    stats: &mut SharingStats,
) -> (ComputedStyle, BTreeMap<String, String>, Option<String>) {
    let mut normal: Vec<Match<'_>> = Vec::new();
    let mut important: Vec<Match<'_>> = Vec::new();
    // Inline declarations are per-element and are never shared.
    let can_share = cache.is_some() && dom.attr(id, "style").is_none();
    // Phase A already matched every candidate; only rule-order-independent
    // bookkeeping remains here (no re-matching, no context/sibling scans).
    for &candidate in matched_ids.iter() {
        let (origin, order, _rule, layer) = rules.ordered[candidate];
        let specificity = rules.specificity[candidate];
        if !rules.prepared[candidate].0.is_empty() {
            normal.push((
                rank(origin, false),
                layer_key(layer, false),
                specificity,
                order,
                &rules.prepared[candidate].0,
            ));
        }
        if !rules.prepared[candidate].1.is_empty() {
            important.push((
                rank(origin, true),
                layer_key(layer, true),
                specificity,
                order,
                &rules.prepared[candidate].1,
            ));
        }
    }
    // Inline style is an author-origin layer above author sheets.
    let inline = dom.attr(id, "style").map(parse_style_attribute);
    let mut inline_normal: Vec<&Declaration> = Vec::new();
    let mut inline_important: Vec<&Declaration> = Vec::new();
    if let Some(declarations) = &inline {
        for declaration in declarations {
            if is_important(declaration) {
                inline_important.push(declaration);
            } else {
                inline_normal.push(declaration);
            }
        }
        if !inline_normal.is_empty() {
            normal.push((
                rank(2, false),
                layer_key(u32::MAX, false),
                (0, 0, 0),
                usize::MAX / 2,
                &inline_normal,
            ));
        }
        if !inline_important.is_empty() {
            important.push((
                rank(2, true),
                layer_key(u32::MAX, true),
                (0, 0, 0),
                usize::MAX / 2,
                &inline_important,
            ));
        }
    }
    // Cache lookup. On a hit the rest of this function would produce an
    // identical style: the matched-rule-id set fixes the whole
    // (origin, specificity, source order, declaration) multiset; the parent
    // hash fixes every inherited value; inline declarations are excluded;
    // and everything below is a deterministic function of those inputs.
    // `!important` is just one more declaration in the multiset, so origin /
    // specificity / `!important` resolution is identical on a hit.
    let shared_key = if can_share {
        matched_ids.sort_unstable();
        let key = SharingKey {
            tag: hash_u64(&dom.tag(id).unwrap_or("")),
            matched: hash_u64(&*matched_ids),
            parent: hash_inherited(parent, parent_custom),
        };
        match cache.as_mut().unwrap().get(&key) {
            Some(hit) => {
                stats.hits += 1;
                return hit;
            }
            None => {
                stats.misses += 1;
                Some(key)
            }
        }
    } else {
        stats.excluded += 1;
        None
    };
    // Stable sort: ties keep source order. Larger rank/layer/specificity/order win last.
    normal.sort_by_key(|m| (m.0, m.1, m.2, m.3));
    important.sort_by_key(|m| (m.0, m.1, m.2, m.3));

    // First pass: the winning raw custom-property values. They inherit from the
    // parent and may reference each other, so resolve them as a set before any
    // ordinary declaration is substituted.
    let mut raw_custom = parent_custom.clone();
    for m in normal.iter().chain(&important) {
        for decl in m.4 {
            let property = decl.property.trim();
            if property.starts_with("--") {
                raw_custom.insert(property.to_string(), declaration_value(decl).to_string());
            }
        }
    }
    let resolved_custom = resolve_custom_properties(&raw_custom);

    // Last declaration for each property wins after the cascade sort. Edge
    // shorthands are expanded to their four longhands as we walk in cascade
    // order, so a later longhand (or a later shorthand) wins by source order.
    // `var()` is substituted on the raw value *before* shorthand parsing; an
    // unresolved reference makes that declaration invalid and drops it.
    let mut winning: HashMap<String, String> = HashMap::new();
    for m in normal.iter().chain(&important) {
        for decl in m.4 {
            let trimmed = decl.property.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            let property = trimmed.to_ascii_lowercase();
            let Some(substituted) = substitute_vars(declaration_value(decl), &resolved_custom)
            else {
                continue;
            };
            // Parse-time calc resolution runs before `var()` substitution, so
            // var-containing calcs were kept raw. Resolve them now that the
            // variables are concrete; `%` calcs stay raw for layout.
            let Some(value) = resolve_calc_now(&substituted) else {
                continue;
            };
            if property == "border" {
                expand_border_shorthand(&value, &mut winning);
                continue;
            }
            if property == "flex" {
                expand_flex_shorthand(&value, &mut winning);
                continue;
            }
            let expanded = edge_longhand_keys(&property)
                .and_then(|keys| edge_shorthand_parts(&value).map(|parts| (keys, parts)));
            if let Some((keys, parts)) = expanded {
                for (key, part) in keys.into_iter().zip(parts) {
                    winning.insert(key.to_string(), part.to_string());
                }
            } else {
                winning.insert(property, value);
            }
        }
    }

    // A `%`-containing `calc()` has no parse-time basis; keep its raw text so
    // layout can resolve it against the containing block (mirroring how plain
    // `width: N%` flows through `width_percent`).
    let deferred_width = winning
        .get("width")
        .filter(|value| value.contains("calc(") && value.contains('%'))
        .cloned();

    let mut style = initial_style();
    // Inherited properties start from the parent's computed values.
    style.color = parent.color;
    style.font_size = parent.font_size;
    style.font_weight = parent.font_weight;
    style.font_family = parent.font_family.clone();
    style.italic = parent.italic;
    style.underline = parent.underline;
    // A Number line-height stays a number; a resolved Length inherits as-is.
    style.line_height = parent.line_height;
    style.text_indent = parent.text_indent;
    style.letter_spacing = parent.letter_spacing;
    style.word_spacing = parent.word_spacing;
    style.white_space = parent.white_space;
    style.vertical_align = parent.vertical_align;
    style.text_align = parent.text_align;
    style.border_collapse = parent.border_collapse;
    style.border_spacing = parent.border_spacing;

    if let Some(v) = winning.get("display") {
        style.display = match v.to_ascii_lowercase().as_str() {
            "block" => Display::Block,
            "none" => Display::None,
            "inline-block" => Display::InlineBlock,
            "flex" => Display::Flex,
            "inline-flex" => Display::InlineFlex,
            "grid" => Display::Grid,
            "inline-grid" => Display::InlineGrid,
            "table" => Display::Table,
            "inline-table" => Display::InlineTable,
            "table-row-group" => Display::TableRowGroup,
            "table-row" => Display::TableRow,
            "table-cell" => Display::TableCell,
            "table-caption" => Display::TableCaption,
            _ => Display::Inline,
        };
    }
    if let Some(v) = winning.get("color").and_then(|v| parse_color(v)) {
        style.color = v;
    }
    if let Some(v) = winning
        .get("background-color")
        .or_else(|| winning.get("background"))
    {
        if let Some(c) = parse_color(v) {
            style.background = c;
        }
    }
    if let Some(v) = winning.get("font-size") {
        style.font_size = parse_font_size(v, parent.font_size);
    }
    if let Some(v) = winning.get("font-weight") {
        style.font_weight = match v.to_ascii_lowercase().as_str() {
            "bold" => 700,
            "normal" => 400,
            other => other.parse().unwrap_or(style.font_weight),
        };
    }
    if let Some(v) = winning.get("font-family") {
        if let Some(stack) = parse_font_family(v) {
            style.font_family = stack;
        }
    }
    if let Some(v) = winning.get("font-style") {
        style.italic = v.to_ascii_lowercase().contains("italic");
    }
    if let Some(v) = winning.get("text-decoration") {
        let lower = v.to_ascii_lowercase();
        style.underline = lower.contains("underline");
        style.line_through = lower.contains("line-through");
    }

    // Shorthands feeding two fields are resolved first so the single pass can
    // still let later longhands win.
    if let Some(v) = winning.get("gap").or_else(|| winning.get("grid-gap")) {
        let mut parts = v.split_whitespace();
        if let Some(col) = parts.next().and_then(|p| parse_length(p, style.font_size)) {
            let row = parts
                .next()
                .and_then(|p| parse_length(p, style.font_size))
                .unwrap_or(col);
            style.gap = col;
            style.column_gap = col;
            style.row_gap = row;
        }
    }
    if let Some(v) = winning.get("grid-column") {
        if let Some(placement) = parse_grid_placement(v) {
            style.grid_column = placement;
        }
    }
    if let Some(v) = winning.get("grid-row") {
        if let Some(placement) = parse_grid_placement(v) {
            style.grid_row = placement;
        }
    }

    // One hash lookup per declared property rather than one per property the
    // engine knows about.
    macro_rules! set {
        ($field:expr, $parsed:expr) => {
            if let Some(value) = $parsed {
                $field = value;
            }
        };
    }
    for (property, value) in &winning {
        match property.as_str() {
            "flex-direction" => set!(style.flex_direction, parse_flex_direction(value)),
            "flex-wrap" => set!(style.flex_wrap, parse_flex_wrap(value)),
            "justify-content" => set!(style.justify_content, parse_justify_content(value)),
            "align-items" => set!(style.align_items, parse_align_items(value)),
            "align-self" => set!(style.align_self, parse_align_self(value)),
            "flex-grow" => set!(style.flex_grow, parse_f32(value)),
            "flex-shrink" => set!(style.flex_shrink, parse_f32(value)),
            "flex-basis" => set!(style.flex_basis, parse_flex_basis(value)),
            "order" => set!(style.order, parse_i32(value)),
            "column-gap" => set!(style.column_gap, parse_length(value, style.font_size)),
            "row-gap" => set!(style.row_gap, parse_length(value, style.font_size)),
            "grid-template-columns" => {
                style.grid_template_columns = parse_track_list(value);
            }
            "grid-template-rows" => {
                style.grid_template_rows = parse_track_list(value);
            }
            "grid-column-start" => set!(style.grid_column.0, parse_grid_line(value)),
            "grid-column-end" => set!(style.grid_column.1, parse_grid_line(value)),
            "grid-row-start" => set!(style.grid_row.0, parse_grid_line(value)),
            "grid-row-end" => set!(style.grid_row.1, parse_grid_line(value)),
            "grid-auto-flow" => {
                if !value.eq_ignore_ascii_case("row") {
                    eprintln!(
                        "hodo-style: grid-auto-flow '{}' parsed but ignored (row only)",
                        value
                    );
                }
            }
            "table-layout" => set!(style.table_layout, parse_table_layout(value)),
            "border-collapse" => set!(style.border_collapse, parse_border_collapse(value)),
            "border-spacing" => {
                let mut parts = value.split_whitespace();
                if let Some(h) = parts.next().and_then(|p| parse_length(p, style.font_size)) {
                    let v = parts
                        .next()
                        .and_then(|p| parse_length(p, style.font_size))
                        .unwrap_or(h);
                    style.border_spacing = (h, v);
                }
            }
            "line-height" => set!(style.line_height, parse_line_height(value, style.font_size)),
            "vertical-align" => {
                set!(
                    style.vertical_align,
                    parse_vertical_align(value, style.font_size)
                )
            }
            "text-align" => set!(style.text_align, parse_text_align(value)),
            "text-indent" => set!(style.text_indent, parse_length(value, style.font_size)),
            "letter-spacing" => set!(style.letter_spacing, parse_length(value, style.font_size)),
            "word-spacing" => set!(style.word_spacing, parse_length(value, style.font_size)),
            "white-space" => set!(style.white_space, parse_white_space(value)),
            "overflow-wrap" => set!(style.overflow_wrap, parse_overflow_wrap(value)),
            "position" => set!(style.position, parse_position(value)),
            "float" => set!(style.float, parse_float(value)),
            "clear" => set!(style.clear, parse_clear(value)),
            "overflow" => set!(style.overflow, parse_overflow(value)),
            "top" => set!(
                style.offsets.top,
                parse_optional_length(value, style.font_size)
            ),
            "right" => set!(
                style.offsets.right,
                parse_optional_length(value, style.font_size)
            ),
            "bottom" => set!(
                style.offsets.bottom,
                parse_optional_length(value, style.font_size)
            ),
            "left" => set!(
                style.offsets.left,
                parse_optional_length(value, style.font_size)
            ),
            "width" => {
                // A percentage has no parse-time basis: keep the raw number for
                // layout to resolve against the containing block's content box.
                let (abs, pct) = parse_length_or_percent(value, style.font_size);
                // Negative width/height are invalid in CSS; clamp here so a
                // `calc(0px - 10px)` cannot produce a negative box width.
                style.width = abs.map(|w| w.max(0.0));
                style.width_percent = pct.map(|p| p.max(0.0));
            }
            "height" => set!(
                style.height,
                parse_optional_length(value, style.font_size).map(|h| h.map(|h| h.max(0.0)))
            ),
            "min-width" => set!(style.min_size.w, parse_length(value, style.font_size)),
            "min-height" => set!(style.min_size.h, parse_length(value, style.font_size)),
            "max-width" => set!(style.max_size.w, parse_length(value, style.font_size)),
            "max-height" => set!(style.max_size.h, parse_length(value, style.font_size)),
            _ => {}
        }
    }

    // Per-side lengths resolve against the element's own (now final) font size.
    apply_edges(
        &mut style.margin,
        Some(&mut style.margin_percent),
        &winning,
        MARGIN_KEYS,
        style.font_size,
    );
    apply_edges(
        &mut style.padding,
        Some(&mut style.padding_percent),
        &winning,
        PADDING_KEYS,
        style.font_size,
    );
    apply_edges(
        &mut style.border_widths,
        None,
        &winning,
        BORDER_WIDTH_KEYS,
        style.font_size,
    );
    // `border_width` is the single value layout uses; keep it as the widest side.
    let b = style.border_widths;
    style.border_width = b.top.max(b.right).max(b.bottom).max(b.left);
    if let Some(v) = winning.get("border-color").and_then(|v| parse_color(v)) {
        style.border_color = v;
    }
    if let Some(key) = shared_key {
        cache
            .as_mut()
            .unwrap()
            .insert(key, &style, &resolved_custom, &deferred_width);
        stats.inserted += 1;
    }
    (style, resolved_custom, deferred_width)
}

fn parse_f32(value: &str) -> Option<f32> {
    value.trim().parse::<f32>().ok().filter(|n| n.is_finite())
}

fn parse_i32(value: &str) -> Option<i32> {
    value.trim().parse::<i32>().ok()
}

fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
    match value.trim().to_ascii_lowercase().as_str() {
        "row" => Some(FlexDirection::Row),
        "row-reverse" => Some(FlexDirection::RowReverse),
        "column" => Some(FlexDirection::Column),
        "column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
    match value.trim().to_ascii_lowercase().as_str() {
        "nowrap" => Some(FlexWrap::NoWrap),
        "wrap" => Some(FlexWrap::Wrap),
        _ => None,
    }
}

fn parse_justify_content(value: &str) -> Option<JustifyContent> {
    match value.trim().to_ascii_lowercase().as_str() {
        "flex-start" => Some(JustifyContent::FlexStart),
        "center" => Some(JustifyContent::Center),
        "flex-end" => Some(JustifyContent::FlexEnd),
        "space-between" => Some(JustifyContent::SpaceBetween),
        "space-around" => Some(JustifyContent::SpaceAround),
        "space-evenly" => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

fn parse_align_items(value: &str) -> Option<AlignItems> {
    match value.trim().to_ascii_lowercase().as_str() {
        "stretch" => Some(AlignItems::Stretch),
        "center" => Some(AlignItems::Center),
        "flex-start" => Some(AlignItems::FlexStart),
        "flex-end" => Some(AlignItems::FlexEnd),
        "baseline" => Some(AlignItems::Baseline),
        _ => None,
    }
}

fn parse_align_self(value: &str) -> Option<AlignSelf> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(AlignSelf::Auto),
        "stretch" => Some(AlignSelf::Stretch),
        "center" => Some(AlignSelf::Center),
        "flex-start" => Some(AlignSelf::FlexStart),
        "flex-end" => Some(AlignSelf::FlexEnd),
        "baseline" => Some(AlignSelf::Baseline),
        _ => None,
    }
}

fn parse_flex_basis(value: &str) -> Option<FlexBasis> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        Some(FlexBasis::Auto)
    } else {
        parse_length(value, 1.0).map(FlexBasis::Length)
    }
}

fn parse_track_list(value: &str) -> Vec<GridTrack> {
    split_top_level(value)
        .into_iter()
        .filter_map(parse_track)
        .collect()
}

/// Split on whitespace outside parentheses so `minmax(a, b)` and
/// `repeat(n, a b)` stay single tokens.
fn split_top_level(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    parts.push(&value[s..i]);
                }
            }
            _ if start.is_none() => start = Some(i),
            _ => {}
        }
    }
    if let Some(s) = start {
        parts.push(&value[s..]);
    }
    parts
}

fn parse_track(value: &str) -> Option<GridTrack> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridTrack::Auto);
    }
    let lower = value.to_ascii_lowercase();
    if let Some(inner) = lower
        .strip_prefix("minmax(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let (a, b) = split_once_top_level(inner)?;
        return Some(GridTrack::MinMax(
            Box::new(parse_track(a)?),
            Box::new(parse_track(b)?),
        ));
    }
    if let Some(inner) = lower
        .strip_prefix("repeat(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let (n, tracks) = split_once_top_level(inner)?;
        let count = n.trim().parse::<u32>().ok()?;
        return Some(GridTrack::Repeat(count, parse_track_list(tracks)));
    }
    if let Some(n) = lower.strip_suffix("fr") {
        return parse_f32(n).map(GridTrack::Fr);
    }
    if let Some(n) = lower.strip_suffix('%') {
        return parse_f32(n).map(GridTrack::Percent);
    }
    parse_length(value, 1.0).map(GridTrack::Fixed)
}

/// Split at the first top-level comma, tracking parentheses.
fn split_once_top_level(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some((&value[..i], &value[i + 1..])),
            _ => {}
        }
    }
    None
}

fn parse_grid_line(value: &str) -> Option<GridLine> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridLine::Auto);
    }
    let lower = value.to_ascii_lowercase();
    if let Some(n) = lower.strip_prefix("span ") {
        return n.trim().parse::<u32>().ok().map(GridLine::Span);
    }
    value.parse::<i32>().ok().map(GridLine::Line)
}

fn parse_grid_placement(value: &str) -> Option<(GridLine, GridLine)> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let (start, end) = match value.split_once('/') {
        Some((a, b)) => (parse_grid_line(a)?, parse_grid_line(b)?),
        None => (parse_grid_line(value)?, GridLine::Auto),
    };
    Some((start, end))
}

fn parse_table_layout(value: &str) -> Option<TableLayout> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TableLayout::Auto),
        "fixed" => Some(TableLayout::Fixed),
        _ => None,
    }
}

fn parse_border_collapse(value: &str) -> Option<BorderCollapse> {
    match value.trim().to_ascii_lowercase().as_str() {
        "separate" => Some(BorderCollapse::Separate),
        "collapse" => Some(BorderCollapse::Collapse),
        _ => None,
    }
}

fn parse_line_height(value: &str, font_size: f32) -> Option<LineHeight> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some(LineHeight::Normal);
    }
    let lower = value.to_ascii_lowercase();
    if lower.ends_with('%') {
        let n = lower.strip_suffix('%')?.trim().parse::<f32>().ok()?;
        return n
            .is_finite()
            .then(|| LineHeight::Length(n / 100.0 * font_size));
    }
    if lower.ends_with("px") || lower.ends_with("em") || lower.ends_with("pt") {
        return parse_length(value, font_size).map(LineHeight::Length);
    }
    parse_f32(value).map(LineHeight::Number)
}

fn parse_vertical_align(value: &str, font_size: f32) -> Option<VerticalAlign> {
    let value = value.trim();
    match value.to_ascii_lowercase().as_str() {
        "baseline" => Some(VerticalAlign::Baseline),
        "sub" => Some(VerticalAlign::Sub),
        "super" => Some(VerticalAlign::Super),
        "top" => Some(VerticalAlign::Top),
        "middle" => Some(VerticalAlign::Middle),
        "bottom" => Some(VerticalAlign::Bottom),
        _ => parse_length_pct(value, font_size, 1.0).map(VerticalAlign::Length),
    }
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" => Some(TextAlign::Left),
        "center" => Some(TextAlign::Center),
        "right" => Some(TextAlign::Right),
        _ => None,
    }
}

fn parse_white_space(value: &str) -> Option<WhiteSpace> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WhiteSpace::Normal),
        "pre" => Some(WhiteSpace::Pre),
        "nowrap" => Some(WhiteSpace::NoWrap),
        "pre-wrap" => Some(WhiteSpace::PreWrap),
        "pre-line" => Some(WhiteSpace::PreLine),
        _ => None,
    }
}

fn parse_overflow_wrap(value: &str) -> Option<OverflowWrap> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(OverflowWrap::Normal),
        "break-word" => Some(OverflowWrap::BreakWord),
        "anywhere" => Some(OverflowWrap::Anywhere),
        _ => None,
    }
}

fn parse_position(value: &str) -> Option<Position> {
    match value.trim().to_ascii_lowercase().as_str() {
        "static" => Some(Position::Static),
        "relative" => Some(Position::Relative),
        "absolute" => Some(Position::Absolute),
        "fixed" => Some(Position::Fixed),
        _ => None,
    }
}

fn parse_float(value: &str) -> Option<Float> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(Float::None),
        "left" => Some(Float::Left),
        "right" => Some(Float::Right),
        _ => None,
    }
}

fn parse_clear(value: &str) -> Option<Clear> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(Clear::None),
        "left" => Some(Clear::Left),
        "right" => Some(Clear::Right),
        "both" => Some(Clear::Both),
        _ => None,
    }
}

fn parse_overflow(value: &str) -> Option<Overflow> {
    match value.trim().to_ascii_lowercase().as_str() {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        _ => None,
    }
}

fn parse_optional_length(value: &str, font_size: f32) -> Option<Option<f32>> {
    if value.trim().eq_ignore_ascii_case("auto") {
        return Some(None);
    }
    parse_length(value, font_size).map(Some)
}

/// Split a declaration into an absolute length and a raw percentage. Exactly
/// one side is `Some` for a valid length/percentage value. Percentages carry no
/// basis here: layout resolves them against the containing block's content box.
fn parse_length_or_percent(value: &str, font_size: f32) -> (Option<f32>, Option<f32>) {
    let value = value.trim();
    if let Some(n) = value.strip_suffix('%') {
        return (None, n.trim().parse::<f32>().ok().filter(|n| n.is_finite()));
    }
    (parse_length(value, font_size), None)
}

/// Resolve a length or percentage into a single f32. `pct_scale` is 1.0 for
/// properties that keep the raw number, or font_size / 100 for line-height.
fn parse_length_pct(value: &str, font_size: f32, pct_scale: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(n) = value.strip_suffix('%') {
        let n = n.trim().parse::<f32>().ok()?;
        return n.is_finite().then_some(n * pct_scale);
    }
    parse_length(value, font_size)
}

fn expand_flex_shorthand(value: &str, winning: &mut HashMap<String, String>) {
    let mut parts = value.split_whitespace();
    let first = parts.next().unwrap_or("");
    let (grow, shrink, basis) = match first.to_ascii_lowercase().as_str() {
        "none" => ("0", "0", "auto"),
        "auto" => ("1", "1", "auto"),
        _ => (
            first,
            parts.next().unwrap_or("1"),
            parts.next().unwrap_or("0px"),
        ),
    };
    winning.insert("flex-grow".to_string(), grow.to_string());
    winning.insert("flex-shrink".to_string(), shrink.to_string());
    winning.insert("flex-basis".to_string(), basis.to_string());
}

fn parse_length(value: &str, font_size: f32) -> Option<f32> {
    let value = value.trim();
    let n = if let Some(n) = value.strip_suffix("px") {
        n.trim().parse::<f32>().ok()
    } else if let Some(n) = value.strip_suffix("em") {
        n.trim().parse::<f32>().ok().map(|n| n * font_size)
    } else if let Some(n) = value.strip_suffix("pt") {
        n.trim().parse::<f32>().ok().map(|n| n * 4.0 / 3.0)
    } else {
        value.parse::<f32>().ok()
    }?;
    n.is_finite().then_some(n)
}

fn parse_font_size(value: &str, parent_size: f32) -> f32 {
    if let Some(n) = value.trim().strip_suffix('%') {
        return n
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|n| n.is_finite())
            .map(|n| parent_size * n / 100.0)
            .unwrap_or(parent_size);
    }
    parse_length(value, parent_size).unwrap_or(parent_size)
}

/// Normalize a `font-family` value into the parsed stack. Quoted and unquoted
/// names and generic keywords are accepted; nothing switches fonts yet.
fn parse_font_family(value: &str) -> Option<String> {
    let mut families = Vec::new();
    for part in value.split(',') {
        let raw = part.trim();
        let family = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(raw)
            .trim();
        if family.is_empty() || family.chars().any(char::is_control) {
            return None;
        }
        families.push(family);
    }
    (!families.is_empty()).then(|| families.join(", "))
}

const MARGIN_KEYS: [&str; 4] = ["margin-top", "margin-right", "margin-bottom", "margin-left"];
const PADDING_KEYS: [&str; 4] = [
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
];
const BORDER_WIDTH_KEYS: [&str; 4] = [
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
];

fn edge_longhand_keys(property: &str) -> Option<[&'static str; 4]> {
    match property {
        "margin" => Some(MARGIN_KEYS),
        "padding" => Some(PADDING_KEYS),
        "border-width" => Some(BORDER_WIDTH_KEYS),
        _ => None,
    }
}

/// Expand `border: <width> <style> <color>` (tokens in any order, partial forms
/// allowed) into the same longhand keys the cascade already walks.
fn expand_border_shorthand(value: &str, winning: &mut HashMap<String, String>) {
    let mut width = None;
    let mut color = None;
    for token in value.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "none"
                | "hidden"
                | "solid"
                | "dashed"
                | "dotted"
                | "double"
                | "groove"
                | "ridge"
                | "inset"
                | "outset"
        ) {
            // shortcut: only solid borders rasterize, per-style patterns when a real border-style engine lands
            continue;
        }
        if parse_color(token).is_some() {
            color = Some(token);
        } else if parse_length(token, 1.0).is_some() {
            width = Some(token);
        }
    }
    if let Some(width) = width {
        for key in BORDER_WIDTH_KEYS {
            winning.insert(key.to_string(), width.to_string());
        }
    }
    if let Some(color) = color {
        winning.insert("border-color".to_string(), color.to_string());
    }
}

/// Split a 1–4 value edge shorthand into its per-side value strings.
fn edge_shorthand_parts(value: &str) -> Option<[&str; 4]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [a] => Some([*a, *a, *a, *a]),
        [a, b] => Some([*a, *b, *a, *b]),
        [a, b, c] => Some([*a, *b, *c, *b]),
        [a, b, c, d] => Some([*a, *b, *c, *d]),
        _ => None,
    }
}

fn apply_edges(
    target: &mut Edges<f32>,
    mut percent: Option<&mut Edges<Option<f32>>>,
    winning: &HashMap<String, String>,
    keys: [&str; 4],
    font_size: f32,
) {
    let mut side = |i: usize, current: f32| {
        let Some(value) = winning.get(keys[i]) else {
            return current;
        };
        let (abs, pct) = parse_length_or_percent(value, font_size);
        if let Some(p) = pct {
            if let Some(percent) = percent.as_deref_mut() {
                match i {
                    0 => percent.top = Some(p),
                    1 => percent.right = Some(p),
                    2 => percent.bottom = Some(p),
                    _ => percent.left = Some(p),
                }
            }
        }
        abs.unwrap_or(current)
    };
    let t = side(0, target.top);
    let r = side(1, target.right);
    let b = side(2, target.bottom);
    let l = side(3, target.left);
    *target = Edges {
        top: t,
        right: r,
        bottom: b,
        left: l,
    };
}

fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Some(Color::TRANSPARENT);
    }
    if value.starts_with('#') {
        return Color::from_css_hex(value);
    }
    let lower = value.to_ascii_lowercase();
    if lower.starts_with("rgba(") {
        return parse_rgba(&lower);
    }
    if lower.starts_with("rgb(") {
        return parse_rgb(&lower);
    }
    Color::from_css_name(value)
}

fn parse_channel(value: &str) -> Option<u8> {
    let n = value.trim().parse::<f32>().ok()?;
    Some(n.clamp(0.0, 255.0).round() as u8)
}

fn parse_rgb(value: &str) -> Option<Color> {
    let inner = value.strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut parts = inner.split(',');
    Some(Color::rgb(
        parse_channel(parts.next()?)?,
        parse_channel(parts.next()?)?,
        parse_channel(parts.next()?)?,
    ))
}

fn parse_rgba(value: &str) -> Option<Color> {
    let inner = value.strip_prefix("rgba(")?.strip_suffix(')')?;
    let mut parts = inner.split(',');
    let r = parse_channel(parts.next()?)?;
    let g = parse_channel(parts.next()?)?;
    let b = parse_channel(parts.next()?)?;
    let a = parts.next()?.trim().parse::<f32>().ok()?.clamp(0.0, 1.0);
    Some(Color::rgba(r, g, b, (a * 255.0).round() as u8))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{parse_stylesheet_with, Selector, Viewport};

    fn rule(selector: Selector, decls: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector,
            declarations: decls
                .iter()
                .map(|&(property, value)| Declaration {
                    property: property.into(),
                    value: value.into(),
                })
                .collect(),
        }
    }

    fn tag(name: &str) -> Selector {
        Selector {
            tag: Some(name.into()),
            id: None,
            classes: vec![],
            universal: false,
        }
    }

    fn id_sel(name: &str) -> Selector {
        Selector {
            tag: None,
            id: Some(name.into()),
            classes: vec![],
            universal: false,
        }
    }

    fn element(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: attrs.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        });
        dom.append(parent, id);
        id
    }

    fn one_element(tag: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, tag, &[]);
        (dom, id)
    }

    #[test]
    fn ua_default_block_and_author_overrides_ua() {
        let (dom, id) = one_element("p");

        let base = compute_styles(&dom, &[]);
        assert_eq!(base.get(id).unwrap().display, Display::Block);

        let sheets = [Stylesheet {
            rules: vec![rule(tag("p"), &[("display", "inline")])],
        }];
        let styled = compute_styles(&dom, &sheets);
        assert_eq!(styled.get(id).unwrap().display, Display::Inline);
        assert_eq!(styled.get(id).unwrap().color, Color::BLACK);
        assert_eq!(styled.get(id).unwrap().font_size, 16.0);
    }

    #[test]
    fn specificity_beats_source_order() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, "p", &[("id", "main")]);
        let sheets = [Stylesheet {
            // High specificity first, low specificity later: high must still win.
            rules: vec![
                rule(id_sel("main"), &[("color", "#00ff00")]),
                rule(tag("p"), &[("color", "#ff0000")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 255, 0));
    }

    #[test]
    fn source_order_tie_break() {
        let (dom, id) = one_element("p");
        let sheets = [Stylesheet {
            rules: vec![
                rule(tag("p"), &[("color", "#ff0000")]),
                rule(tag("p"), &[("color", "#0000ff")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn inheritance_chain_and_no_background_inheritance() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let p = element(&mut dom, body, "p", &[]);
        let sheets = [Stylesheet {
            rules: vec![rule(
                tag("body"),
                &[("color", "red"), ("background-color", "#ff0000")],
            )],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(p).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(map.get(p).unwrap().background, Color::TRANSPARENT);
        assert_eq!(map.get(body).unwrap().background, Color::rgb(255, 0, 0));
    }

    #[test]
    fn font_size_em_and_percent_accumulate() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let div = element(&mut dom, body, "div", &[]);
        let span = element(&mut dom, div, "span", &[]);
        let sheets = [Stylesheet {
            rules: vec![
                rule(tag("body"), &[("font-size", "2em")]),
                rule(tag("div"), &[("font-size", "50%")]),
                rule(tag("span"), &[("font-size", "1.5em")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(body).unwrap().font_size, 32.0);
        assert_eq!(map.get(div).unwrap().font_size, 16.0);
        assert_eq!(map.get(span).unwrap().font_size, 24.0);
    }

    #[test]
    fn display_none() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![rule(tag("div"), &[("display", "none")])],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().display, Display::None);
    }

    #[test]
    fn margin_four_value_shorthand() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![rule(tag("div"), &[("margin", "1px 2px 3px 4px")])],
        }];
        let map = compute_styles(&dom, &sheets);
        let m = map.get(id).unwrap().margin;
        assert_eq!((m.top, m.right, m.bottom, m.left), (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn longhand_after_shorthand_wins() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![
                rule(tag("div"), &[("margin", "1px 2px 3px 4px")]),
                rule(tag("div"), &[("margin-top", "9px")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        let m = map.get(id).unwrap().margin;
        assert_eq!((m.top, m.right, m.bottom, m.left), (9.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn shorthand_after_longhand_wins() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![
                rule(tag("div"), &[("margin-top", "9px")]),
                rule(tag("div"), &[("margin", "1px 2px 3px 4px")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        let m = map.get(id).unwrap().margin;
        assert_eq!((m.top, m.right, m.bottom, m.left), (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn padding_and_border_longhands_and_shorthands_apply() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![rule(
                tag("div"),
                &[
                    ("padding", "1px 2px"),
                    ("padding-bottom", "5px"),
                    ("border-width", "1px 2px 3px 4px"),
                    ("border-right-width", "9px"),
                ],
            )],
        }];
        let map = compute_styles(&dom, &sheets);
        let s = map.get(id).unwrap();
        assert_eq!(
            (
                s.padding.top,
                s.padding.right,
                s.padding.bottom,
                s.padding.left
            ),
            (1.0, 2.0, 5.0, 2.0)
        );
        assert_eq!(
            (
                s.border_widths.top,
                s.border_widths.right,
                s.border_widths.bottom,
                s.border_widths.left
            ),
            (1.0, 9.0, 3.0, 4.0)
        );
        assert_eq!(s.border_width, 9.0);
    }

    #[test]
    fn border_shorthand_sets_widths_and_color() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![rule(tag("div"), &[("border", "2px solid #3366aa")])],
        }];
        let map = compute_styles(&dom, &sheets);
        let s = map.get(id).unwrap();
        assert_eq!(
            (
                s.border_widths.top,
                s.border_widths.right,
                s.border_widths.bottom,
                s.border_widths.left
            ),
            (2.0, 2.0, 2.0, 2.0)
        );
        assert_eq!(s.border_width, 2.0);
        assert_eq!(s.border_color, Color::rgb(0x33, 0x66, 0xaa));
    }

    #[test]
    fn border_shorthand_color_only_leaves_widths_alone() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![rule(
                tag("div"),
                &[("border-width", "3px"), ("border", "red")],
            )],
        }];
        let map = compute_styles(&dom, &sheets);
        let s = map.get(id).unwrap();
        assert_eq!(s.border_widths.top, 3.0);
        assert_eq!(s.border_width, 3.0);
        assert_eq!(s.border_color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn border_shorthand_and_longhands_follow_source_order() {
        let (dom, id) = one_element("div");
        let sheets = [Stylesheet {
            rules: vec![
                rule(tag("div"), &[("border", "2px solid #0000ff")]),
                rule(
                    tag("div"),
                    &[("border-top-width", "9px"), ("border-color", "#ff0000")],
                ),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        let s = map.get(id).unwrap();
        assert_eq!(
            (
                s.border_widths.top,
                s.border_widths.right,
                s.border_widths.bottom,
                s.border_widths.left
            ),
            (9.0, 2.0, 2.0, 2.0)
        );
        assert_eq!(s.border_color, Color::rgb(255, 0, 0));

        // Reversed: the later shorthand overwrites both longhands.
        let sheets = [Stylesheet {
            rules: vec![
                rule(
                    tag("div"),
                    &[("border-top-width", "9px"), ("border-color", "#ff0000")],
                ),
                rule(tag("div"), &[("border", "4px dashed #00ff00")]),
            ],
        }];
        let map = compute_styles(&dom, &sheets);
        let s = map.get(id).unwrap();
        assert_eq!(
            (
                s.border_widths.top,
                s.border_widths.right,
                s.border_widths.bottom,
                s.border_widths.left
            ),
            (4.0, 4.0, 4.0, 4.0)
        );
        assert_eq!(s.border_color, Color::rgb(0, 255, 0));
    }

    #[test]
    fn rgba_alpha_is_parsed() {
        let (dom, id) = one_element("p");
        let sheets = [Stylesheet {
            rules: vec![rule(tag("p"), &[("color", "rgba(255, 0, 0, 0.5)")])],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgba(255, 0, 0, 128));
    }

    #[test]
    fn text_nodes_inherit_parent_style() {
        let mut dom = Dom::new();
        let root = dom.root();
        let p = element(&mut dom, root, "p", &[]);
        let text = dom.create(NodeKind::Text("hi".into()));
        dom.append(p, text);
        let sheets = [Stylesheet {
            rules: vec![rule(tag("p"), &[("color", "#123456")])],
        }];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(text).unwrap().color, map.get(p).unwrap().color);
        assert_eq!(
            map.get(text).unwrap().font_size,
            map.get(p).unwrap().font_size
        );
    }

    #[test]
    fn style_map_with_len_get_set() {
        let mut map = StyleMap::with_len(2);
        assert!(map.get(NodeId(0)).is_none());
        map.set(NodeId(0), initial_style());
        assert_eq!(map.get(NodeId(0)).unwrap().font_size, 16.0);
        assert!(map.get(NodeId(1)).is_none());
    }

    #[test]
    fn detached_slot_reuse_does_not_alias_styles_or_reattach_via_stale_handle() {
        let mut dom = Dom::new();
        let root = dom.root();
        // A occupies a slot and is styled red.
        let a = element(&mut dom, root, "p", &[]);
        let sheets = [sheet("p { color: red } div { color: blue }")];
        let before = compute_styles(&dom, &sheets);
        assert_eq!(before.get(a).unwrap().color, Color::rgb(255, 0, 0));

        // Detaching A puts its slot on the free list with a bumped generation.
        dom.detach_subtree(a);
        assert!(!dom.validate(a));

        // Re-creating reuses A's slot but must hand out a fresh handle.
        let c = element(&mut dom, root, "div", &[]);
        assert_ne!(c, a, "recycled slot must not reuse the stale handle");
        assert_eq!(c.slot(), a.slot(), "recycled slot is the same arena index");

        // Appending through the stale handle is ignored, so C stays under root.
        dom.append(a, c);
        assert_eq!(dom.parent(c), Some(root));

        // The fresh cascade styles C blue at the recycled slot, never A's
        // stale red style.
        let after = compute_styles(&dom, &sheets);
        assert_eq!(after.get(c).unwrap().color, Color::rgb(0, 0, 255));
    }

    // --- M7.1: cascade extensions ---------------------------------------

    fn sheet(css: &str) -> Stylesheet {
        hodo_css::parse_stylesheet(css)
    }

    #[test]
    fn author_important_beats_specificity_and_source_order() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, "p", &[("id", "main")]);
        let sheets = [sheet("#main { color: blue } p { color: red !important }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn ua_important_beats_author_important() {
        let (dom, id) = one_element("div");
        let sheets = [sheet("div { color: red !important }")];
        let map = compute_styles_inner(&dom, &sheets, "div { color: green !important }");
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 128, 0));
    }

    #[test]
    fn inline_style_is_highest_normal_origin() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(
            &mut dom,
            root,
            "p",
            &[("style", "color: green; font-size: 20px; margin: 3px")],
        );
        let sheets = [sheet("p { color: red; font-size: 12px; margin: 9px }")];
        let map = compute_styles(&dom, &sheets);
        let style = map.get(id).unwrap();
        assert_eq!(style.color, Color::rgb(0, 128, 0));
        assert_eq!(style.font_size, 20.0);
        assert_eq!(style.margin.top, 3.0);
    }

    #[test]
    fn origin_ladder_from_spec() {
        let mut dom = Dom::new();
        let root = dom.root();
        // UA !important beats every author-origin declaration, including
        // inline !important.
        let id = element(
            &mut dom,
            root,
            "div",
            &[("style", "color: cyan !important")],
        );
        let sheets = [sheet("div { color: red !important }")];
        let map = compute_styles_inner(&dom, &sheets, "div { color: blue !important }");
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 0, 255));

        let plain = element(&mut dom, root, "p", &[("style", "color: cyan")]);
        let sheets = [sheet("p { color: red !important }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(plain).unwrap().color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn specificity_only_breaks_ties_within_origin_and_importance() {
        // A high-specificity author !important still loses to a low-specificity
        // UA !important, because origin outranks specificity.
        let (dom, id) = one_element("div");
        let sheets = [sheet("#target { color: red !important }")];
        let mut dom = dom;
        dom.set_attr(id, "id", "target".to_string());
        let map = compute_styles_inner(&dom, &sheets, "div { color: green !important }");
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 128, 0));

        // Within one origin+importance level, specificity still decides.
        let sheets = [sheet("div { color: red } #target { color: blue }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 0, 255));
    }

    // --- @layer + custom properties ---------------------------------------

    #[test]
    fn unlayered_beats_layered_and_later_layer_wins() {
        let (_, _, s) = css_style("@layer base { div { color: red } } div { color: blue }");
        assert_eq!(s.color, Color::rgb(0, 0, 255));

        let (_, _, s) =
            css_style("@layer a { div { color: red } } @layer b { div { color: blue } }");
        assert_eq!(s.color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn specificity_breaks_ties_within_a_layer_not_across_layers() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, "div", &[("id", "target")]);

        // Within one layer, specificity still beats source order.
        let sheets = [sheet(
            "@layer early { div { color: blue } #target { color: red } }",
        )];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(255, 0, 0));

        // Across layers, layer order outranks specificity (spec cascade order).
        let sheets = [sheet(
            "@layer early { #target { color: red } } @layer late { div { color: blue } }",
        )];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn important_inverts_layer_order() {
        let (_, _, s) = css_style(
            "@layer a { div { color: red !important } } \
             @layer b { div { color: blue !important } }",
        );
        assert_eq!(s.color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn layer_and_media_wrappers_compose() {
        let (_, _, s) = css_style("@layer base { @media screen { div { color: red } } }");
        assert_eq!(s.color, Color::rgb(255, 0, 0));

        let (_, _, s) = css_style("@media screen { @layer base { div { color: green } } }");
        assert_eq!(s.color, Color::rgb(0, 128, 0));

        let (_, _, s) =
            css_style("@layer base { @media print { div { color: red } } } div { color: blue }");
        assert_eq!(s.color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn same_name_layer_merges_across_sheets() {
        let first = sheet("@layer a { div { color: red } }");
        let second = sheet("@layer b { div { color: green } } @layer a { div { color: blue } }");
        assert_eq!(
            resolved_layer_order(&[first.clone(), second.clone()]),
            ["a", "b"]
        );
        let (dom, id) = one_element("div");
        let map = compute_styles(&dom, &[first, second]);
        // Global `b` (rank 1) outranks `a` (rank 0) even though sheet 2's
        // per-sheet rank for `a` is 1; a repeated name must not restart.
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 128, 0));
    }

    #[test]
    fn nested_layer_keeps_its_own_path_vs_a_flat_layer() {
        let (_, _, s) = css_style(
            "@layer inner { div { color: blue } } \
             @layer outer { @layer inner { div { color: red } } }",
        );
        // `outer.inner` is a distinct, later layer than flat `inner`.
        assert_eq!(s.color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn comma_layer_list_duplicates_the_rule_into_each_layer() {
        let (_, _, s) =
            css_style("@layer b { div { color: blue } } @layer a, b { div { color: red } }");
        // The copy in `b` shares b's rank with the blue rule but comes later.
        assert_eq!(s.color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn var_inside_calc_resolves_at_cascade_time() {
        let (_, _, s) = css_style("div { --w: 50px; width: calc(var(--w) + 5px) }");
        assert_eq!(s.width, Some(55.0));

        let (_, _, s) = css_style("div { --b: 10px; --a: calc(var(--b) * 2); width: var(--a) }");
        assert_eq!(s.width, Some(20.0));
    }

    #[test]
    fn percent_calc_is_deferred_to_layout_basis() {
        let (dom, id) = one_element("div");
        let map = compute_styles(&dom, &[sheet("div { width: calc(100% - 32px) }")]);
        assert!(map.get(id).unwrap().width.is_none());
        assert_eq!(map.resolve_width_with_basis(id, 1000.0), Some(968.0));
        assert_eq!(map.resolve_width_with_basis(id, 500.0), Some(468.0));
    }

    #[test]
    fn unitless_calc_line_height_is_a_number() {
        let (_, _, s) = css_style("div { line-height: calc(2) }");
        assert_eq!(s.line_height, LineHeight::Number(2.0));
    }

    #[test]
    fn negative_calc_width_and_height_clamp_to_zero() {
        let (_, _, s) = css_style("div { width: calc(0px - 10px); height: calc(0px - 5px) }");
        assert_eq!(s.width, Some(0.0));
        assert_eq!(s.height, Some(0.0));
    }

    #[test]
    fn visited_and_link_style_split() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, "a", &[("href", "https://example.com")]);
        let sheets = [sheet("a:link { color: blue } a:visited { color: red }")];

        let unvisited = compute_styles_with_visited(&dom, &sheets, &HashSet::new());
        assert_eq!(unvisited.get(id).unwrap().color, Color::rgb(0, 0, 255));

        let visited = HashSet::from(["https://example.com".to_string()]);
        let styled = compute_styles_with_visited(&dom, &sheets, &visited);
        assert_eq!(styled.get(id).unwrap().color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn positional_pseudos_are_detected_inside_is_and_where() {
        let first = Compound {
            pseudos: vec![PseudoClass::FirstChild],
            ..Compound::default()
        };
        let in_is = Compound {
            pseudos: vec![PseudoClass::Is(vec![Compound::default(), first.clone()])],
            ..Compound::default()
        };
        let in_where = Compound {
            pseudos: vec![PseudoClass::Where(vec![first])],
            ..Compound::default()
        };
        assert!(compound_has_positional(&in_is));
        assert!(compound_has_positional(&in_where));
    }

    #[test]
    fn custom_property_inherits_and_substitutes() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = element(&mut dom, root, "div", &[]);
        let child = element(&mut dom, parent, "span", &[]);
        let sheets = [sheet("div { --brand: red } span { color: var(--brand) }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(child).unwrap().color, Color::rgb(255, 0, 0));

        let sheets = [sheet("div { --brand: blue } span { color: var(--brand) }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(child).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn var_fallback_and_guaranteed_invalid_drop() {
        let (_, _, fallback) = css_style("div { color: var(--missing, green) }");
        assert_eq!(fallback.color, Color::rgb(0, 128, 0));

        let (_, _, dropped) = css_style("div { color: var(--missing) }");
        assert_eq!(dropped.color, Color::BLACK);
    }

    #[test]
    fn var_works_in_shorthand_values() {
        let (_, _, s) = css_style("div { --w: 3px; border: var(--w) solid red }");
        assert_eq!(s.border_widths.top, 3.0);
        assert_eq!(s.border_color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn custom_property_cycle_drops_all_involved() {
        let (_, _, with_fallback) =
            css_style("div { --a: var(--b); --b: var(--a); color: var(--a, green) }");
        assert_eq!(with_fallback.color, Color::rgb(0, 128, 0));

        let (_, _, invalid) = css_style("div { --a: var(--b); --b: var(--a); color: var(--a) }");
        assert_eq!(invalid.color, Color::BLACK);
    }

    #[test]
    fn inherited_custom_tokens_are_part_of_the_sharing_key() {
        let mut dom = Dom::new();
        let root = dom.root();
        let a = element(&mut dom, root, "div", &[("class", "pa")]);
        let b = element(&mut dom, root, "div", &[("class", "pb")]);
        let ca = element(&mut dom, a, "span", &[("class", "child")]);
        let cb = element(&mut dom, b, "span", &[("class", "child")]);
        let sheets = [sheet(
            ".pa { --token: red } .pb { --token: blue } .child { color: var(--token) }",
        )];
        let cached = compute_styles(&dom, &sheets);
        let uncached = compute_styles_uncached(&dom, &sheets);
        assert_eq!(cached.get(ca).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(cached.get(cb).unwrap().color, Color::rgb(0, 0, 255));
        assert_eq!(
            cached.get(ca).unwrap().color,
            uncached.get(ca).unwrap().color
        );
        assert_eq!(
            cached.get(cb).unwrap().color,
            uncached.get(cb).unwrap().color
        );
    }

    #[test]
    fn font_family_parses_and_survives_cascade() {
        let (_, _, stack) = css_style("div { font-family: Verdana, sans-serif }");
        assert_eq!(stack.font_family, "Verdana, sans-serif");

        let (_, _, quoted) = css_style("div { font-family: \"My Font\", serif }");
        assert_eq!(quoted.font_family, "My Font, serif");

        let (_, _, generic) = css_style("div { font-family: monospace }");
        assert_eq!(generic.font_family, "monospace");

        let (_, _, winning) =
            css_style("div { font-family: Verdana, sans-serif } div { font-family: Courier }");
        assert_eq!(winning.font_family, "Courier");
    }

    #[test]
    fn font_family_inherits_to_children() {
        let mut dom = Dom::new();
        let root = dom.root();
        let parent = element(&mut dom, root, "div", &[]);
        let child = element(&mut dom, parent, "span", &[]);
        let sheets = [sheet("div { font-family: Verdana, sans-serif }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(parent).unwrap().font_family, "Verdana, sans-serif");
        assert_eq!(map.get(child).unwrap().font_family, "Verdana, sans-serif");
    }

    #[test]
    fn viewport_lengths_reach_the_cascade_output() {
        let vp = Viewport {
            width: 1200,
            height: 800,
            ..Viewport::default()
        };
        let (dom, id) = one_element("div");
        for (css, expected) in [
            ("div { padding: 25vh }", 200.0),
            ("div { padding: 10vw }", 120.0),
            ("div { padding: 5vmin }", 40.0),
            ("div { padding: 5vmax }", 60.0),
        ] {
            let map = compute_styles(&dom, &[parse_stylesheet_with(vp, css)]);
            let padding = map.get(id).unwrap().padding;
            assert_eq!(padding.top, expected, "css {css}");
            assert!(padding.top.is_finite());
        }

        let map = compute_styles(
            &dom,
            &[parse_stylesheet_with(vp, "div { padding: 25vh 2em 2em }")],
        );
        let padding = map.get(id).unwrap().padding;
        assert_eq!(padding.top, 200.0);
        assert_eq!(padding.right, 32.0);
        assert!(padding.bottom.is_finite() && padding.left.is_finite());

        // Unknown unit keeps the existing fallback path (no padding).
        let map = compute_styles(&dom, &[parse_stylesheet_with(vp, "div { padding: 10vi }")]);
        assert_eq!(map.get(id).unwrap().padding.top, 0.0);
    }

    #[test]
    fn calc_lengths_reach_the_cascade_output() {
        let (dom, id) = one_element("div");
        for (css, expected) in [
            ("div { padding: calc(25vh - 8px) }", 192.0),
            ("div { padding: calc(2*(10px+5px)) }", 30.0),
            ("div { width: calc(100px + 20px) }", 120.0),
            ("div { padding: calc(1px / 0) }", 0.0),
        ] {
            let map = compute_styles(&dom, &[sheet(css)]);
            let style = map.get(id).unwrap();
            if css.contains("width") {
                assert_eq!(style.width, Some(expected), "css {css}");
            } else {
                assert_eq!(style.padding.top, expected, "css {css}");
            }
        }
    }

    #[test]
    fn complex_selectors_drive_cascade() {
        let mut dom = Dom::new();
        let root = dom.root();
        let html = element(&mut dom, root, "html", &[]);
        let body = element(&mut dom, html, "body", &[]);
        let list = element(&mut dom, body, "ul", &[]);
        let first = element(&mut dom, list, "li", &[("class", "first")]);
        let target = element(&mut dom, list, "li", &[("id", "target")]);
        element(&mut dom, list, "li", &[]);
        let sheets = [sheet(
            "ul li { color: red } ul > li + li { color: green } \
             li:first-child { color: blue } .first + #target { color: yellow }",
        )];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(first).unwrap().color, Color::rgb(0, 0, 255));
        assert_eq!(map.get(target).unwrap().color, Color::rgb(255, 255, 0));
    }

    #[test]
    fn invalid_selector_only_drops_its_rule_in_cascade() {
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(&mut dom, root, "p", &[]);
        let sheets = [sheet("::before { color: red } p { color: blue }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(id).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn deep_dom_style_pass_completes_under_budget() {
        let mut dom = Dom::new();
        let mut cursor = dom.root();
        for _ in 0..1000 {
            cursor = element(&mut dom, cursor, "div", &[]);
        }
        let sheets = [sheet("div div div { color: red }")];
        let start = std::time::Instant::now();
        let map = compute_styles(&dom, &sheets);
        let elapsed = start.elapsed();
        assert_eq!(map.get(cursor).unwrap().color, Color::rgb(255, 0, 0));
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "took {elapsed:?}"
        );
    }

    // --- M8: frozen style-contract parser tests --------------------------

    fn css_style(css: &str) -> (Dom, NodeId, ComputedStyle) {
        let (dom, id) = one_element("div");
        let sheets = [sheet(css)];
        let map = compute_styles(&dom, &sheets);
        let style = map.get(id).unwrap().clone();
        (dom, id, style)
    }

    #[test]
    fn flex_and_alignment_properties_parse_happy_and_bad() {
        let (_, _, s) = css_style(
            "div { flex-direction: column-reverse; flex-wrap: wrap; \
             justify-content: space-evenly; align-items: baseline; align-self: center; \
             flex-grow: 2; flex-shrink: 3; flex-basis: 10px; order: 4; }",
        );
        assert_eq!(s.flex_direction, FlexDirection::ColumnReverse);
        assert_eq!(s.flex_wrap, FlexWrap::Wrap);
        assert_eq!(s.justify_content, JustifyContent::SpaceEvenly);
        assert_eq!(s.align_items, AlignItems::Baseline);
        assert_eq!(s.align_self, AlignSelf::Center);
        assert_eq!(s.flex_grow, 2.0);
        assert_eq!(s.flex_shrink, 3.0);
        assert_eq!(s.flex_basis, FlexBasis::Length(10.0));
        assert_eq!(s.order, 4);

        let (_, _, bad) = css_style(
            "div { flex-direction: sideways; flex-wrap: sometimes; \
             justify-content: nowhere; align-items: weird; align-self: odd; \
             flex-grow: nope; flex-shrink: nope; flex-basis: nope; order: nope; }",
        );
        assert_eq!(bad.flex_direction, FlexDirection::Row);
        assert_eq!(bad.flex_wrap, FlexWrap::NoWrap);
        assert_eq!(bad.justify_content, JustifyContent::FlexStart);
        assert_eq!(bad.align_items, AlignItems::Stretch);
        assert_eq!(bad.align_self, AlignSelf::Auto);
        assert_eq!(bad.flex_grow, 0.0);
        assert_eq!(bad.flex_shrink, 1.0);
        assert_eq!(bad.flex_basis, FlexBasis::Auto);
        assert_eq!(bad.order, 0);
    }

    #[test]
    fn flex_shorthand_expands_to_grow_shrink_basis() {
        let (_, _, s) = css_style("div { flex: 2 3 8px }");
        assert_eq!((s.flex_grow, s.flex_shrink), (2.0, 3.0));
        assert_eq!(s.flex_basis, FlexBasis::Length(8.0));
        let (_, _, none) = css_style("div { flex: none }");
        assert_eq!((none.flex_grow, none.flex_shrink), (0.0, 0.0));
        assert_eq!(none.flex_basis, FlexBasis::Auto);
    }

    #[test]
    fn gap_longhands_and_aliases_parse() {
        let (_, _, s) = css_style("div { gap: 1px 2px; column-gap: 7px; row-gap: 4px }");
        assert_eq!(s.gap, 1.0);
        assert_eq!(s.column_gap, 7.0);
        assert_eq!(s.row_gap, 4.0);
        let (_, _, alias) = css_style("div { grid-gap: 3px }");
        assert_eq!(
            (alias.gap, alias.column_gap, alias.row_gap),
            (3.0, 3.0, 3.0)
        );
        let (_, _, bad) = css_style("div { gap: nonsense }");
        assert_eq!((bad.gap, bad.column_gap, bad.row_gap), (0.0, 0.0, 0.0));
    }

    #[test]
    fn grid_track_lists_parse() {
        let (_, _, s) = css_style(
            "div { grid-template-columns: 100px 1fr auto minmax(10px, 2fr) repeat(2, 3px auto); \
             grid-template-rows: 40%; }",
        );
        assert_eq!(
            s.grid_template_columns,
            vec![
                GridTrack::Fixed(100.0),
                GridTrack::Fr(1.0),
                GridTrack::Auto,
                GridTrack::MinMax(
                    Box::new(GridTrack::Fixed(10.0)),
                    Box::new(GridTrack::Fr(2.0)),
                ),
                GridTrack::Repeat(2, vec![GridTrack::Fixed(3.0), GridTrack::Auto]),
            ]
        );
        assert_eq!(s.grid_template_rows, vec![GridTrack::Percent(40.0)]);
        let (_, _, bad) = css_style("div { grid-template-columns: blah blah }");
        assert!(bad.grid_template_columns.is_empty());
    }

    #[test]
    fn grid_line_placements_parse() {
        let (_, _, s) = css_style(
            "div { grid-column: 2 / span 3; grid-row: -1 / auto; \
             grid-column-start: 4; grid-row-end: 6 }",
        );
        // The later start/end longhands win over the shorthand halves.
        assert_eq!(s.grid_column, (GridLine::Line(4), GridLine::Span(3)));
        assert_eq!(s.grid_row, (GridLine::Line(-1), GridLine::Line(6)));
        let (_, _, bad) = css_style("div { grid-column: nowhere; grid-row: nope }");
        assert_eq!(bad.grid_column, (GridLine::Auto, GridLine::Auto));
        assert_eq!(bad.grid_row, (GridLine::Auto, GridLine::Auto));
    }

    #[test]
    fn grid_auto_flow_is_parsed_and_ignored() {
        let (_, _, s) = css_style("div { grid-auto-flow: column dense }");
        assert!(s.grid_template_columns.is_empty());
        assert_eq!(s.grid_column, (GridLine::Auto, GridLine::Auto));
    }

    #[test]
    fn table_properties_parse_happy_and_bad() {
        let (_, _, s) = css_style(
            "div { table-layout: fixed; border-collapse: collapse; border-spacing: 3px 4px }",
        );
        assert_eq!(s.table_layout, TableLayout::Fixed);
        assert_eq!(s.border_collapse, BorderCollapse::Collapse);
        assert_eq!(s.border_spacing, (3.0, 4.0));
        let (_, _, bad) =
            css_style("div { table-layout: weird; border-collapse: weird; border-spacing: bad }");
        assert_eq!(bad.table_layout, TableLayout::Auto);
        assert_eq!(bad.border_collapse, BorderCollapse::Separate);
        assert_eq!(bad.border_spacing, (2.0, 2.0));
    }

    #[test]
    fn text_properties_parse_happy_and_bad() {
        let (_, _, s) = css_style(
            "div { line-height: 150%; vertical-align: -3px; text-align: right; \
             text-indent: 4px; letter-spacing: 1px; word-spacing: 2px; \
             white-space: pre-wrap; overflow-wrap: anywhere; \
             text-decoration: underline line-through }",
        );
        assert_eq!(s.line_height, LineHeight::Length(24.0));
        assert_eq!(s.vertical_align, VerticalAlign::Length(-3.0));
        assert_eq!(s.text_align, TextAlign::Right);
        assert_eq!(s.text_indent, 4.0);
        assert_eq!(s.letter_spacing, 1.0);
        assert_eq!(s.word_spacing, 2.0);
        assert_eq!(s.white_space, WhiteSpace::PreWrap);
        assert_eq!(s.overflow_wrap, OverflowWrap::Anywhere);
        assert!(s.underline);
        assert!(s.line_through);

        let (_, _, none) = css_style("div { text-decoration: none }");
        assert!(!none.underline);
        assert!(!none.line_through);

        let (_, _, bad) = css_style(
            "div { line-height: bogus; vertical-align: sideways; text-align: justify; \
             white-space: whenever; overflow-wrap: wherever }",
        );
        assert_eq!(bad.line_height, LineHeight::Normal);
        assert_eq!(bad.vertical_align, VerticalAlign::Baseline);
        assert_eq!(bad.text_align, TextAlign::Left);
        assert_eq!(bad.white_space, WhiteSpace::Normal);
        assert_eq!(bad.overflow_wrap, OverflowWrap::Normal);
    }

    #[test]
    fn line_height_forms_distinguish_number_length_and_normal() {
        let (_, _, number) = css_style("div { line-height: 1.5 }");
        assert_eq!(number.line_height, LineHeight::Number(1.5));
        let (_, _, px) = css_style("div { line-height: 20px }");
        assert_eq!(px.line_height, LineHeight::Length(20.0));
        let (_, _, normal) = css_style("div { line-height: normal }");
        assert_eq!(normal.line_height, LineHeight::Normal);
        let (_, _, em) = css_style("div { line-height: 2em }");
        assert_eq!(em.line_height, LineHeight::Length(32.0));
    }

    #[test]
    fn positioning_and_box_properties_parse() {
        let (_, _, s) = css_style(
            "div { position: absolute; float: left; clear: both; overflow: hidden; \
             top: 1px; right: 2px; bottom: 3px; left: 4px; width: 100px; height: 50px; \
             min-width: 10px; min-height: 11px; max-width: 200px; max-height: 201px }",
        );
        assert_eq!(s.position, Position::Absolute);
        assert_eq!(s.float, Float::Left);
        assert_eq!(s.clear, Clear::Both);
        assert_eq!(s.overflow, Overflow::Hidden);
        assert_eq!(
            (
                s.offsets.top,
                s.offsets.right,
                s.offsets.bottom,
                s.offsets.left
            ),
            (Some(1.0), Some(2.0), Some(3.0), Some(4.0))
        );
        assert_eq!((s.width, s.height), (Some(100.0), Some(50.0)));
        assert_eq!((s.min_size.w, s.min_size.h), (10.0, 11.0));
        assert_eq!((s.max_size.w, s.max_size.h), (200.0, 201.0));

        let (_, _, bad) = css_style(
            "div { position: somewhere; float: middle; clear: neither; overflow: scroll; \
             width: auto; height: auto; min-width: nope }",
        );
        assert_eq!(bad.position, Position::Static);
        assert_eq!(bad.float, Float::None);
        assert_eq!(bad.clear, Clear::None);
        assert_eq!(bad.overflow, Overflow::Visible);
        assert_eq!((bad.width, bad.height), (None, None));
        assert_eq!(bad.offsets.top, None);
        assert_eq!((bad.min_size.w, bad.min_size.h), (0.0, 0.0));
    }

    #[test]
    fn non_finite_lengths_are_rejected() {
        let (_, _, s) = css_style(
            "div { width: NaNpx; height: infpx; margin: infinitypx; font-size: NaN; \
             flex-grow: inf; line-height: NaN }",
        );
        assert_eq!(s.width, None);
        assert_eq!(s.height, None);
        assert_eq!(s.margin.top, 0.0);
        assert_eq!(s.font_size, 16.0);
        assert_eq!(s.flex_grow, 0.0);
        assert_eq!(s.line_height, LineHeight::Normal);
    }

    #[test]
    fn every_display_value_parses_and_unknown_falls_back_to_inline() {
        let cases: [(&str, Display); 14] = [
            ("block", Display::Block),
            ("inline", Display::Inline),
            ("none", Display::None),
            ("inline-block", Display::InlineBlock),
            ("flex", Display::Flex),
            ("inline-flex", Display::InlineFlex),
            ("grid", Display::Grid),
            ("inline-grid", Display::InlineGrid),
            ("table", Display::Table),
            ("inline-table", Display::InlineTable),
            ("table-row-group", Display::TableRowGroup),
            ("table-row", Display::TableRow),
            ("table-cell", Display::TableCell),
            ("table-caption", Display::TableCaption),
        ];
        for (value, expected) in cases {
            let (_, _, s) = css_style(&format!("div {{ display: {value} }}"));
            assert_eq!(s.display, expected, "display: {value}");
        }
        let (_, _, unknown) = css_style("div { display: contents }");
        assert_eq!(unknown.display, Display::Inline);
    }

    #[test]
    fn ua_stylesheet_declares_table_rules() {
        let ua = hodo_css::parse_stylesheet(ua_stylesheet());
        let find = |tag_name: &str| {
            ua.rules
                .iter()
                .find(|r| r.selector.tag.as_deref() == Some(tag_name))
        };
        for tag_name in [
            "table", "caption", "thead", "tbody", "tfoot", "tr", "td", "th",
        ] {
            assert!(find(tag_name).is_some(), "UA sheet must style {tag_name}");
        }
        let table = find("table").unwrap();
        for (property, value) in [
            ("display", "table"),
            ("table-layout", "auto"),
            ("border-collapse", "separate"),
            ("border-spacing", "2px 2px"),
        ] {
            assert!(
                table
                    .declarations
                    .iter()
                    .any(|d| d.property == property && d.value == value),
                "table must declare {property}: {value}"
            );
        }
        let td = find("td").unwrap();
        assert!(td
            .declarations
            .iter()
            .any(|d| { d.property == "display" && d.value == "table-cell" }));
        assert!(find("input")
            .unwrap()
            .declarations
            .iter()
            .any(|d| { d.property == "display" && d.value == "inline-block" }));
    }

    #[test]
    fn white_space_and_line_height_inherit_appropriately() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let p = element(&mut dom, body, "p", &[]);
        let sheets = [sheet("body { white-space: pre; line-height: 1.5 }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(p).unwrap().white_space, WhiteSpace::Pre);
        assert_eq!(map.get(p).unwrap().line_height, LineHeight::Number(1.5));

        let sheets = [sheet("body { line-height: 200% }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(p).unwrap().line_height, LineHeight::Length(32.0));

        // Position-like properties must not inherit.
        let sheets = [sheet("body { position: absolute; float: left }")];
        let map = compute_styles(&dom, &sheets);
        assert_eq!(map.get(p).unwrap().position, Position::Static);
        assert_eq!(map.get(p).unwrap().float, Float::None);
    }

    // --- Style sharing cache ---------------------------------------------

    fn ua() -> &'static Stylesheet {
        UA_SHEET.get_or_init(|| parse_stylesheet(ua_stylesheet()))
    }

    /// Compare every field of every computed style. `Debug` is derived for the
    /// whole struct (including `Edges`/`GridTrack`), so this is an exact
    /// field-by-field comparison without adding a public `PartialEq`.
    fn assert_same_styles(shared: &StyleMap, plain: &StyleMap, dom: &Dom) {
        dom.walk(|id, _| {
            assert_eq!(
                format!("{:?}", shared.get(id)),
                format!("{:?}", plain.get(id)),
                "style mismatch for {id:?}"
            );
        });
    }

    /// `HODO_NO_RULE_INDEX=1` routes the public entry through brute force;
    /// exercising the mode seam directly keeps the test independent of env
    /// mutation in a parallel test run. The result must stay byte-for-byte the
    /// same as the indexed path (the corpus's EXACT equality, end to end).
    #[test]
    fn no_rule_index_flag_produces_identical_styles() {
        let (dom, sheets) = style_corpus(64, 20, true);
        let indexed =
            compute_styles_tracked_with_index(&dom, &sheets, ua(), true, 1, &HashSet::new(), true)
                .0;
        let brute =
            compute_styles_tracked_with_index(&dom, &sheets, ua(), true, 1, &HashSet::new(), false)
                .0;
        assert_same_styles(&indexed, &brute, &dom);
    }

    /// 33 distinct attribute names overflow the 32 dedicated buckets; the
    /// fallback rule must still win a specificity tie by source order.
    #[test]
    fn rule_index_33rd_attribute_keeps_source_order_winner() {
        let mut css = String::new();
        for i in 0..=32 {
            css.push_str(&format!("[data-k{i:02}] {{ color: #ff0000 }}\n"));
        }
        css.push_str(".target { color: #0000ff }\n");
        let sheets = [sheet(&css)];
        let mut dom = Dom::new();
        let root = dom.root();
        let id = element(
            &mut dom,
            root,
            "div",
            &[("class", "target"), ("data-k32", "x")],
        );
        let map = compute_styles(&dom, &sheets);
        assert_eq!(
            map.get(id).unwrap().color,
            Color::rgb(0, 0, 255),
            "equal specificity: the later .target rule must beat the 33rd attr rule"
        );
    }

    fn same_class_siblings(n: usize, extra_css: &str) -> (Dom, Vec<NodeId>, Vec<Stylesheet>) {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let ids: Vec<NodeId> = (0..n)
            .map(|_| element(&mut dom, body, "div", &[("class", "foo")]))
            .collect();
        let css = format!(
            ".foo {{ color: red; background: white; padding: 2px; font-size: 12px }}\n{extra_css}"
        );
        (dom, ids, vec![sheet(&css)])
    }

    #[test]
    fn sharing_matches_uncached_for_500_same_class_siblings() {
        let (dom, ids, sheets) = same_class_siblings(500, "");
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let plain = compute_styles_uncached(&dom, &sheets);
        assert_same_styles(&shared, &plain, &dom);
        assert!(stats.hits >= 498, "expected near-total sharing: {stats:?}");
        assert_eq!(stats.excluded, 0, "{stats:?}");
        assert_eq!(shared.get(ids[0]).unwrap().color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn inline_style_elements_are_excluded_from_sharing() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let plain = element(&mut dom, body, "div", &[("class", "foo")]);
        let inline = element(
            &mut dom,
            body,
            "div",
            &[("class", "foo"), ("style", "color: blue")],
        );
        let sheets = [sheet(".foo { color: red }")];

        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let plain_map = compute_styles_uncached(&dom, &sheets);
        assert_same_styles(&shared, &plain_map, &dom);
        assert_eq!(shared.get(plain).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(shared.get(inline).unwrap().color, Color::rgb(0, 0, 255));
        assert!(
            stats.excluded >= 1,
            "inline element must bail out: {stats:?}"
        );
    }

    #[test]
    fn positional_pseudo_sheet_disables_sharing() {
        let (dom, ids, sheets) = same_class_siblings(
            10,
            ".foo:nth-child(odd) { color: red }\n.foo:nth-child(even) { color: blue }",
        );
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let plain = compute_styles_uncached(&dom, &sheets);
        assert_same_styles(&shared, &plain, &dom);
        assert_eq!(
            stats.hits, 0,
            "positional pseudo must gate the cache: {stats:?}"
        );
        assert!(stats.excluded > 0, "{stats:?}");
        // Sanity-check that the fixture really distinguishes positions.
        assert_eq!(shared.get(ids[0]).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(shared.get(ids[1]).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn important_cascade_is_preserved_through_the_cache() {
        let (dom, ids, sheets) =
            same_class_siblings(16, ".foo { color: red !important }\n.foo { color: green }");
        // UA important outranks author important; sharing must not change that.
        let ua = parse_stylesheet("div { color: blue !important }");
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, &ua, true, &HashSet::new());
        let plain = compute_styles_parsed(&dom, &sheets, &ua, false, &HashSet::new());
        assert_same_styles(&shared, &plain, &dom);
        assert!(stats.hits > 0, "expected sharing here: {stats:?}");
        for id in ids {
            assert_eq!(shared.get(id).unwrap().color, Color::rgb(0, 0, 255));
        }
    }

    #[test]
    fn two_hundred_deep_nesting_matches_uncached() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let mut parent = body;
        let chain: Vec<NodeId> = (0..200)
            .map(|_| {
                parent = element(&mut dom, parent, "div", &[("class", "lvl")]);
                parent
            })
            .collect();
        let sheets = [sheet(
            "body { color: purple; font-weight: bold }\n.lvl { padding: 1px; margin: 2px }",
        )];
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let plain = compute_styles_uncached(&dom, &sheets);
        assert_same_styles(&shared, &plain, &dom);
        assert!(stats.hits >= 198, "expected deep-chain sharing: {stats:?}");
        assert_eq!(
            shared.get(chain[199]).unwrap().color,
            Color::rgb(128, 0, 128)
        );
    }

    #[test]
    fn inherited_parent_context_is_part_of_the_key() {
        // Same matched-rule set under parents with different inherited colors
        // must NOT share. This is why the key hashes the parent's inherited
        // values rather than only the element's own matched rules.
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let a = element(&mut dom, body, "div", &[("id", "a")]);
        let b = element(&mut dom, body, "div", &[("id", "b")]);
        let kid_a = element(&mut dom, a, "div", &[("class", "kid")]);
        let kid_b = element(&mut dom, b, "div", &[("class", "kid")]);
        let sheets = [sheet(
            "#a { color: red } #b { color: blue } .kid { padding: 1px }",
        )];

        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let plain = compute_styles_uncached(&dom, &sheets);
        assert_same_styles(&shared, &plain, &dom);
        assert_eq!(shared.get(kid_a).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(shared.get(kid_b).unwrap().color, Color::rgb(0, 0, 255));
        assert_eq!(
            stats.hits, 0,
            "different inherited context must not share: {stats:?}"
        );
    }

    /// In-crate A/B harness. Run with
    /// `cargo test --release -p hodo-style ab_sharing_2000_same_class -- --nocapture`.
    #[test]
    fn ab_sharing_2000_same_class() {
        use std::time::Instant;

        let (dom, _ids, sheets) = same_class_siblings(2000, "");
        let iterations = 7;
        let mut on = std::time::Duration::ZERO;
        let mut off = std::time::Duration::ZERO;
        let mut last_stats = SharingStats::default();
        for _ in 0..iterations {
            let start = Instant::now();
            let (shared, stats) =
                compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
            on += start.elapsed();
            last_stats = stats;
            assert_eq!(shared.styles.len(), dom.len());
            let start = Instant::now();
            let plain = compute_styles_tracked(&dom, &sheets, ua(), false, &HashSet::new()).0;
            off += start.elapsed();
            assert_eq!(plain.styles.len(), dom.len());
        }
        let on_ms = on.as_secs_f64() * 1000.0 / iterations as f64;
        let off_ms = off.as_secs_f64() * 1000.0 / iterations as f64;
        eprintln!(
            "A/B styles (2000 same-class divs): cache on {on_ms:.2} ms, off {off_ms:.2} ms, \
             delta {:.1}% | stats {last_stats:?}",
            if off_ms > 0.0 {
                (on_ms - off_ms) / off_ms * 100.0
            } else {
                0.0
            }
        );
    }

    // --- Parallel matching (phase A) ---------------------------------------

    /// Build a shallow, big-doc-like tree with varied tags/classes, attribute
    /// selectors and some inline styles. `positional` adds sibling-position
    /// rules, which exercise the per-element sibling scan and disable sharing.
    fn style_corpus(n: usize, rules: usize, positional: bool) -> (Dom, Vec<Stylesheet>) {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let tags = ["div", "p", "span", "li"];
        let mut parent = body;
        for i in 0..n {
            if i % 25 == 0 {
                parent = body;
            }
            let class = match i % 3 {
                0 => "foo bar",
                1 => "foo",
                _ => "baz",
            };
            let mut attrs: Vec<(&str, &str)> = vec![("class", class), ("data-kind", "mixed")];
            if i % 17 == 0 {
                attrs.push(("style", "color: teal"));
            }
            parent = element(&mut dom, parent, tags[i % tags.len()], &attrs);
        }
        let mut selectors = vec![
            ".foo { color: red; padding: 1px }",
            "div p { color: blue; margin: 1px }",
            "span + li { background: white }",
            "[data-kind=\"mixed\"] { border-width: 1px }",
            "div > span { text-indent: 2px }",
            ".baz { font-size: 13px }",
        ];
        if positional {
            selectors.push(".bar:nth-child(2n+1) { color: green }");
            selectors.push("li:first-child { font-weight: bold }");
        }
        let css: String = (0..rules)
            .map(|i| format!("{}\n", selectors[i % selectors.len()]))
            .collect();
        (dom, vec![sheet(&css)])
    }

    /// Uncached+sequential is the reference; cached+sequential and the
    /// production path (cached + auto threads) must be byte-identical to it.
    fn assert_three_way(dom: &Dom, sheets: &[Stylesheet]) {
        let seq_uncached =
            compute_styles_tracked_with(dom, sheets, ua(), false, 1, &HashSet::new()).0;
        let seq_cached = compute_styles_tracked_with(dom, sheets, ua(), true, 1, &HashSet::new()).0;
        let parallel = compute_styles_tracked_with(dom, sheets, ua(), true, 0, &HashSet::new()).0;
        assert_same_styles(&seq_uncached, &seq_cached, dom);
        assert_same_styles(&seq_uncached, &parallel, dom);
    }

    #[test]
    fn parallel_matches_sequential_on_2000_element_fixture() {
        let (dom, sheets) = style_corpus(2000, 200, false);
        assert_three_way(&dom, &sheets);
        let (_, stats) = compute_styles_tracked_with(&dom, &sheets, ua(), true, 0, &HashSet::new());
        assert!(stats.hits > 0, "sharing should be active here: {stats:?}");
        assert!(LAST_MATCH_THREADS.with(|count| count.get()) >= 2);
    }

    #[test]
    fn parallel_matches_sequential_on_big_doc_corpus() {
        // Sibling-position rules, descendant chains and attribute rules over a
        // 2000-element tree: the hodo-perf LARGE corpus shape.
        let (dom, sheets) = style_corpus(2000, 200, true);
        assert_three_way(&dom, &sheets);
        assert!(LAST_MATCH_THREADS.with(|count| count.get()) >= 2);
    }

    #[test]
    fn parallel_matches_sequential_on_200_deep_chain() {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = element(&mut dom, root, "body", &[]);
        let mut parent = body;
        for _ in 0..200 {
            parent = element(&mut dom, parent, "div", &[("class", "lvl")]);
        }
        let sheets = [sheet(
            "body { color: purple; font-weight: bold } \
             .lvl { padding: 1px; margin: 2px } div div div { color: red }",
        )];
        // Only 202 elements, so the auto policy stays sequential; force 8
        // threads to prove the chunked matcher reproduces the chain exactly.
        let seq = compute_styles_tracked_with(&dom, &sheets, ua(), false, 1, &HashSet::new()).0;
        let par = compute_styles_tracked_with(&dom, &sheets, ua(), false, 8, &HashSet::new()).0;
        assert_same_styles(&seq, &par, &dom);
        assert_eq!(seq.get(parent).unwrap().color, Color::rgb(255, 0, 0));
    }

    #[test]
    fn parallel_positional_pseudos_are_chunk_independent() {
        // 601 elements: the sibling scan for each `:nth-child` rule reads only
        // `dom.parent(id)` + `dom.children(parent)`, so splitting elements
        // across chunks must yield the same odd/even colours as one thread.
        let (dom, ids, sheets) = same_class_siblings(
            600,
            ".foo:nth-child(odd) { color: red }\n.foo:nth-child(even) { color: blue }",
        );
        let seq = compute_styles_tracked_with(&dom, &sheets, ua(), false, 1, &HashSet::new()).0;
        let par = compute_styles_tracked_with(&dom, &sheets, ua(), false, 0, &HashSet::new()).0;
        assert_same_styles(&seq, &par, &dom);
        assert!(LAST_MATCH_THREADS.with(|count| count.get()) >= 2);
        assert_eq!(par.get(ids[0]).unwrap().color, Color::rgb(255, 0, 0));
        assert_eq!(par.get(ids[1]).unwrap().color, Color::rgb(0, 0, 255));
    }

    #[test]
    fn matching_thread_threshold_is_512_elements() {
        assert_eq!(match_thread_count(0), 1);
        assert_eq!(match_thread_count(511), 1);
        assert!(match_thread_count(512) >= 2);

        let (small, _) = one_element("p");
        let _ = compute_styles(&small, &[]);
        assert_eq!(LAST_MATCH_THREADS.with(|count| count.get()), 1);

        let (below, sheets) = style_corpus(510, 1, false); // 511 elements
        let _ = compute_styles(&below, &sheets);
        assert_eq!(LAST_MATCH_THREADS.with(|count| count.get()), 1);

        let (at, sheets) = style_corpus(511, 1, false); // 512 elements
        let _ = compute_styles(&at, &sheets);
        assert!(LAST_MATCH_THREADS.with(|count| count.get()) >= 2);
    }

    /// Run with `cargo test --release -p hodo-style ab_matching_parallel -- --nocapture`.
    #[test]
    fn ab_matching_parallel_vs_sequential() {
        use std::time::Instant;

        let (dom, sheets) = style_corpus(2000, 200, false);
        let iterations = 5;
        let mut seq = std::time::Duration::ZERO;
        let mut par = std::time::Duration::ZERO;
        for _ in 0..iterations {
            let start = Instant::now();
            let a = compute_styles_tracked_with(&dom, &sheets, ua(), true, 1, &HashSet::new()).0;
            seq += start.elapsed();
            let start = Instant::now();
            let b = compute_styles_tracked_with(&dom, &sheets, ua(), true, 0, &HashSet::new()).0;
            par += start.elapsed();
            assert_eq!(a.styles.len(), b.styles.len());
        }
        let seq_ms = seq.as_secs_f64() * 1000.0 / iterations as f64;
        let par_ms = par.as_secs_f64() * 1000.0 / iterations as f64;
        eprintln!(
            "A/B matching (2000 elements, 200 rules): sequential {seq_ms:.2} ms, \
             parallel {par_ms:.2} ms ({:.2}x)",
            if par_ms > 0.0 { seq_ms / par_ms } else { 0.0 }
        );
    }

    // --- Style change classifier -------------------------------------------

    fn classify_change(mutate: impl FnOnce(&mut ComputedStyle)) -> StyleChangeClass {
        let old = initial_style();
        let mut new = old.clone();
        mutate(&mut new);
        style_change_class(&old, &new)
    }

    #[test]
    fn style_change_class_paint_only_fields() {
        use StyleChangeClass::PaintOnly;
        assert_eq!(
            classify_change(|s| s.color = Color::rgb(1, 2, 3)),
            PaintOnly
        );
        assert_eq!(
            classify_change(|s| s.background = Color::rgb(1, 2, 3)),
            PaintOnly
        );
        assert_eq!(
            classify_change(|s| s.border_color = Color::rgb(1, 2, 3)),
            PaintOnly
        );
        assert_eq!(classify_change(|s| s.underline = true), PaintOnly);
        assert_eq!(classify_change(|s| s.line_through = true), PaintOnly);
        // No change is safe to report as paint-only.
        assert_eq!(
            style_change_class(&initial_style(), &initial_style()),
            PaintOnly
        );
    }

    #[test]
    fn style_change_class_layout_affecting_fields() {
        use StyleChangeClass::LayoutAffecting as L;
        assert_eq!(classify_change(|s| s.display = Display::None), L);
        assert_eq!(classify_change(|s| s.font_size = 17.0), L);
        assert_eq!(classify_change(|s| s.font_weight = 700), L);
        assert_eq!(classify_change(|s| s.font_family = "serif".into()), L);
        assert_eq!(classify_change(|s| s.italic = true), L);
        assert_eq!(classify_change(|s| s.margin.top = 1.0), L);
        assert_eq!(classify_change(|s| s.padding.left = 1.0), L);
        assert_eq!(classify_change(|s| s.border_width = 2.0), L);
        assert_eq!(classify_change(|s| s.border_widths.right = 2.0), L);
        assert_eq!(
            classify_change(|s| s.flex_direction = FlexDirection::Column),
            L
        );
        assert_eq!(classify_change(|s| s.flex_wrap = FlexWrap::Wrap), L);
        assert_eq!(
            classify_change(|s| s.justify_content = JustifyContent::Center),
            L
        );
        assert_eq!(classify_change(|s| s.align_items = AlignItems::Center), L);
        assert_eq!(classify_change(|s| s.align_self = AlignSelf::Stretch), L);
        assert_eq!(classify_change(|s| s.gap = 3.0), L);
        assert_eq!(classify_change(|s| s.flex_grow = 1.0), L);
        assert_eq!(classify_change(|s| s.flex_shrink = 2.0), L);
        assert_eq!(
            classify_change(|s| s.flex_basis = FlexBasis::Length(4.0)),
            L
        );
        assert_eq!(classify_change(|s| s.order = 1), L);
        assert_eq!(
            classify_change(|s| s.grid_template_columns = vec![GridTrack::Fixed(1.0)]),
            L
        );
        assert_eq!(
            classify_change(|s| s.grid_template_rows = vec![GridTrack::Fixed(1.0)]),
            L
        );
        assert_eq!(
            classify_change(|s| s.grid_column = (GridLine::Line(1), GridLine::Auto)),
            L
        );
        assert_eq!(
            classify_change(|s| s.grid_row = (GridLine::Line(1), GridLine::Auto)),
            L
        );
        assert_eq!(classify_change(|s| s.column_gap = 5.0), L);
        assert_eq!(classify_change(|s| s.row_gap = 6.0), L);
        assert_eq!(classify_change(|s| s.table_layout = TableLayout::Fixed), L);
        assert_eq!(
            classify_change(|s| s.border_collapse = BorderCollapse::Collapse),
            L
        );
        assert_eq!(classify_change(|s| s.border_spacing = (3.0, 3.0)), L);
        assert_eq!(
            classify_change(|s| s.line_height = LineHeight::Number(2.0)),
            L
        );
        assert_eq!(
            classify_change(|s| s.vertical_align = VerticalAlign::Top),
            L
        );
        assert_eq!(classify_change(|s| s.text_align = TextAlign::Center), L);
        assert_eq!(classify_change(|s| s.text_indent = 7.0), L);
        assert_eq!(classify_change(|s| s.letter_spacing = 1.0), L);
        assert_eq!(classify_change(|s| s.word_spacing = 1.0), L);
        assert_eq!(classify_change(|s| s.white_space = WhiteSpace::Pre), L);
        assert_eq!(
            classify_change(|s| s.overflow_wrap = OverflowWrap::BreakWord),
            L
        );
        assert_eq!(classify_change(|s| s.position = Position::Absolute), L);
        assert_eq!(classify_change(|s| s.float = Float::Left), L);
        assert_eq!(classify_change(|s| s.clear = Clear::Both), L);
        assert_eq!(classify_change(|s| s.overflow = Overflow::Hidden), L);
        assert_eq!(classify_change(|s| s.offsets.bottom = Some(1.0)), L);
        assert_eq!(classify_change(|s| s.width = Some(10.0)), L);
        assert_eq!(classify_change(|s| s.width_percent = Some(50.0)), L);
        assert_eq!(classify_change(|s| s.margin_percent.left = Some(10.0)), L);
        assert_eq!(classify_change(|s| s.padding_percent.top = Some(10.0)), L);
        assert_eq!(classify_change(|s| s.height = Some(10.0)), L);
        assert_eq!(classify_change(|s| s.min_size = Size { w: 1.0, h: 1.0 }), L);
        assert_eq!(classify_change(|s| s.max_size = Size { w: 1.0, h: 1.0 }), L);
    }

    // --- Sharing extension measurement -------------------------------------

    /// Miniature of `hodo-perf`'s LARGE generator: 200 blocks with the same
    /// class / id / attribute rule shapes, short text (text cannot affect
    /// styles). Fixed LCG seed, no `rand` dependency.
    fn large_like_corpus() -> (Dom, Vec<Stylesheet>) {
        const BLOCKS: usize = 200;
        const CLASSES: usize = 50;
        const SPANS: usize = 25;
        let mut rng = 0x004c_4152_4745_0002u64;
        let mut next = || {
            rng = rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            rng
        };
        let mut hex = || format!("#{:06x}", next() & 0x00ff_ffff);
        let mut css = String::from(".root{display:block;padding:4px}\n");
        for i in 0..80 {
            let selector = if i % 2 == 0 {
                format!(".s{}", i % CLASSES)
            } else {
                format!(".sp{}", i % SPANS)
            };
            css.push_str(&format!(
                "{selector}{{color:{};background-color:{};padding:1px}}\n",
                hex(),
                hex()
            ));
        }
        for i in 0..60 {
            css.push_str(&format!(
                "#blk{i}{{border-width:1px;border-color:{};margin:2px}}\n",
                hex()
            ));
        }
        for i in 0..59 {
            let selector = match i % 3 {
                0 => "[data-kind=\"mixed\"]".to_string(),
                1 => "[width=\"120\"]".to_string(),
                _ => format!("[data-idx=\"{}\"]", i % 20),
            };
            css.push_str(&format!(
                "{selector}{{font-size:{}px;line-height:1.4}}\n",
                13 + i % 4
            ));
        }

        let mut dom = Dom::new();
        let root = dom.root();
        let main = element(&mut dom, root, "main", &[("class", "root")]);
        for block in 0..BLOCKS {
            let bclass = format!("b{}", block % 20);
            let block_id = format!("blk{block}");
            let block_idx = format!("{block}");
            let outer = element(
                &mut dom,
                main,
                "div",
                &[
                    ("class", &bclass),
                    ("id", &block_id),
                    ("data-kind", "mixed"),
                    ("data-idx", &block_idx),
                ],
            );
            let d1 = element(&mut dom, outer, "div", &[("class", "d1")]);
            let d2 = element(&mut dom, d1, "div", &[("class", "d2")]);
            let d3 = element(&mut dom, d2, "div", &[("class", "d3")]);
            let d4 = element(&mut dom, d3, "div", &[("class", "d4")]);
            for paragraph in 0..3 {
                let class = format!("s{}", (block * 3 + paragraph) % CLASSES);
                let p = element(&mut dom, d4, "p", &[("class", &class)]);
                for span in 0..if paragraph == 2 { 3 } else { 2 } {
                    let sclass = format!("sp{}", (block + span) % SPANS);
                    let _ = element(&mut dom, p, "span", &[("class", &sclass)]);
                }
            }
            let list = element(&mut dom, d4, "ul", &[("class", "list")]);
            let _ = element(&mut dom, list, "li", &[("class", "li0")]);
            let _ = element(&mut dom, list, "li", &[("class", "li1")]);
            let _ = element(&mut dom, d4, "img", &[("src", "pic.png"), ("width", "120")]);
        }
        (dom, vec![sheet(&css)])
    }

    /// Gate for rung 4: how much sharing would a subtree-content hash unlock
    /// beyond the current (matched-set + tag + inherited) key? Below 5% the
    /// extension is not worth building and is deliberately skipped.
    ///
    /// Measured on this LARGE-shaped corpus: 171 of 3801 elements (4.50%) have
    /// an identical computed style under a different key, while the existing
    /// cache already hits 3390/3801 (89.2%). 4.50% < 5%, so the subtree-content
    /// hash is NOT built. shortcut: skip subtree hashing; revisit only if the
    /// corpus changes to repeat whole subtrees without unique ids/data-idx.
    #[test]
    fn sharing_untapped_potential_large_corpus() {
        let (dom, sheets) = large_like_corpus();
        let (map, stats) = compute_styles_tracked(&dom, &sheets, ua(), true, &HashSet::new());
        let mut total = 0usize;
        let mut distinct_styles: HashSet<u64> = HashSet::new();
        let mut distinct_style_and_key: HashSet<(u64, String)> = HashSet::new();
        dom.walk(|id, node| {
            if !matches!(node.kind, NodeKind::Element { .. }) {
                return;
            }
            total += 1;
            let style_hash = hash_u64(&format!("{:?}", map.get(id)));
            // A conservative stand-in for the current SharingKey: unique ids
            // and data-idx attributes are exactly what splits LARGE's repeated
            // blocks into distinct matched sets today.
            let key = format!(
                "{:?}|{:?}|{:?}|{:?}",
                dom.tag(id),
                dom.attr(id, "id"),
                dom.attr(id, "class"),
                dom.attr(id, "data-idx")
            );
            distinct_styles.insert(style_hash);
            distinct_style_and_key.insert((style_hash, key));
        });
        let untapped = distinct_style_and_key.len() - distinct_styles.len();
        let potential = untapped as f64 / total as f64 * 100.0;
        eprintln!(
            "Sharing LARGE corpus: {total} elements, cache hits {}, misses {}, inserted {}, excluded {}; \
             distinct computed styles {}, distinct (style,key) {}, untapped {untapped} ({potential:.2}%)",
            stats.hits, stats.misses, stats.inserted, stats.excluded,
            distinct_styles.len(),
            distinct_style_and_key.len()
        );
    }
}

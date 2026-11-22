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
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use hodo_css::{
    complex, declaration_value, is_compound, is_important, matches_selector_ref,
    parse_style_attribute, parse_stylesheet, specificity_ref, AncestorBloom, Compound, Declaration,
    MatchContext, PseudoClass, RuleIndex, StyleRule, Stylesheet,
};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_types::{Color, Edges, Size};

type OrderedRule<'a> = (u8, usize, &'a StyleRule);
// (origin rank, specificity, source order, pre-split declaration references)
type Match<'a> = (u8, (u32, u32, u32), usize, &'a [&'a Declaration]);

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
    pub height: Option<f32>,
    pub min_size: Size,
    pub max_size: Size,
}

#[derive(Debug, Clone, Default)]
pub struct StyleMap {
    styles: Vec<Option<ComputedStyle>>,
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

    pub fn with_len(len: usize) -> Self {
        Self {
            styles: vec![None; len],
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
    let ua = UA_SHEET.get_or_init(|| parse_stylesheet(ua_stylesheet()));
    compute_styles_parsed(dom, sheets, ua, true)
}

#[cfg(test)]
fn compute_styles_inner(dom: &Dom, sheets: &[Stylesheet], ua_css: &str) -> StyleMap {
    let ua = parse_stylesheet(ua_css);
    compute_styles_parsed(dom, sheets, &ua, true)
}

/// Same resolution as `compute_styles`, but with style sharing turned off.
/// Used by the equality tests and the in-crate A/B harness.
#[cfg(test)]
fn compute_styles_uncached(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    let ua = UA_SHEET.get_or_init(|| parse_stylesheet(ua_stylesheet()));
    compute_styles_parsed(dom, sheets, ua, false)
}

fn compute_styles_parsed(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
) -> StyleMap {
    compute_styles_tracked(dom, sheets, ua, use_cache).0
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
fn match_one(
    dom: &Dom,
    ordered: &[OrderedRule<'_>],
    index: &RuleIndex,
    bloom: AncestorBloom,
    id: NodeId,
    candidates: &mut Vec<usize>,
) -> Vec<usize> {
    let mut ctx = MatchContext::new();
    ctx.ancestors = bloom;
    index.candidates(dom, id, candidates);
    let mut matched = Vec::new();
    for &candidate in candidates.iter() {
        let Some(&(_, _, rule)) = ordered.get(candidate) else {
            continue;
        };
        if matches_selector_ref(&rule.selector, dom, id, &mut ctx) {
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
    needs_bloom: bool,
    threads: usize,
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
        for &id in &elements {
            let slot = id.slot() as usize;
            matched[slot] = match_one(dom, ordered, index, blooms[slot], id, &mut candidates);
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
                for &id in ids {
                    let slot = id.slot() as usize;
                    slice[slot - base] =
                        match_one(dom, ordered, index, blooms[slot], id, &mut candidates);
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
) -> (StyleMap, SharingStats) {
    compute_styles_tracked_with(dom, sheets, ua, use_cache, 0)
}

/// `threads == 0` selects the policy automatically; tests force a value to
/// compare single-threaded and parallel matching byte-for-byte.
fn compute_styles_tracked_with(
    dom: &Dom,
    sheets: &[Stylesheet],
    ua: &Stylesheet,
    use_cache: bool,
    threads: usize,
) -> (StyleMap, SharingStats) {
    // (origin, source order, rule); origin 0 = UA, 1 = author.
    let mut ordered: Vec<OrderedRule<'_>> = Vec::new();
    let mut order = 0;
    for rule in &ua.rules {
        ordered.push((0, order, rule));
        order += 1;
    }
    for sheet in sheets {
        for rule in &sheet.rules {
            ordered.push((1, order, rule));
            order += 1;
        }
    }
    let index = RuleIndex::new(ordered.iter().map(|(_, _, rule)| *rule));
    let needs_bloom = ordered
        .iter()
        .any(|(_, _, rule)| !is_compound(&rule.selector));
    // Split declarations once per style pass, not once per element × rule.
    let prepared: Vec<(Vec<&Declaration>, Vec<&Declaration>)> = ordered
        .iter()
        .map(|(_, _, rule)| {
            let mut normal = Vec::new();
            let mut important = Vec::new();
            for declaration in &rule.declarations {
                if is_important(declaration) {
                    important.push(declaration);
                } else {
                    normal.push(declaration);
                }
            }
            (normal, important)
        })
        .collect();

    // Phase A — matching only: reads immutable DOM + sheets, writes one result
    // list per element slot. This is the part that runs in parallel.
    let mut matched = match_all(dom, &ordered, &index, needs_bloom, threads);

    // Phase B — cascade + style sharing stay sequential: they consume the
    // precomputed matched lists and share one mutable LRU / parent context.
    let mut map = StyleMap::with_len(dom.len());
    let mut rules = CascadeRules {
        ordered: &ordered,
        prepared: &prepared,
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
    // Pre-order guarantees a node's parent is already resolved when it is visited.
    dom.walk(|id, node| {
        let parent = dom
            .parent(id)
            .and_then(|p| map.get(p).cloned())
            .unwrap_or_else(initial_style);
        match node.kind {
            NodeKind::Text(_) => map.set(id, parent),
            NodeKind::Element { .. } => {
                map.set(
                    id,
                    cascade(
                        dom,
                        id,
                        &mut rules,
                        &mut matched[id.slot() as usize],
                        &parent,
                        cache.as_mut(),
                        &mut stats,
                    ),
                );
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

fn initial_style() -> ComputedStyle {
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
fn hash_inherited(style: &ComputedStyle) -> u64 {
    let mut h = DefaultHasher::new();
    (style.color.r, style.color.g, style.color.b, style.color.a).hash(&mut h);
    style.font_size.to_bits().hash(&mut h);
    style.font_weight.hash(&mut h);
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

/// Pure-std LRU: O(1) hits, O(capacity) eviction. `clock` gives each entry a
/// monotonically increasing stamp; the smallest stamp is least recently used.
struct SharingCache {
    entries: HashMap<SharingKey, (u64, ComputedStyle)>,
    clock: u64,
}

impl SharingCache {
    fn new() -> Self {
        Self {
            entries: HashMap::with_capacity(SHARING_CACHE_CAP),
            clock: 0,
        }
    }

    fn get(&mut self, key: &SharingKey) -> Option<ComputedStyle> {
        self.clock += 1;
        let stamp = self.clock;
        let entry = self.entries.get_mut(key)?;
        entry.0 = stamp;
        Some(entry.1.clone())
    }

    fn insert(&mut self, key: SharingKey, style: &ComputedStyle) {
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
                .min_by_key(|(_, (s, _))| *s)
                .map(|(key, _)| *key);
            if let Some(oldest) = oldest {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(key, (stamp, style.clone()));
    }
}

fn compound_has_positional(compound: &Compound) -> bool {
    compound.pseudos.iter().any(|pseudo| match pseudo {
        PseudoClass::FirstChild
        | PseudoClass::LastChild
        | PseudoClass::OnlyChild
        | PseudoClass::NthChild(_, _) => true,
        PseudoClass::Not(inner) => compound_has_positional(inner),
        _ => false,
    })
}

/// Conservative document-wide guard: sibling-position selectors make the set
/// of matched rules depend on where an element sits, not just on its tag and
/// class. With the matched-rule-set key this is strictly redundant, but the
/// guard keeps the cache sound if the key is ever coarsened toward Servo's
/// class-set-only both-hares trick.
fn sheet_has_positional_pseudo(ordered: &[OrderedRule<'_>]) -> bool {
    ordered.iter().any(|(_, _, rule)| {
        complex(&rule.selector)
            .compounds
            .iter()
            .any(compound_has_positional)
    })
}

fn cascade(
    dom: &Dom,
    id: NodeId,
    rules: &mut CascadeRules<'_>,
    matched_ids: &mut Vec<usize>,
    parent: &ComputedStyle,
    mut cache: Option<&mut SharingCache>,
    stats: &mut SharingStats,
) -> ComputedStyle {
    let mut normal: Vec<Match<'_>> = Vec::new();
    let mut important: Vec<Match<'_>> = Vec::new();
    // Inline declarations are per-element and are never shared.
    let can_share = cache.is_some() && dom.attr(id, "style").is_none();
    // Phase A already matched every candidate; only rule-order-independent
    // bookkeeping remains here (no re-matching, no context/sibling scans).
    for &candidate in matched_ids.iter() {
        let (origin, order, rule) = rules.ordered[candidate];
        let specificity = specificity_ref(&rule.selector);
        if !rules.prepared[candidate].0.is_empty() {
            normal.push((
                rank(origin, false),
                specificity,
                order,
                &rules.prepared[candidate].0,
            ));
        }
        if !rules.prepared[candidate].1.is_empty() {
            important.push((
                rank(origin, true),
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
            normal.push((rank(2, false), (0, 0, 0), usize::MAX / 2, &inline_normal));
        }
        if !inline_important.is_empty() {
            important.push((rank(2, true), (0, 0, 0), usize::MAX / 2, &inline_important));
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
            parent: hash_inherited(parent),
        };
        match cache.as_mut().unwrap().get(&key) {
            Some(style) => {
                stats.hits += 1;
                return style;
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
    // Stable sort: ties keep source order. Larger rank/specificity/order win last.
    normal.sort_by_key(|m| (m.0, m.1, m.2));
    important.sort_by_key(|m| (m.0, m.1, m.2));

    // Last declaration for each property wins after the cascade sort. Edge
    // shorthands are expanded to their four longhands as we walk in cascade
    // order, so a later longhand (or a later shorthand) wins by source order.
    let mut winning: HashMap<String, String> = HashMap::new();
    for m in normal.iter().chain(&important) {
        for decl in m.3 {
            let property = decl.property.trim().to_ascii_lowercase();
            let value = declaration_value(decl).to_string();
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

    let mut style = initial_style();
    // Inherited properties start from the parent's computed values.
    style.color = parent.color;
    style.font_size = parent.font_size;
    style.font_weight = parent.font_weight;
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
            "width" => set!(style.width, parse_optional_length(value, style.font_size)),
            "height" => set!(style.height, parse_optional_length(value, style.font_size)),
            "min-width" => set!(style.min_size.w, parse_length(value, style.font_size)),
            "min-height" => set!(style.min_size.h, parse_length(value, style.font_size)),
            "max-width" => set!(style.max_size.w, parse_length(value, style.font_size)),
            "max-height" => set!(style.max_size.h, parse_length(value, style.font_size)),
            _ => {}
        }
    }

    // Per-side lengths resolve against the element's own (now final) font size.
    apply_edges(&mut style.margin, &winning, MARGIN_KEYS, style.font_size);
    apply_edges(&mut style.padding, &winning, PADDING_KEYS, style.font_size);
    apply_edges(
        &mut style.border_widths,
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
        cache.as_mut().unwrap().insert(key, &style);
        stats.inserted += 1;
    }
    style
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
    winning: &HashMap<String, String>,
    keys: [&str; 4],
    font_size: f32,
) {
    let side = |key: &str, current: f32| {
        winning
            .get(key)
            .and_then(|v| parse_length(v, font_size))
            .unwrap_or(current)
    };
    *target = Edges {
        top: side(keys[0], target.top),
        right: side(keys[1], target.right),
        bottom: side(keys[2], target.bottom),
        left: side(keys[3], target.left),
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
    use hodo_css::Selector;

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
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
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

        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
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
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
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
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, &ua, true);
        let plain = compute_styles_parsed(&dom, &sheets, &ua, false);
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
        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
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

        let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
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
            let (shared, stats) = compute_styles_tracked(&dom, &sheets, ua(), true);
            on += start.elapsed();
            last_stats = stats;
            assert_eq!(shared.styles.len(), dom.len());
            let start = Instant::now();
            let plain = compute_styles_tracked(&dom, &sheets, ua(), false).0;
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
        let seq_uncached = compute_styles_tracked_with(dom, sheets, ua(), false, 1).0;
        let seq_cached = compute_styles_tracked_with(dom, sheets, ua(), true, 1).0;
        let parallel = compute_styles_tracked_with(dom, sheets, ua(), true, 0).0;
        assert_same_styles(&seq_uncached, &seq_cached, dom);
        assert_same_styles(&seq_uncached, &parallel, dom);
    }

    #[test]
    fn parallel_matches_sequential_on_2000_element_fixture() {
        let (dom, sheets) = style_corpus(2000, 200, false);
        assert_three_way(&dom, &sheets);
        let (_, stats) = compute_styles_tracked_with(&dom, &sheets, ua(), true, 0);
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
        let seq = compute_styles_tracked_with(&dom, &sheets, ua(), false, 1).0;
        let par = compute_styles_tracked_with(&dom, &sheets, ua(), false, 8).0;
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
        let seq = compute_styles_tracked_with(&dom, &sheets, ua(), false, 1).0;
        let par = compute_styles_tracked_with(&dom, &sheets, ua(), false, 0).0;
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
            let a = compute_styles_tracked_with(&dom, &sheets, ua(), true, 1).0;
            seq += start.elapsed();
            let start = Instant::now();
            let b = compute_styles_tracked_with(&dom, &sheets, ua(), true, 0).0;
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
}

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
//! - Vertical margins sum, never collapse. Unknown nodes are transparent
//!   containers.
//!
//! Implementation lives in `block` (block flow) and `inline` (inline
//! formatting); `table`, `flex`, `grid` and `forms` are reserved stubs.

mod block;
mod flex;
mod forms;
mod grid;
mod inline;
mod table;

pub use block::layout;

pub(crate) use flex::layout_flex;
pub(crate) use grid::layout_grid;
pub(crate) use inline::{
    feed_text, flush_line, has_content, inline_block_atom, left_inset, push_atom, right_inset,
    Atom, Flow, InlineBox,
};
pub(crate) use table::layout_table;

use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_style::{
    AlignItems, AlignSelf, BorderCollapse, Clear, ComputedStyle, Display, FlexBasis, FlexDirection,
    FlexWrap, Float, GridLine, JustifyContent, LineHeight, Overflow, OverflowWrap, Position,
    StyleMap, TableLayout, TextAlign, VerticalAlign, WhiteSpace,
};
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
        underline: bool,
        line_through: bool,
    },
    Image {
        rect: Rect,
        path: String,
    },
    Control {
        rect: Rect,
        kind: ControlKind,
        text: String,
        name: String,
        masked: bool,
        disabled: bool,
        readonly: bool,
    },
}

/// Semantic form control kinds emitted by form layout and drawn by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    TextInput,
    Checkbox,
    Radio,
    Button,
    Select,
    Textarea,
    Option,
}

#[derive(Debug, Clone, Default)]
pub struct DisplayList {
    pub items: Vec<DisplayItem>,
    /// Owning element of each display item: `node_map[i]` owns `items[i]`.
    /// Text items point at the text node's parent element (events target
    /// elements, not text nodes). Kept index-parallel to `items` so a shell
    /// can hit-test an item back to its DOM element.
    pub node_map: Vec<NodeId>,

    /// Fixed-position items, painted in a separate channel from the scrolled
    /// page items. `fixed_node_map[i]` owns `fixed_items[i]` by the same
    /// index-parallel rule as `items`/`node_map`.
    ///
    /// Frozen channel contract: the shell draws `items` first (scrolled), then
    /// `fixed_items` unshifted by scroll, then chrome/popup on top.
    pub fixed_items: Vec<DisplayItem>,
    pub fixed_node_map: Vec<NodeId>,
}

impl DisplayList {
    /// The single emission seam: every display item is pushed with its owner.
    pub(crate) fn push(&mut self, owner: NodeId, item: DisplayItem) {
        self.items.push(item);
        self.node_map.push(owner);
    }

    /// Emission seam for the fixed channel: index-parallel to `fixed_items`.
    pub(crate) fn push_fixed(&mut self, owner: NodeId, item: DisplayItem) {
        self.fixed_items.push(item);
        self.fixed_node_map.push(owner);
    }

    /// Append a scratch list, keeping both item/owner pairs index-parallel.
    /// Merging the fixed channel here lets fixed elements nested inside
    /// flex/grid/table scratch lists reach the root list.
    pub(crate) fn append(&mut self, mut other: DisplayList) {
        self.items.append(&mut other.items);
        self.node_map.append(&mut other.node_map);
        self.fixed_items.append(&mut other.fixed_items);
        self.fixed_node_map.append(&mut other.fixed_node_map);
    }
}

/// Borrowed inputs shared by the recursive layout walk.
pub(crate) struct Ctx<'a> {
    pub(crate) dom: &'a Dom,
    pub(crate) styles: &'a StyleMap,
    pub(crate) list: &'a mut DisplayList,
}

/// The available box handed to a block: left edge, top edge and usable width.
pub(crate) struct Avail {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
}

/// Style used for text nodes that have no computed style of their own.
pub(crate) fn fallback_style() -> ComputedStyle {
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
        height: None,
        min_size: Size { w: 0.0, h: 0.0 },
        max_size: Size {
            w: f32::INFINITY,
            h: f32::INFINITY,
        },
    }
}

pub(crate) fn push_rect(list: &mut DisplayList, owner: NodeId, rect: Rect, color: Color) -> usize {
    let idx = list.items.len();
    list.push(
        owner,
        DisplayItem::Rect {
            rect,
            color,
            border_color: None,
            radius: 0.0,
        },
    );
    idx
}

pub(crate) fn patch_rect(list: &mut DisplayList, idx: usize, f: impl FnOnce(&mut Rect)) {
    if let DisplayItem::Rect { rect, .. } = &mut list.items[idx] {
        f(rect);
    }
}

/// Route one non-fixed child to its display engine, returning the vertical
/// space it consumes. Mirrors the display match in `block::layout_child_inner`
/// so flex/grid/table item sites agree on which engine paints a child.
/// Callers handle line flushing, floats and relative offsets themselves.
pub(crate) fn dispatch_child(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    link: Option<String>,
) -> f32 {
    let tag = ctx.dom.tag(id).unwrap_or("");
    if forms::is_form_control(tag) {
        return forms::layout_control_atom(ctx, style, id, avail, link);
    }
    match style.display {
        Display::Flex | Display::InlineFlex => layout_flex(ctx, style, id, avail, link),
        Display::Grid | Display::InlineGrid => layout_grid(ctx, style, id, avail, link),
        Display::Table | Display::InlineTable => layout_table(ctx, style, id, avail, link),
        Display::InlineBlock => inline_block_atom(ctx, style, id, avail, link),
        _ => block::layout_block(ctx, style, id, avail, link),
    }
}

/// Whether a child participates as a block-level sibling for anonymous-block
/// splitting. `<img>` is an atomic replaced element in this engine.
pub(crate) fn is_block_level(ctx: &Ctx<'_>, id: NodeId) -> bool {
    // Text nodes inherit their parent's computed `display`, so only elements
    // may be classified as block-level siblings here.
    matches!(ctx.dom.get(id).kind, NodeKind::Element { .. })
        && (ctx.styles.get(id).is_some_and(|s| {
            matches!(
                s.display,
                Display::Block | Display::Flex | Display::Grid | Display::Table
            )
        }) || ctx
            .dom
            .tag(id)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("img")))
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

    #[test]
    fn flex_grid_table_stubs_render_without_panic() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "flexbox");
        text(&mut dom, f, "flex content");
        let g = elem(&mut dom, root, "gridbox");
        text(&mut dom, g, "grid content");
        let t = elem(&mut dom, root, "tablebox");
        text(&mut dom, t, "table content");

        let styles = styled(
            &dom,
            vec![
                tag_rule("flexbox", &[("display", "flex")]),
                tag_rule("gridbox", &[("display", "grid")]),
                tag_rule("tablebox", &[("display", "table")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        // M7 engines landed: flex/grid/table are real now — render without panic
        // and paint actual content.
        assert!(
            !list.items.is_empty(),
            "flex/grid/table containers must paint their content"
        );
    }

    // ---- node_map: display item -> owning element -------------------------

    fn owners_for(list: &DisplayList, pick: impl Fn(&DisplayItem) -> bool) -> Vec<NodeId> {
        list.items
            .iter()
            .enumerate()
            .filter(|(_, item)| pick(item))
            .map(|(i, _)| list.node_map[i])
            .collect()
    }

    #[test]
    fn node_map_text_item_maps_to_parent_element() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        text(&mut dom, d, "hello");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        let owners = owners_for(&list, |it| matches!(it, DisplayItem::Text { .. }));
        assert_eq!(
            owners,
            vec![d],
            "text item must be owned by its parent element"
        );
        assert_eq!(list.items.len(), list.node_map.len());
    }

    #[test]
    fn node_map_nested_inline_spans_map_each_to_their_own_element() {
        let mut dom = Dom::new();
        let root = dom.root();
        let d = elem(&mut dom, root, "div");
        let outer = elem_with_attrs(&mut dom, d, "span", &[("style", "background:#0000ff")]);
        let inner = elem_with_attrs(&mut dom, outer, "span", &[("style", "background:#00ff00")]);
        text(&mut dom, inner, "inner");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        // Text inside a span is owned by the span (its parent element), not by
        // the text node nor by the outer inline box.
        assert_eq!(
            owners_for(&list, |it| matches!(it, DisplayItem::Text { .. })),
            vec![inner]
        );

        // Each span's painted fragment is owned by that span.
        let blue = Color::rgb(0, 0, 255);
        let green = Color::rgb(0, 255, 0);
        let outer_bg = owners_for(
            &list,
            |it| matches!(it, DisplayItem::Rect { color, .. } if *color == blue),
        );
        let inner_bg = owners_for(
            &list,
            |it| matches!(it, DisplayItem::Rect { color, .. } if *color == green),
        );
        assert!(!outer_bg.is_empty() && outer_bg.iter().all(|&o| o == outer));
        assert!(!inner_bg.is_empty() && inner_bg.iter().all(|&o| o == inner));
        assert_eq!(list.items.len(), list.node_map.len());
    }

    #[test]
    fn node_map_flex_children_map_to_their_own_elements() {
        let mut dom = Dom::new();
        let root = dom.root();
        let f = elem(&mut dom, root, "div");
        let a = elem(&mut dom, f, "item");
        text(&mut dom, a, "A");
        let b = elem(&mut dom, f, "item");
        text(&mut dom, b, "B");
        let styles = styled(
            &dom,
            vec![
                tag_rule("div", &[("display", "flex")]),
                tag_rule("item", &[("display", "block"), ("background", "#ff0000")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let red = Color::rgb(255, 0, 0);
        assert_eq!(
            owners_for(&list, |it| {
                matches!(it, DisplayItem::Rect { color, .. } if *color == red)
            }),
            vec![a, b],
            "each flex item's box is owned by that item"
        );
        assert_eq!(
            owners_for(&list, |it| matches!(it, DisplayItem::Text { .. })),
            vec![a, b]
        );
    }

    #[test]
    fn node_map_grid_children_map_to_their_own_elements() {
        let mut dom = Dom::new();
        let root = dom.root();
        let g = elem(&mut dom, root, "div");
        let a = elem(&mut dom, g, "item");
        text(&mut dom, a, "A");
        let b = elem(&mut dom, g, "item");
        text(&mut dom, b, "B");
        let styles = styled(
            &dom,
            vec![
                tag_rule(
                    "div",
                    &[
                        ("display", "grid"),
                        ("grid-template-columns", "100px 100px"),
                    ],
                ),
                tag_rule("item", &[("display", "block"), ("background", "#ff0000")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let red = Color::rgb(255, 0, 0);
        assert_eq!(
            owners_for(&list, |it| {
                matches!(it, DisplayItem::Rect { color, .. } if *color == red)
            }),
            vec![a, b],
            "each grid item's box is owned by that item"
        );
    }

    #[test]
    fn node_map_table_cells_map_to_their_own_elements() {
        let mut dom = Dom::new();
        let root = dom.root();
        let t = elem(&mut dom, root, "table");
        let tr = elem(&mut dom, t, "tr");
        let a = elem(&mut dom, tr, "td");
        text(&mut dom, a, "A");
        let b = elem(&mut dom, tr, "td");
        text(&mut dom, b, "B");
        let styles = styled(&dom, vec![tag_rule("td", &[("background", "#ff0000")])]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        let red = Color::rgb(255, 0, 0);
        assert_eq!(
            owners_for(&list, |it| {
                matches!(it, DisplayItem::Rect { color, .. } if *color == red)
            }),
            vec![a, b],
            "each table cell's box is owned by that cell"
        );
        assert_eq!(
            owners_for(&list, |it| matches!(it, DisplayItem::Text { .. })),
            vec![a, b]
        );
    }

    #[test]
    fn node_map_form_controls_map_to_their_own_elements() {
        let mut dom = Dom::new();
        let root = dom.root();
        let a = elem_with_attrs(&mut dom, root, "input", &[("name", "a")]);
        let b = elem_with_attrs(&mut dom, root, "button", &[("name", "b")]);
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert_eq!(
            owners_for(&list, |it| matches!(it, DisplayItem::Control { .. })),
            vec![a, b],
            "each control is owned by its own element"
        );
    }

    #[test]
    fn node_map_image_maps_to_img_element() {
        let mut dom = Dom::new();
        let root = dom.root();
        let img = elem_with_attrs(&mut dom, root, "img", &[("src", "/p.png")]);
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        assert_eq!(
            owners_for(&list, |it| matches!(it, DisplayItem::Image { .. })),
            vec![img]
        );
        assert_eq!(list.items.len(), list.node_map.len());
    }

    #[test]
    fn node_map_parity_holds_at_200_levels_of_nesting() {
        let mut dom = Dom::new();
        let root = dom.root();
        let mut ids = Vec::new();
        let mut cur = root;
        for _ in 0..200 {
            cur = elem(&mut dom, cur, "div");
            ids.push(cur);
        }
        text(&mut dom, cur, "deep");
        let styles = styled(&dom, vec![tag_rule("div", &[("background", "#ff0000")])]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        assert_eq!(list.items.len(), list.node_map.len());
        assert!(
            list.node_map.iter().all(|owner| ids.contains(owner)),
            "every owner must be one of the created elements"
        );
    }

    #[test]
    fn node_map_styled_fixture_asserts_concrete_ids_at_known_indices() {
        let mut dom = Dom::new();
        let root = dom.root();
        let outer = elem(&mut dom, root, "div");
        let a = elem(&mut dom, outer, "span");
        text(&mut dom, a, "A");
        let b = elem(&mut dom, outer, "span");
        text(&mut dom, b, "B");
        let styles = styled(
            &dom,
            vec![
                tag_rule("div", &[("background", "#ff0000")]),
                tag_rule("span", &[("background", "#0000ff")]),
            ],
        );
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        assert_eq!(list.node_map.len(), 4);
        assert_eq!(list.node_map[0], outer, "div background");
        assert_eq!(list.node_map[1], a, "first span background");
        assert_eq!(list.node_map[2], b, "second span background");
        assert_eq!(
            list.node_map[3], a,
            "line Text owned by first glyph's element"
        );
    }

    // ---- fixed channel: position:fixed never rides the scrolled item flow ---

    fn fixed_texts(list: &DisplayList) -> Vec<&str> {
        list.fixed_items
            .iter()
            .filter_map(|it| match it {
                DisplayItem::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn fixed_element_items_land_in_fixed_channel_not_page_items() {
        let mut dom = Dom::new();
        let root = dom.root();
        let fixed = elem_with_attrs(
            &mut dom,
            root,
            "div",
            &[(
                "style",
                "position:fixed; left:10px; top:5px; background:#ff0000",
            )],
        );
        text(&mut dom, fixed, "pinned");
        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        assert!(
            list.items.is_empty(),
            "fixed element must not emit into scrolled page items: {:?}",
            list.items
        );
        assert!(
            !list.fixed_items.is_empty(),
            "fixed element must emit into fixed_items"
        );
        assert_eq!(fixed_texts(&list), ["pinned"]);
        assert_eq!(list.fixed_items.len(), list.fixed_node_map.len());
        assert!(
            list.fixed_node_map.iter().all(|&o| o == fixed),
            "every fixed item must be owned by the fixed element"
        );
    }

    #[test]
    fn fixed_nested_in_blocks_and_flex_bubbles_to_root_fixed_channel() {
        let mut dom = Dom::new();
        let root = dom.root();
        let outer = elem_with_attrs(&mut dom, root, "div", &[("style", "display:block")]);
        let mid = elem_with_attrs(&mut dom, outer, "div", &[("style", "display:block")]);
        let deep = elem_with_attrs(
            &mut dom,
            mid,
            "div",
            &[("style", "position:fixed; left:7px; background:#ff0000")],
        );
        text(&mut dom, deep, "deep-fixed");

        let flex = elem_with_attrs(&mut dom, root, "div", &[("style", "display:flex")]);
        let flex_fixed = elem_with_attrs(
            &mut dom,
            flex,
            "div",
            &[(
                "style",
                "position:fixed; left:9px; width:100px; background:#00ff00",
            )],
        );
        text(&mut dom, flex_fixed, "flex-fixed");

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        assert!(
            list.items.is_empty(),
            "only fixed content exists; page items must stay empty: {:?}",
            list.items
        );
        assert_eq!(
            fixed_texts(&list),
            ["deep-fixed", "flex-fixed"],
            "fixed nested two blocks deep and in a flex item must reach the root fixed channel"
        );
        assert_eq!(list.fixed_items.len(), list.fixed_node_map.len());
    }

    #[test]
    fn mixed_static_fixed_absolute_channels_are_separate() {
        let mut dom = Dom::new();
        let root = dom.root();
        let stat = elem_with_attrs(&mut dom, root, "div", &[("style", "background:#0000ff")]);
        text(&mut dom, stat, "static");
        let abs = elem_with_attrs(
            &mut dom,
            root,
            "div",
            &[("style", "position:absolute; left:40px; background:#ff0000")],
        );
        text(&mut dom, abs, "absolute");
        let fix = elem_with_attrs(
            &mut dom,
            root,
            "div",
            &[("style", "position:fixed; left:70px; background:#00ff00")],
        );
        text(&mut dom, fix, "fixed");

        let styles = compute_styles(&dom, &[]);
        let list = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });

        assert_eq!(
            texts(&list),
            ["static", "absolute"],
            "page items keep static + absolute content only"
        );
        assert_eq!(
            fixed_texts(&list),
            ["fixed"],
            "fixed channel holds only fixed content"
        );
        assert_eq!(list.items.len(), list.node_map.len());
        assert_eq!(list.fixed_items.len(), list.fixed_node_map.len());
        assert!(
            list.fixed_node_map.iter().all(|&o| o == fix),
            "fixed items must be owned by the fixed element"
        );
    }
}

//! Block-flow layout producing a flat display list.
//!
//! MVP scope and documented limits:
//! - Layout is a pure, pre-order walk of the DOM; it keeps no global state.
//! - Block elements produce a background rect (when non-transparent) and,
//!   optionally, four thin border rects. Nested blocks stack vertically.
//! - Text is flowed into visual lines using approximate metrics:
//!   char width = 0.6 * font_size, line height = 1.2 * font_size.
//! - Inline elements do not create boxes: their text is folded into the
//!   surrounding block's inline flow.
//! - Vertical margins are summed, never collapsed.
//! - Unknown/unlisted nodes are treated as transparent containers.

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
    },
}

#[derive(Debug, Clone, Default)]
pub struct DisplayList {
    pub items: Vec<DisplayItem>,
}

/// Block-flow layout. Approximate metrics: char width = 0.6 * font_size,
/// line height = 1.2 * font_size.
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
        layout_children(&mut ctx, &mut flow, &fallback, dom.root());
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

/// One rendered word (or a single collapsed space) waiting to be placed.
struct Atom {
    text: String,
    font_size: f32,
    color: Color,
    weight: u16,
}

/// Mutable inline-flow state for a single block's content area.
struct Flow {
    left: f32,
    right: f32,
    y: f32,
    line: Vec<Atom>,
    line_w: f32,
    pending_space: Option<f32>,
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
        }
    }
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

/// Walk a node's children in order, stacking blocks and flowing text.
fn layout_children(ctx: &mut Ctx<'_>, flow: &mut Flow, fallback: &ComputedStyle, parent: NodeId) {
    let children: Vec<NodeId> = ctx.dom.children(parent).to_vec();
    for child in children {
        let s = ctx.styles.get(child).cloned();
        if s.as_ref().is_some_and(|st| st.display == Display::None) {
            continue;
        }
        match &ctx.dom.get(child).kind {
            NodeKind::Text(t) => {
                let st = s.as_ref().unwrap_or(fallback);
                feed_text(flow, ctx.list, t, st);
            }
            NodeKind::Element { .. } => {
                if s.as_ref().is_some_and(|st| st.display == Display::Block) {
                    flush_line(flow, ctx.list);
                    let avail = Avail {
                        x: flow.left,
                        y: flow.y,
                        width: flow.right - flow.left,
                    };
                    let consumed = layout_block(ctx, s.as_ref().unwrap(), child, avail);
                    // shortcut: vertical margins sum, no collapsing (MVP); upgrade when
                    // CSS margin collapsing is required for spec fidelity.
                    flow.y += consumed;
                } else {
                    // shortcut: inline elements create no box; their text joins this
                    // block's flow. Upgrade for true inline box layout/vertical alignment.
                    layout_children(ctx, flow, fallback, child);
                }
            }
            NodeKind::Document => layout_children(ctx, flow, fallback, child),
        }
    }
}

/// Lay out one block box and return the total vertical space it consumes,
/// including its own top and bottom margins.
fn layout_block(ctx: &mut Ctx<'_>, style: &ComputedStyle, id: NodeId, avail: Avail) -> f32 {
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
    layout_children(ctx, &mut flow, style, id);
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

/// Commit the current visual line as one Text item and advance the y cursor.
fn flush_line(flow: &mut Flow, list: &mut Vec<DisplayItem>) {
    if flow.line.is_empty() {
        return;
    }
    let first = &flow.line[0];
    let font_size = first.font_size;
    let color = first.color;
    let weight = first.weight;
    let mut text = String::new();
    let mut line_h = 0.0_f32;
    for atom in &flow.line {
        text.push_str(&atom.text);
        line_h = line_h.max(1.2 * atom.font_size);
    }
    list.push(DisplayItem::Text {
        rect: Rect {
            x: flow.left,
            y: flow.y,
            w: flow.line_w,
            h: line_h,
        },
        text,
        color,
        font_size,
        weight,
    });
    flow.y += line_h;
    flow.line.clear();
    flow.line_w = 0.0;
    flow.pending_space = None;
}

/// Place a word into the flow, wrapping at the content width. Words longer
/// than a whole line are hard-broken into as many characters as fit.
fn place_word(flow: &mut Flow, list: &mut Vec<DisplayItem>, atom: Atom) {
    let cw = 0.6 * atom.font_size;
    let max_w = flow.right - flow.left;
    let mut word: &str = &atom.text;
    loop {
        if word.is_empty() {
            break;
        }
        let word_w = word.chars().count() as f32 * cw;
        let space_w = flow.pending_space.unwrap_or(0.0);
        if !flow.line.is_empty() && flow.line_w + space_w + word_w > max_w {
            flush_line(flow, list);
            flow.pending_space = None;
            continue;
        }
        if word_w > max_w {
            let n = ((max_w / cw).floor() as usize).max(1);
            let mut end = 0;
            for (count, (i, ch)) in word.char_indices().enumerate() {
                if count == n {
                    break;
                }
                end = i + ch.len_utf8();
            }
            let (chunk, rest) = word.split_at(end);
            flow.line.push(Atom {
                text: chunk.to_string(),
                font_size: atom.font_size,
                color: atom.color,
                weight: atom.weight,
            });
            flow.line_w += chunk.chars().count() as f32 * cw;
            flush_line(flow, list);
            flow.pending_space = None;
            word = rest;
            continue;
        }
        if let Some(sw) = flow.pending_space {
            if !flow.line.is_empty() {
                flow.line.push(Atom {
                    text: " ".to_string(),
                    font_size: atom.font_size,
                    color: atom.color,
                    weight: atom.weight,
                });
                flow.line_w += sw;
            }
        }
        flow.pending_space = None;
        flow.line.push(Atom {
            text: word.to_string(),
            font_size: atom.font_size,
            color: atom.color,
            weight: atom.weight,
        });
        flow.line_w += word_w;
        break;
    }
}

/// Feed a text node into the inline flow: whitespace runs collapse to a
/// single space, pure-whitespace nodes are skipped.
fn feed_text(flow: &mut Flow, list: &mut Vec<DisplayItem>, text: &str, style: &ComputedStyle) {
    if text.chars().all(char::is_whitespace) {
        return;
    }
    let fs = style.font_size;
    let color = style.color;
    let weight = style.font_weight;
    if !flow.line.is_empty() && text.chars().next().is_some_and(char::is_whitespace) {
        flow.pending_space = Some(0.6 * fs);
    }
    let mut first = true;
    for word in text.split_whitespace() {
        if !first && !flow.line.is_empty() {
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
        let id = dom.create(NodeKind::Element {
            tag: tag.to_string(),
            attrs: Vec::new(),
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
}

//! Style resolution: UA sheet + author sheets, cascaded by specificity and order.

use std::collections::HashMap;

use hodo_css::{
    matches_element, parse_stylesheet, specificity, Declaration, StyleRule, Stylesheet,
};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_types::{Color, Edges};

type OrderedRule<'a> = (u32, usize, &'a StyleRule);
type Match<'a> = (u32, (u32, u32, u32), usize, &'a [Declaration]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    None,
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
}

#[derive(Debug, Clone, Default)]
pub struct StyleMap {
    styles: Vec<Option<ComputedStyle>>,
}

impl StyleMap {
    pub fn get(&self, id: NodeId) -> Option<&ComputedStyle> {
        self.styles.get(id.0 as usize).and_then(Option::as_ref)
    }

    pub(crate) fn set(&mut self, id: NodeId, s: ComputedStyle) {
        let index = id.0 as usize;
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
     td { display: inline; }\n"
}

/// Apply the UA sheet first, then author sheets; cascade by specificity then order.
/// `color` and `font_size` inherit from the parent.
pub fn compute_styles(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    let ua = parse_stylesheet(ua_stylesheet());

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

    let mut map = StyleMap::with_len(dom.len());
    // Pre-order guarantees a node's parent is already resolved when it is visited.
    dom.walk(|id, node| {
        let parent = dom
            .parent(id)
            .and_then(|p| map.get(p).cloned())
            .unwrap_or_else(initial_style);
        match node.kind {
            NodeKind::Text(_) => map.set(id, parent),
            NodeKind::Element { .. } => map.set(id, cascade(dom, id, &ordered, &parent)),
            NodeKind::Document => {}
        }
    });
    map
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
    }
}

fn cascade(
    dom: &Dom,
    id: NodeId,
    ordered: &[OrderedRule<'_>],
    parent: &ComputedStyle,
) -> ComputedStyle {
    let mut matched: Vec<Match<'_>> = Vec::new();
    for &(origin, order, rule) in ordered {
        if matches_element(&rule.selector, dom, id) {
            matched.push((
                origin,
                specificity(&rule.selector),
                order,
                &rule.declarations,
            ));
        }
    }
    // Stable sort: ties keep source order. Larger origin/specificity/order win last.
    matched.sort_by_key(|m| (m.0, m.1, m.2));

    // Last declaration for each property wins after the cascade sort. Edge
    // shorthands are expanded to their four longhands as we walk in cascade
    // order, so a later longhand (or a later shorthand) wins by source order.
    let mut winning: HashMap<String, String> = HashMap::new();
    for m in &matched {
        for decl in m.3 {
            let property = decl.property.trim().to_ascii_lowercase();
            let value = decl.value.trim().to_string();
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

    if let Some(v) = winning.get("display") {
        style.display = match v.to_ascii_lowercase().as_str() {
            "block" => Display::Block,
            "none" => Display::None,
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
        style.underline = v.to_ascii_lowercase().contains("underline");
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
    style
}

fn parse_length(value: &str, font_size: f32) -> Option<f32> {
    let value = value.trim();
    if let Some(n) = value.strip_suffix("px") {
        n.trim().parse().ok()
    } else if let Some(n) = value.strip_suffix("em") {
        n.trim().parse::<f32>().ok().map(|n| n * font_size)
    } else if let Some(n) = value.strip_suffix("pt") {
        n.trim().parse::<f32>().ok().map(|n| n * 4.0 / 3.0)
    } else {
        value.parse().ok()
    }
}

fn parse_font_size(value: &str, parent_size: f32) -> f32 {
    if let Some(n) = value.trim().strip_suffix('%') {
        return n
            .trim()
            .parse::<f32>()
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
}

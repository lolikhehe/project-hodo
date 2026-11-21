//! Style resolution: UA sheet + author sheets + inline `style`, cascaded by
//! origin, `!important`, specificity and source order.
//!
//! Limits inherited from `hodo-css`: at most 64 compounds per selector, `:not()`
//! depth 2, 10 000 rules per sheet. Each element tests only rules selected by
//! the rightmost-compound index; descendant rules are fast-rejected by a
//! 64-bit ancestor Bloom filter, sibling rules use a cached sibling scan.
//! Matching is iterative and a 1000-level DOM stays under the 1 s test budget.

use std::collections::HashMap;

use hodo_css::{
    declaration_value, is_compound, is_important, matches_selector_ref, parse_style_attribute,
    parse_stylesheet, specificity_ref, AncestorBloom, Declaration, MatchContext, RuleIndex,
    StyleRule, Stylesheet,
};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_types::{Color, Edges};

type OrderedRule<'a> = (u8, usize, &'a StyleRule);
// (origin rank, specificity, source order, pre-split declaration references)
type Match<'a> = (u8, (u32, u32, u32), usize, &'a [&'a Declaration]);

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
     td { display: inline; }\n"
}

/// Apply the UA sheet, then author sheets, then the element's inline `style`
/// attribute. Cascade order is UA normal < author normal < inline normal <
/// author `!important` < inline `!important` < UA `!important`, then
/// specificity, then source order. `color` and `font_size` inherit.
pub fn compute_styles(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    compute_styles_inner(dom, sheets, ua_stylesheet())
}

fn compute_styles_inner(dom: &Dom, sheets: &[Stylesheet], ua_css: &str) -> StyleMap {
    let ua = parse_stylesheet(ua_css);

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

    let mut map = StyleMap::with_len(dom.len());
    let mut blooms = vec![AncestorBloom::default(); dom.len()];
    let mut ctx = MatchContext::new();
    let mut rules = CascadeRules {
        ordered: &ordered,
        index: &index,
        prepared: &prepared,
        candidates: Vec::new(),
    };
    // Pre-order guarantees a node's parent is already resolved when it is visited.
    dom.walk(|id, node| {
        let parent = dom
            .parent(id)
            .and_then(|p| map.get(p).cloned())
            .unwrap_or_else(initial_style);
        match node.kind {
            NodeKind::Text(_) => map.set(id, parent),
            NodeKind::Element { .. } => {
                let ancestors = if needs_bloom {
                    dom.parent(id)
                        .map(|p| AncestorBloom::child_of(blooms[p.slot() as usize], dom, p))
                        .unwrap_or_default()
                } else {
                    AncestorBloom::default()
                };
                if let Some(slot) = blooms.get_mut(id.slot() as usize) {
                    *slot = ancestors;
                }
                ctx.ancestors = ancestors;
                map.set(id, cascade(dom, id, &mut rules, &mut ctx, &parent));
            }
            NodeKind::Document => {}
        }
    });
    map
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
    }
}

/// Per-pass rule data shared by every element's cascade.
struct CascadeRules<'a> {
    ordered: &'a [OrderedRule<'a>],
    index: &'a RuleIndex,
    prepared: &'a [(Vec<&'a Declaration>, Vec<&'a Declaration>)],
    candidates: Vec<usize>,
}

fn cascade(
    dom: &Dom,
    id: NodeId,
    rules: &mut CascadeRules<'_>,
    ctx: &mut MatchContext,
    parent: &ComputedStyle,
) -> ComputedStyle {
    let mut normal: Vec<Match<'_>> = Vec::new();
    let mut important: Vec<Match<'_>> = Vec::new();
    rules.index.candidates(dom, id, &mut rules.candidates);
    for &candidate in &rules.candidates {
        let Some(&(origin, order, rule)) = rules.ordered.get(candidate) else {
            continue;
        };
        if !matches_selector_ref(&rule.selector, dom, id, ctx) {
            continue;
        }
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
}

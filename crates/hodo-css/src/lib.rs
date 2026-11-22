//! CSS parsing, selector matching and cascade support.
//!
//! Limits (local constants, pending centralization in `hodo-types`): at most
//! 64 compounds per selector, `:not()` nesting depth 2, at most 10 000 rules per
//! stylesheet, and a 64-bit ancestor Bloom filter for descendant rejection.
//! Matching walks ancestors/siblings iteratively; a 1000-level DOM with the
//! longest allowed descendant selector completes in well under the 1 s budget
//! asserted in tests. An invalid selector drops only its own rule.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};

use cssparser::{Delimiter, ParseError, Parser, ToCss, Token};
use hodo_dom::{Dom, NodeId};

/// Maximum number of compounds in one selector (spec starting proposal).
pub const MAX_COMPOUNDS: usize = 64;
/// Maximum `:not()` nesting depth.
pub const MAX_NOT_DEPTH: u8 = 2;
/// Maximum number of qualified rules kept from one stylesheet.
/// Deliberately stricter than `hodo_types::limits::MAX_CSS_RULES` (100 000):
/// that table value bounds the global rule index, while this per-sheet parse
/// cap stays local until the two budgets are reconciled.
pub const MAX_RULES: usize = 10_000;
/// Nested-parenthesis cap inside `calc()` expressions.
const MAX_CALC_DEPTH: u8 = 4;
/// `em`/`rem` inside `calc()` resolve at parse time, where no element font-size
/// context exists. TODO: thread the computed font size and use it here.
const CALC_FALLBACK_FONT_PX: f64 = 16.0;
/// Rendered device scale: one CSS pixel per device pixel.
const DEVICE_DPPX: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

/// Media-feature evaluation context for `@media` blocks **and** the descriptor
/// viewport units (`vh`/`vw`/`vmin`/`vmax`) resolve against while parsing.
/// `parse_stylesheet` uses [`Viewport::default`]; callers with a real window
/// should pass theirs through [`parse_stylesheet_with`]. Lengths resolve at
/// parse time into fixed `px` text (see [`resolve_lengths`]); a resize
/// mid-session does not re-resolve already-parsed declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub media_type: MediaType,
    /// Reserved for a future `scripting` feature; wave-1 queries ignore it.
    pub scripting_enabled: bool,
    /// Shell integrators thread the chrome dark-theme flag here so
    /// `prefers-color-scheme` matches the visible theme; default is light.
    pub dark: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            media_type: MediaType::Screen,
            scripting_enabled: true,
            dark: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MediaType {
    #[default]
    Screen,
    Print,
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

/// A compound selector (tag, `#id`, `.class`, `*`), historically also used as
/// the whole selector. Complex chains are carried alongside the rightmost
/// compound via an internal marker in `classes` so this public shape stays
/// compatible with hand-built selectors in other crates.
#[derive(Debug, Clone)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub universal: bool,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    /// Value text; a trailing `! important` is retained and interpreted by
    /// [`is_important`] / [`declaration_value`].
    pub value: String,
}

/// Synthetic declaration property carrying a rule's per-sheet cascade-layer
/// rank. Kept in `Declaration` so the public `StyleRule` shape stays stable for
/// the other crates that build rules by hand; `hodo-style` reads it via
/// [`cascade_layer`] and skips it when cascading.
pub const LAYER_MARKER: &str = "\0hodo-layer";
/// Synthetic declaration carrying the layer's full dotted name. Paired with
/// [`LAYER_MARKER`] so `hodo-style` can resolve one GLOBAL layer order across
/// all sheets (same name in two sheets merges into the first-seen rank).
pub const LAYER_NAME_MARKER: &str = "\0hodo-layer-name";

/// Per-sheet cascade-layer rank of a rule (`None` = unlayered), recorded at
/// parse time. Use [`cascade_layer_name`] for cross-sheet resolution.
pub fn cascade_layer(rule: &StyleRule) -> Option<u32> {
    rule.declarations
        .iter()
        .find(|decl| decl.property == LAYER_MARKER)
        .and_then(|decl| decl.value.parse().ok())
}

/// Full dotted layer name of a rule (`None` = unlayered). Nested layers are
/// composed as `outer.inner`; comma lists duplicate the rule into each name.
pub fn cascade_layer_name(rule: &StyleRule) -> Option<&str> {
    rule.declarations
        .iter()
        .find(|decl| decl.property == LAYER_NAME_MARKER)
        .map(|decl| decl.value.as_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
    Adjacent,
    GeneralSibling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
    Exists,
    Equal,
    Includes,
    DashMatch,
    Prefix,
    Suffix,
    Substring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    pub name: String,
    pub op: AttrOp,
    pub value: String,
    pub case_insensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoClass {
    FirstChild,
    LastChild,
    NthChild(i32, i32),
    OnlyChild,
    Not(Box<Compound>),
    /// `:where()` — matches any inner compound, zero specificity.
    Where(Vec<Compound>),
    /// `:is()` — matches any inner compound, specificity of the best one.
    Is(Vec<Compound>),
    /// `:link` — an `a[href]` whose href is not in `MatchContext::visited`.
    Link,
    /// `:visited` — an `a[href]` whose href IS in `MatchContext::visited`.
    Visited,
    /// Interaction pseudos parse but never match in a static render.
    Hover,
    Focus,
    Active,
    Root,
    Empty,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Compound {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub universal: bool,
    pub attrs: Vec<AttributeSelector>,
    pub pseudos: Vec<PseudoClass>,
}

impl Compound {
    fn is_empty(&self) -> bool {
        self.tag.is_none()
            && self.id.is_none()
            && self.classes.is_empty()
            && !self.universal
            && self.attrs.is_empty()
            && self.pseudos.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexSelector {
    pub compounds: Vec<Compound>,
    pub combinators: Vec<Combinator>,
}

/// Return the decoded compound chain of a selector (one compound when the
/// selector has no combinators, attributes or pseudo-classes).
pub fn complex(sel: &Selector) -> ComplexSelector {
    for class in &sel.classes {
        if let Some(payload) = class.strip_prefix(CHAIN_PREFIX) {
            if let Some(chain) = decode_chain(payload) {
                return chain;
            }
        }
    }
    ComplexSelector {
        compounds: vec![Compound {
            tag: sel.tag.clone(),
            id: sel.id.clone(),
            classes: sel
                .classes
                .iter()
                .filter(|c| !c.starts_with(CHAIN_PREFIX))
                .cloned()
                .collect(),
            universal: sel.universal,
            attrs: Vec::new(),
            pseudos: Vec::new(),
        }],
        combinators: Vec::new(),
    }
}

// ---- stylesheet parsing -------------------------------------------------

/// Nesting guard for block at-rules: hostile input cannot overflow the stack
/// before the rule cap applies.
const MAX_AT_RULE_DEPTH: u8 = 16;

/// One warning per stylesheet. Unsupported conditions drop their inner rules
/// but keep parsing, mirroring the `grid-auto-flow` diagnostic precedent.
#[derive(Default)]
struct WarnOnce {
    emitted: bool,
}

impl WarnOnce {
    fn note(&mut self, message: &str) {
        if !self.emitted {
            self.emitted = true;
            eprintln!("hodo-css: {message}");
        }
    }
}

/// First-seen `@layer` order for one stylesheet parse. Each name gets a stable
/// rank; rules carry both the rank and the full dotted name in synthetic
/// declarations so `hodo-style` can resolve one GLOBAL order across all sheets
/// (first-seen name wins ties, so `@layer a` in sheet 2 merges into sheet 1's
/// `a` instead of restarting at rank 0).
struct LayerOrder {
    names: Vec<String>,
    /// Makes anonymous layer names unique per sheet so they cannot merge in
    /// the style crate's global resolution (only same-NAMED layers merge).
    sheet_id: u64,
}

impl Default for LayerOrder {
    fn default() -> Self {
        static NEXT_SHEET_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self {
            names: Vec::new(),
            sheet_id: NEXT_SHEET_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

impl LayerOrder {
    fn rank(&mut self, name: &str) -> u32 {
        if let Some(index) = self.names.iter().position(|n| n == name) {
            index as u32
        } else {
            let index = self.names.len() as u32;
            self.names.push(name.to_string());
            index
        }
    }
}

fn layer_names(prelude: &str) -> Vec<String> {
    prelude
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

/// Compose a nested layer's dotted path. A parent path may itself hold several
/// paths (from an outer comma list), in which case each gets the child suffix.
fn compose_layer_paths(parent: &[String], name: &str) -> Vec<String> {
    if parent.is_empty() {
        vec![name.to_string()]
    } else {
        parent.iter().map(|path| format!("{path}.{name}")).collect()
    }
}

/// Parse a stylesheet with the default viewport (1200x800, `screen`,
/// scripting enabled). Invalid selectors drop only their own rule; at-rules
/// and malformed declarations recover as before.
pub fn parse_stylesheet(css: &str) -> Stylesheet {
    parse_stylesheet_with(Viewport::default(), css)
}

/// Parse a stylesheet, evaluating `@media` / `@supports` against `viewport`.
/// Inner rules are kept only for matching conditions; unsupported conditions
/// are dropped with a single warning per stylesheet. `@layer` both establishes
/// order (statement form) and tags its inner rules (block form).
pub fn parse_stylesheet_with(viewport: Viewport, css: &str) -> Stylesheet {
    let mut parser = Parser::new(css);
    let mut rules = Vec::new();
    let mut warn = WarnOnce::default();
    let mut layers = LayerOrder::default();
    parse_rules(
        &mut parser,
        viewport,
        &mut warn,
        &mut rules,
        0,
        &[],
        &mut layers,
    );
    Stylesheet { rules }
}

fn parse_rules(
    parser: &mut Parser<'_>,
    viewport: Viewport,
    warn: &mut WarnOnce,
    rules: &mut Vec<StyleRule>,
    depth: u8,
    layer: &[String],
    layers: &mut LayerOrder,
) {
    loop {
        let state = parser.state();
        match parser.next() {
            Ok(Token::CDO | Token::CDC | Token::Semicolon) => continue,
            Ok(Token::AtKeyword(name)) => {
                let name = name.to_string();
                parse_at_rule(parser, &name, viewport, warn, rules, depth, layer, layers);
            }
            Ok(Token::CurlyBracketBlock) => {}
            Ok(_) => {
                parser.reset(&state);
                if let Some(parsed) = parse_qualified_rule(parser, viewport, warn) {
                    for rule in parsed {
                        if layer.is_empty() {
                            rules.push(rule);
                        } else {
                            // A comma list duplicates the rule into every
                            // named layer (per CSS Cascade 5), each with its
                            // own rank + full dotted name marker.
                            for name in layer {
                                let rank = layers.rank(name);
                                let mut layered = rule.clone();
                                layered.declarations.push(Declaration {
                                    property: LAYER_MARKER.into(),
                                    value: rank.to_string(),
                                });
                                layered.declarations.push(Declaration {
                                    property: LAYER_NAME_MARKER.into(),
                                    value: name.clone(),
                                });
                                rules.push(layered);
                            }
                        }
                    }
                }
            }
            Err(_) => break,
        }
        if rules.len() >= MAX_RULES {
            rules.truncate(MAX_RULES);
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_at_rule(
    parser: &mut Parser<'_>,
    name: &str,
    viewport: Viewport,
    warn: &mut WarnOnce,
    rules: &mut Vec<StyleRule>,
    depth: u8,
    layer: &[String],
    layers: &mut LayerOrder,
) {
    let prelude = parser
        .parse_until_before(Delimiter::CurlyBracketBlock | Delimiter::Semicolon, |p| {
            Ok::<_, ParseError<()>>(serialize_value(p))
        })
        .unwrap_or_default();
    let is_block = matches!(parser.next(), Ok(Token::CurlyBracketBlock));
    let mut inner_layer = layer.to_vec();
    let applies = match name.to_ascii_lowercase().as_str() {
        "media" => media_applies(&prelude, viewport, warn),
        "supports" => supports_applies(&prelude, warn),
        "import" => {
            warn.note("@import ignored (fetching stylesheets in the parser is out of scope)");
            false
        }
        "layer" => {
            let mut names = layer_names(&prelude);
            if names.is_empty() {
                // Anonymous layer: unique per occurrence, composed with any
                // enclosing layer path.
                names.push(format!("\0anon:{}:{}", layers.sheet_id, layers.names.len()));
            }
            if is_block {
                // Comma lists create one layer per name; nested layers compose
                // a dotted path (`outer.inner`) instead of replacing `outer`.
                inner_layer = names
                    .iter()
                    .flat_map(|name| compose_layer_paths(layer, name))
                    .collect();
                // Establishes the names' order even before the first rule.
                for path in &inner_layer {
                    layers.rank(path);
                }
                true
            } else {
                for name in names {
                    for path in compose_layer_paths(layer, &name) {
                        layers.rank(&path);
                    }
                }
                false
            }
        }
        // @font-face, @keyframes, @page, @charset etc. keep the old skip path.
        _ => false,
    };
    if !is_block {
        return;
    }
    // Consume the block whether or not it matches, so recovery stays intact.
    let _ = parser.parse_nested_block(|p| {
        if applies && depth < MAX_AT_RULE_DEPTH {
            parse_rules(p, viewport, warn, rules, depth + 1, &inner_layer, layers);
        }
        Ok::<_, ParseError<()>>(())
    });
}

/// Split a query into whitespace-separated items, keeping each balanced
/// `(...)` (including nested parentheses) as one item.
fn query_items(query: &str) -> Option<Vec<String>> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth = 0u8;
    for ch in query.trim().chars() {
        match ch {
            '(' => {
                if depth == 0 && !current.is_empty() {
                    items.push(std::mem::take(&mut current));
                }
                depth += 1;
                if depth > MAX_CALC_DEPTH {
                    return None;
                }
                current.push(ch);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                current.push(ch);
                if depth == 0 {
                    items.push(std::mem::take(&mut current));
                }
            }
            ch if ch.is_whitespace() && depth == 0 => {
                if !current.is_empty() {
                    items.push(std::mem::take(&mut current));
                }
            }
            ch => current.push(ch),
        }
    }
    if depth != 0 {
        return None;
    }
    if !current.is_empty() {
        items.push(current);
    }
    Some(items)
}

/// True when `or` starts at byte `i` as a standalone keyword outside parens.
fn is_or_keyword(query: &str, i: usize) -> bool {
    let rest = &query[i..];
    if !rest
        .get(..2)
        .is_some_and(|word| word.eq_ignore_ascii_case("or"))
    {
        return false;
    }
    let word_char = |ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_');
    query[..i].chars().last().is_none_or(|ch| !word_char(ch))
        && rest
            .get(2..)
            .and_then(|tail| tail.chars().next())
            .is_none_or(|ch| !word_char(ch))
}

/// Strip one pair of parentheses wrapping the whole condition.
fn strip_wrapper(query: &str) -> Option<&str> {
    let mut depth = 0i32;
    for (i, ch) in query.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return (i + 1 == query.len()).then(|| &query[1..i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split a media prelude on top-level commas (OR). Commas inside feature
/// parentheses or quoted strings are data, not alternatives.
fn split_media_query(query: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0u8;
    let mut quote = None;
    let mut start = 0;
    for (i, ch) in query.char_indices() {
        match ch {
            '\'' | '"' if quote.is_none() => quote = Some(ch),
            q if quote == Some(q) => quote = None,
            '(' if quote.is_none() => {
                depth += 1;
                if depth > MAX_CALC_DEPTH {
                    return None;
                }
            }
            ')' if quote.is_none() => depth = depth.checked_sub(1)?,
            ',' if quote.is_none() && depth == 0 => {
                parts.push(query[start..i].trim());
                start = i + 1;
            }
            'o' | 'O' if quote.is_none() && depth == 0 && is_or_keyword(query, i) => {
                parts.push(query[start..i].trim());
                start = i + 2;
            }
            _ => {}
        }
    }
    parts.push(query[start..].trim());
    Some(parts)
}

fn media_applies(query: &str, viewport: Viewport, warn: &mut WarnOnce) -> bool {
    let Some(alternatives) = split_media_query(query) else {
        warn.note("@media condition unsupported (safe default: rules dropped)");
        return false;
    };
    let mut matched = false;
    let mut saw_unsupported = false;
    for alternative in alternatives {
        if alternative.is_empty() {
            saw_unsupported = true;
            continue;
        }
        match media_condition_matches(alternative, viewport) {
            Some(true) => matched = true,
            Some(false) => {}
            None => saw_unsupported = true,
        }
    }
    if saw_unsupported {
        warn.note("@media condition unsupported (safe default: rules dropped)");
    }
    matched
}

/// Evaluate one comma-separated media alternative. `not` inverts the whole
/// alternative; the legacy `only` qualifier is a no-op.
fn media_condition_matches(query: &str, viewport: Viewport) -> Option<bool> {
    let mut query = query.trim();
    for _ in 0..MAX_CALC_DEPTH {
        match strip_wrapper(query) {
            Some(inner) if inner.trim_start().starts_with('(') => query = inner.trim(),
            _ => break,
        }
    }
    let mut items = query_items(query)?;
    if items.is_empty() {
        return None;
    }
    let invert = items[0].eq_ignore_ascii_case("not");
    if invert {
        items.remove(0);
    }
    if items
        .first()
        .is_some_and(|item| item.eq_ignore_ascii_case("only"))
    {
        items.remove(0);
    }
    if items.is_empty() {
        return None;
    }
    let mut index = 0;
    let mut matched = true;
    if !items[0].starts_with('(') {
        matched = match items[0].to_ascii_lowercase().as_str() {
            "screen" => viewport.media_type == MediaType::Screen,
            "print" => viewport.media_type == MediaType::Print,
            "all" => true,
            _ => return None,
        };
        index = 1;
    }
    while index < items.len() {
        if items[index].eq_ignore_ascii_case("and") {
            index += 1;
            continue;
        }
        matched &= feature_matches(&items[index], viewport)?;
        index += 1;
    }
    Some(if invert { !matched } else { matched })
}

fn feature_matches(item: &str, viewport: Viewport) -> Option<bool> {
    let inner = item.strip_prefix('(')?.strip_suffix(')')?.trim();
    // MQ4 range syntax: `(width>=48rem)`, `(height<=500px)`, `(width=1px)`.
    if let Some(split) = inner.find(['<', '>', '=']) {
        let name = &inner[..split];
        let tail = &inner[split..];
        let (op, value) = if let Some(rest) = tail.strip_prefix(">=") {
            (">=", rest)
        } else if let Some(rest) = tail.strip_prefix("<=") {
            ("<=", rest)
        } else if let Some(rest) = tail.strip_prefix('>') {
            (">", rest)
        } else if let Some(rest) = tail.strip_prefix('<') {
            ("<", rest)
        } else {
            ("=", tail.strip_prefix('=')?)
        };
        let actual = match name.trim().to_ascii_lowercase().as_str() {
            "width" => f64::from(viewport.width),
            "height" => f64::from(viewport.height),
            _ => return None,
        };
        let target = media_length(value, viewport)?;
        return Some(match op {
            ">=" => actual >= target,
            "<=" => actual <= target,
            ">" => actual > target,
            "<" => actual < target,
            "=" => actual == target,
            _ => return None,
        });
    }
    // Boolean form: `(prefers-reduced-motion)` means "not no-preference".
    if !inner.contains(':') {
        return match inner.to_ascii_lowercase().as_str() {
            "prefers-reduced-motion" | "pointer" => Some(true),
            "hover" | "forced-colors" => Some(false),
            _ => None,
        };
    }
    let (feature, value) = inner.split_once(':')?;
    let value = value.trim();
    let feature = feature.trim().to_ascii_lowercase();
    match feature.as_str() {
        "min-width" => Some(px_value(value)? <= f64::from(viewport.width)),
        "max-width" => Some(px_value(value)? >= f64::from(viewport.width)),
        "min-height" => Some(px_value(value)? <= f64::from(viewport.height)),
        "max-height" => Some(px_value(value)? >= f64::from(viewport.height)),
        "orientation" => match value.to_ascii_lowercase().as_str() {
            "landscape" => Some(viewport.width >= viewport.height),
            "portrait" => Some(viewport.width < viewport.height),
            _ => None,
        },
        // Static render: no hover-capable pointer and no theme tracking beyond
        // the shell's dark flag.
        "hover" => match value.to_ascii_lowercase().as_str() {
            "none" => Some(true),
            "hover" => Some(false),
            _ => None,
        },
        "prefers-color-scheme" => match value.to_ascii_lowercase().as_str() {
            "dark" => Some(viewport.dark),
            "light" => Some(!viewport.dark),
            _ => None,
        },
        "min-resolution" => Some(dppx_value(value)? <= DEVICE_DPPX),
        "max-resolution" => Some(dppx_value(value)? >= DEVICE_DPPX),
        "-webkit-min-device-pixel-ratio"
        | "-moz-min-device-pixel-ratio"
        | "min-device-pixel-ratio" => Some(dppx_value(value)? <= DEVICE_DPPX),
        // Static render: no animation preference, no forced colors, fine pointer.
        "prefers-reduced-motion" => match value.to_ascii_lowercase().as_str() {
            "no-preference" => Some(true),
            "reduce" => Some(false),
            _ => None,
        },
        "forced-colors" => match value.to_ascii_lowercase().as_str() {
            "none" => Some(true),
            "active" => Some(false),
            _ => None,
        },
        "pointer" => match value.to_ascii_lowercase().as_str() {
            "fine" => Some(true),
            "coarse" | "none" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn px_value(value: &str) -> Option<f64> {
    value.strip_suffix("px")?.trim().parse().ok()
}

/// Length in a media-range feature. `rem`/`em` use the same parse-time 16px
/// fallback as `calc()`; the media context has no element font size either.
fn media_length(value: &str, viewport: Viewport) -> Option<f64> {
    let mut value = value.trim().to_string();
    if value.contains("calc(") {
        value = resolve_calcs(&value, viewport).ok()?;
    }
    let value = value.trim();
    let number = if let Some(n) = value
        .strip_suffix("rem")
        .or_else(|| value.strip_suffix("em"))
    {
        n.trim().parse::<f64>().ok()? * CALC_FALLBACK_FONT_PX
    } else if let Some(n) = value.strip_suffix("px") {
        n.trim().parse::<f64>().ok()?
    } else {
        value.parse::<f64>().ok()?
    };
    number.is_finite().then_some(number)
}

/// Device-pixel ratio: bare numbers, `dppx` and `dpi` (96 per dppx) are
/// accepted so vendor-prefixed aliases share one evaluation path.
fn dppx_value(value: &str) -> Option<f64> {
    let value = value.trim();
    let ratio = if let Some(n) = value.strip_suffix("dppx") {
        n.trim().parse::<f64>().ok()?
    } else if let Some(n) = value.strip_suffix("dpi") {
        n.trim().parse::<f64>().ok()? / 96.0
    } else {
        value.parse::<f64>().ok()?
    };
    ratio.is_finite().then_some(ratio)
}

fn supports_applies(condition: &str, warn: &mut WarnOnce) -> bool {
    // Whitelist: one plain `(property: value)` we actually implement,
    // optionally negated with `not`. `and`/`or`/`selector()` and
    // `position: sticky` stay non-matching.
    let trimmed = condition.trim();
    let negated = trimmed
        .get(..3)
        .is_some_and(|p| p.eq_ignore_ascii_case("not"))
        && trimmed
            .get(3..)
            .is_some_and(|rest| rest.starts_with(char::is_whitespace));
    let (condition, invert) = if negated {
        (trimmed.get(3..).unwrap_or_default().trim(), true)
    } else {
        (trimmed, false)
    };
    let known = condition
        .strip_prefix('(')
        .and_then(|text| text.strip_suffix(')'))
        .and_then(|inner| inner.split_once(':'))
        .is_some_and(|(property, value)| {
            let is_display = property.trim().eq_ignore_ascii_case("display");
            let value = value.trim();
            is_display
                && (value.eq_ignore_ascii_case("flex")
                    || value.eq_ignore_ascii_case("grid")
                    || value.eq_ignore_ascii_case("table"))
        });
    let supported = known && !invert;
    if !supported {
        warn.note("@supports condition unsupported (safe default: rules dropped)");
    }
    supported
}

fn skip_rule(parser: &mut Parser<'_>) {
    loop {
        match parser.next() {
            Ok(Token::CurlyBracketBlock) => {
                let _ = parser.parse_nested_block(|p| {
                    while p.next().is_ok() {}
                    Ok::<_, ParseError<()>>(())
                });
                return;
            }
            Ok(Token::Semicolon) | Err(_) => return,
            Ok(_) => {}
        }
    }
}

fn parse_qualified_rule(
    parser: &mut Parser<'_>,
    viewport: Viewport,
    warn: &mut WarnOnce,
) -> Option<Vec<StyleRule>> {
    // Collect the prelude once. Plain compounds use the allocation-light token
    // parser; selectors with blocks/functions are re-parsed from their text so
    // nested blocks can be expanded.
    let start = parser.position();
    let prelude = parser
        .parse_until_before(Delimiter::CurlyBracketBlock, |p| {
            let mut tokens = Vec::new();
            while let Ok(token) = p.next_including_whitespace() {
                tokens.push(token.clone());
            }
            Ok::<_, ParseError<()>>(tokens)
        })
        .ok()?;

    if !matches!(parser.next(), Ok(Token::CurlyBracketBlock)) {
        return None;
    }
    let declarations = parser
        .parse_nested_block(|p| Ok::<_, ParseError<()>>(parse_declarations(p, viewport, warn)))
        .unwrap_or_default();

    let mut rules = Vec::new();
    // (selector-list index, rule index) for groups needing nested-block parsing.
    let mut complex: Vec<(usize, usize)> = Vec::new();
    let mut current = Vec::new();
    let mut group_index = 0;
    for token in prelude {
        if matches!(token, Token::Comma) {
            if !current.is_empty() {
                match selector_from_tokens(&current) {
                    Ok(selector) => rules.push(StyleRule {
                        selector,
                        declarations: declarations.clone(),
                    }),
                    Err(()) => {
                        complex.push((group_index, rules.len()));
                        rules.push(StyleRule {
                            selector: Selector {
                                tag: None,
                                id: None,
                                classes: Vec::new(),
                                universal: true,
                            },
                            declarations: declarations.clone(),
                        });
                    }
                }
            }
            // Advance once per group, including empty ones: the streaming
            // parser emits a `None` slot for every comma-separated group.
            group_index += 1;
            current.clear();
        } else {
            current.push(token);
        }
    }
    if !current.is_empty() {
        match selector_from_tokens(&current) {
            Ok(selector) => rules.push(StyleRule {
                selector,
                declarations,
            }),
            Err(()) => {
                complex.push((group_index, rules.len()));
                rules.push(StyleRule {
                    selector: Selector {
                        tag: None,
                        id: None,
                        classes: Vec::new(),
                        universal: true,
                    },
                    declarations,
                });
            }
        }
    }

    // Selectors with nested blocks are parsed from the original prelude text.
    if !complex.is_empty() {
        let text = parser.slice(start..parser.position()).to_string();
        let mut stream = Parser::new(&text);
        let streamed = parse_selector_list_streaming(&mut stream);
        let mut drop = Vec::new();
        for (group_index, rule_index) in complex {
            match streamed.get(group_index).and_then(Option::as_ref) {
                Some(selector) => rules[rule_index].selector = selector.clone(),
                None => drop.push(rule_index),
            }
        }
        for index in drop.into_iter().rev() {
            rules.remove(index);
        }
    }
    Some(rules)
}

fn parse_selector_list_streaming(parser: &mut Parser<'_>) -> Vec<Option<Selector>> {
    let mut selectors = Vec::new();
    loop {
        match parse_selector_stream(parser) {
            Ok(selector) => {
                selectors.push(Some(selector));
                if !matches!(parser.next(), Ok(Token::Comma)) {
                    break;
                }
            }
            Err(_) => {
                selectors.push(None);
                let mut comma = false;
                while let Ok(token) = parser.next() {
                    if matches!(token, Token::Comma) {
                        comma = true;
                        break;
                    }
                }
                if !comma {
                    break;
                }
            }
        }
    }
    selectors
}

/// Parse a plain compound selector from already-tokenized prelude tokens.
/// `Err` means the group needs the streaming parser (nested blocks, pseudos or
/// combinators). Whitespace is preserved in the token stream, so any whitespace
/// (including between two simple selectors) delegates to the streaming parser,
/// which is the only place descendant combinators are built.
fn selector_from_tokens(tokens: &[Token<'_>]) -> Result<Selector, ()> {
    let mut selector = Selector {
        tag: None,
        id: None,
        classes: Vec::new(),
        universal: false,
    };
    let mut idents = 0;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::WhiteSpace(_)
            | Token::Function(_)
            | Token::SquareBracketBlock
            | Token::Colon
            | Token::Delim('>' | '+' | '~') => return Err(()),
            Token::Ident(name) => {
                idents += 1;
                if idents > 1 {
                    return Err(());
                }
                selector.tag = Some(name.to_string());
                i += 1;
            }
            Token::Hash(name) | Token::IDHash(name) => {
                selector.id = Some(name.to_string());
                i += 1;
            }
            Token::Delim('.') => {
                if let Some(Token::Ident(name)) = tokens.get(i + 1) {
                    selector.classes.push(name.to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            Token::Delim('*') => {
                selector.universal = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    if selector.tag.is_none() && selector.id.is_none() && selector.classes.is_empty() {
        selector.universal = true;
    }
    Ok(selector)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalcError {
    Invalid,
}

/// Resolve the `calc()` expressions that can be evaluated at parse time and
/// viewport-relative lengths in a declaration value to `px` text using the
/// stylesheet's `Viewport`. Two kinds of `calc()` are **deferred** and kept as
/// raw text instead of being dropped:
///
/// * `var(...)` — `hodo-style` substitutes variables later, at cascade time,
///   then calls [`resolve_calc_now`] on the substituted value.
/// * `%` — only layout knows the containing-block basis; the raw expression is
///   carried into `StyleMap` and resolved with [`resolve_calc_with_basis`].
///
/// Invalid `calc()` still drops its declaration; quoted text is left verbatim.
///
/// NOTE(parse-time): like `px`, a viewport unit becomes a fixed number while
/// parsing. A window resized mid-session does not re-resolve cached
/// declarations — revisit by re-parsing with the new `Viewport` when resize
/// support lands.
fn resolve_lengths(value: &str, viewport: Viewport, _warn: &mut WarnOnce) -> Option<String> {
    let (head, tail) = match value.split_once('!') {
        Some((head, tail)) => (head, Some(tail)),
        None => (value, None),
    };
    let head = resolve_calcs(head, viewport).ok()?;
    let mut out = resolve_viewport_lengths(&head, viewport);
    if let Some(tail) = tail {
        out.push('!');
        out.push_str(tail);
    }
    Some(out)
}

/// Resolve deferred `calc()` text at cascade time, after `var()` substitution.
/// Still-deferred `%` expressions are left verbatim for layout. Returns `None`
/// only for a genuinely invalid expression.
pub fn resolve_calc_now(text: &str) -> Option<String> {
    resolve_lengths(text, Viewport::default(), &mut WarnOnce::default())
}

/// Resolve a deferred percentage `calc()` against a known containing-block
/// basis: each `%` operand becomes `basis / 100`. `var()` must already be
/// substituted. This is the layout-time half of the `calc(100% - 32px)` path;
/// `hodo-style` carries the raw text until `StyleMap::resolve_width_with_basis`
/// calls this with the containing block's content width.
pub fn resolve_calc_with_basis(text: &str, basis: f32) -> Option<f32> {
    let text = text.trim();
    let inner = text
        .strip_prefix("calc(")
        .and_then(|rest| rest.strip_suffix(')'))?;
    let (value, _) = CalcParser::new_with_basis(inner, Viewport::default(), Some(f64::from(basis)))
        .parse()
        .ok()?;
    value.is_finite().then_some(value as f32)
}

/// Rewrite every eager `calc(...)` in a value as `<number>px` (or a bare
/// `<number>` when every operand is unitless). Calcs containing `var(` or `%`
/// are copied verbatim for later resolution.
fn resolve_calcs(value: &str, viewport: Viewport) -> Result<String, CalcError> {
    let lower = value.to_ascii_lowercase();
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut pos = 0;
    while pos < value.len() {
        let is_calc = lower[pos..].starts_with("calc(")
            && !bytes
                .get(pos.wrapping_sub(1))
                .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'));
        if is_calc {
            let open = pos + 4;
            let mut depth = 0u8;
            let mut close = None;
            for (i, ch) in lower[open..].char_indices() {
                match ch {
                    '(' => {
                        depth += 1;
                        if depth > MAX_CALC_DEPTH {
                            return Err(CalcError::Invalid);
                        }
                    }
                    ')' => {
                        depth = depth.checked_sub(1).ok_or(CalcError::Invalid)?;
                        if depth == 0 {
                            close = Some(open + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let close = close.ok_or(CalcError::Invalid)?;
            let inner = &value[open + 1..close];
            let lower_inner = inner.to_ascii_lowercase();
            if lower_inner.contains("var(") || inner.contains('%') {
                // Deferred: keep the whole `calc(...)` text verbatim.
                out.push_str(&value[pos..=close]);
            } else {
                let (number, has_unit) = CalcParser::new(inner, viewport).parse()?;
                if has_unit {
                    out.push_str(&format!("{number}px"));
                } else {
                    out.push_str(&format!("{number}"));
                }
            }
            pos = close + 1;
        } else {
            let ch = value[pos..].chars().next().ok_or(CalcError::Invalid)?;
            if matches!(ch, '\'' | '"') {
                let start = pos;
                let quote = ch;
                pos += ch.len_utf8();
                let mut escaped = false;
                while pos < value.len() {
                    let ch = value[pos..].chars().next().ok_or(CalcError::Invalid)?;
                    pos += ch.len_utf8();
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == quote {
                        break;
                    }
                }
                out.push_str(&value[start..pos]);
            } else {
                out.push(ch);
                pos += ch.len_utf8();
            }
        }
    }
    Ok(out)
}

/// Recursive-descent evaluator for the inside of one `calc()`:
/// `expr = term (('+'|'-') term)*`, `term = factor (('*'|'/') factor)*`.
struct CalcParser<'a> {
    expr: &'a [u8],
    pos: usize,
    depth: u8,
    viewport: Viewport,
    /// `Some(basis)` enables `%` evaluation at layout time; `None` defers.
    percent_basis: Option<f64>,
    /// Whether any operand carried a unit (incl. `%`); drives px vs bare number.
    has_unit: bool,
}

impl<'a> CalcParser<'a> {
    fn new(expr: &'a str, viewport: Viewport) -> Self {
        Self::new_with_basis(expr, viewport, None)
    }

    fn new_with_basis(expr: &'a str, viewport: Viewport, percent_basis: Option<f64>) -> Self {
        Self {
            expr: expr.as_bytes(),
            pos: 0,
            depth: 0,
            viewport,
            percent_basis,
            has_unit: false,
        }
    }

    /// Returns the value and whether any operand carried a unit.
    fn parse(mut self) -> Result<(f64, bool), CalcError> {
        let value = self.expr()?;
        self.ws();
        if self.pos != self.expr.len() || !value.is_finite() {
            return Err(CalcError::Invalid);
        }
        Ok((value, self.has_unit))
    }

    fn ws(&mut self) {
        while self.expr.get(self.pos).is_some_and(u8::is_ascii_whitespace) {
            self.pos += 1;
        }
    }

    fn expr(&mut self) -> Result<f64, CalcError> {
        let mut value = self.term()?;
        loop {
            self.ws();
            match self.expr.get(self.pos) {
                Some(b'+') => {
                    self.pos += 1;
                    value += self.term()?;
                }
                Some(b'-') => {
                    self.pos += 1;
                    value -= self.term()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn term(&mut self) -> Result<f64, CalcError> {
        let mut value = self.factor()?;
        loop {
            self.ws();
            match self.expr.get(self.pos) {
                Some(b'*') => {
                    self.pos += 1;
                    value *= self.factor()?;
                }
                Some(b'/') => {
                    self.pos += 1;
                    let divisor = self.factor()?;
                    if divisor == 0.0 {
                        return Err(CalcError::Invalid);
                    }
                    value /= divisor;
                }
                _ => return Ok(value),
            }
        }
    }

    fn factor(&mut self) -> Result<f64, CalcError> {
        self.ws();
        match self.expr.get(self.pos).copied() {
            Some(b'+') => {
                self.pos += 1;
                self.factor()
            }
            Some(b'-') => {
                self.pos += 1;
                Ok(-self.factor()?)
            }
            Some(b'(') => self.parens(),
            Some(_) if self.expr[self.pos..].starts_with(b"calc(") => {
                self.pos += 5;
                self.parens()
            }
            Some(_) => self.number(),
            None => Err(CalcError::Invalid),
        }
    }

    fn parens(&mut self) -> Result<f64, CalcError> {
        self.pos += 1;
        self.depth += 1;
        if self.depth > MAX_CALC_DEPTH {
            return Err(CalcError::Invalid);
        }
        let value = self.expr()?;
        self.ws();
        if self.expr.get(self.pos) != Some(&b')') {
            return Err(CalcError::Invalid);
        }
        self.pos += 1;
        self.depth -= 1;
        Ok(value)
    }

    fn number(&mut self) -> Result<f64, CalcError> {
        let start = self.pos;
        while self
            .expr
            .get(self.pos)
            .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
        {
            self.pos += 1;
        }
        // Only consume `e`/`E` as an exponent when digits follow; `em` is a
        // unit, not a malformed exponent.
        if self
            .expr
            .get(self.pos)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            let mut scan = self.pos + 1;
            if self
                .expr
                .get(scan)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                scan += 1;
            }
            if self.expr.get(scan).is_some_and(u8::is_ascii_digit) {
                self.pos = scan + 1;
                while self.expr.get(self.pos).is_some_and(u8::is_ascii_digit) {
                    self.pos += 1;
                }
            }
        }
        if self.pos == start {
            return Err(CalcError::Invalid);
        }
        let text =
            std::str::from_utf8(&self.expr[start..self.pos]).map_err(|_| CalcError::Invalid)?;
        let number = text.parse::<f64>().map_err(|_| CalcError::Invalid)?;
        let (vp, vh) = (
            f64::from(self.viewport.width),
            f64::from(self.viewport.height),
        );
        let rest = &self.expr[self.pos..];
        let (unit_len, scale) = if rest.starts_with(b"vmin") {
            (4, vp.min(vh) / 100.0)
        } else if rest.starts_with(b"vmax") {
            (4, vp.max(vh) / 100.0)
        } else if rest.starts_with(b"rem") {
            (3, CALC_FALLBACK_FONT_PX)
        } else if rest.starts_with(b"vh") {
            (2, vh / 100.0)
        } else if rest.starts_with(b"vw") {
            (2, vp / 100.0)
        } else if rest.starts_with(b"px") {
            (2, 1.0)
        } else if rest.starts_with(b"em") {
            (2, CALC_FALLBACK_FONT_PX)
        } else if rest.starts_with(b"%") {
            let Some(basis) = self.percent_basis else {
                return Err(CalcError::Invalid);
            };
            (1, basis / 100.0)
        } else {
            (0, 1.0)
        };
        if rest
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'%')
            && unit_len == 0
        {
            return Err(CalcError::Invalid);
        }
        if unit_len > 0 {
            self.has_unit = true;
        }
        self.pos += unit_len;
        let value = number * scale;
        value.is_finite().then_some(value).ok_or(CalcError::Invalid)
    }
}

/// Resolve bare viewport units (outside `calc()`) as before. Quoted text and
/// unknown units are left alone.
fn resolve_viewport_lengths(value: &str, viewport: Viewport) -> String {
    const UNITS: [&str; 4] = ["vmin", "vmax", "vh", "vw"];
    if !UNITS.iter().any(|unit| value.contains(unit)) {
        return value.to_string();
    }
    let (head, tail) = match value.split_once('!') {
        Some((head, tail)) => (head, Some(tail)),
        None => (value, None),
    };
    let sides = [
        ("vmin", viewport.width.min(viewport.height)),
        ("vmax", viewport.width.max(viewport.height)),
        ("vh", viewport.height),
        ("vw", viewport.width),
    ];
    let mut out = String::with_capacity(value.len());
    for piece in head.split_inclusive(char::is_whitespace) {
        let lead = piece.len() - piece.trim_start().len();
        let trimmed = piece.trim();
        let end = piece.trim_end().len();
        let resolved = sides.into_iter().find_map(|(unit, side)| {
            let number = trimmed.strip_suffix(unit)?.trim().parse::<f32>().ok()?;
            number
                .is_finite()
                .then(|| format!("{}px", number * side as f32 / 100.0))
        });
        if let Some(px) = resolved {
            out.push_str(&piece[..lead]);
            out.push_str(&px);
            out.push_str(&piece[end..]);
        } else {
            out.push_str(piece);
        }
    }
    if let Some(tail) = tail {
        out.push('!');
        out.push_str(tail);
    }
    out
}

fn parse_declarations(
    parser: &mut Parser<'_>,
    viewport: Viewport,
    warn: &mut WarnOnce,
) -> Vec<Declaration> {
    let mut declarations = Vec::new();

    loop {
        let state = parser.state();
        match parser.next() {
            Ok(Token::Semicolon) => continue,
            Ok(Token::AtKeyword(_)) => {
                parser.reset(&state);
                skip_rule(parser);
            }
            Ok(Token::Ident(property)) => {
                // Custom property names are case-sensitive; ordinary ones are not.
                let property = if property.starts_with("--") {
                    property.to_string()
                } else {
                    property.to_lowercase()
                };
                if parser.expect_colon().is_err() {
                    skip_to_semicolon(parser);
                    continue;
                }
                let value = parser
                    .parse_until_before(Delimiter::Semicolon, |p| {
                        Ok::<_, ParseError<()>>(serialize_value(p))
                    })
                    .unwrap_or_default();
                let Some(value) = resolve_lengths(&value, viewport, warn) else {
                    continue;
                };
                declarations.push(Declaration { property, value });
            }
            Ok(_) => skip_to_semicolon(parser),
            Err(_) => break,
        }
    }

    declarations
}

/// Parse an element's `style="..."` attribute as a declaration list.
pub fn parse_style_attribute(css: &str) -> Vec<Declaration> {
    let mut parser = Parser::new(css);
    parse_declarations(&mut parser, Viewport::default(), &mut WarnOnce::default())
}

/// Value text of a declaration with a trailing `! important` removed.
pub fn declaration_value(decl: &Declaration) -> &str {
    split_important(&decl.value).0
}

/// Whether a declaration carries the `!important` flag.
pub fn is_important(decl: &Declaration) -> bool {
    split_important(&decl.value).1
}

fn split_important(value: &str) -> (&str, bool) {
    let trimmed = value.trim();
    match trimmed.rsplit_once('!') {
        Some((head, tail)) if tail.trim().eq_ignore_ascii_case("important") => (head.trim(), true),
        _ => (trimmed, false),
    }
}

// ---- custom properties -------------------------------------------------
//
// Values are stored as raw text (the parser already applies the existing
// parse-time `vh`/`vw` resolution to them). Resizing mid-session therefore
// does not re-resolve lengths held in custom properties — same limitation as
// ordinary declarations, documented on `resolve_viewport_lengths`.

/// Resolve every custom property against the raw set, dropping any whose
/// `var()` references are cyclic or unresolved (the spec's
/// guaranteed-invalid value). Values may reference each other in any order.
pub fn resolve_custom_properties(raw: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for name in raw.keys() {
        if let Some(value) = raw
            .get(name)
            .and_then(|text| resolve_text(text, raw, &mut vec![name.clone()]))
        {
            out.insert(name.clone(), value);
        }
    }
    out
}

/// Substitute `var(--name)` / `var(--name, fallback)` in a declaration value.
/// Returns `None` when a reference has no value and no fallback — the whole
/// declaration is then invalid and must be dropped by the caller.
pub fn substitute_vars(value: &str, resolved: &BTreeMap<String, String>) -> Option<String> {
    resolve_text(value, resolved, &mut Vec::new())
}

fn is_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}

fn find_var_end(text: &str, start: usize) -> Option<usize> {
    let mut depth = 1usize;
    for (offset, byte) in text.as_bytes()[start..].iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

// shortcut: fallback commas inside quotes are treated as separators; upgrade
// when a full CSS value parser is available.
fn split_fallback(inner: &str) -> (&str, Option<&str>) {
    let mut depth = 0usize;
    for (i, byte) in inner.as_bytes().iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => return (&inner[..i], Some(&inner[i + 1..])),
            _ => {}
        }
    }
    (inner, None)
}

fn resolve_text(
    text: &str,
    raw: &BTreeMap<String, String>,
    stack: &mut Vec<String>,
) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if text[i..].starts_with("var(") && (i == 0 || !is_name_char(bytes[i - 1])) {
            let start = i + 4;
            let end = find_var_end(text, start)?;
            let (name, fallback) = split_fallback(&text[start..end]);
            let name = name.trim();
            if !name.starts_with("--") {
                return None;
            }
            let replacement = match raw.get(name) {
                Some(value) => {
                    if stack.iter().any(|seen| seen == name) {
                        return None;
                    }
                    stack.push(name.to_string());
                    let resolved = resolve_text(value, raw, stack);
                    stack.pop();
                    resolved?
                }
                None => {
                    let value = fallback?;
                    resolve_text(value.trim(), raw, stack)?
                }
            };
            out.push_str(&replacement);
            i = end + 1;
        } else {
            let ch = text[i..].chars().next()?;
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    Some(out)
}

fn skip_to_semicolon(parser: &mut Parser<'_>) {
    while !matches!(parser.next(), Ok(Token::Semicolon) | Err(_)) {}
}

fn serialize_value(parser: &mut Parser<'_>) -> String {
    let mut out = String::new();
    let mut pending_space = false;

    while let Ok(token) = parser.next_including_whitespace() {
        let token = token.clone();

        match token {
            Token::WhiteSpace(_) => {
                if !out.is_empty() {
                    pending_space = true;
                }
            }
            Token::Comment(_) => {}
            Token::Semicolon => break,
            Token::Function(ref name) => {
                let name = name.to_string();
                push_space(&mut out, pending_space);
                out.push_str(&name);
                out.push('(');
                let inner = parser
                    .parse_nested_block(|p| Ok::<_, ParseError<()>>(serialize_value(p)))
                    .unwrap_or_default();
                out.push_str(inner.trim());
                out.push(')');
                pending_space = false;
            }
            Token::ParenthesisBlock => serialize_block(parser, &mut out, pending_space, '(', ')'),
            Token::SquareBracketBlock => serialize_block(parser, &mut out, pending_space, '[', ']'),
            Token::CurlyBracketBlock => serialize_block(parser, &mut out, pending_space, '{', '}'),
            Token::BadString(_) | Token::BadUrl(_) => {
                pending_space = false;
            }
            token if token.is_parse_error() => {
                pending_space = false;
            }
            token => {
                push_space(&mut out, pending_space);
                out.push_str(&token.to_css_string());
                pending_space = false;
            }
        }
    }

    out.trim().to_string()
}

fn serialize_block(
    parser: &mut Parser<'_>,
    out: &mut String,
    pending_space: bool,
    open: char,
    close: char,
) {
    push_space(out, pending_space);
    out.push(open);
    let inner = parser
        .parse_nested_block(|p| Ok::<_, ParseError<()>>(serialize_value(p)))
        .unwrap_or_default();
    out.push_str(inner.trim());
    out.push(close);
}

fn push_space(out: &mut String, pending: bool) {
    if pending && !out.is_empty() && !out.ends_with(' ') {
        out.push(' ');
    }
}

// ---- selector parsing ---------------------------------------------------

fn parse_selector_stream<'i>(parser: &mut Parser<'i>) -> Result<Selector, ParseError<()>> {
    let mut first = parse_compound(parser, 0)?;
    // Avoid allocating the chain vectors for the common single-compound case.
    let mut compounds: Vec<Compound> = Vec::new();
    let mut combinators: Vec<Combinator> = Vec::new();

    loop {
        let state = parser.state();
        let token = match parser.next_including_whitespace() {
            Ok(token) => token.clone(),
            Err(_) => {
                parser.reset(&state);
                break;
            }
        };
        let combinator = match token {
            Token::WhiteSpace(_) => {
                let after_space = parser.state();
                let next = parser.next().ok().cloned();
                match next {
                    Some(Token::Delim('>')) => Some(Combinator::Child),
                    Some(Token::Delim('+')) => Some(Combinator::Adjacent),
                    Some(Token::Delim('~')) => Some(Combinator::GeneralSibling),
                    Some(
                        Token::Ident(_)
                        | Token::Hash(_)
                        | Token::IDHash(_)
                        | Token::Function(_)
                        | Token::Colon
                        | Token::Delim('.' | '*')
                        | Token::SquareBracketBlock,
                    ) => {
                        parser.reset(&after_space);
                        Some(Combinator::Descendant)
                    }
                    // Trailing whitespace before `,` / `{` / EOF is not a
                    // combinator; leave the parser on the next token.
                    _ => break,
                }
            }
            Token::Delim('>') => Some(Combinator::Child),
            Token::Delim('+') => Some(Combinator::Adjacent),
            Token::Delim('~') => Some(Combinator::GeneralSibling),
            _ => None,
        };

        let Some(combinator) = combinator else {
            parser.reset(&state);
            break;
        };
        let next = parse_compound(parser, 0)?;
        if next.is_empty() {
            break;
        }
        if compounds.is_empty() {
            compounds.push(std::mem::take(&mut first));
        }
        compounds.push(next);
        combinators.push(combinator);
        if compounds.len() > MAX_COMPOUNDS {
            return Err(parser.new_error_for_next_token());
        }
    }

    if compounds.is_empty() && first.is_empty() {
        // `new_error_for_next_token` advances past the offending token; keep
        // the comma so the selector-list recovery loop can find the next group.
        let state = parser.state();
        let error = parser.new_error_for_next_token();
        parser.reset(&state);
        return Err(error);
    }
    if compounds.is_empty() {
        if first.attrs.is_empty() && first.pseudos.is_empty() {
            return Ok(Selector {
                tag: first.tag,
                id: first.id,
                classes: first.classes,
                universal: first.universal,
            });
        }
        compounds.push(first);
    }
    Ok(selector_from_chain(&ComplexSelector {
        compounds,
        combinators,
    }))
}

fn parse_compound<'i>(parser: &mut Parser<'i>, not_depth: u8) -> Result<Compound, ParseError<()>> {
    let mut compound = Compound::default();
    loop {
        let state = parser.state();
        let token = match parser.next_including_whitespace() {
            Ok(token) => token.clone(),
            Err(_) => {
                parser.reset(&state);
                break;
            }
        };
        match token {
            // Leading whitespace is insignificant; internal whitespace ends
            // this compound and leaves a descendant combinator for the caller.
            // Inside `:not()` the leftover makes `parse_nested_block` fail, so
            // nested combinators are rejected instead of silently folded.
            Token::WhiteSpace(_) if compound.is_empty() => continue,
            Token::WhiteSpace(_) => {
                parser.reset(&state);
                break;
            }
            Token::Ident(name) if compound.tag.is_none() => {
                compound.tag = Some(name.to_string());
            }
            Token::Hash(name) | Token::IDHash(name) => {
                compound.id = Some(name.to_string());
            }
            Token::Delim('*') => compound.universal = true,
            Token::Delim('.') => match parser.next()? {
                Token::Ident(name) => compound.classes.push(name.to_string()),
                _ => return Err(parser.new_error_for_next_token()),
            },
            Token::Colon => {
                let name = match parser.next()? {
                    Token::Colon => return Err(parser.new_error_for_next_token()),
                    Token::Ident(name) => {
                        let name = name.to_string();
                        compound
                            .pseudos
                            .push(match name.to_ascii_lowercase().as_str() {
                                "first-child" => PseudoClass::FirstChild,
                                "last-child" => PseudoClass::LastChild,
                                "only-child" => PseudoClass::OnlyChild,
                                "root" => PseudoClass::Root,
                                "empty" => PseudoClass::Empty,
                                "link" => PseudoClass::Link,
                                "visited" => PseudoClass::Visited,
                                "hover" => PseudoClass::Hover,
                                "focus" => PseudoClass::Focus,
                                "active" => PseudoClass::Active,
                                _ => return Err(parser.new_error_for_next_token()),
                            });
                        continue;
                    }
                    Token::Function(name) => name.to_string(),
                    _ => return Err(parser.new_error_for_next_token()),
                };
                let pseudo = match name.to_ascii_lowercase().as_str() {
                    "nth-child" => {
                        let inner = parse_nested_text(parser)?;
                        let nth =
                            parse_nth(&inner).ok_or_else(|| parser.new_error_for_next_token())?;
                        PseudoClass::NthChild(nth.0, nth.1)
                    }
                    "not" => {
                        // One compound only: a nested combinator leaves tokens
                        // unconsumed in the nested block, which makes
                        // `parse_nested_block` fail and drops the whole rule.
                        if not_depth >= MAX_NOT_DEPTH {
                            return Err(parser.new_error_for_next_token());
                        }
                        let arg = parser
                            .parse_nested_block(|p| parse_compound(p, not_depth + 1))
                            .map_err(|_| parser.new_error_for_next_token())?;
                        if arg.is_empty() {
                            return Err(parser.new_error_for_next_token());
                        }
                        PseudoClass::Not(Box::new(arg))
                    }
                    // One compound per variant; a combinator leaves tokens
                    // unconsumed, so the whole rule drops as with `:not()`.
                    "where" => PseudoClass::Where(parse_compound_list(parser, not_depth)?),
                    "is" => PseudoClass::Is(parse_compound_list(parser, not_depth)?),
                    _ => return Err(parser.new_error_for_next_token()),
                };
                compound.pseudos.push(pseudo);
            }
            Token::SquareBracketBlock => {
                compound.attrs.push(parse_attribute(parser)?);
            }
            _ => {
                parser.reset(&state);
                break;
            }
        }
    }
    Ok(compound)
}

/// Parse a comma-separated list of single compounds inside `:is()` / `:where()`.
/// Shares the `:not()` depth cap; an inner combinator leaves tokens unconsumed
/// and drops the rule through `parse_nested_block`'s end-of-input check.
fn parse_compound_list<'i>(
    parser: &mut Parser<'i>,
    not_depth: u8,
) -> Result<Vec<Compound>, ParseError<()>> {
    if not_depth >= MAX_NOT_DEPTH {
        return Err(parser.new_error_for_next_token());
    }
    parser
        .parse_nested_block(|p| {
            let mut list = Vec::new();
            loop {
                let compound = parse_compound(p, not_depth + 1)?;
                if compound.is_empty() {
                    return Err(p.new_error_for_next_token());
                }
                list.push(compound);
                match p.next() {
                    Ok(Token::Comma) => continue,
                    Ok(_) => return Err(p.new_error_for_next_token()),
                    Err(_) => break, // end of the nested block
                }
            }
            Ok(list)
        })
        .map_err(|_| parser.new_error_for_next_token())
}

fn parse_nested_text<'i>(parser: &mut Parser<'i>) -> Result<String, ParseError<()>> {
    parser
        .parse_nested_block(|p| {
            let mut out = String::new();
            while let Ok(token) = p.next() {
                out.push_str(&token.to_css_string());
            }
            Ok::<_, ParseError<()>>(out)
        })
        .map_err(|_| parser.new_error_for_next_token())
}

fn parse_attribute<'i>(parser: &mut Parser<'i>) -> Result<AttributeSelector, ParseError<()>> {
    parser
        .parse_nested_block(|p| {
            let name = match p.next()? {
                Token::Ident(name) => name.to_string(),
                _ => return Err(p.new_error_for_next_token()),
            };
            let state = p.state();
            let first = p.next().ok().cloned();
            let (op, has_value) = match first {
                Some(Token::Delim('=')) => (AttrOp::Equal, true),
                Some(Token::IncludeMatch) => (AttrOp::Includes, true),
                Some(Token::DashMatch) => (AttrOp::DashMatch, true),
                Some(Token::PrefixMatch) => (AttrOp::Prefix, true),
                Some(Token::SuffixMatch) => (AttrOp::Suffix, true),
                Some(Token::SubstringMatch) => (AttrOp::Substring, true),
                None => return Ok((name, AttrOp::Exists, String::new(), false)),
                _ => {
                    p.reset(&state);
                    (AttrOp::Exists, false)
                }
            };
            let mut value = String::new();
            if has_value {
                value = match p.next()? {
                    Token::Ident(name) | Token::QuotedString(name) => name.to_string(),
                    _ => return Err(p.new_error_for_next_token()),
                };
            }
            let case_insensitive = match p.next() {
                Ok(Token::Ident(flag)) => flag.eq_ignore_ascii_case("i"),
                Err(_) => false,
                Ok(_) => return Err(p.new_error_for_next_token()),
            };
            Ok::<_, ParseError<()>>((name, op, value, case_insensitive))
        })
        .map_err(|_| parser.new_error_for_next_token())
        .map(|(name, op, value, case_insensitive)| AttributeSelector {
            name,
            op,
            value,
            case_insensitive,
        })
}

fn parse_nth(text: &str) -> Option<(i32, i32)> {
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let lower = text.to_ascii_lowercase();
    if lower == "odd" {
        return Some((2, 1));
    }
    if lower == "even" {
        return Some((2, 0));
    }
    if let Some(n_pos) = lower.find('n') {
        let a = match &lower[..n_pos] {
            "" => 1,
            "+" => 1,
            "-" => -1,
            part => part.parse::<i32>().ok()?,
        };
        let b = match &text[n_pos + 1..] {
            "" => 0,
            part => part.parse::<i32>().ok()?,
        };
        Some((a, b))
    } else {
        let b = text.parse::<i32>().ok()?;
        Some((0, b))
    }
}

// ---- internal chain encoding -------------------------------------------

const CHAIN_PREFIX: &str = "\u{0}chain:";
const COMP_SEP: char = '\u{1f}';
const FIELD_SEP: char = '\u{1d}';
/// Separates `:is()` / `:where()` variants inside one encoded pseudo atom.
/// Must differ from `COMP_SEP` (chain compounds) and every depth separator.
const PSEUDO_LIST_SEP: char = '\u{1a}';

fn sep_for_depth(depth: u8) -> char {
    match depth {
        0 => '\u{1e}',
        1 => '\u{1c}',
        _ => '\u{1b}',
    }
}

fn sanitize(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

fn selector_from_chain(chain: &ComplexSelector) -> Selector {
    let last = chain
        .compounds
        .last()
        .cloned()
        .unwrap_or_else(Compound::default);
    if chain.compounds.len() == 1 && last.attrs.is_empty() && last.pseudos.is_empty() {
        return Selector {
            tag: last.tag,
            id: last.id,
            classes: last.classes,
            universal: last.universal,
        };
    }
    let mut classes = last.classes.clone();
    classes.push(format!("{CHAIN_PREFIX}{}", encode_chain(chain)));
    Selector {
        tag: last.tag,
        id: last.id,
        classes,
        universal: last.universal,
    }
}

fn encode_chain(chain: &ComplexSelector) -> String {
    let mut out = String::new();
    for (i, compound) in chain.compounds.iter().enumerate() {
        if i > 0 {
            out.push(COMP_SEP);
            out.push(match chain.combinators[i - 1] {
                Combinator::Descendant => ' ',
                Combinator::Child => '>',
                Combinator::Adjacent => '+',
                Combinator::GeneralSibling => '~',
            });
        } else {
            out.push('\u{0}');
        }
        out.push_str(&encode_compound(compound, 0));
    }
    out
}

fn encode_compound(compound: &Compound, depth: u8) -> String {
    let sep = sep_for_depth(depth);
    let mut atoms: Vec<String> = Vec::new();
    if let Some(tag) = &compound.tag {
        atoms.push(format!("t{}", sanitize(tag)));
    }
    if let Some(id) = &compound.id {
        atoms.push(format!("i{}", sanitize(id)));
    }
    for class in &compound.classes {
        atoms.push(format!("c{}", sanitize(class)));
    }
    if compound.universal {
        atoms.push("u".to_string());
    }
    for attr in &compound.attrs {
        let op = match attr.op {
            AttrOp::Exists => '0',
            AttrOp::Equal => '1',
            AttrOp::Includes => '2',
            AttrOp::DashMatch => '3',
            AttrOp::Prefix => '4',
            AttrOp::Suffix => '5',
            AttrOp::Substring => '6',
        };
        atoms.push(format!(
            "a{}{FIELD_SEP}{op}{FIELD_SEP}{}{FIELD_SEP}{}",
            sanitize(&attr.name),
            sanitize(&attr.value),
            if attr.case_insensitive { "i" } else { "" }
        ));
    }
    for pseudo in &compound.pseudos {
        let atom = match pseudo {
            PseudoClass::FirstChild => "pf".to_string(),
            PseudoClass::LastChild => "pl".to_string(),
            PseudoClass::OnlyChild => "po".to_string(),
            PseudoClass::Root => "pr".to_string(),
            PseudoClass::Empty => "pe".to_string(),
            PseudoClass::NthChild(a, b) => format!("pn{a},{b}"),
            PseudoClass::Not(arg) => format!(
                "p!{}{FIELD_SEP}{}",
                char::from(b'0' + depth + 1),
                encode_compound(arg, depth + 1)
            ),
            PseudoClass::Where(args) | PseudoClass::Is(args) => {
                let code = if matches!(pseudo, PseudoClass::Where(_)) {
                    'W'
                } else {
                    'I'
                };
                let inner = args
                    .iter()
                    .map(|arg| encode_compound(arg, depth + 1))
                    .collect::<Vec<_>>()
                    .join(&PSEUDO_LIST_SEP.to_string());
                format!("p{code}{}{FIELD_SEP}{inner}", char::from(b'0' + depth + 1))
            }
            PseudoClass::Link => "pL".to_string(),
            PseudoClass::Visited => "pV".to_string(),
            PseudoClass::Hover => "pH".to_string(),
            PseudoClass::Focus => "pF".to_string(),
            PseudoClass::Active => "pA".to_string(),
        };
        atoms.push(atom);
    }
    atoms.join(&sep.to_string())
}

fn decode_chain(payload: &str) -> Option<ComplexSelector> {
    let mut compounds = Vec::new();
    let mut combinators = Vec::new();
    for (i, segment) in payload.split(COMP_SEP).enumerate() {
        let mut chars = segment.chars();
        let marker = chars.next()?;
        if i == 0 {
            if marker != '\u{0}' {
                return None;
            }
        } else {
            combinators.push(match marker {
                ' ' => Combinator::Descendant,
                '>' => Combinator::Child,
                '+' => Combinator::Adjacent,
                '~' => Combinator::GeneralSibling,
                _ => return None,
            });
        }
        compounds.push(decode_compound(&chars.collect::<String>(), 0)?);
    }
    if compounds.is_empty()
        || compounds.len() > MAX_COMPOUNDS
        || combinators.len() + 1 != compounds.len()
    {
        return None;
    }
    Some(ComplexSelector {
        compounds,
        combinators,
    })
}

fn decode_compound(text: &str, depth: u8) -> Option<Compound> {
    let mut compound = Compound::default();
    for atom in text.split(sep_for_depth(depth)) {
        if atom.is_empty() {
            continue;
        }
        let mut chars = atom.chars();
        let kind = chars.next()?;
        let rest = chars.collect::<String>();
        match kind {
            't' => compound.tag = Some(rest),
            'i' => compound.id = Some(rest),
            'c' => compound.classes.push(rest),
            'u' => compound.universal = true,
            'a' => {
                let parts: Vec<&str> = rest.split(FIELD_SEP).collect();
                if parts.len() != 4 {
                    return None;
                }
                let op = match parts[1] {
                    "0" => AttrOp::Exists,
                    "1" => AttrOp::Equal,
                    "2" => AttrOp::Includes,
                    "3" => AttrOp::DashMatch,
                    "4" => AttrOp::Prefix,
                    "5" => AttrOp::Suffix,
                    "6" => AttrOp::Substring,
                    _ => return None,
                };
                compound.attrs.push(AttributeSelector {
                    name: parts[0].to_string(),
                    op,
                    value: parts[2].to_string(),
                    case_insensitive: parts[3] == "i",
                });
            }
            'p' => {
                let mut chars = rest.chars();
                let code = chars.next()?;
                let arg = chars.collect::<String>();
                let pseudo = match code {
                    'f' => PseudoClass::FirstChild,
                    'l' => PseudoClass::LastChild,
                    'o' => PseudoClass::OnlyChild,
                    'r' => PseudoClass::Root,
                    'e' => PseudoClass::Empty,
                    'n' => {
                        let (a, b) = arg.split_once(',')?;
                        PseudoClass::NthChild(a.parse().ok()?, b.parse().ok()?)
                    }
                    '!' => {
                        let nested_depth = arg.chars().next()?.to_digit(10)? as u8;
                        if nested_depth > MAX_NOT_DEPTH || nested_depth <= depth {
                            return None;
                        }
                        let (_, nested) = arg.split_once(FIELD_SEP)?;
                        PseudoClass::Not(Box::new(decode_compound(nested, nested_depth)?))
                    }
                    'W' | 'I' => {
                        let nested_depth = arg.chars().next()?.to_digit(10)? as u8;
                        if nested_depth > MAX_NOT_DEPTH || nested_depth <= depth {
                            return None;
                        }
                        let (_, list) = arg.split_once(FIELD_SEP)?;
                        let args = list
                            .split(PSEUDO_LIST_SEP)
                            .map(|part| decode_compound(part, nested_depth))
                            .collect::<Option<Vec<_>>>()?;
                        if args.is_empty() {
                            return None;
                        }
                        if code == 'W' {
                            PseudoClass::Where(args)
                        } else {
                            PseudoClass::Is(args)
                        }
                    }
                    'L' => PseudoClass::Link,
                    'V' => PseudoClass::Visited,
                    'H' => PseudoClass::Hover,
                    'F' => PseudoClass::Focus,
                    'A' => PseudoClass::Active,
                    _ => return None,
                };
                compound.pseudos.push(pseudo);
            }
            _ => return None,
        }
    }
    Some(compound)
}

// ---- specificity and matching ------------------------------------------

/// Compound-only specificity `(ids, classes, tags)` for the rightmost
/// compound. Use [`specificity_full`] for the whole compound chain.
pub fn specificity(sel: &Selector) -> (u32, u32, u32) {
    (
        u32::from(sel.id.is_some()),
        sel.classes
            .iter()
            .filter(|c| !c.starts_with(CHAIN_PREFIX))
            .count() as u32,
        u32::from(sel.tag.is_some()),
    )
}

/// Full specificity including combinator compounds, attributes, pseudo-classes
/// and `:not()` arguments, as `(ids, classes, tags)`.
pub fn specificity_full(sel: &Selector) -> (u32, u32, u32) {
    specificity_of(&complex(sel))
}

/// Specificity of an already-decoded compound chain.
pub fn specificity_of(chain: &ComplexSelector) -> (u32, u32, u32) {
    let mut ids = 0;
    let mut classes = 0;
    let mut tags = 0;
    for compound in &chain.compounds {
        add_specificity(compound, &mut ids, &mut classes, &mut tags);
    }
    (ids, classes, tags)
}

fn add_specificity(compound: &Compound, ids: &mut u32, classes: &mut u32, tags: &mut u32) {
    *ids += u32::from(compound.id.is_some());
    *classes += compound.classes.len() as u32;
    *classes += compound.attrs.len() as u32;
    *classes += compound
        .pseudos
        .iter()
        .filter(|pseudo| {
            !matches!(
                pseudo,
                PseudoClass::Not(_) | PseudoClass::Where(_) | PseudoClass::Is(_)
            )
        })
        .count() as u32;
    *tags += u32::from(compound.tag.is_some());
    for pseudo in &compound.pseudos {
        match pseudo {
            PseudoClass::Not(arg) => add_specificity(arg, ids, classes, tags),
            // `:where()` contributes nothing; `:is()` takes its most specific
            // variant, but only breaks ties inside this origin/importance level.
            PseudoClass::Is(args) => {
                let mut best = (0, 0, 0);
                for arg in args {
                    let (mut i, mut c, mut t) = (0, 0, 0);
                    add_specificity(arg, &mut i, &mut c, &mut t);
                    best = best.max((i, c, t));
                }
                *ids += best.0;
                *classes += best.1;
                *tags += best.2;
            }
            _ => {}
        }
    }
}

/// Match a selector against one element. This is the compound-only entry point
/// kept for compatibility; it now honours the full chain as well.
pub fn matches_element(sel: &Selector, dom: &Dom, id: NodeId) -> bool {
    matches_selector(sel, dom, id)
}

/// Match a full selector against one element with fresh context.
pub fn matches_selector(sel: &Selector, dom: &Dom, id: NodeId) -> bool {
    if dom.tag(id).is_none() {
        return false;
    }
    let mut ctx = MatchContext::new();
    matches_selector_with(sel, dom, id, &mut ctx)
}

/// Match a selector without decoding when it is a plain compound (the common
/// case); fall back to the full chain matcher for combinators/attrs/pseudos.
pub fn matches_selector_ref(sel: &Selector, dom: &Dom, id: NodeId, ctx: &mut MatchContext) -> bool {
    if sel
        .classes
        .iter()
        .any(|class| class.starts_with(CHAIN_PREFIX))
    {
        matches_selector_with(sel, dom, id, ctx)
    } else {
        matches_plain(sel, dom, id)
    }
}

/// Whether a selector is a plain compound with no chain to decode.
pub fn is_compound(sel: &Selector) -> bool {
    !sel.classes
        .iter()
        .any(|class| class.starts_with(CHAIN_PREFIX))
}

/// Specificity without decoding when the selector is a plain compound.
pub fn specificity_ref(sel: &Selector) -> (u32, u32, u32) {
    if sel
        .classes
        .iter()
        .any(|class| class.starts_with(CHAIN_PREFIX))
    {
        specificity_full(sel)
    } else {
        specificity(sel)
    }
}

/// Fast path for a plain compound (no chain, attributes or pseudo-classes).
/// Allocation-free and independent of `MatchContext` — the caller needs no
/// ancestor Bloom or sibling cache. Used by `hodo-style`'s candidate matcher.
pub fn matches_plain(sel: &Selector, dom: &Dom, id: NodeId) -> bool {
    let Some(tag) = dom.tag(id) else {
        return false;
    };
    if let Some(expected) = &sel.tag {
        if !tag.eq_ignore_ascii_case(expected) {
            return false;
        }
    }
    if let Some(expected) = &sel.id {
        if dom.attr(id, "id") != Some(expected.as_str()) {
            return false;
        }
    }
    if !sel.classes.is_empty() {
        let classes = dom.attr(id, "class").unwrap_or("");
        if !sel
            .classes
            .iter()
            .all(|c| classes.split_whitespace().any(|actual| actual == c))
        {
            return false;
        }
    }
    true
}

/// Match a full selector using reusable ancestor/sibling caches.
pub fn matches_selector_with(
    sel: &Selector,
    dom: &Dom,
    id: NodeId,
    ctx: &mut MatchContext,
) -> bool {
    matches_compiled(&complex(sel), dom, id, ctx)
}

/// Match an already-decoded compound chain using reusable caches.
pub fn matches_compiled(
    chain: &ComplexSelector,
    dom: &Dom,
    id: NodeId,
    ctx: &mut MatchContext,
) -> bool {
    let mut compounds = chain.compounds.iter();
    let Some(last) = compounds.next_back() else {
        return false;
    };
    if !matches_compound(last, dom, id, ctx) {
        return false;
    }

    let mut current = id;
    let mut combinators = chain.combinators.iter().rev();
    for compound in compounds.rev() {
        let Some(combinator) = combinators.next() else {
            return false;
        };
        match combinator {
            Combinator::Descendant => {
                if !ctx.ancestors.might_match(compound) {
                    return false;
                }
                let mut found = false;
                let mut cursor = dom.parent(current);
                while let Some(ancestor) = cursor {
                    if matches_compound(compound, dom, ancestor, ctx) {
                        current = ancestor;
                        found = true;
                        break;
                    }
                    cursor = dom.parent(ancestor);
                }
                if !found {
                    return false;
                }
            }
            Combinator::Child => {
                let Some(parent) = dom.parent(current) else {
                    return false;
                };
                if !matches_compound(compound, dom, parent, ctx) {
                    return false;
                }
                current = parent;
            }
            Combinator::Adjacent => {
                let (_, previous, _) = ctx.siblings.entry(dom, current);
                let Some(previous) = previous else {
                    return false;
                };
                if !matches_compound(compound, dom, previous, ctx) {
                    return false;
                }
                current = previous;
            }
            Combinator::GeneralSibling => {
                let mut cursor = ctx.siblings.entry(dom, current).1;
                let mut found = false;
                while let Some(previous) = cursor {
                    if matches_compound(compound, dom, previous, ctx) {
                        current = previous;
                        found = true;
                        break;
                    }
                    cursor = ctx.siblings.entry(dom, previous).1;
                }
                if !found {
                    return false;
                }
            }
        }
    }
    true
}

fn matches_compound(compound: &Compound, dom: &Dom, id: NodeId, ctx: &mut MatchContext) -> bool {
    if let Some(tag) = &compound.tag {
        let Some(actual) = dom.tag(id) else {
            return false;
        };
        if !actual.eq_ignore_ascii_case(tag) {
            return false;
        }
    }
    if let Some(expected) = &compound.id {
        if dom.attr(id, "id") != Some(expected.as_str()) {
            return false;
        }
    }
    if !compound.classes.is_empty() {
        let classes = dom.attr(id, "class").unwrap_or("");
        if !compound
            .classes
            .iter()
            .all(|c| classes.split_whitespace().any(|actual| actual == c))
        {
            return false;
        }
    }
    for attr in &compound.attrs {
        let Some(actual) = attr_value(dom, id, &attr.name) else {
            return false;
        };
        if !attr_matches(attr, actual) {
            return false;
        }
    }
    for pseudo in &compound.pseudos {
        let (index, previous, next) = ctx.siblings.entry(dom, id);
        let matched = match pseudo {
            PseudoClass::FirstChild => index == 1,
            PseudoClass::LastChild => next.is_none(),
            PseudoClass::OnlyChild => previous.is_none() && next.is_none(),
            PseudoClass::NthChild(a, b) => nth_matches(*a, *b, index),
            PseudoClass::Not(arg) => !matches_compound(arg, dom, id, ctx),
            PseudoClass::Where(args) | PseudoClass::Is(args) => {
                args.iter().any(|arg| matches_compound(arg, dom, id, ctx))
            }
            PseudoClass::Link | PseudoClass::Visited => {
                let is_anchor = dom.tag(id).is_some_and(|tag| tag.eq_ignore_ascii_case("a"));
                let href = attr_value(dom, id, "href");
                let visited = href.map(|href| ctx.is_visited(href)).unwrap_or(false);
                is_anchor
                    && href.is_some()
                    && match pseudo {
                        PseudoClass::Link => !visited,
                        PseudoClass::Visited => visited,
                        _ => unreachable!(),
                    }
            }
            // Static render has no pointer/focus state; never match.
            PseudoClass::Hover | PseudoClass::Focus | PseudoClass::Active => false,
            PseudoClass::Root => dom.parent(id) == Some(dom.root()),
            PseudoClass::Empty => dom
                .children(id)
                .iter()
                .all(|&child| dom.text(child).is_none_or(|text| text.is_empty())),
        };
        if !matched {
            return false;
        }
    }
    true
}

fn nth_matches(a: i32, b: i32, index: usize) -> bool {
    let index = index as i32;
    if a == 0 {
        return index == b;
    }
    if a > 0 {
        index >= b && (index - b) % a == 0
    } else {
        index <= b && (b - index) % (-a) == 0
    }
}

/// HTML attribute names are ASCII case-insensitive, but `Dom::attr` (and the
/// interner behind it) looks names up exactly. The HTML parser stores names in
/// lower case, so retry the other common spelling on a miss; this keeps
/// selectors like `[HREF]` and mixed-case DOM attributes working in both
/// `Dom::new()` and `Dom::with_interning()` modes.
///
/// TODO(fast path): once selector names are normalised at parse time, replace
/// this with a single `dom.attr_interned(id, name)` and compare its u32 id in
/// `attr_matches` — no per-candidate byte walking.
fn attr_value<'a>(dom: &'a Dom, id: NodeId, name: &str) -> Option<&'a str> {
    if let Some(value) = dom.attr(id, name) {
        return Some(value);
    }
    // Matches (the hot case) stay allocation-free; only misses allocate.
    if name.bytes().any(|b| b.is_ascii_uppercase()) {
        return dom.attr(id, &name.to_ascii_lowercase());
    }
    if name.bytes().any(|b| b.is_ascii_lowercase()) {
        return dom.attr(id, &name.to_ascii_uppercase());
    }
    None
}

fn attr_matches(attr: &AttributeSelector, actual: &str) -> bool {
    let (expected, actual) = if attr.case_insensitive {
        (
            std::borrow::Cow::Owned(attr.value.to_ascii_lowercase()),
            std::borrow::Cow::Owned(actual.to_ascii_lowercase()),
        )
    } else {
        (
            std::borrow::Cow::Borrowed(attr.value.as_str()),
            std::borrow::Cow::Borrowed(actual),
        )
    };
    match attr.op {
        AttrOp::Exists => true,
        AttrOp::Equal => actual == expected,
        AttrOp::Includes => actual
            .split_whitespace()
            .any(|token| token == expected.as_ref()),
        AttrOp::DashMatch => actual == expected || actual.starts_with(&format!("{expected}-")),
        AttrOp::Prefix => actual.starts_with(expected.as_ref()),
        AttrOp::Suffix => actual.ends_with(expected.as_ref()),
        AttrOp::Substring => actual.contains(expected.as_ref()),
    }
}

/// Caches shared by all rules matched for one element during a style pass.
#[derive(Debug, Default)]
pub struct MatchContext {
    pub ancestors: AncestorBloom,
    pub siblings: SiblingCache,
    /// Hrefs the browser has visited. Empty by default, so `:link` matches every
    /// `a[href]` and `:visited` none. The shell should seed this from its
    /// redirect-visited-set via `hodo_style::compute_styles_with_visited`.
    pub visited: std::collections::HashSet<String>,
}

impl MatchContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the visited-href set (typically the shell's history set).
    pub fn with_visited<I, S>(visited: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            visited: visited.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    fn is_visited(&self, href: &str) -> bool {
        self.visited.contains(href)
    }
}

/// 64-bit Bloom filter over tags, ids and classes seen on the ancestor path.
#[derive(Debug, Clone, Copy, Default)]
pub struct AncestorBloom {
    bits: u64,
    has_ancestors: bool,
}

impl AncestorBloom {
    /// Build the filter for an element from its parent's filter.
    pub fn child_of(parent: AncestorBloom, dom: &Dom, parent_id: NodeId) -> Self {
        let mut filter = parent;
        filter.has_ancestors = true;
        if let Some(tag) = dom.tag(parent_id) {
            filter.add_str(tag);
        }
        if let Some(id) = dom.attr(parent_id, "id") {
            filter.add_str(id);
        }
        if let Some(classes) = dom.attr(parent_id, "class") {
            for class in classes.split_whitespace() {
                filter.add_str(class);
            }
        }
        filter
    }

    fn add_str(&mut self, text: &str) {
        let (first, second) = hashes(text);
        self.bits |= 1 << first;
        self.bits |= 1 << second;
    }

    fn contains(&self, text: &str) -> bool {
        let (first, second) = hashes(text);
        self.bits & (1 << first) != 0 && self.bits & (1 << second) != 0
    }

    /// Conservative check: false means no ancestor can satisfy the compound.
    pub fn might_match(&self, compound: &Compound) -> bool {
        if !self.has_ancestors {
            return true;
        }
        compound.tag.as_deref().is_none_or(|t| self.contains(t))
            && compound.id.as_deref().is_none_or(|i| self.contains(i))
            && compound.classes.iter().all(|c| self.contains(c))
    }
}

fn hashes(text: &str) -> (u32, u32) {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();
    ((hash & 0x3f) as u32, ((hash >> 32) & 0x3f) as u32)
}

/// Lazily filled 1-based element index and previous/next element siblings.
#[derive(Debug, Default)]
pub struct SiblingCache {
    entries: HashMap<NodeId, (usize, Option<NodeId>, Option<NodeId>)>,
}

impl SiblingCache {
    pub fn entry(&mut self, dom: &Dom, id: NodeId) -> (usize, Option<NodeId>, Option<NodeId>) {
        if let Some(entry) = self.entries.get(&id) {
            return *entry;
        }
        let mut entry = (1, None, None);
        if let Some(parent) = dom.parent(id) {
            let mut index = 0;
            let mut previous = None;
            for &child in dom.children(parent) {
                if dom.tag(child).is_none() {
                    continue;
                }
                index += 1;
                if child == id {
                    entry = (index, previous, None);
                    break;
                }
                previous = Some(child);
            }
            if entry.2.is_none() {
                let mut seen_self = false;
                for &child in dom.children(parent) {
                    if child == id {
                        seen_self = true;
                        continue;
                    }
                    if seen_self && dom.tag(child).is_some() {
                        entry.2 = Some(child);
                        break;
                    }
                }
            }
        }
        self.entries.insert(id, entry);
        entry
    }
}

/// Cap on distinct attribute names kept as dedicated buckets. Beyond this the
/// least frequent names fall back to `universal`, so collecting candidates is
/// O(1) in the number of attribute names rather than O(distinct names).
/// shortcut: 32 names; raise it if a sheet legitimately keys on more attributes.
const MAX_ATTR_BUCKETS: usize = 32;

/// A necessary condition on an element, read from a rule's rightmost compound.
/// A rule is stored under exactly one of these (the rarest), so an element that
/// fails the chosen condition can never match the rule and never sees it as a
/// candidate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RuleKey {
    Id(String),
    Class(String),
    Tag(String),
    Attr(String),
}

/// Rule index keyed by one necessary condition of each rule's rightmost
/// compound. Collecting candidates is O(bucket sizes), never O(all rules), and
/// every rule lives in exactly one bucket so there is no cross-bucket blow-up.
///
/// Bucket choice, in priority order:
/// * an `#id` wins outright (an element with a different id cannot match);
/// * otherwise the rarest of the rule's classes, tag and attribute names wins
///   ("rarest" = fewest other rules carry the same key);
/// * a rule with no such condition goes into `universal`.
///
/// Source order is preserved by sorting the collected rule indices: ties in
/// specificity are decided by the original rule order in `hodo-style`.
#[derive(Debug, Default)]
pub struct RuleIndex {
    by_id: HashMap<String, Vec<u32>>,
    by_class: HashMap<String, Vec<u32>>,
    by_tag: HashMap<String, Vec<u32>>,
    by_attr: HashMap<String, Vec<u32>>,
    universal: Vec<u32>,
    /// Attribute names with a dedicated bucket, most frequent first. Bounded by
    /// [`MAX_ATTR_BUCKETS`] so the per-element scan stays constant-time.
    attr_names: Vec<String>,
    /// Decoded chains for non-plain rules as `(rule index, chain, specificity)`,
    /// sorted by index. Specificity is computed once here so the cascade never
    /// decodes a chain per element. Plain compounds need no decode at match
    /// time; keeping only the non-plain ones avoids a per-rule `Option` slot.
    chains: Vec<(u32, ComplexSelector, (u32, u32, u32))>,
}

impl RuleIndex {
    pub fn new<'a>(rules: impl IntoIterator<Item = &'a StyleRule>) -> Self {
        let rules: Vec<&StyleRule> = rules.into_iter().collect();
        // Pass 1: collect every necessary key and count how many rules share
        // it, so the rarest key can be picked in pass 2.
        let mut all: HashMap<RuleKey, Vec<u32>> = HashMap::new();
        let mut keys_per_rule: Vec<Vec<RuleKey>> = Vec::with_capacity(rules.len());
        let mut chains: Vec<(u32, ComplexSelector, (u32, u32, u32))> = Vec::new();
        for (i, rule) in rules.iter().enumerate() {
            let chain = complex(&rule.selector);
            let mut keys = Vec::new();
            if let Some(last) = chain.compounds.last() {
                if let Some(id) = &last.id {
                    keys.push(RuleKey::Id(id.clone()));
                } else {
                    for class in &last.classes {
                        keys.push(RuleKey::Class(class.clone()));
                    }
                    if let Some(tag) = &last.tag {
                        keys.push(RuleKey::Tag(tag.to_ascii_lowercase()));
                    }
                    for attr in &last.attrs {
                        keys.push(RuleKey::Attr(attr.name.to_ascii_lowercase()));
                    }
                }
            }
            for key in &keys {
                all.entry(key.clone()).or_default().push(i as u32);
            }
            if !is_compound(&rule.selector) {
                let specificity = specificity_of(&chain);
                chains.push((i as u32, chain, specificity));
            }
            keys_per_rule.push(keys);
        }

        // Keep only the most frequent attribute names as dedicated buckets.
        let mut attr_freq: Vec<(String, usize)> = all
            .iter()
            .filter_map(|(key, list)| match key {
                RuleKey::Attr(name) => Some((name.clone(), list.len())),
                _ => None,
            })
            .collect();
        attr_freq.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        attr_freq.truncate(MAX_ATTR_BUCKETS);
        let attr_allowed: HashSet<String> = attr_freq.iter().map(|(n, _)| n.clone()).collect();

        let mut index = Self {
            attr_names: attr_freq.into_iter().map(|(n, _)| n).collect(),
            chains,
            ..Self::default()
        };
        for (i, keys) in keys_per_rule.iter().enumerate() {
            // `Id` uses weight 0, always beating any real frequency (>= 1).
            let chosen = keys.iter().min_by_key(|key| match key {
                RuleKey::Id(_) => 0,
                _ => all.get(*key).map_or(usize::MAX, Vec::len),
            });
            match chosen {
                Some(RuleKey::Id(id)) => index.by_id.entry(id.clone()).or_default().push(i as u32),
                Some(RuleKey::Class(class)) => index
                    .by_class
                    .entry(class.clone())
                    .or_default()
                    .push(i as u32),
                Some(RuleKey::Tag(tag)) => {
                    index.by_tag.entry(tag.clone()).or_default().push(i as u32)
                }
                Some(RuleKey::Attr(name)) if attr_allowed.contains(name.as_str()) => index
                    .by_attr
                    .entry(name.clone())
                    .or_default()
                    .push(i as u32),
                _ => index.universal.push(i as u32),
            }
        }
        index
    }

    /// Fill `out` with sorted, deduplicated candidate rule indices for one
    /// element. Callers reuse the buffer across elements.
    pub fn candidates(&self, dom: &Dom, id: NodeId, out: &mut Vec<usize>) {
        out.clear();
        if let Some(value) = attr_value(dom, id, "id") {
            if let Some(indices) = self.by_id.get(value) {
                out.extend(indices.iter().map(|&i| i as usize));
            }
        }
        if let Some(classes) = attr_value(dom, id, "class") {
            for class in classes.split_whitespace() {
                if let Some(indices) = self.by_class.get(class) {
                    out.extend(indices.iter().map(|&i| i as usize));
                }
            }
        }
        if let Some(tag) = dom.tag(id) {
            if let Some(indices) = self.by_tag.get(&tag.to_ascii_lowercase()) {
                out.extend(indices.iter().map(|&i| i as usize));
            }
        }
        for name in &self.attr_names {
            if attr_value(dom, id, name).is_some() {
                if let Some(indices) = self.by_attr.get(name) {
                    out.extend(indices.iter().map(|&i| i as usize));
                }
            }
        }
        out.extend(self.universal.iter().map(|&i| i as usize));
        out.sort_unstable();
        out.dedup();
    }

    /// Pre-decoded chain for a non-plain rule, or `None` for a plain compound.
    /// Lets callers skip the `CHAIN_PREFIX` scan and `complex()` decode on every
    /// match, while keeping the generic chain matcher for everything else.
    pub fn chain(&self, rule_index: usize) -> Option<&ComplexSelector> {
        self.chains
            .binary_search_by_key(&(rule_index as u32), |(i, _, _)| *i)
            .ok()
            .map(|position| &self.chains[position].1)
    }

    /// Specificity of a rule without decoding its chain: cached from the
    /// one-shot build decode for non-plain rules, compound-only for plain ones.
    pub fn specificity(&self, rule_index: usize, sel: &Selector) -> (u32, u32, u32) {
        match self
            .chains
            .binary_search_by_key(&(rule_index as u32), |(i, _, _)| *i)
        {
            Ok(position) => self.chains[position].2,
            Err(_) => specificity(sel),
        }
    }

    /// Approximate bytes held by the final index (rule lists + key strings +
    /// table slots). Used by the RAM report in the tests; excludes the
    /// one-shot build scratch maps, which are freed when `new` returns.
    pub fn heap_bytes(&self) -> usize {
        let mut bytes = self.universal.capacity() * std::mem::size_of::<u32>();
        for (_, chain, _) in &self.chains {
            bytes += std::mem::size_of_val(chain);
            for compound in &chain.compounds {
                bytes += compound.tag.as_ref().map_or(0, String::len)
                    + compound.id.as_ref().map_or(0, String::len)
                    + compound.classes.iter().map(String::len).sum::<usize>()
                    + compound
                        .attrs
                        .iter()
                        .map(|a| a.name.len() + a.value.len())
                        .sum::<usize>();
            }
        }
        bytes +=
            self.chains.capacity() * std::mem::size_of::<(u32, ComplexSelector, (u32, u32, u32))>();
        for map in [&self.by_id, &self.by_class, &self.by_tag, &self.by_attr] {
            bytes += map.capacity()
                * (std::mem::size_of::<String>()
                    + std::mem::size_of::<Vec<u32>>()
                    + std::mem::size_of::<usize>());
            for (key, list) in map {
                bytes += key.len() + list.capacity() * std::mem::size_of::<u32>();
            }
        }
        for name in &self.attr_names {
            bytes += name.len();
        }
        bytes + std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_dom::NodeKind;
    use std::time::{Duration, Instant};

    fn selector(s: &str) -> Selector {
        let sheet = parse_stylesheet(&format!("{s} {{ color: red }}"));
        assert_eq!(sheet.rules.len(), 1, "selector should be accepted: {s}");
        sheet.rules[0].selector.clone()
    }

    fn declarations(css: &str) -> Vec<Declaration> {
        parse_stylesheet(css).rules[0].declarations.clone()
    }

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.into(),
            attrs: attrs.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        });
        dom.append(parent, id);
        id
    }

    // --- original suite --------------------------------------------------

    #[test]
    fn parses_realistic_stylesheet() {
        let css = r#"
            * { margin: 0; padding: 0 }
            body { font-family: "My Font", serif; background-color: #fff }
            div.card#main { color: rgb(10, 20, 30); margin: 10px auto 5% 0 }
            .warning { border: 1px solid red }
            #footer { display: none }
        "#;
        let sheet = parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 5);

        assert!(sheet.rules[0].selector.universal);

        assert_eq!(sheet.rules[1].selector.tag.as_deref(), Some("body"));
        assert_eq!(sheet.rules[1].declarations.len(), 2);
        assert_eq!(sheet.rules[1].declarations[0].property, "font-family");
        assert_eq!(sheet.rules[1].declarations[0].value, "\"My Font\", serif");
        assert_eq!(sheet.rules[1].declarations[1].property, "background-color");
        assert_eq!(sheet.rules[1].declarations[1].value, "#fff");

        let main = &sheet.rules[2];
        assert_eq!(main.selector.tag.as_deref(), Some("div"));
        assert_eq!(main.selector.id.as_deref(), Some("main"));
        assert_eq!(main.selector.classes, ["card"]);
        assert_eq!(main.declarations[0].value, "rgb(10, 20, 30)");
        assert_eq!(main.declarations[1].value, "10px auto 5% 0");

        assert_eq!(sheet.rules[3].selector.classes, ["warning"]);
        assert_eq!(sheet.rules[4].selector.id.as_deref(), Some("footer"));
    }

    #[test]
    fn selector_list_produces_one_rule_each() {
        let sheet = parse_stylesheet("h1, h2, .card { color: red; margin: 0 }");
        assert_eq!(sheet.rules.len(), 3);
        assert_eq!(sheet.rules[0].selector.tag.as_deref(), Some("h1"));
        assert_eq!(sheet.rules[1].selector.tag.as_deref(), Some("h2"));
        assert_eq!(sheet.rules[2].selector.classes, ["card"]);
        for rule in &sheet.rules {
            assert_eq!(rule.declarations.len(), 2);
        }
    }

    #[test]
    fn malformed_declarations_recover_to_next_rule() {
        let css = "broken { color red; color: blue } p { margin: 5px; : bad; font-size: 12px }";
        let sheet = parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 2);
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        assert_eq!(sheet.rules[1].declarations.len(), 2);
        assert_eq!(sheet.rules[1].declarations[1].property, "font-size");
        assert_eq!(sheet.rules[1].declarations[1].value, "12px");
    }

    #[test]
    fn skips_at_rules_with_and_without_blocks() {
        let css = r#"
            @charset "UTF-8";
            @import "theme.css";
            @media (max-width: 600px) { .x { color: red } @font-face { src: url(a.woff) } }
            @keyframes fade { from { opacity: 0 } to { opacity: 1 } }
            p { color: green }
        "#;
        let sheet = parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector.tag.as_deref(), Some("p"));
        assert_eq!(sheet.rules[0].declarations[0].value, "green");

        let css = "div { @supports (display: grid) { color: red } color: blue }";
        let sheet = parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
    }

    #[test]
    fn preserves_declaration_values() {
        let decls = declarations(
            r#"a { font-family: "My Font", serif; background: url("img/a b.png") no-repeat;
                   width: 50%; opacity: 0.5; transform: translate(10px, -2px) }"#,
        );
        assert_eq!(decls[0].value, "\"My Font\", serif");
        assert_eq!(decls[1].value, "url(\"img/a b.png\") no-repeat");
        assert_eq!(decls[2].value, "50%");
        assert_eq!(decls[3].value, "0.5");
        assert_eq!(decls[4].value, "translate(10px, -2px)");
    }

    #[test]
    fn property_names_are_lowercased_except_custom_ones() {
        let decls = declarations("a { COLOR: red; --Brand-Color: #abc; -webkit-transform: none }");
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[1].property, "--Brand-Color");
        assert_eq!(decls[2].property, "-webkit-transform");
    }

    #[test]
    fn specificity_counts_ids_classes_tags() {
        assert_eq!(specificity(&selector("*")), (0, 0, 0));
        assert_eq!(specificity(&selector("div")), (0, 0, 1));
        assert_eq!(specificity(&selector(".a.b")), (0, 2, 0));
        assert_eq!(specificity(&selector("div.a#main")), (1, 1, 1));
        assert!(specificity(&selector(".a")) < specificity(&selector("div.a#main")));
    }

    #[test]
    fn matches_element_against_dom_tree() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = elem(
            &mut dom,
            root,
            "DIV",
            &[("id", "main"), ("class", "card  highlighted")],
        );
        let text = dom.create(NodeKind::Text("hello".to_string()));
        dom.append(root, text);

        assert!(matches_element(&selector("div"), &dom, div));
        assert!(matches_element(&selector("DIV.card"), &dom, div));
        assert!(matches_element(
            &selector("div.card.highlighted#main"),
            &dom,
            div
        ));
        assert!(matches_element(&selector("*"), &dom, div));
        assert!(!matches_element(&selector("span"), &dom, div));
        assert!(!matches_element(&selector("div#other"), &dom, div));
        assert!(!matches_element(&selector("div.missing"), &dom, div));
        assert!(!matches_element(&selector("div"), &dom, text));
    }

    #[test]
    fn never_panics_on_malformed_css() {
        let css = "@media { div { color: red } } </broken> { color: ; ; ::: } a { color: blue";
        let _ = parse_stylesheet(css);
        let _ = parse_stylesheet("");
        let _ = parse_stylesheet("}{)(@@#");
        let _ = parse_stylesheet(":not( { color: red } p { color: blue }");
        let _ = parse_stylesheet("[=] { color: red } a { color: blue }");
    }

    // --- M7.1: specificity truth table ----------------------------------

    #[test]
    fn specificity_full_truth_table() {
        let cases: &[(&str, (u32, u32, u32))] = &[
            ("*", (0, 0, 0)),
            ("div", (0, 0, 1)),
            (".a.b", (0, 2, 0)),
            ("div.a#main", (1, 1, 1)),
            ("[a=b]", (0, 1, 0)),
            (":root", (0, 1, 0)),
            ("li:nth-child(2n+1)", (0, 1, 1)),
            ("a:not(.b)", (0, 1, 1)),
            ("a:not(#id)", (1, 0, 1)),
            (":not(:not(.x))", (0, 1, 0)),
            ("#a .b > c", (1, 1, 1)),
            ("ul > li + a", (0, 0, 3)),
        ];
        for &(text, expected) in cases {
            assert_eq!(
                specificity_full(&selector(text)),
                expected,
                "specificity of {text}"
            );
        }
    }

    #[test]
    fn nth_forms() {
        assert_eq!(parse_nth("odd"), Some((2, 1)));
        assert_eq!(parse_nth("even"), Some((2, 0)));
        assert_eq!(parse_nth("2n+1"), Some((2, 1)));
        assert_eq!(parse_nth("-n+3"), Some((-1, 3)));
        assert_eq!(parse_nth("3"), Some((0, 3)));
        assert!(nth_matches(2, 1, 5));
        assert!(!nth_matches(2, 1, 4));
        assert!(nth_matches(-1, 3, 1));
        assert!(nth_matches(0, 3, 3));
        assert!(!nth_matches(0, 3, 4));
    }

    // --- M7.1: combinators and pseudo-classes ----------------------------

    fn sample_tree() -> (Dom, NodeId, NodeId, NodeId, NodeId) {
        let mut dom = Dom::new();
        let root = dom.root();
        let html = elem(&mut dom, root, "html", &[]);
        let body = elem(&mut dom, html, "body", &[]);
        let list = elem(&mut dom, body, "ul", &[("id", "list")]);
        let first = elem(&mut dom, list, "li", &[("class", "first")]);
        let target = elem(&mut dom, list, "li", &[("id", "target")]);
        let last = elem(&mut dom, list, "li", &[("class", "last")]);
        elem(&mut dom, body, "div", &[("class", "wrap")]);
        (dom, html, first, target, last)
    }

    #[test]
    fn combinators_match_right_to_left() {
        let (dom, html, first, target, last) = sample_tree();
        assert!(matches_selector(
            &selector("html body ul li#target"),
            &dom,
            target
        ));
        assert!(matches_selector(&selector("ul > li#target"), &dom, target));
        assert!(matches_selector(
            &selector("li.first + li#target"),
            &dom,
            target
        ));
        assert!(matches_selector(
            &selector("li.first ~ li#target"),
            &dom,
            target
        ));
        assert!(matches_selector(
            &selector("li.first ~ li.last"),
            &dom,
            last
        ));
        assert!(matches_selector(
            &selector("body > ul > li:first-child"),
            &dom,
            first
        ));
        assert!(!matches_selector(
            &selector("li.first + li.last"),
            &dom,
            last
        ));
        assert!(!matches_selector(&selector("body > li"), &dom, target));
        assert!(!matches_selector(&selector("div.wrap li"), &dom, target));
        assert!(matches_selector(&selector(":root"), &dom, html));
        assert!(!matches_selector(&selector(":root"), &dom, target));
    }

    #[test]
    fn descendant_selectors_are_not_folded_into_one_compound() {
        let mut dom = Dom::new();
        let root = dom.root();
        let outer = elem(&mut dom, root, "div", &[("class", "a")]);
        let nested = elem(
            &mut dom,
            outer,
            "span",
            &[("class", "b"), ("id", "b"), ("data-x", "1")],
        );
        let second = elem(&mut dom, outer, "span", &[("class", "c")]);
        // A single node carrying both classes/id/attr must not satisfy a
        // descendant selector: `.a .b` needs two nodes in an ancestor chain.
        let both = elem(
            &mut dom,
            root,
            "div",
            &[("class", "a b"), ("id", "b"), ("data-x", "1")],
        );

        for text in [".a .b", ".a #b", ".a *", ".a [data-x]", ".a :first-child"] {
            let sel = selector(text);
            assert!(
                !matches_selector(&sel, &dom, both),
                "{text} must not match a single compound node"
            );
            assert!(
                matches_selector(&sel, &dom, nested),
                "{text} must match the nested node"
            );
        }
        assert!(!matches_selector(&selector(".a .b"), &dom, second));
        // The sibling (2nd child of root) has class `a` but is not itself a
        // descendant of another `.a`, so it cannot match `.a :nth-child(2)`.
        assert!(!matches_selector(&selector(".a :nth-child(2)"), &dom, both));
        assert!(matches_selector(
            &selector(".a :nth-child(2)"),
            &dom,
            second
        ));
    }

    #[test]
    fn nth_child_after_whitespace_is_a_descendant() {
        let mut dom = Dom::new();
        let root = dom.root();
        let outer = elem(&mut dom, root, "div", &[("class", "a")]);
        let first = elem(&mut dom, outer, "li", &[]);
        let second = elem(&mut dom, outer, "li", &[]);
        assert!(matches_selector(
            &selector(".a :nth-child(2)"),
            &dom,
            second
        ));
        assert!(!matches_selector(
            &selector(".a :nth-child(2)"),
            &dom,
            first
        ));
        assert!(!matches_selector(
            &selector(".a :nth-child(2)"),
            &dom,
            outer
        ));
    }

    #[test]
    fn empty_comma_groups_do_not_misalign_streamed_selectors() {
        let sheet = parse_stylesheet(", :not(.x) { color: red } p { color: blue }");
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["red", "blue"]);
        let mut dom = Dom::new();
        let root = dom.root();
        let x = elem(&mut dom, root, "div", &[("class", "x")]);
        let plain = elem(&mut dom, root, "div", &[]);
        assert!(!matches_selector(&sheet.rules[0].selector, &dom, x));
        assert!(matches_selector(&sheet.rules[0].selector, &dom, plain));

        let sheet = parse_stylesheet(":not(.a), , :not(.b) { color: red } p { color: blue }");
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["red", "red", "blue"]);
        let a = elem(&mut dom, root, "div", &[("class", "a")]);
        let b = elem(&mut dom, root, "div", &[("class", "b")]);
        assert!(!matches_selector(&sheet.rules[0].selector, &dom, a));
        assert!(matches_selector(&sheet.rules[0].selector, &dom, b));
        assert!(matches_selector(&sheet.rules[1].selector, &dom, a));
        assert!(!matches_selector(&sheet.rules[1].selector, &dom, b));
    }

    #[test]
    fn not_rejects_nested_combinators_without_affecting_siblings() {
        let sheet = parse_stylesheet(":not(.a .b) { color: red } p { color: blue }");
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["blue"]);
        let sheet = parse_stylesheet("a:not(.x .y) { color: red } p { color: blue }");
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["blue"]);
    }

    #[test]
    fn link_and_visited_split_on_the_visited_set() {
        let mut dom = Dom::new();
        let root = dom.root();
        let link = elem(&mut dom, root, "a", &[("href", "https://example.com")]);
        let bare = elem(&mut dom, root, "a", &[]);
        let other = elem(&mut dom, root, "div", &[("href", "#")]);
        let href = "https://example.com";

        for (text, expected_unvisited, expected_visited) in [
            ("a:link", true, false),
            ("a:visited", false, true),
            (":link", true, false),
            (":visited", false, true),
        ] {
            let sel = selector(text);
            let mut ctx = MatchContext::new();
            assert_eq!(
                matches_selector_with(&sel, &dom, link, &mut ctx),
                expected_unvisited,
                "{text} on an unvisited link"
            );
            let mut ctx = MatchContext::with_visited([href]);
            assert_eq!(
                matches_selector_with(&sel, &dom, link, &mut ctx),
                expected_visited,
                "{text} on a visited link"
            );
            let mut ctx = MatchContext::new();
            assert!(!matches_selector_with(&sel, &dom, bare, &mut ctx));
            assert!(!matches_selector_with(&sel, &dom, other, &mut ctx));
        }
        // HN-shaped selector lists survive parsing; one arm always applies.
        let sheet = parse_stylesheet("a:link, a:visited { color: blue } p { color: red }");
        assert_eq!(sheet.rules.len(), 3);
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        assert_eq!(sheet.rules[1].declarations[0].value, "blue");
        let mut ctx = MatchContext::new();
        assert!(matches_selector_with(
            &sheet.rules[0].selector,
            &dom,
            link,
            &mut ctx
        ));
        let mut ctx = MatchContext::with_visited([href]);
        assert!(matches_selector_with(
            &sheet.rules[1].selector,
            &dom,
            link,
            &mut ctx
        ));
    }

    #[test]
    fn interaction_pseudos_parse_but_never_match_statically() {
        let mut dom = Dom::new();
        let root = dom.root();
        let link = elem(&mut dom, root, "a", &[("href", "#")]);
        for text in ["a:hover", "a:focus", "a:active", ":hover"] {
            let sel = selector(text); // must survive parsing
            assert!(!matches_selector(&sel, &dom, link), "{text} must not match");
        }
        // A non-matching interaction arm must not sink its sibling link arm.
        let sheet = parse_stylesheet("a:hover, a:link { color: blue } p { color: red }");
        assert_eq!(sheet.rules.len(), 3);
        assert!(!matches_selector(&sheet.rules[0].selector, &dom, link));
        assert!(matches_selector(&sheet.rules[1].selector, &dom, link));
    }

    #[test]
    fn where_and_is_match_with_expected_specificity() {
        let mut dom = Dom::new();
        let root = dom.root();
        let wrapper = elem(&mut dom, root, "div", &[("id", "menu"), ("class", "menu")]);
        let link = elem(&mut dom, wrapper, "a", &[("href", "#")]);
        let flagged = elem(&mut dom, root, "p", &[("id", "b")]);
        let other = elem(&mut dom, root, "p", &[("class", "a")]);

        assert_eq!(specificity_full(&selector(":where(#id) a")), (0, 0, 1));
        assert_eq!(specificity_full(&selector(":where(.menu) a")), (0, 0, 1));
        assert_eq!(specificity_full(&selector(":is(.a, #b)")), (1, 0, 0));

        assert!(matches_selector(&selector(":where(#menu) a"), &dom, link));
        assert!(matches_selector(&selector(":where(.menu) a"), &dom, link));
        assert!(!matches_selector(
            &selector(":where(#menu) a"),
            &dom,
            wrapper
        ));
        assert!(matches_selector(&selector(":is(.a, #b)"), &dom, flagged));
        assert!(matches_selector(&selector(":is(.a, #b)"), &dom, other));

        // Inner combinators are unsupported: the rule drops, siblings survive.
        for css in [
            ":is(.a .b) { color: red } p { color: blue }",
            ":where(> .a) { color: red } p { color: blue }",
            ":where(:is(:where(.a))) { color: red } p { color: blue }",
        ] {
            let sheet = parse_stylesheet(css);
            assert_eq!(sheet.rules.len(), 1, "unsupported inner: {css}");
            assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        }
    }

    #[test]
    fn child_pseudo_classes_and_empty() {
        let (mut dom, _, first, target, last) = sample_tree();
        assert!(matches_selector(&selector("li:first-child"), &dom, first));
        assert!(matches_selector(&selector("li:last-child"), &dom, last));
        assert!(matches_selector(
            &selector("li:nth-child(even)"),
            &dom,
            target
        ));
        assert!(matches_selector(
            &selector("li:nth-child(2n)"),
            &dom,
            target
        ));
        assert!(!matches_selector(
            &selector("li:nth-child(odd)"),
            &dom,
            target
        ));
        assert!(!matches_selector(&selector("li:only-child"), &dom, target));
        assert!(!matches_selector(&selector("ul:empty"), &dom, dom.root()));
        let root = dom.root();
        let empty = elem(&mut dom, root, "div", &[]);
        assert!(matches_selector(&selector("div:empty"), &dom, empty));
        let text = dom.create(NodeKind::Text("x".into()));
        dom.append(empty, text);
        assert!(!matches_selector(&selector("div:empty"), &dom, empty));
    }

    #[test]
    fn not_takes_simple_argument_and_nests_twice() {
        let (dom, _, first, target, _) = sample_tree();
        assert!(matches_selector(&selector("li:not(.first)"), &dom, target));
        assert!(!matches_selector(&selector("li:not(.first)"), &dom, first));
        assert!(matches_selector(
            &selector(":not(div):not(#nope)"),
            &dom,
            target
        ));
        assert!(matches_selector(&selector(":not(:not(li))"), &dom, target));
        assert!(!matches_selector(
            &selector(":not(:not(div))"),
            &dom,
            target
        ));
    }

    // --- M7.1: attribute operators ---------------------------------------

    #[test]
    fn attribute_operators_and_case_flag() {
        let mut dom = Dom::new();
        let root = dom.root();
        let link = elem(
            &mut dom,
            root,
            "a",
            &[
                ("href", "https://example.com"),
                ("class", "one two"),
                ("LANG", "en-US"),
                ("data-x", "abc"),
            ],
        );
        let cases: &[(&str, bool)] = &[
            ("[href]", true),
            ("[missing]", false),
            ("[href='https://example.com']", true),
            ("[href='https://example.org']", false),
            ("[class~='two']", true),
            ("[class~='two-three']", false),
            ("[lang|='en']", true),
            ("[lang|='en-US-x']", false),
            ("[href^='https']", true),
            ("[href^='http:']", false),
            ("[href$='com']", true),
            ("[href$='org']", false),
            ("[href*='example']", true),
            ("[href*='nope']", false),
            ("[lang='EN-US' i]", true),
            ("[class~='TWO' i]", true),
            ("[lang='EN-US']", false),
            ("[data-x]", true),
        ];
        for &(text, expected) in cases {
            assert_eq!(
                matches_selector(&selector(text), &dom, link),
                expected,
                "attribute selector {text}"
            );
        }
    }

    #[test]
    fn attribute_selectors_match_with_interning() {
        // Mirror of the attribute-selector path with per-node strings stripped:
        // the pool must serve tag/attr reads identically to Dom::new() mode.
        let mut dom = Dom::with_interning();
        let root = dom.root();
        let link = elem(
            &mut dom,
            root,
            "a",
            &[("href", "https://example.com"), ("LANG", "en-US")],
        );
        for &(text, expected) in &[
            ("a[href]", true),
            ("a[href$='com']", true),
            ("a[lang]", true),
            ("a[lang|='en']", true),
            ("a[HREF^='https' i]", true),
            ("a[missing]", false),
        ] {
            assert_eq!(
                matches_selector(&selector(text), &dom, link),
                expected,
                "interned DOM: attribute selector {text}"
            );
        }
    }

    // --- M7.1: error recovery, limits, robustness ------------------------

    #[test]
    fn invalid_selector_drops_only_its_own_rule() {
        let css = ":bogus { color: red } p { color: blue } > { color: green } \
                   div::before { color: yellow } .ok:not() { color: pink } a { color: black }";
        let sheet = parse_stylesheet(css);
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["blue", "black"]);
    }

    #[test]
    fn selector_compound_and_not_depth_limits() {
        let many = (0..MAX_COMPOUNDS)
            .map(|_| "div")
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            parse_stylesheet(&format!("{many} {{ color: red }}"))
                .rules
                .len()
                == 1
        );
        let too_many = (0..=MAX_COMPOUNDS)
            .map(|_| "div")
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            parse_stylesheet(&format!("{too_many} {{ color: red }} p {{ color: blue }}"))
                .rules
                .len()
                == 1
        );
        assert!(
            parse_stylesheet(":not(:not(.a)) { color: red }")
                .rules
                .len()
                == 1
        );
        assert!(
            parse_stylesheet(":not(:not(:not(.a))) { color: red } p { color: blue }")
                .rules
                .len()
                == 1
        );
    }

    #[test]
    fn pathological_deep_dom_completes_under_budget() {
        let mut dom = Dom::new();
        let mut cursor = dom.root();
        for _ in 0..1000 {
            cursor = elem(&mut dom, cursor, "div", &[]);
        }
        let deep = selector(
            &(0..MAX_COMPOUNDS)
                .map(|_| "div")
                .collect::<Vec<_>>()
                .join(" "),
        );
        let start = Instant::now();
        assert!(matches_selector(&deep, &dom, cursor));
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(1),
            "matching took {elapsed:?}"
        );
    }

    #[test]
    fn rule_index_buckets_rightmost_compound() {
        let sheet = parse_stylesheet(
            "#main { color: red } .card { color: blue } \
                                      div { color: green } * { color: black }",
        );
        let index = RuleIndex::new(sheet.rules.iter());
        let mut dom = Dom::new();
        let root = dom.root();
        let node = elem(&mut dom, root, "DIV", &[("id", "main"), ("class", "card")]);
        let mut out = Vec::new();
        index.candidates(&dom, node, &mut out);
        assert_eq!(out, vec![0, 1, 2, 3]);
        let other = elem(&mut dom, root, "span", &[]);
        index.candidates(&dom, other, &mut out);
        assert_eq!(out, vec![3]);
    }

    #[test]
    fn rule_index_id_first_wins_and_uses_rarest_class() {
        // 20 `.common` rules and one `.rare`: a `.rare.common` rule must be
        // indexed under `.rare`, not the crowded `.common` bucket.
        let mut css = String::from(".rare.common { color: red }\n");
        for _ in 0..20 {
            css.push_str(".common { color: blue }\n");
        }
        let sheet = parse_stylesheet(&css);
        let index = RuleIndex::new(sheet.rules.iter());
        let mut dom = Dom::new();
        let root = dom.root();
        let both = elem(&mut dom, root, "div", &[("class", "rare common")]);
        let mut out = Vec::new();
        index.candidates(&dom, both, &mut out);
        assert_eq!(out, (0..21).collect::<Vec<_>>());
        let common_only = elem(&mut dom, root, "div", &[("class", "common")]);
        index.candidates(&dom, common_only, &mut out);
        assert_eq!(
            out,
            (1..21).collect::<Vec<_>>(),
            "rarest-class rule gated out"
        );

        // An id rule is never reachable from a class bucket: an element with
        // the class but a different id cannot match `#main.card`.
        let sheet = parse_stylesheet("#main.card { color: red } .card { color: blue }");
        let index = RuleIndex::new(sheet.rules.iter());
        let wrong_id = elem(&mut dom, root, "div", &[("id", "other"), ("class", "card")]);
        index.candidates(&dom, wrong_id, &mut out);
        assert_eq!(out, vec![1]);
        let right_id = elem(&mut dom, root, "div", &[("id", "main"), ("class", "card")]);
        index.candidates(&dom, right_id, &mut out);
        assert_eq!(out, vec![0, 1]);
    }

    #[test]
    fn rule_index_preserves_source_order_within_and_across_buckets() {
        // Same specificity, different buckets: collected candidates must stay
        // in ascending rule order so `hodo-style` cascades by source order.
        let sheet = parse_stylesheet(
            "[data-kind=\"mixed\"] { color: #010101 }\n\
             .b { color: #020202 }\n\
             div { color: #030303 }\n\
             #i { color: #040404 }\n\
             * { color: #050505 }\n\
             .a { color: #060606 }\n",
        );
        let index = RuleIndex::new(sheet.rules.iter());
        let mut dom = Dom::new();
        let root = dom.root();
        let node = elem(
            &mut dom,
            root,
            "div",
            &[("id", "i"), ("class", "b a"), ("data-kind", "mixed")],
        );
        let mut out = Vec::new();
        index.candidates(&dom, node, &mut out);
        assert_eq!(out, vec![0, 1, 2, 3, 4, 5], "source order must survive");
    }

    /// Fixed-seed synthetic elements over the demo stylesheet shapes. The LCG
    /// is the same one used by `hodo-perf`, so the corpus is reproducible with
    /// no `rand` dependency.
    fn ab_corpus() -> (Dom, Vec<NodeId>, Vec<Stylesheet>) {
        let mut dom = Dom::new();
        let root = dom.root();
        let body = elem(&mut dom, root, "body", &[]);
        let tags = ["div", "p", "span", "li", "a", "section", "ul"];
        let ids = ["blk0", "blk1", "main", "footer"];
        let class_pool = [
            "s0", "s1", "s2", "s3", "s4", "s5", "foo", "bar", "card", "sp0", "sp1", "sp2",
        ];
        let mut rng = 0x5eed_1234_5678_9abcu64;
        let mut next = || {
            rng = rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            rng
        };
        let mut class_text: Vec<String> = Vec::new();
        let mut nodes = Vec::new();
        let mut parent = body;
        for i in 0..420 {
            if i % 13 == 0 {
                parent = body;
            }
            let tag = tags[(next() >> 33) as usize % tags.len()];
            let id = if i % 9 == 0 {
                Some(ids[(next() >> 33) as usize % ids.len()])
            } else {
                None
            };
            let a = class_pool[(next() >> 33) as usize % class_pool.len()];
            let b = class_pool[(next() >> 33) as usize % class_pool.len()];
            let text = if i % 4 == 0 {
                format!("{a} {b}")
            } else {
                a.to_string()
            };
            let mut attrs: Vec<(&str, &str)> = Vec::new();
            if let Some(id) = id {
                attrs.push(("id", id));
            }
            class_text.push(text);
            attrs.push(("class", class_text.last().unwrap()));
            if i % 7 == 0 {
                attrs.push(("data-kind", "mixed"));
            }
            if i % 11 == 0 {
                attrs.push(("data-idx", "3"));
            }
            if i % 5 == 0 {
                attrs.push(("href", "https://example.com"));
            }
            let node = elem(&mut dom, parent, tag, &attrs);
            nodes.push(node);
            if i % 3 == 0 {
                parent = node;
            }
        }
        // Two demo-shaped sheets: LARGE-like class/id/attr rules plus a
        // chained/positional sheet exercising the rightmost-compound decode.
        let mut large_css = String::from("* { margin: 0 }\n");
        for c in class_pool.iter().take(6) {
            large_css.push_str(&format!(".{c} {{ color: red; padding: 1px }}\n"));
        }
        for id in ids {
            large_css.push_str(&format!("#{id} {{ border-width: 1px; margin: 2px }}\n"));
        }
        for i in 0..12 {
            large_css.push_str(&format!(
                "[data-idx=\"{}\"] {{ font-size: {}px }}\n",
                i % 5,
                12 + i
            ));
        }
        large_css.push_str("[data-kind=\"mixed\"] { background: blue }\n");
        for tag in tags {
            large_css.push_str(&format!("{tag} {{ display: block }}\n"));
        }
        let chain_css = "div p { color: green }\n\
                         span + li { text-indent: 2px }\n\
                         ul > li { font-weight: bold }\n\
                         .card:first-child { color: teal }\n\
                         a:link { color: navy }\n\
                         a:visited { color: purple }\n\
                         :not(.foo) { line-height: 1.4 }\n\
                         :is(.bar, .card) { letter-spacing: 1px }\n";
        (
            dom,
            nodes,
            vec![parse_stylesheet(&large_css), parse_stylesheet(chain_css)],
        )
    }

    #[test]
    fn rule_index_equals_brute_force_on_fixed_seed_corpus() {
        let (dom, nodes, sheets) = ab_corpus();
        let rules: Vec<&StyleRule> = sheets.iter().flat_map(|s| s.rules.iter()).collect();
        let index = RuleIndex::new(rules.iter().copied());
        let mut blooms = vec![AncestorBloom::default(); dom.len()];
        dom.walk(|id, _| {
            if let Some(parent) = dom.parent(id) {
                blooms[id.slot() as usize] =
                    AncestorBloom::child_of(blooms[parent.slot() as usize], &dom, parent);
            }
        });

        let mut out = Vec::new();
        let mut compared = 0usize;
        for &id in &nodes {
            let bloom = blooms[id.slot() as usize];
            index.candidates(&dom, id, &mut out);
            let mut bucketed_ctx = MatchContext::new();
            bucketed_ctx.ancestors = bloom;
            let mut brute_ctx = MatchContext::new();
            brute_ctx.ancestors = bloom;
            let bucketed: Vec<usize> = out
                .iter()
                .copied()
                .filter(|&i| matches_selector_ref(&rules[i].selector, &dom, id, &mut bucketed_ctx))
                .collect();
            let brute: Vec<usize> = (0..rules.len())
                .filter(|&i| matches_selector_ref(&rules[i].selector, &dom, id, &mut brute_ctx))
                .collect();
            assert_eq!(
                bucketed,
                brute,
                "slot {} ({:?}) diverged",
                id.slot(),
                dom.tag(id)
            );
            compared += 1;
        }
        assert!(compared >= 400, "corpus too small: {compared}");
        eprintln!(
            "A/B equality: {compared} elements x {} rules x 2 sheets — EXACT",
            rules.len()
        );
    }

    /// The hot path reads cached specificity; this pins the cached value to
    /// the old per-element `specificity_ref` result for both plain and chained
    /// rules, and proves every non-plain rule has a pre-decoded chain.
    #[test]
    fn rule_index_caches_chain_specificity_without_per_element_decode() {
        let sheet = parse_stylesheet(
            "#main.card { color: red }\n\
             div p span { color: blue }\n\
             .card:first-child { color: green }\n\
             :is(.a, #b) > li { color: teal }\n\
             .plain { color: black }\n",
        );
        let index = RuleIndex::new(sheet.rules.iter());
        let mut chained = 0;
        for (i, rule) in sheet.rules.iter().enumerate() {
            assert_eq!(
                index.specificity(i, &rule.selector),
                specificity_ref(&rule.selector),
                "rule {i}: cached specificity must equal the one-shot decode"
            );
            if !is_compound(&rule.selector) {
                chained += 1;
                assert!(index.chain(i).is_some(), "rule {i}: chain not cached");
            }
        }
        assert!(chained >= 3, "test must exercise cached chains: {chained}");
    }

    /// 33rd distinct attribute name (all frequency 1) lands in `universal`
    /// because only `MAX_ATTR_BUCKETS` names get dedicated buckets. The
    /// rule still arrives in source order and matches exactly like brute force.
    #[test]
    fn rule_index_33rd_attribute_uses_universal_bucket_in_source_order() {
        let mut css = String::new();
        for i in 0..=MAX_ATTR_BUCKETS {
            css.push_str(&format!("[data-k{i:02}] {{ color: #{i:06x} }}\n"));
        }
        // Two competitors that also match the element, placed after the 33rd
        // attribute rule so source order is observable in the candidate list.
        css.push_str(".target { border-width: 1px }\n");
        css.push_str("#t32 { margin: 2px }\n");
        let sheet = parse_stylesheet(&css);
        let index = RuleIndex::new(sheet.rules.iter());
        assert_eq!(sheet.rules.len(), MAX_ATTR_BUCKETS + 3);

        let mut dom = Dom::new();
        let root = dom.root();
        let node = elem(
            &mut dom,
            root,
            "div",
            &[("id", "t32"), ("class", "target"), ("data-k32", "x")],
        );
        let mut out = Vec::new();
        index.candidates(&dom, node, &mut out);

        let mut bucketed_ctx = MatchContext::new();
        let bucketed: Vec<usize> = out
            .iter()
            .copied()
            .filter(|&i| {
                matches_selector_ref(&sheet.rules[i].selector, &dom, node, &mut bucketed_ctx)
            })
            .collect();
        let mut brute_ctx = MatchContext::new();
        let brute: Vec<usize> = (0..sheet.rules.len())
            .filter(|&i| matches_selector_ref(&sheet.rules[i].selector, &dom, node, &mut brute_ctx))
            .collect();
        assert_eq!(out, brute, "candidate order must match brute force");
        assert_eq!(bucketed, brute, "matches must match brute force");
        assert_eq!(
            brute,
            vec![MAX_ATTR_BUCKETS, MAX_ATTR_BUCKETS + 1, MAX_ATTR_BUCKETS + 2],
            "33rd attribute rule plus both competitors, in source order"
        );
    }

    #[test]
    fn rule_index_ram_report_100k_rules() {
        let template = parse_stylesheet(".c0 { color: red }").rules[0].clone();
        // Realistic distribution: 100 000 rules over 500 shared classes.
        let realistic: Vec<StyleRule> = (0..100_000)
            .map(|i| {
                let mut rule = template.clone();
                rule.selector.classes = vec![format!("c{}", i % 500)];
                rule
            })
            .collect();
        let realistic_bytes = RuleIndex::new(realistic.iter()).heap_bytes();
        eprintln!(
            "RuleIndex RAM (100k rules / 500 classes): {} bytes ({:.2} MiB)",
            realistic_bytes,
            realistic_bytes as f64 / (1024.0 * 1024.0)
        );
        assert!(
            realistic_bytes < 1024 * 1024,
            "index must stay under 1 MiB: {realistic_bytes}"
        );
        // Worst case (every rule a unique key) is reported, not gated: the
        // HashMap's per-key slot — not the u32 rule lists — dominates there.
        let unique: Vec<StyleRule> = (0..100_000)
            .map(|i| {
                let mut rule = template.clone();
                rule.selector.classes = vec![format!("c{i}")];
                rule
            })
            .collect();
        let unique_index = RuleIndex::new(unique.iter());
        let unique_bytes = unique_index.heap_bytes();
        eprintln!(
            "RuleIndex RAM (100k unique classes): {} bytes ({:.2} MiB); rule-list payload alone = {} bytes",
            unique_bytes,
            unique_bytes as f64 / (1024.0 * 1024.0),
            100_000 * std::mem::size_of::<u32>()
        );
    }

    #[test]
    fn inline_declarations_and_important_flag() {
        let decls = parse_style_attribute("color: red !important; margin: 0; font-size: 12px");
        assert_eq!(decls.len(), 3);
        assert!(is_important(&decls[0]));
        assert_eq!(declaration_value(&decls[0]), "red");
        assert!(!is_important(&decls[1]));
        assert_eq!(declaration_value(&decls[1]), "0");
        assert_eq!(declaration_value(&decls[2]), "12px");
    }

    #[test]
    fn viewport_units_resolve_to_px_at_parse_time() {
        let vp = Viewport {
            width: 1200,
            height: 800,
            ..Viewport::default()
        };
        let sheet = parse_stylesheet_with(
            vp,
            "div { padding: 25vh 10vw 5vmin 5vmax; width: 10vi; height: 25vh !important }",
        );
        let decls = &sheet.rules[0].declarations;
        assert_eq!(declaration_value(&decls[0]), "200px 120px 40px 60px");
        assert_eq!(declaration_value(&decls[1]), "10vi");
        assert!(is_important(&decls[2]));
        assert_eq!(declaration_value(&decls[2]), "200px");
    }

    #[test]
    fn viewport_unit_resolution_leaves_other_values_untouched() {
        let vp = Viewport {
            width: 1200,
            height: 800,
            ..Viewport::default()
        };
        let sheet = parse_stylesheet_with(
            vp,
            "div { font-family: '25vh'; width: calc(25vh - 1px); margin: 2em; top: 50% }",
        );
        let values: Vec<&str> = sheet.rules[0]
            .declarations
            .iter()
            .map(|d| d.value.as_str())
            .collect();
        assert_eq!(values, ["\"25vh\"", "199px", "2em", "50%"]);
    }

    #[test]
    fn calc_resolves_lengths_and_nested_parens() {
        let vp = Viewport {
            width: 1200,
            height: 800,
            ..Viewport::default()
        };
        let css = "div { width: calc(2*(10px+5px)); height: calc(100px / 4); \
                   padding: calc( 10px + 5px ); margin: calc(10vw + 5vh); \
                   top: calc(2em + 1rem); left: calc(-1px + 3px); \
                   right: calc(1px + 2px) calc(3px * 2) }";
        let sheet = parse_stylesheet_with(vp, css);
        let values: Vec<&str> = sheet.rules[0]
            .declarations
            .iter()
            .map(declaration_value)
            .collect();
        assert_eq!(
            values,
            ["30px", "25px", "15px", "160px", "48px", "2px", "3px 6px"]
        );
    }

    #[test]
    fn calc_invalid_drops_only_the_declaration() {
        for value in [
            "calc(10px / 0)",
            "calc(10px +)",
            "calc(((((1px))))))",
            "calc(1px + 10deg)",
        ] {
            let css = format!("div {{ width: {value}; height: 5px }} p {{ color: blue }}");
            let sheet = parse_stylesheet(&css);
            assert_eq!(sheet.rules.len(), 2, "value {value}");
            assert_eq!(sheet.rules[0].declarations.len(), 1, "value {value}");
            assert_eq!(sheet.rules[0].declarations[0].property, "height");
            assert_eq!(sheet.rules[1].declarations[0].value, "blue");
        }
    }

    #[test]
    fn calc_in_importance_and_string_stays_intact() {
        let sheet = parse_stylesheet(
            "div { width: calc(10px + 5px) !important; font-family: 'calc(1px + 2px)' }",
        );
        let decls = &sheet.rules[0].declarations;
        assert_eq!(declaration_value(&decls[0]), "15px");
        assert!(is_important(&decls[0]));
        assert_eq!(decls[1].value, "\"calc(1px + 2px)\"");
    }

    // --- wave-1 at-rules -------------------------------------------------

    #[test]
    fn media_screen_applies_and_print_does_not() {
        let sheet = parse_stylesheet("@media screen { .a { color: red } }");
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector.classes, ["a"]);
        assert_eq!(sheet.rules[0].declarations[0].value, "red");

        let sheet = parse_stylesheet("@media print { .a { color: red } }");
        assert!(sheet.rules.is_empty());

        let print = Viewport {
            media_type: MediaType::Print,
            ..Viewport::default()
        };
        let sheet = parse_stylesheet_with(print, "@media print { .a { color: red } }");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn media_width_and_orientation_features_use_viewport() {
        let sheet = parse_stylesheet("@media (min-width: 1000px) { .a { color: red } }");
        assert_eq!(sheet.rules.len(), 1);
        let sheet = parse_stylesheet("@media (min-width: 2000px) { .a { color: red } }");
        assert!(sheet.rules.is_empty());

        let narrow = Viewport {
            width: 800,
            height: 600,
            ..Viewport::default()
        };
        let css = "@media (max-width: 900px) and (orientation: landscape) { .a { color: red } }";
        let sheet = parse_stylesheet_with(narrow, css);
        assert_eq!(sheet.rules.len(), 1);
        let sheet = parse_stylesheet_with(
            narrow,
            "@media (orientation: portrait) { .a { color: red } }",
        );
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn media_unsupported_features_are_non_matching() {
        for query in [
            "(hover: hover)",
            "screen and (pointer: coarse)",
            "(min-resolution: 2dppx)",
            "(-webkit-min-device-pixel-ratio: 2)",
            "not (hover: none)",
            "",
        ] {
            let css = format!("@media {query} {{ .a {{ color: red }} }} p {{ color: blue }}");
            let sheet = parse_stylesheet(&css);
            assert_eq!(sheet.rules.len(), 1, "query {query}");
            assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        }
    }

    #[test]
    fn media_broadened_features_match() {
        let dark = Viewport {
            dark: true,
            ..Viewport::default()
        };
        for (query, viewport) in [
            ("(hover: none)", Viewport::default()),
            ("(prefers-color-scheme: light)", Viewport::default()),
            ("(prefers-color-scheme: dark)", dark),
            ("(min-resolution: 1dppx)", Viewport::default()),
            ("(min-resolution: 96dpi)", Viewport::default()),
            ("(max-resolution: 1dppx)", Viewport::default()),
            ("(-webkit-min-device-pixel-ratio: 1)", Viewport::default()),
            ("not (hover: hover)", Viewport::default()),
            ("not print", Viewport::default()),
            ("(min-width: 1000px), print", Viewport::default()),
            ("print, (min-width: 1000px)", Viewport::default()),
            ("only screen and (hover: none)", Viewport::default()),
            ("(width>=1200px)", Viewport::default()),
            ("(width<=1200px)", Viewport::default()),
            ("(height=800px)", Viewport::default()),
            ("(width>=75rem)", Viewport::default()),
            ("((min-width: 100px))", Viewport::default()),
            (
                "(min-width: 100px) or (max-width: 10px)",
                Viewport::default(),
            ),
            (
                "((width>=802px) and (width<=1279.99px)) or (width>=1284px)",
                Viewport::default(),
            ),
            ("(width>=calc(48rem + 100px))", Viewport::default()),
            (
                "(prefers-reduced-motion: no-preference)",
                Viewport::default(),
            ),
            ("(forced-colors: none)", Viewport::default()),
            ("(pointer: fine)", Viewport::default()),
        ] {
            let css = format!("@media {query} {{ .a {{ color: red }} }}");
            let sheet = parse_stylesheet_with(viewport, &css);
            assert_eq!(sheet.rules.len(), 1, "query {query}");
        }
    }

    #[test]
    fn supports_whitelist_is_literal() {
        for condition in ["(display: grid)", "(display:flex)", "(display: table)"] {
            let sheet =
                parse_stylesheet(&format!("@supports {condition} {{ .a {{ color: red }} }}"));
            assert_eq!(sheet.rules.len(), 1, "condition {condition}");
        }
        for condition in [
            "(position: sticky)",
            "selector(:has(*))",
            "not (display: grid)",
            "(display: grid) or (display: flex)",
        ] {
            let css =
                format!("@supports {condition} {{ .a {{ color: red }} }} p {{ color: blue }}");
            let sheet = parse_stylesheet(&css);
            assert_eq!(sheet.rules.len(), 1, "condition {condition}");
            assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        }
    }

    #[test]
    fn import_is_rejected_explicitly() {
        let sheet = parse_stylesheet("@import url(theme.css); @import 'x.css'; p { color: blue }");
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
    }

    #[test]
    fn layer_blocks_are_parsed_with_recovery() {
        let sheet = parse_stylesheet(
            "@layer base { a { color: red } } @layer utilities { p { color: blue } } \
             q { color: green }",
        );
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["red", "blue", "green"]);
        let layers: Vec<Option<u32>> = sheet.rules.iter().map(cascade_layer).collect();
        assert_eq!(layers, [Some(0), Some(1), None]);
    }

    #[test]
    fn layer_statement_establishes_order_before_block_use() {
        let sheet = parse_stylesheet(
            "@layer reset, theme; @layer theme { .b { color: blue } } \
             @layer reset { .a { color: red } }",
        );
        let layers: Vec<Option<u32>> = sheet.rules.iter().map(cascade_layer).collect();
        assert_eq!(layers, [Some(1), Some(0)]);
    }

    #[test]
    fn nested_layer_composes_a_dotted_path() {
        let sheet = parse_stylesheet(
            "@layer outer { @layer inner { .a { color: red } } .b { color: blue } }",
        );
        let layers: Vec<(Option<u32>, Option<&str>)> = sheet
            .rules
            .iter()
            .map(|r| (cascade_layer(r), cascade_layer_name(r)))
            .collect();
        // `outer` is rank 0; the nested `outer.inner` is a distinct rank 1.
        assert_eq!(
            layers,
            [(Some(1), Some("outer.inner")), (Some(0), Some("outer"))]
        );
    }

    #[test]
    fn anonymous_layers_stay_distinct_across_sheets() {
        let first = parse_stylesheet("@layer { .a { color: red } }");
        let second = parse_stylesheet("@layer { .b { color: blue } }");
        assert_ne!(
            cascade_layer_name(&first.rules[0]),
            cascade_layer_name(&second.rules[0])
        );
    }

    #[test]
    fn comma_layer_list_applies_to_every_name() {
        let sheet = parse_stylesheet("@layer a, b { .x { color: red } }");
        let layers: Vec<(Option<u32>, Option<&str>)> = sheet
            .rules
            .iter()
            .map(|r| (cascade_layer(r), cascade_layer_name(r)))
            .collect();
        assert_eq!(layers, [(Some(0), Some("a")), (Some(1), Some("b"))]);
        assert!(sheet.rules.iter().all(|r| r.declarations[0].value == "red"));
    }

    #[test]
    fn var_calc_is_deferred_until_cascade_substitution() {
        let sheet = parse_stylesheet("div { --w: 50px; width: calc(var(--w) + 5px) }");
        let width = &sheet.rules[0].declarations[1];
        assert_eq!(width.value, "calc(var(--w) + 5px)");
        let resolved = BTreeMap::from([("--w".to_string(), "50px".to_string())]);
        let substituted = substitute_vars(&width.value, &resolved).unwrap();
        assert_eq!(resolve_calc_now(&substituted).unwrap(), "55px");
    }

    #[test]
    fn percent_calc_defers_to_a_layout_basis() {
        let sheet = parse_stylesheet("div { width: calc(100% - 32px) }");
        let width = &sheet.rules[0].declarations[0];
        assert_eq!(width.value, "calc(100% - 32px)");
        assert_eq!(resolve_calc_with_basis(&width.value, 1000.0), Some(968.0));
        assert_eq!(resolve_calc_with_basis(&width.value, 500.0), Some(468.0));
    }

    #[test]
    fn unitless_calc_emits_a_bare_number() {
        let sheet = parse_stylesheet("div { line-height: calc(2); opacity: calc(1 / 2) }");
        let values: Vec<&str> = sheet.rules[0]
            .declarations
            .iter()
            .map(|d| d.value.as_str())
            .collect();
        assert_eq!(values, ["2", "0.5"]);
    }

    #[test]
    fn layer_and_media_compose_in_both_nestings() {
        let sheet = parse_stylesheet(
            "@layer base { @media screen { .a { color: red } } \
             @media print { .b { color: blue } } } \
             @media screen { @layer util { .c { color: green } } }",
        );
        let rules: Vec<(&str, Option<u32>)> = sheet
            .rules
            .iter()
            .map(|r| (r.declarations[0].value.as_str(), cascade_layer(r)))
            .collect();
        assert_eq!(rules, [("red", Some(0)), ("green", Some(1))]);
    }

    #[test]
    fn custom_properties_are_case_sensitive_raw_declarations() {
        let decls = parse_stylesheet("p { --Brand: 12px; --brand: Hi; COLOR: red }").rules[0]
            .declarations
            .clone();
        assert_eq!(decls[0].property, "--Brand");
        assert_eq!(decls[0].value, "12px");
        assert_eq!(decls[1].property, "--brand");
        assert_eq!(decls[2].property, "color");
    }

    #[test]
    fn var_substitution_fallback_and_cycles() {
        let mut raw = BTreeMap::new();
        raw.insert("--a".into(), "var(--b)".into());
        raw.insert("--b".into(), "var(--a)".into());
        raw.insert("--width".into(), "4px".into());
        let resolved = resolve_custom_properties(&raw);
        assert_eq!(resolved.get("--width").map(String::as_str), Some("4px"));
        assert!(!resolved.contains_key("--a"));
        assert!(!resolved.contains_key("--b"));

        assert_eq!(
            substitute_vars("var(--width) solid", &resolved).as_deref(),
            Some("4px solid")
        );
        assert_eq!(
            substitute_vars("var(--missing, 2px)", &resolved).as_deref(),
            Some("2px")
        );
        assert_eq!(substitute_vars("var(--missing)", &resolved), None);
    }

    #[test]
    fn media_blocks_match_flat_rule_count_and_recover() {
        let nested = "@media screen { .a { color: red } } \
                      @media (max-width: 2000px) { .b { color: green } } \
                      @media print { .c { color: blue } } p { color: black }";
        let flat = ".a { color: red } .b { color: green } p { color: black }";
        let nested = parse_stylesheet(nested);
        let flat = parse_stylesheet(flat);
        assert_eq!(nested.rules.len(), flat.rules.len());
        fn values(sheet: &Stylesheet) -> Vec<&str> {
            sheet
                .rules
                .iter()
                .map(|rule| rule.declarations[0].value.as_str())
                .collect()
        }
        assert_eq!(values(&nested), values(&flat));

        let sheet = parse_stylesheet(
            "@media screen { :bogus { color: red } .ok { color: green } } p { color: blue }",
        );
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|rule| rule.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["green", "blue"]);
    }

    #[test]
    fn fuzz_style_inputs_never_panic() {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64;
        let alphabet: Vec<char> = "#.[]:=~|^$*!olerant-+> ,(){};\\\"'nthcid\n\t\u{1f}\u{0}"
            .chars()
            .collect();
        for _ in 0..400 {
            let len = (state % 80) as usize;
            let mut text = String::new();
            for _ in 0..len {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let index = (state as usize) % alphabet.len();
                text.push(alphabet[index]);
            }
            let sheet = parse_stylesheet(&text);
            let index = RuleIndex::new(sheet.rules.iter());
            let mut dom = Dom::new();
            let root = dom.root();
            let node = elem(&mut dom, root, "div", &[("class", "x")]);
            for rule in &sheet.rules {
                let _ = complex(&rule.selector);
                let _ = specificity_full(&rule.selector);
                let _ = matches_selector(&rule.selector, &dom, node);
            }
            let mut out = Vec::new();
            index.candidates(&dom, node, &mut out);
        }
    }
}

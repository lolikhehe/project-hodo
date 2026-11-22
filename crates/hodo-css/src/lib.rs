//! CSS parsing, selector matching and cascade support.
//!
//! Limits (local constants, pending centralization in `hodo-types`): at most
//! 64 compounds per selector, `:not()` nesting depth 2, at most 10 000 rules per
//! stylesheet, and a 64-bit ancestor Bloom filter for descendant rejection.
//! Matching walks ancestors/siblings iteratively; a 1000-level DOM with the
//! longest allowed descendant selector completes in well under the 1 s budget
//! asserted in tests. An invalid selector drops only its own rule.

use std::collections::HashMap;
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

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

/// Media-feature evaluation context for `@media` blocks **and** the descriptor
/// viewport units (`vh`/`vw`/`vmin`/`vmax`) resolve against while parsing.
/// `parse_stylesheet` uses [`Viewport::default`]; callers with a real window
/// should pass theirs through [`parse_stylesheet_with`]. Lengths resolve at
/// parse time into fixed `px` text (see [`resolve_viewport_lengths`]); a resize
/// mid-session does not re-resolve already-parsed declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub media_type: MediaType,
    /// Reserved for a future `scripting` feature; wave-1 queries ignore it.
    pub scripting_enabled: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            media_type: MediaType::Screen,
            scripting_enabled: true,
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
    /// `:link` / `:visited` — both match `a[href]` this wave (no history).
    Link,
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

/// Parse a stylesheet with the default viewport (1200x800, `screen`,
/// scripting enabled). Invalid selectors drop only their own rule; at-rules
/// and malformed declarations recover as before.
pub fn parse_stylesheet(css: &str) -> Stylesheet {
    parse_stylesheet_with(Viewport::default(), css)
}

/// Parse a stylesheet, evaluating `@media` / `@supports` against `viewport`.
/// Inner rules are kept only for matching conditions; unsupported conditions
/// are dropped with a single warning per stylesheet.
pub fn parse_stylesheet_with(viewport: Viewport, css: &str) -> Stylesheet {
    let mut parser = Parser::new(css);
    let mut rules = Vec::new();
    let mut warn = WarnOnce::default();
    parse_rules(&mut parser, viewport, &mut warn, &mut rules, 0);
    Stylesheet { rules }
}

fn parse_rules(
    parser: &mut Parser<'_>,
    viewport: Viewport,
    warn: &mut WarnOnce,
    rules: &mut Vec<StyleRule>,
    depth: u8,
) {
    loop {
        let state = parser.state();
        match parser.next() {
            Ok(Token::CDO | Token::CDC | Token::Semicolon) => continue,
            Ok(Token::AtKeyword(name)) => {
                let name = name.to_string();
                parse_at_rule(parser, &name, viewport, warn, rules, depth);
            }
            Ok(Token::CurlyBracketBlock) => {}
            Ok(_) => {
                parser.reset(&state);
                if let Some(parsed) = parse_qualified_rule(parser, viewport) {
                    rules.extend(parsed);
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

fn parse_at_rule(
    parser: &mut Parser<'_>,
    name: &str,
    viewport: Viewport,
    warn: &mut WarnOnce,
    rules: &mut Vec<StyleRule>,
    depth: u8,
) {
    let prelude = parser
        .parse_until_before(Delimiter::CurlyBracketBlock | Delimiter::Semicolon, |p| {
            Ok::<_, ParseError<()>>(serialize_value(p))
        })
        .unwrap_or_default();
    let is_block = matches!(parser.next(), Ok(Token::CurlyBracketBlock));
    let applies = match name.to_ascii_lowercase().as_str() {
        "media" => media_applies(&prelude, viewport, warn),
        "supports" => supports_applies(&prelude, warn),
        "import" => {
            warn.note("@import ignored (fetching stylesheets in the parser is out of scope)");
            false
        }
        "layer" => {
            // Layers are parsed and diagnosed, but their cascade layer is not
            // modelled yet: inner rules stay dropped this wave.
            warn.note("@layer ignored (cascade layers unsupported; inner rules dropped)");
            false
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
            parse_rules(p, viewport, warn, rules, depth + 1);
        }
        Ok::<_, ParseError<()>>(())
    });
}

/// Split a query into whitespace-separated items, keeping each `(...)` as one
/// item. Nested parentheses are rejected as unsupported (safe default).
fn query_items(query: &str) -> Option<Vec<String>> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_condition = false;
    for ch in query.trim().chars() {
        match ch {
            '(' if in_condition => return None,
            '(' => {
                if !current.is_empty() {
                    items.push(std::mem::take(&mut current));
                }
                current.push(ch);
                in_condition = true;
            }
            ')' if !in_condition => return None,
            ')' => {
                current.push(ch);
                items.push(std::mem::take(&mut current));
                in_condition = false;
            }
            ch if ch.is_whitespace() && !in_condition => {
                if !current.is_empty() {
                    items.push(std::mem::take(&mut current));
                }
            }
            ch => current.push(ch),
        }
    }
    if in_condition {
        return None;
    }
    if !current.is_empty() {
        items.push(current);
    }
    Some(items)
}

fn media_applies(query: &str, viewport: Viewport, warn: &mut WarnOnce) -> bool {
    let Some(mut items) = query_items(query) else {
        warn.note("@media condition unsupported (safe default: rules dropped)");
        return false;
    };
    if items.is_empty() {
        warn.note("@media condition unsupported (safe default: rules dropped)");
        return false;
    }
    // Legacy `only` qualifier is a no-op; `not`, comma lists and `or` are
    // deliberately unsupported and fall through to the non-matching path.
    if items[0].eq_ignore_ascii_case("only") {
        items.remove(0);
    }
    let mut index = 0;
    if let Some(first) = items.first() {
        if !first.starts_with('(') {
            match first.to_ascii_lowercase().as_str() {
                "screen" if viewport.media_type != MediaType::Screen => return false,
                "print" if viewport.media_type != MediaType::Print => return false,
                "screen" | "print" | "all" => {}
                _ => {
                    warn.note("@media condition unsupported (safe default: rules dropped)");
                    return false;
                }
            }
            index = 1;
        }
    }
    while index < items.len() {
        if items[index].eq_ignore_ascii_case("and") {
            index += 1;
            continue;
        }
        let Some(matched) = feature_matches(&items[index], viewport) else {
            warn.note("@media condition unsupported (safe default: rules dropped)");
            return false;
        };
        if !matched {
            return false;
        }
        index += 1;
    }
    true
}

fn feature_matches(item: &str, viewport: Viewport) -> Option<bool> {
    let inner = item.strip_prefix('(')?.strip_suffix(')')?;
    let (feature, value) = inner.split_once(':')?;
    let value = value.trim();
    match feature.trim().to_ascii_lowercase().as_str() {
        "min-width" => Some(px_value(value)? <= f64::from(viewport.width)),
        "max-width" => Some(px_value(value)? >= f64::from(viewport.width)),
        "min-height" => Some(px_value(value)? <= f64::from(viewport.height)),
        "max-height" => Some(px_value(value)? >= f64::from(viewport.height)),
        "orientation" => match value.to_ascii_lowercase().as_str() {
            "landscape" => Some(viewport.width >= viewport.height),
            "portrait" => Some(viewport.width < viewport.height),
            _ => None,
        },
        _ => None,
    }
}

fn px_value(value: &str) -> Option<f64> {
    value.strip_suffix("px")?.trim().parse().ok()
}

fn supports_applies(condition: &str, warn: &mut WarnOnce) -> bool {
    // Wave-1 whitelist: one plain `(property: value)` we actually implement.
    // `not`/`and`/`or`/`selector()` and `position: sticky` stay non-matching.
    let supported = condition
        .trim()
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

fn parse_qualified_rule(parser: &mut Parser<'_>, viewport: Viewport) -> Option<Vec<StyleRule>> {
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
        .parse_nested_block(|p| Ok::<_, ParseError<()>>(parse_declarations(p, viewport)))
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

/// Resolve viewport-relative lengths in a declaration value to `px` text using
/// the stylesheet's `Viewport`. Only whole `<number><unit>` words are touched;
/// quoted text, `calc()` and unknown units are left alone.
///
/// NOTE(parse-time): like `px`, a viewport unit becomes a fixed number while
/// parsing. A window resized mid-session does not re-resolve cached
/// declarations — revisit by re-parsing with the new `Viewport` when resize
/// support lands.
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

fn parse_declarations(parser: &mut Parser<'_>, viewport: Viewport) -> Vec<Declaration> {
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
                let property = property.to_lowercase();
                if parser.expect_colon().is_err() {
                    skip_to_semicolon(parser);
                    continue;
                }
                let value = parser
                    .parse_until_before(Delimiter::Semicolon, |p| {
                        Ok::<_, ParseError<()>>(serialize_value(p))
                    })
                    .unwrap_or_default();
                declarations.push(Declaration {
                    property,
                    value: resolve_viewport_lengths(&value, viewport),
                });
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
    parse_declarations(&mut parser, Viewport::default())
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
        matches_simple(sel, dom, id)
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

fn matches_simple(sel: &Selector, dom: &Dom, id: NodeId) -> bool {
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
            // No visited-history tracking this wave: both are plain `a[href]`.
            PseudoClass::Link | PseudoClass::Visited => {
                dom.tag(id).is_some_and(|tag| tag.eq_ignore_ascii_case("a"))
                    && attr_value(dom, id, "href").is_some()
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
}

impl MatchContext {
    pub fn new() -> Self {
        Self::default()
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

/// Rule index keyed by the rightmost compound: id, class, tag, universal.
#[derive(Debug, Default)]
pub struct RuleIndex {
    by_id: HashMap<String, Vec<usize>>,
    by_class: HashMap<String, Vec<usize>>,
    by_tag: HashMap<String, Vec<usize>>,
    universal: Vec<usize>,
}

impl RuleIndex {
    pub fn new<'a>(rules: impl IntoIterator<Item = &'a StyleRule>) -> Self {
        let mut index = Self::default();
        for (i, rule) in rules.into_iter().enumerate() {
            let sel = &rule.selector;
            if let Some(id) = &sel.id {
                index.by_id.entry(id.clone()).or_default().push(i);
            }
            for class in &sel.classes {
                if !class.starts_with(CHAIN_PREFIX) {
                    index.by_class.entry(class.clone()).or_default().push(i);
                }
            }
            match &sel.tag {
                Some(tag) => index
                    .by_tag
                    .entry(tag.to_ascii_lowercase())
                    .or_default()
                    .push(i),
                None if sel.id.is_none()
                    && sel
                        .classes
                        .iter()
                        .all(|class| class.starts_with(CHAIN_PREFIX)) =>
                {
                    index.universal.push(i)
                }
                None => {}
            }
        }
        index
    }

    /// Fill `out` with sorted, deduplicated candidate rule indices for one
    /// element. Callers reuse the buffer across elements.
    pub fn candidates(&self, dom: &Dom, id: NodeId, out: &mut Vec<usize>) {
        let candidates = out;
        candidates.clear();
        if let Some(value) = dom.attr(id, "id") {
            if let Some(indices) = self.by_id.get(value) {
                candidates.extend_from_slice(indices);
            }
        }
        if let Some(classes) = dom.attr(id, "class") {
            for class in classes.split_whitespace() {
                if let Some(indices) = self.by_class.get(class) {
                    candidates.extend_from_slice(indices);
                }
            }
        }
        if let Some(tag) = dom.tag(id) {
            if let Some(indices) = self.by_tag.get(&tag.to_ascii_lowercase()) {
                candidates.extend_from_slice(indices);
            }
        }
        candidates.extend_from_slice(&self.universal);
        candidates.sort_unstable();
        candidates.dedup();
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
    fn property_names_are_lowercased_and_unknown_kept() {
        let decls = declarations("a { COLOR: red; --Brand-Color: #abc; -webkit-transform: none }");
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[1].property, "--brand-color");
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
    fn link_and_visited_match_anchors_with_href() {
        let mut dom = Dom::new();
        let root = dom.root();
        let link = elem(&mut dom, root, "a", &[("href", "https://example.com")]);
        let bare = elem(&mut dom, root, "a", &[]);
        let other = elem(&mut dom, root, "div", &[("href", "#")]);
        for text in ["a:link", "a:visited", ":link", ":visited"] {
            let sel = selector(text);
            assert!(
                matches_selector(&sel, &dom, link),
                "{text} must match a[href]"
            );
            assert!(
                !matches_selector(&sel, &dom, bare),
                "{text} must skip bare a"
            );
            assert!(
                !matches_selector(&sel, &dom, other),
                "{text} must skip non-anchor"
            );
        }
        // HN-shaped selector lists survive parsing; only the link arms apply.
        let sheet = parse_stylesheet("a:link, a:visited { color: blue } p { color: red }");
        assert_eq!(sheet.rules.len(), 3);
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");
        assert_eq!(sheet.rules[1].declarations[0].value, "blue");
        assert!(matches_selector(&sheet.rules[0].selector, &dom, link));
        assert!(matches_selector(&sheet.rules[1].selector, &dom, link));
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
        assert_eq!(values, ["\"25vh\"", "calc(25vh - 1px)", "2em", "50%"]);
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
            "(prefers-color-scheme: dark)",
            "((min-width: 100px))",
            "screen and (pointer: coarse)",
            "(min-width: 1000px), print",
            "",
        ] {
            let css = format!("@media {query} {{ .a {{ color: red }} }} p {{ color: blue }}");
            let sheet = parse_stylesheet(&css);
            assert_eq!(sheet.rules.len(), 1, "query {query}");
            assert_eq!(sheet.rules[0].declarations[0].value, "blue");
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
    fn layer_blocks_are_dropped_with_recovery() {
        let sheet = parse_stylesheet(
            "@layer base { a { color: red } } @layer utilities { p { color: blue } } \
             q { color: green }",
        );
        let values: Vec<&str> = sheet
            .rules
            .iter()
            .map(|r| r.declarations[0].value.as_str())
            .collect();
        assert_eq!(values, ["green"]);
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

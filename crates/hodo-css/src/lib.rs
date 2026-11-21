//! Minimal CSS parsing and selector matching.

use cssparser::{Delimiter, ParseError, Parser, ToCss, Token};
use hodo_dom::{Dom, NodeId};

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

/// MVP compound selector: tag, #id, .class, and combinations.
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
    pub value: String,
}

/// Parse a stylesheet.
///
/// MVP limitations: each item in a selector list is a single *compound*
/// selector (tag, `#id`, `.class`, `*` and combinations such as `div.card#main`).
/// Combinators (descendant/child/sibling) and pseudo-classes/elements are not
/// interpreted yet; their tokens are consumed but ignored. At-rules are
/// skipped. Roadmap: combinators and pseudo selectors.
pub fn parse_stylesheet(css: &str) -> Stylesheet {
    let mut parser = Parser::new(css);
    let mut rules = Vec::new();

    loop {
        let state = parser.state();
        match parser.next() {
            // Ignore top-level trivia and stray semicolons.
            Ok(Token::CDO | Token::CDC | Token::Semicolon) => continue,
            // At-rules: consume the prelude; `next()` transparently skips a
            // balanced `{ ... }` block, and an `@import ...;` ends at the `;`.
            Ok(Token::AtKeyword(_)) => skip_rule(&mut parser),
            // A stray top-level block: `next()` skips its balanced contents.
            Ok(Token::CurlyBracketBlock) => {}
            // Start of a qualified rule: rewind and parse it properly.
            Ok(_) => {
                parser.reset(&state);
                if let Some(parsed) = parse_qualified_rule(&mut parser) {
                    rules.extend(parsed);
                }
            }
            Err(_) => break,
        }
    }

    Stylesheet { rules }
}

/// Skip an at-rule: consume its prelude, then either a balanced `{ ... }`
/// block (after which the rule ends) or a `;` for blockless at-rules such as
/// `@import` and `@charset`.
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

/// Parse one qualified rule into one `StyleRule` per selector in its list.
fn parse_qualified_rule(parser: &mut Parser<'_>) -> Option<Vec<StyleRule>> {
    // Prelude: everything before the `{` at this nesting level.
    let prelude = parser
        .parse_until_before(Delimiter::CurlyBracketBlock, |p| {
            let mut tokens = Vec::new();
            while let Ok(token) = p.next() {
                tokens.push(token.clone());
            }
            Ok::<_, ParseError<()>>(tokens)
        })
        .ok()?;

    if !matches!(parser.next(), Ok(Token::CurlyBracketBlock)) {
        return None;
    }

    let declarations = parser
        .parse_nested_block(|p| Ok::<_, ParseError<()>>(parse_declarations(p)))
        .unwrap_or_default();

    // A selector list is comma-separated; produce one rule per compound selector.
    let mut rules = Vec::new();
    let mut current = Vec::new();
    for token in prelude {
        if matches!(token, Token::Comma) {
            if let Some(selector) = parse_compound_selector(&current) {
                rules.push(StyleRule {
                    selector,
                    declarations: declarations.clone(),
                });
            }
            current.clear();
        } else {
            current.push(token);
        }
    }
    if let Some(selector) = parse_compound_selector(&current) {
        rules.push(StyleRule {
            selector,
            declarations,
        });
    }
    Some(rules)
}

/// Build a compound selector from a prelude token slice.
fn parse_compound_selector(tokens: &[Token<'_>]) -> Option<Selector> {
    if tokens.is_empty() {
        return None;
    }

    let mut selector = Selector {
        tag: None,
        id: None,
        classes: Vec::new(),
        universal: false,
    };

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Ident(name) if selector.tag.is_none() => {
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
            // Pseudo selectors are ignored (roadmap); skip the marker and name.
            Token::Colon => {
                i += 1;
                if matches!(tokens.get(i), Some(Token::Function(_) | Token::Ident(_))) {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    // A bare `*` (or nothing recognizable) is the universal selector.
    if selector.tag.is_none() && selector.id.is_none() && selector.classes.is_empty() {
        selector.universal = true;
    }
    Some(selector)
}

/// Parse the contents of a declaration block, recovering from each bad
/// declaration by skipping to the next `;`.
fn parse_declarations(parser: &mut Parser<'_>) -> Vec<Declaration> {
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
                declarations.push(Declaration { property, value });
            }
            Ok(_) => skip_to_semicolon(parser),
            Err(_) => break,
        }
    }

    declarations
}

fn skip_to_semicolon(parser: &mut Parser<'_>) {
    while !matches!(parser.next(), Ok(Token::Semicolon) | Err(_)) {}
}

/// Serialize a declaration value from tokens, collapsing runs of whitespace
/// to single spaces while preserving strings, urls, numbers, percentages and
/// functions/blocks.
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
                // Unmatched closing bracket or similar: drop it.
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

/// Cascade specificity as (ids, classes, tags).
pub fn specificity(sel: &Selector) -> (u32, u32, u32) {
    (
        u32::from(sel.id.is_some()),
        sel.classes.len() as u32,
        u32::from(sel.tag.is_some()),
    )
}

pub fn matches_element(sel: &Selector, dom: &Dom, id: NodeId) -> bool {
    // Selectors only match Element nodes (not Document or Text).
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
    for class in &sel.classes {
        let classes = dom.attr(id, "class").unwrap_or("");
        if !classes.split_whitespace().any(|c| c == class) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_dom::NodeKind;

    fn selector(s: &str) -> Selector {
        let sheet = parse_stylesheet(&format!("{s} {{ color: red }}"));
        sheet.rules[0].selector.clone()
    }

    fn declarations(css: &str) -> Vec<Declaration> {
        parse_stylesheet(css).rules[0].declarations.clone()
    }

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

        // `color red` has no colon and is dropped; `color: blue` survives.
        assert_eq!(sheet.rules[0].declarations.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        assert_eq!(sheet.rules[0].declarations[0].value, "blue");

        // `: bad` is garbage and dropped; `font-size: 12px` survives.
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

        // At-rules nested inside declarations are skipped too.
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

        let less = specificity(&selector(".a"));
        let more = specificity(&selector("div.a#main"));
        assert!(less < more);
    }

    #[test]
    fn matches_element_against_dom_tree() {
        let mut dom = Dom::new();
        let root = dom.root();
        let div = dom.create(NodeKind::Element {
            tag: "DIV".to_string(),
            attrs: vec![
                ("id".to_string(), "main".to_string()),
                ("class".to_string(), "card  highlighted".to_string()),
            ],
        });
        dom.append(root, div);
        let text = dom.create(NodeKind::Text("hello".to_string()));
        dom.append(root, text);

        // Tag match is case-insensitive; id/class must match exactly.
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
    }
}

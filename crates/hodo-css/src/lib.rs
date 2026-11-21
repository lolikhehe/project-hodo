//! Minimal CSS parsing and selector matching.

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

/// Parse a stylesheet (MVP: compound selectors only).
pub fn parse_stylesheet(css: &str) -> Stylesheet {
    todo!("parse_stylesheet({})", css)
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
    todo!("matches_element({:?}, {:?}, {:?})", sel, dom, id)
}

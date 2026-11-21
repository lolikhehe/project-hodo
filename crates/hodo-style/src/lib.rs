//! Style resolution: UA sheet + author sheets, cascaded by specificity and order.

use hodo_css::Stylesheet;
use hodo_dom::{Dom, NodeId};
use hodo_types::{Color, Edges};

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
    "html, body, div, p, h1, h2, h3, h4, h5, h6, ul, ol, li, section, article { display: block; }\n\
     span, a, em, strong, b, i, code { display: inline; }\n\
     html { color: #000000; background: transparent; font-size: 16px; }\n"
}

/// Apply the UA sheet first, then author sheets; cascade by specificity then order.
/// `color` and `font_size` inherit from the parent.
pub fn compute_styles(dom: &Dom, sheets: &[Stylesheet]) -> StyleMap {
    todo!("compute_styles({:?}, {} sheets)", dom, sheets.len())
}

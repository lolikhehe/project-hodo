//! Block-flow layout producing a flat display list.

#[allow(unused_imports)]
use hodo_dom::{Dom, NodeId};
#[allow(unused_imports)]
use hodo_style::{Display, StyleMap};
use hodo_types::{Color, Rect, Size};

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
    todo!("layout({:?}, {:?}, {:?})", dom, styles, viewport)
}

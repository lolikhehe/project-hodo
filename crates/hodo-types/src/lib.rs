//! Shared value types for the hodo browser engine.

/// Axis-aligned rectangle in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Width and height in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

/// 8-bit per channel RGBA color (non-premultiplied).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::rgba(r, g, b, 255)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    /// Parse `#rgb` or `#rrggbb`.
    pub fn from_css_hex(s: &str) -> Option<Color> {
        todo!("from_css_hex({s})")
    }

    /// Parse one of the 16 common CSS color names.
    pub fn from_css_name(s: &str) -> Option<Color> {
        todo!("from_css_name({s})")
    }
}

/// Per-edge spacing: top, right, bottom, left.
#[derive(Debug, Clone, Copy, Default)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Default> Edges<T> {
    pub fn zero() -> Self {
        Self::default()
    }
}

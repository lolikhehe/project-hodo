//! Shared value types for the hodo browser engine.

pub mod limits;

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
        let hex = s.strip_prefix('#')?;
        if !matches!(hex.len(), 3 | 6) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let nib = |b: u8| (b as char).to_digit(16).map(|d| d as u8);
        let bytes = hex.as_bytes();
        if hex.len() == 3 {
            Some(Color::rgb(
                nib(bytes[0])? * 17,
                nib(bytes[1])? * 17,
                nib(bytes[2])? * 17,
            ))
        } else {
            Some(Color::rgb(
                nib(bytes[0])? * 16 + nib(bytes[1])?,
                nib(bytes[2])? * 16 + nib(bytes[3])?,
                nib(bytes[4])? * 16 + nib(bytes[5])?,
            ))
        }
    }

    /// Parse one of the 16 common CSS color names.
    pub fn from_css_name(s: &str) -> Option<Color> {
        let name = s.to_ascii_lowercase();
        let color = match name.as_str() {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::rgb(255, 0, 0),
            "green" => Color::rgb(0, 128, 0),
            "blue" => Color::rgb(0, 0, 255),
            "yellow" => Color::rgb(255, 255, 0),
            "cyan" | "aqua" => Color::rgb(0, 255, 255),
            "magenta" | "fuchsia" => Color::rgb(255, 0, 255),
            "gray" | "grey" => Color::rgb(128, 128, 128),
            "silver" => Color::rgb(192, 192, 192),
            "maroon" => Color::rgb(128, 0, 0),
            "olive" => Color::rgb(128, 128, 0),
            "lime" => Color::rgb(0, 255, 0),
            "navy" => Color::rgb(0, 0, 128),
            "teal" => Color::rgb(0, 128, 128),
            "purple" => Color::rgb(128, 0, 128),
            _ => return None,
        };
        Some(color)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_short_forms() {
        assert_eq!(Color::from_css_hex("#000"), Some(Color::BLACK));
        assert_eq!(Color::from_css_hex("#fff"), Some(Color::WHITE));
        assert_eq!(
            Color::from_css_hex("#abc"),
            Some(Color::rgb(0xaa, 0xbb, 0xcc))
        );
        assert_eq!(
            Color::from_css_hex("#ABC"),
            Some(Color::rgb(0xaa, 0xbb, 0xcc))
        );
        assert_eq!(Color::from_css_hex("#F0A"), Some(Color::rgb(255, 0, 170)));
        assert_eq!(
            Color::from_css_hex("#123"),
            Some(Color::rgb(0x11, 0x22, 0x33))
        );
    }

    #[test]
    fn hex_long_forms() {
        assert_eq!(Color::from_css_hex("#000000"), Some(Color::BLACK));
        assert_eq!(Color::from_css_hex("#ffffff"), Some(Color::WHITE));
        assert_eq!(Color::from_css_hex("#FFFFFF"), Some(Color::WHITE));
        assert_eq!(Color::from_css_hex("#ff0000"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(
            Color::from_css_hex("#AbCdEf"),
            Some(Color::rgb(0xab, 0xcd, 0xef))
        );
        assert_eq!(Color::from_css_hex("#008000"), Some(Color::rgb(0, 128, 0)));
    }

    #[test]
    fn hex_invalid() {
        let invalid = [
            "#fff8",
            "#GGG",
            "#gggggg",
            "fff",
            "",
            "#",
            "#12",
            "#12345",
            "#1234567",
            "#FFFFFF00",
            "#fff ",
            " #fff",
            "#fff\n",
            "##fff",
            "#fffg",
            "#€fff",
        ];
        for s in invalid {
            assert_eq!(Color::from_css_hex(s), None, "input: {s:?}");
        }
    }

    #[test]
    fn named_colors() {
        let cases = [
            ("black", Color::BLACK),
            ("white", Color::WHITE),
            ("red", Color::rgb(255, 0, 0)),
            ("green", Color::rgb(0, 128, 0)),
            ("blue", Color::rgb(0, 0, 255)),
            ("yellow", Color::rgb(255, 255, 0)),
            ("cyan", Color::rgb(0, 255, 255)),
            ("magenta", Color::rgb(255, 0, 255)),
            ("gray", Color::rgb(128, 128, 128)),
            ("silver", Color::rgb(192, 192, 192)),
            ("maroon", Color::rgb(128, 0, 0)),
            ("olive", Color::rgb(128, 128, 0)),
            ("lime", Color::rgb(0, 255, 0)),
            ("navy", Color::rgb(0, 0, 128)),
            ("teal", Color::rgb(0, 128, 128)),
            ("purple", Color::rgb(128, 0, 128)),
        ];
        for (name, expected) in cases {
            assert_eq!(Color::from_css_name(name), Some(expected), "name: {name}");
        }
    }

    #[test]
    fn named_aliases_and_case() {
        assert_eq!(Color::from_css_name("aqua"), Color::from_css_name("cyan"));
        assert_eq!(
            Color::from_css_name("fuchsia"),
            Color::from_css_name("magenta")
        );
        assert_eq!(Color::from_css_name("grey"), Color::from_css_name("gray"));
        assert_eq!(Color::from_css_name("RED"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(Color::from_css_name("WhItE"), Some(Color::WHITE));
        assert_eq!(Color::from_css_name("LIME"), Some(Color::rgb(0, 255, 0)));
    }

    #[test]
    fn named_invalid() {
        let invalid = [
            "",
            "orange",
            "transparent",
            "lightgray",
            "re d",
            "#fff",
            "red\n",
        ];
        for s in invalid {
            assert_eq!(Color::from_css_name(s), None, "input: {s:?}");
        }
    }

    #[test]
    fn names_match_hex_values() {
        assert_eq!(
            Color::from_css_name("green"),
            Color::from_css_hex("#008000")
        );
        assert_eq!(Color::from_css_name("lime"), Color::from_css_hex("#00ff00"));
        assert_eq!(
            Color::from_css_name("olive"),
            Color::from_css_hex("#808000")
        );
        assert_eq!(
            Color::from_css_name("silver"),
            Color::from_css_hex("#c0c0c0")
        );
        assert_eq!(
            Color::from_css_name("purple"),
            Color::from_css_hex("#800080")
        );
        assert_eq!(
            Color::from_css_name("fuchsia"),
            Color::from_css_hex("#ff00ff")
        );
    }

    #[test]
    fn hex_roundtrip() {
        let colors = [
            Color::BLACK,
            Color::WHITE,
            Color::TRANSPARENT,
            Color::rgb(1, 2, 3),
            Color::rgba(0xab, 0xcd, 0xef, 128),
        ];
        for c in colors {
            let short = format!("#{:x}{:x}{:x}", c.r >> 4, c.g >> 4, c.b >> 4);
            assert_eq!(
                Color::from_css_hex(&short),
                Some(Color::rgb(
                    (c.r >> 4) * 17,
                    (c.g >> 4) * 17,
                    (c.b >> 4) * 17,
                )),
                "input: {short}",
            );
            let long = format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b);
            assert_eq!(
                Color::from_css_hex(&long),
                Some(Color::rgb(c.r, c.g, c.b)),
                "input: {long}",
            );
        }
    }

    #[test]
    fn constructors_and_constants() {
        assert_eq!(Color::rgb(1, 2, 3), Color::rgba(1, 2, 3, 255));
        assert_eq!(
            Color::rgba(1, 2, 3, 4),
            Color {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
        );
        assert_eq!(Color::WHITE, Color::rgba(255, 255, 255, 255));
        assert_eq!(Color::BLACK, Color::rgba(0, 0, 0, 255));
        assert_eq!(Color::TRANSPARENT, Color::rgba(0, 0, 0, 0));
        assert_eq!(Color::from_css_hex("#ffffff"), Some(Color::WHITE));
        assert_eq!(Color::from_css_name("white"), Some(Color::WHITE));
    }
}

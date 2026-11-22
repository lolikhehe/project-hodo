/// 8-bit RGBA color. All UI colors come from generated tokens; no raw hex
/// literals live in this crate's hand-written sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Standard source-over composite of `overlay` on top of `self`.
    /// Both layers may be translucent: the resulting alpha is
    /// `as + ab*(1-as)` and each channel is weighted by its source alpha.
    pub fn over(self, overlay: Color) -> Color {
        let sa = f32::from(overlay.a) / 255.0;
        let ba = f32::from(self.a) / 255.0;
        let out_a = sa + ba * (1.0 - sa);
        if out_a <= 0.0 {
            return Color::rgba(0, 0, 0, 0);
        }
        let mix = |base: u8, top: u8| {
            ((f32::from(top) * sa + f32::from(base) * ba * (1.0 - sa)) / out_a).round() as u8
        };
        Color::rgba(
            mix(self.r, overlay.r),
            mix(self.g, overlay.g),
            mix(self.b, overlay.b),
            (out_a * 255.0).round() as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn state_layer_over_is_flat_alpha() {
        let base = Color::rgb(103, 80, 164);
        let layer = Color::rgba(255, 255, 255, 20); // ~8%
        let out = base.over(layer);
        assert!(out.r > base.r && out.r < 255);
        assert_eq!(out.a, 255);
    }

    #[test]
    fn over_composites_a_translucent_base() {
        let base = Color::rgba(0, 0, 255, 128);
        let top = Color::rgba(255, 0, 0, 128);
        let out = base.over(top);
        assert_eq!(out.a, 192, "src-over must combine both alphas");
        assert_eq!(out.r, 170, "both layers must contribute to red");
        assert_eq!(out.b, 85, "the base must still show through");
    }

    #[test]
    fn over_two_transparent_layers_stays_transparent() {
        let out = Color::rgba(255, 0, 0, 0).over(Color::rgba(0, 255, 0, 0));
        assert_eq!(out, Color::rgba(0, 0, 0, 0));
    }
}

use crate::color::Color;
use crate::tokens::{self, Theme};

/// Every color role of the M3 baseline scheme (the fields of generated `Theme`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorRole {
    Primary,
    OnPrimary,
    PrimaryContainer,
    OnPrimaryContainer,
    Secondary,
    OnSecondary,
    SecondaryContainer,
    OnSecondaryContainer,
    Tertiary,
    OnTertiary,
    TertiaryContainer,
    OnTertiaryContainer,
    Error,
    OnError,
    ErrorContainer,
    OnErrorContainer,
    Surface,
    OnSurface,
    OnSurfaceVariant,
    SurfaceContainerLowest,
    SurfaceContainerLow,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    Outline,
    OutlineVariant,
    InverseSurface,
    InverseOnSurface,
    InversePrimary,
    Scrim,
}

/// The six hodo-specific type roles defined in M9 / §K.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeRole {
    Toolbar,
    Url,
    Tabs,
    Strip,
    Error,
    About,
}

/// State-layer opacity steps from §K.7.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateLayer {
    Hover,
    Focus,
    Pressed,
    Dragged,
}

pub fn state_layer_alpha(layer: StateLayer) -> f32 {
    match layer {
        StateLayer::Hover => tokens::STATE_HOVER,
        StateLayer::Focus => tokens::STATE_FOCUS,
        StateLayer::Pressed => tokens::STATE_PRESSED,
        StateLayer::Dragged => tokens::STATE_DRAGGED,
    }
}

/// Read-only access to a theme's colors and type styles. The restyle agent
/// programs against this trait, not against raw generated fields.
pub trait ThemeAccess {
    fn color(&self, role: ColorRole) -> Color;
    fn scrim(&self) -> Color;
    fn type_style(&self, role: TypeRole) -> tokens::TypeStyle;
}

impl ThemeAccess for Theme {
    fn color(&self, role: ColorRole) -> Color {
        use ColorRole::*;
        match role {
            Primary => self.primary,
            OnPrimary => self.on_primary,
            PrimaryContainer => self.primary_container,
            OnPrimaryContainer => self.on_primary_container,
            Secondary => self.secondary,
            OnSecondary => self.on_secondary,
            SecondaryContainer => self.secondary_container,
            OnSecondaryContainer => self.on_secondary_container,
            Tertiary => self.tertiary,
            OnTertiary => self.on_tertiary,
            TertiaryContainer => self.tertiary_container,
            OnTertiaryContainer => self.on_tertiary_container,
            Error => self.error,
            OnError => self.on_error,
            ErrorContainer => self.error_container,
            OnErrorContainer => self.on_error_container,
            Surface => self.surface,
            OnSurface => self.on_surface,
            OnSurfaceVariant => self.on_surface_variant,
            SurfaceContainerLowest => self.surface_container_lowest,
            SurfaceContainerLow => self.surface_container_low,
            SurfaceContainer => self.surface_container,
            SurfaceContainerHigh => self.surface_container_high,
            SurfaceContainerHighest => self.surface_container_highest,
            Outline => self.outline,
            OutlineVariant => self.outline_variant,
            InverseSurface => self.inverse_surface,
            InverseOnSurface => self.inverse_on_surface,
            InversePrimary => self.inverse_primary,
            Scrim => self.scrim,
        }
    }

    fn scrim(&self) -> Color {
        self.scrim
    }

    fn type_style(&self, role: TypeRole) -> tokens::TypeStyle {
        match role {
            TypeRole::Toolbar => tokens::TYPE_TOOLBAR,
            TypeRole::Url => tokens::TYPE_URL,
            TypeRole::Tabs => tokens::TYPE_TABS,
            TypeRole::Strip => tokens::TYPE_STRIP,
            TypeRole::Error => tokens::TYPE_ERROR,
            TypeRole::About => tokens::TYPE_ABOUT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_themes_expose_every_role() {
        for theme in [tokens::LIGHT, tokens::DARK] {
            for role in [
                ColorRole::Primary,
                ColorRole::Surface,
                ColorRole::InversePrimary,
                ColorRole::Scrim,
            ] {
                let _ = theme.color(role);
            }
            assert_eq!(
                theme.scrim().a,
                (tokens::LIGHT.scrim_alpha * 255.0).round() as u8
            );
            assert_eq!(theme.type_style(TypeRole::Toolbar).size, 14);
        }
    }

    #[test]
    fn state_layer_steps_match_tokens() {
        assert!((state_layer_alpha(StateLayer::Hover) - 0.08).abs() < f32::EPSILON);
        assert!((state_layer_alpha(StateLayer::Dragged) - 0.16).abs() < f32::EPSILON);
    }
}

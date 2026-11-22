//! hodo-ui: small retained-mode widget toolkit and design tokens (§K, decision A).
//!
//! All colors, shapes, type and motion values are generated into `tokens` from
//! `design/tokens.toml` by the standalone `xtask-design` binary. Hand-written
//! sources contain no raw hex literals.

pub mod color;
pub mod lru;
pub mod shape;
pub mod spring;
pub mod theme;
pub mod tokens;
pub mod widget;

pub use color::Color;
pub use lru::{PathCache, PathKey};
pub use shape::{pill, resolve_radius, rounded_rect_path};
pub use spring::{
    Spring, SpringSpec, DEFAULT_EFFECT, DEFAULT_SPATIAL, EFFECTS, EXPRESSIVE_SPATIAL,
    STANDARD_SPATIAL,
};
pub use theme::{state_layer_alpha, ColorRole, StateLayer, ThemeAccess, TypeRole};
pub use widget::{Widget, WidgetState, WIDGETS};

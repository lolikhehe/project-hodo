// GENERATED from design/tokens.toml v1 — do not edit
use crate::color::Color;

pub const TOKEN_SET_VERSION: u32 = 1;
pub const SEED: &str = "#6750A4";
pub const VERIFIED_AGAINST: &str = "m3.material.io baseline table";
pub const VERIFIED_ROLES: [&str; 2] = ["inverse_on_surface", "inverse_primary"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub error: Color,
    pub error_container: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,
    pub inverse_surface: Color,
    pub on_error: Color,
    pub on_error_container: Color,
    pub on_primary: Color,
    pub on_primary_container: Color,
    pub on_secondary: Color,
    pub on_secondary_container: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub on_tertiary: Color,
    pub on_tertiary_container: Color,
    pub outline: Color,
    pub outline_variant: Color,
    pub primary: Color,
    pub primary_container: Color,
    pub secondary: Color,
    pub secondary_container: Color,
    pub surface: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub surface_container_low: Color,
    pub surface_container_lowest: Color,
    pub tertiary: Color,
    pub tertiary_container: Color,
    pub scrim: Color,
    pub scrim_alpha: f32,
}

pub const LIGHT: Theme = Theme {
    error: Color::rgb(179, 38, 30),
    error_container: Color::rgb(249, 222, 220),
    inverse_on_surface: Color::rgb(245, 239, 247),
    inverse_primary: Color::rgb(208, 188, 255),
    inverse_surface: Color::rgb(50, 47, 53),
    on_error: Color::rgb(255, 255, 255),
    on_error_container: Color::rgb(140, 29, 24),
    on_primary: Color::rgb(255, 255, 255),
    on_primary_container: Color::rgb(79, 55, 139),
    on_secondary: Color::rgb(255, 255, 255),
    on_secondary_container: Color::rgb(74, 68, 88),
    on_surface: Color::rgb(29, 27, 32),
    on_surface_variant: Color::rgb(73, 69, 79),
    on_tertiary: Color::rgb(255, 255, 255),
    on_tertiary_container: Color::rgb(99, 59, 72),
    outline: Color::rgb(121, 116, 126),
    outline_variant: Color::rgb(202, 196, 208),
    primary: Color::rgb(103, 80, 164),
    primary_container: Color::rgb(234, 221, 255),
    secondary: Color::rgb(98, 91, 113),
    secondary_container: Color::rgb(232, 222, 248),
    surface: Color::rgb(254, 247, 255),
    surface_container: Color::rgb(243, 237, 247),
    surface_container_high: Color::rgb(236, 230, 240),
    surface_container_highest: Color::rgb(230, 224, 233),
    surface_container_low: Color::rgb(247, 242, 250),
    surface_container_lowest: Color::rgb(255, 255, 255),
    tertiary: Color::rgb(125, 82, 96),
    tertiary_container: Color::rgb(255, 216, 228),
    scrim: Color::rgba(0, 0, 0, 82),
    scrim_alpha: 0.32,
};

pub const DARK: Theme = Theme {
    error: Color::rgb(242, 184, 181),
    error_container: Color::rgb(140, 29, 24),
    inverse_on_surface: Color::rgb(50, 47, 53),
    inverse_primary: Color::rgb(103, 80, 164),
    inverse_surface: Color::rgb(230, 224, 233),
    on_error: Color::rgb(96, 20, 16),
    on_error_container: Color::rgb(249, 222, 220),
    on_primary: Color::rgb(56, 30, 114),
    on_primary_container: Color::rgb(234, 221, 255),
    on_secondary: Color::rgb(51, 45, 65),
    on_secondary_container: Color::rgb(232, 222, 248),
    on_surface: Color::rgb(230, 224, 233),
    on_surface_variant: Color::rgb(202, 196, 208),
    on_tertiary: Color::rgb(73, 37, 50),
    on_tertiary_container: Color::rgb(255, 216, 228),
    outline: Color::rgb(147, 143, 153),
    outline_variant: Color::rgb(73, 69, 79),
    primary: Color::rgb(208, 188, 255),
    primary_container: Color::rgb(79, 55, 139),
    secondary: Color::rgb(204, 194, 220),
    secondary_container: Color::rgb(74, 68, 88),
    surface: Color::rgb(20, 18, 24),
    surface_container: Color::rgb(33, 31, 38),
    surface_container_high: Color::rgb(43, 41, 48),
    surface_container_highest: Color::rgb(54, 52, 59),
    surface_container_low: Color::rgb(29, 27, 32),
    surface_container_lowest: Color::rgb(15, 13, 19),
    tertiary: Color::rgb(239, 184, 200),
    tertiary_container: Color::rgb(99, 59, 72),
    scrim: Color::rgba(0, 0, 0, 82),
    scrim_alpha: 0.32,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeStyle {
    pub style: &'static str,
    pub size: u32,
    pub weight: u32,
    pub line_height: u32,
}

pub const TYPE_ABOUT: TypeStyle = TypeStyle {
    style: "headline-small",
    size: 24,
    weight: 400,
    line_height: 32,
};
pub const TYPE_ERROR: TypeStyle = TypeStyle {
    style: "title-large",
    size: 22,
    weight: 700,
    line_height: 28,
};
pub const TYPE_STRIP: TypeStyle = TypeStyle {
    style: "label-small",
    size: 11,
    weight: 500,
    line_height: 16,
};
pub const TYPE_TABS: TypeStyle = TypeStyle {
    style: "label-medium",
    size: 12,
    weight: 500,
    line_height: 16,
};
pub const TYPE_TOOLBAR: TypeStyle = TypeStyle {
    style: "label-large",
    size: 14,
    weight: 500,
    line_height: 20,
};
pub const TYPE_URL: TypeStyle = TypeStyle {
    style: "body-large",
    size: 16,
    weight: 400,
    line_height: 24,
};

pub const STATE_HOVER: f32 = 0.08;
pub const STATE_FOCUS: f32 = 0.1;
pub const STATE_PRESSED: f32 = 0.1;
pub const STATE_DRAGGED: f32 = 0.16;

pub const SHAPE_CORNER_STEPS: [f32; 9] = [0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 28.0, 36.0];
pub const SHAPE_FULL: f32 = -1.0;

pub const MOTION_HOVER_MS: u32 = 100;
pub const MOTION_HOVER_EASING: [f32; 4] = [0.31, 0.94, 0.34, 1.0];
pub const MOTION_POPUP_OPEN_MS: u32 = 150;
pub const MOTION_POPUP_OPEN_EASING: [f32; 4] = [0.31, 0.94, 0.34, 1.0];
pub const MOTION_PRESS_MS: u32 = 50;
pub const MOTION_PRESS_EASING: [f32; 4] = [0.31, 0.94, 0.34, 1.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowClass {
    pub name: &'static str,
    pub min_dp: i32,
    pub max_dp: i32,
    pub url_max: u32,
    pub popup_width: u32,
    pub popup_height: u32,
}

pub const WINDOW_CLASSES: [WindowClass; 5] = [
    WindowClass {
        name: "compact",
        min_dp: 0,
        max_dp: 599,
        url_max: 480,
        popup_width: 320,
        popup_height: 400,
    },
    WindowClass {
        name: "medium",
        min_dp: 600,
        max_dp: 839,
        url_max: 640,
        popup_width: 360,
        popup_height: 480,
    },
    WindowClass {
        name: "expanded",
        min_dp: 840,
        max_dp: 1199,
        url_max: 720,
        popup_width: 420,
        popup_height: 560,
    },
    WindowClass {
        name: "large",
        min_dp: 1200,
        max_dp: 1599,
        url_max: 720,
        popup_width: 420,
        popup_height: 560,
    },
    WindowClass {
        name: "xl",
        min_dp: 1600,
        max_dp: -1,
        url_max: 720,
        popup_width: 420,
        popup_height: 560,
    },
];

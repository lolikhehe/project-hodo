//! Standalone codegen: design/tokens.toml -> hodo-ui tokens.rs + tokens.css.
//! Run: cargo run --manifest-path xtask-design/Cargo.toml

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const STAMP: &str = "// GENERATED from design/tokens.toml v1 — do not edit";

#[derive(Debug, Deserialize)]
struct Tokens {
    version: u32,
    seed: String,
    #[serde(default)]
    verification: Verification,
    color: Colors,
    #[serde(rename = "type")]
    typography: BTreeMap<String, TypeStyle>,
    shape: Shape,
    state: State,
    motion: Motion,
    #[serde(default)]
    switch: Switch,
    window: Window,
}

/// M3 Expressive switch geometry. Values are dp; `touch_target` is the hit box,
/// the track and thumb are the painted shape centered inside it.
#[derive(Debug, Deserialize)]
struct Switch {
    touch_target: f32,
    track_width: f32,
    track_height: f32,
    thumb: f32,
}

impl Default for Switch {
    fn default() -> Self {
        Self {
            touch_target: 44.0,
            track_width: 52.0,
            track_height: 32.0,
            thumb: 28.0,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct Verification {
    verified_against: Option<String>,
    #[serde(default)]
    flagged_roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Colors {
    light: BTreeMap<String, String>,
    dark: BTreeMap<String, String>,
    scrim: Scrim,
}

#[derive(Debug, Deserialize)]
struct Scrim {
    color: String,
    alpha: f32,
}

#[derive(Debug, Deserialize)]
struct TypeStyle {
    style: String,
    size: u32,
    weight: u32,
    line_height: u32,
}

#[derive(Debug, Deserialize)]
struct Shape {
    corner_steps: Vec<u32>,
    full: f32,
}

#[derive(Debug, Deserialize)]
struct State {
    hover: f32,
    focus: f32,
    pressed: f32,
    dragged: f32,
}

#[derive(Debug, Deserialize)]
struct Motion {
    eased: BTreeMap<String, Eased>,
}

#[derive(Debug, Deserialize)]
struct Eased {
    duration_ms: u32,
    easing: [f32; 4],
}

#[derive(Debug, Deserialize)]
struct Window {
    class: Vec<WindowClass>,
}

#[derive(Debug, Deserialize)]
struct WindowClass {
    name: String,
    min_dp: i32,
    max_dp: i32,
    url_max: u32,
    popup_width: u32,
    popup_height: u32,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask-design lives at the workspace root")
        .to_path_buf()
}

fn load_tokens(root: &Path) -> Tokens {
    let path = root.join("design/tokens.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    toml::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn parse_hex(s: &str) -> (u8, u8, u8) {
    let h = s.trim_start_matches('#');
    assert!(h.len() == 6, "expected #RRGGBB, got {s}");
    (
        u8::from_str_radix(&h[0..2], 16).expect("hex red"),
        u8::from_str_radix(&h[2..4], 16).expect("hex green"),
        u8::from_str_radix(&h[4..6], 16).expect("hex blue"),
    )
}

fn kebab(s: &str) -> String {
    s.replace('_', "-")
}

fn float_lit(v: f32) -> String {
    let s = format!("{}", v);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn sorted_roles(t: &Tokens) -> Vec<&String> {
    let mut roles: Vec<&String> = t.color.light.keys().collect();
    roles.sort();
    roles
}

fn generate_rust(t: &Tokens) -> String {
    let mut out = String::new();
    out.push_str(STAMP);
    out.push('\n');
    out.push_str("use crate::color::Color;\n\n");
    out.push_str(&format!(
        "pub const TOKEN_SET_VERSION: u32 = {};\n",
        t.version
    ));
    out.push_str(&format!("pub const SEED: &str = \"{}\";\n", t.seed));
    if let Some(v) = &t.verification.verified_against {
        out.push_str(&format!("pub const VERIFIED_AGAINST: &str = \"{v}\";\n"));
    }
    let flagged: Vec<String> = t
        .verification
        .flagged_roles
        .iter()
        .map(|r| format!("\"{r}\""))
        .collect();
    out.push_str(&format!(
        "pub const VERIFIED_ROLES: [&str; {}] = [{}];\n",
        flagged.len(),
        flagged.join(", ")
    ));
    out.push('\n');

    // Theme struct: all color roles, sorted, plus scrim.
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Theme {\n");
    for role in sorted_roles(t) {
        out.push_str(&format!("    pub {role}: Color,\n"));
    }
    out.push_str("    pub scrim: Color,\n    pub scrim_alpha: f32,\n}\n\n");

    for (name, table) in [("LIGHT", &t.color.light), ("DARK", &t.color.dark)] {
        out.push_str(&format!("pub const {name}: Theme = Theme {{\n"));
        for role in sorted_roles(t) {
            let (r, g, b) = parse_hex(&table[role]);
            out.push_str(&format!("    {role}: Color::rgb({r}, {g}, {b}),\n"));
        }
        let (r, g, b) = parse_hex(&t.color.scrim.color);
        let alpha_255 = (t.color.scrim.alpha * 255.0).round() as u8;
        out.push_str(&format!(
            "    scrim: Color::rgba({r}, {g}, {b}, {alpha_255}),\n"
        ));
        out.push_str(&format!(
            "    scrim_alpha: {},\n",
            float_lit(t.color.scrim.alpha)
        ));
        out.push_str("};\n\n");
    }

    // Type roles.
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\npub struct TypeStyle {\n");
    out.push_str("    pub style: &'static str,\n    pub size: u32,\n    pub weight: u32,\n    pub line_height: u32,\n}\n\n");
    for (role, ts) in &t.typography {
        out.push_str(&format!(
            "pub const TYPE_{}: TypeStyle = TypeStyle {{\n    style: \"{}\",\n    size: {},\n    weight: {},\n    line_height: {},\n}};\n",
            role.to_ascii_uppercase(), ts.style, ts.size, ts.weight, ts.line_height
        ));
    }
    out.push('\n');

    // State layers.
    for (name, value) in [
        ("HOVER", t.state.hover),
        ("FOCUS", t.state.focus),
        ("PRESSED", t.state.pressed),
        ("DRAGGED", t.state.dragged),
    ] {
        out.push_str(&format!(
            "pub const STATE_{name}: f32 = {};\n",
            float_lit(value)
        ));
    }
    out.push('\n');

    // Shape scale; -1 is the pill/full sentinel resolved in shape::resolve_radius.
    let steps: Vec<String> = t
        .shape
        .corner_steps
        .iter()
        .map(|s| format!("{s}.0"))
        .collect();
    out.push_str(&format!(
        "pub const SHAPE_CORNER_STEPS: [f32; {}] = [{}];\n",
        steps.len(),
        steps.join(", ")
    ));
    out.push_str(&format!(
        "pub const SHAPE_FULL: f32 = {};\n",
        float_lit(t.shape.full)
    ));
    out.push('\n');

    // M3 Expressive switch geometry (touch target / track / thumb).
    out.push_str(&format!(
        "pub const SWITCH_TOUCH_TARGET: f32 = {};\n",
        float_lit(t.switch.touch_target)
    ));
    out.push_str(&format!(
        "pub const SWITCH_TRACK_WIDTH: f32 = {};\n",
        float_lit(t.switch.track_width)
    ));
    out.push_str(&format!(
        "pub const SWITCH_TRACK_HEIGHT: f32 = {};\n",
        float_lit(t.switch.track_height)
    ));
    out.push_str(&format!(
        "pub const SWITCH_THUMB: f32 = {};\n",
        float_lit(t.switch.thumb)
    ));
    out.push('\n');

    // Eased motion.
    for (name, eased) in &t.motion.eased {
        let n = name.to_ascii_uppercase();
        out.push_str(&format!(
            "pub const MOTION_{n}_MS: u32 = {};\n",
            eased.duration_ms
        ));
        let curve: Vec<String> = eased.easing.iter().map(|v| float_lit(*v)).collect();
        out.push_str(&format!(
            "pub const MOTION_{n}_EASING: [f32; 4] = [{}];\n",
            curve.join(", ")
        ));
    }
    out.push('\n');

    // Adaptive window classes.
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\npub struct WindowClass {\n");
    out.push_str("    pub name: &'static str,\n    pub min_dp: i32,\n    pub max_dp: i32,\n");
    out.push_str(
        "    pub url_max: u32,\n    pub popup_width: u32,\n    pub popup_height: u32,\n}\n\n",
    );
    out.push_str(&format!(
        "pub const WINDOW_CLASSES: [WindowClass; {}] = [\n",
        t.window.class.len()
    ));
    for c in &t.window.class {
        out.push_str(&format!(
            "    WindowClass {{\n        name: \"{}\",\n        min_dp: {},\n        max_dp: {},\n        url_max: {},\n        popup_width: {},\n        popup_height: {},\n    }},\n",
            c.name, c.min_dp, c.max_dp, c.url_max, c.popup_width, c.popup_height
        ));
    }
    out.push_str("];\n");
    out
}

fn generate_css(t: &Tokens) -> String {
    let mut out = String::new();
    out.push_str(STAMP);
    out.push('\n');

    for (selector, table) in [
        (":root", &t.color.light),
        ("[data-theme=\"dark\"]", &t.color.dark),
    ] {
        let mut vars: BTreeMap<String, String> = BTreeMap::new();
        for (role, hex) in table {
            vars.insert(format!("--md-sys-color-{}", kebab(role)), hex.clone());
        }
        vars.insert("--md-sys-color-scrim".into(), t.color.scrim.color.clone());
        vars.insert(
            "--md-sys-color-scrim-alpha".into(),
            float_lit(t.color.scrim.alpha),
        );
        out.push_str(&format!("{selector} {{\n"));
        for (k, v) in vars {
            out.push_str(&format!("  {k}: {v};\n"));
        }
        out.push_str("}\n\n");
    }

    let mut shared: BTreeMap<String, String> = BTreeMap::new();
    for (role, ts) in &t.typography {
        let base = format!("--md-sys-type-{}-", kebab(role));
        shared.insert(format!("{base}size"), format!("{}px", ts.size));
        shared.insert(format!("{base}weight"), ts.weight.to_string());
        shared.insert(
            format!("{base}line-height"),
            format!("{}px", ts.line_height),
        );
    }
    shared.insert("--md-sys-state-hover".into(), float_lit(t.state.hover));
    shared.insert("--md-sys-state-focus".into(), float_lit(t.state.focus));
    shared.insert("--md-sys-state-pressed".into(), float_lit(t.state.pressed));
    shared.insert("--md-sys-state-dragged".into(), float_lit(t.state.dragged));
    for (i, step) in t.shape.corner_steps.iter().enumerate() {
        shared.insert(format!("--md-sys-shape-corner-{i}"), format!("{step}px"));
    }
    shared.insert("--md-sys-shape-corner-full".into(), "9999px".into());
    shared.insert(
        "--md-sys-switch-touch-target".into(),
        format!("{}px", float_lit(t.switch.touch_target)),
    );
    shared.insert(
        "--md-sys-switch-track-width".into(),
        format!("{}px", float_lit(t.switch.track_width)),
    );
    shared.insert(
        "--md-sys-switch-track-height".into(),
        format!("{}px", float_lit(t.switch.track_height)),
    );
    shared.insert(
        "--md-sys-switch-thumb".into(),
        format!("{}px", float_lit(t.switch.thumb)),
    );
    for (name, eased) in &t.motion.eased {
        let base = format!("--md-sys-motion-{}-", kebab(name));
        shared.insert(
            format!("{base}duration"),
            format!("{}ms", eased.duration_ms),
        );
        let curve: Vec<String> = eased.easing.iter().map(|v| float_lit(*v)).collect();
        shared.insert(
            format!("{base}easing"),
            format!("cubic-bezier({})", curve.join(", ")),
        );
    }
    for c in &t.window.class {
        let base = format!("--md-sys-window-{}-", c.name);
        shared.insert(format!("{base}url-max"), format!("{}px", c.url_max));
        shared.insert(format!("{base}popup-width"), format!("{}px", c.popup_width));
        shared.insert(
            format!("{base}popup-height"),
            format!("{}px", c.popup_height),
        );
    }
    out.push_str(":root {\n");
    for (k, v) in shared {
        out.push_str(&format!("  {k}: {v};\n"));
    }
    out.push_str("}\n");
    out
}

fn generate(t: &Tokens) -> (String, String) {
    (generate_rust(t), generate_css(t))
}

fn write_generated(root: &Path, t: &Tokens) -> (PathBuf, PathBuf) {
    let (rs, css) = generate(t);
    let rs_path = root.join("crates/hodo-ui/src/tokens.rs");
    let css_path = root.join("crates/hodo-ui/src/tokens.css");
    fs::write(&rs_path, rs).expect("write tokens.rs");
    fs::write(&css_path, css).expect("write tokens.css");
    (rs_path, css_path)
}

fn main() {
    let root = workspace_root();
    let tokens = load_tokens(&root);
    let (rs, css) = write_generated(&root, &tokens);
    println!("generated {}", rs.display());
    println!("generated {}", css.display());
    println!(
        "version={} roles={} classes={}",
        tokens.version,
        tokens.color.light.len(),
        tokens.window.class.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_matches_checked_in_files() {
        let root = workspace_root();
        let t = load_tokens(&root);
        let (rs, css) = generate(&t);
        assert_eq!(
            fs::read_to_string(root.join("crates/hodo-ui/src/tokens.rs")).unwrap(),
            rs
        );
        assert_eq!(
            fs::read_to_string(root.join("crates/hodo-ui/src/tokens.css")).unwrap(),
            css
        );
    }

    #[test]
    fn generation_is_deterministic() {
        let t = load_tokens(&workspace_root());
        assert_eq!(generate(&t), generate(&t));
    }

    #[test]
    fn light_and_dark_share_the_same_role_set() {
        let t = load_tokens(&workspace_root());
        let light: Vec<_> = t.color.light.keys().collect();
        let dark: Vec<_> = t.color.dark.keys().collect();
        assert_eq!(light, dark);
        assert_eq!(light.len(), 29);
    }

    #[test]
    fn helper_conversions_are_stable() {
        assert_eq!(kebab("surface_container_low"), "surface-container-low");
        assert_eq!(float_lit(-1.0), "-1.0");
        assert_eq!(float_lit(0.08), "0.08");
        assert_eq!(parse_hex("#6750A4"), (0x67, 0x50, 0xA4));
    }

    #[test]
    fn stamp_is_present_in_both_outputs() {
        let (rs, css) = generate(&load_tokens(&workspace_root()));
        assert!(rs.starts_with(STAMP));
        assert!(css.starts_with(STAMP));
    }
}

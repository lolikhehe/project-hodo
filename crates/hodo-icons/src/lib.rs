//! Frozen icon API for hodo: rasterize vendored Material Symbols Outlined glyphs.
//!
//! No SVG is parsed at runtime. [`paths::icon_path`] returns a normalized,
//! absolute-only path-d string on a 24x24 canvas; [`rasterize`] parses that
//! string with a tiny-skia `PathBuilder` and fills it into a `Pixmap`. The
//! finished pixmap is kept in a bounded process-wide LRU keyed by
//! (name, size, color), so repeated toolbar draws only copy pixels.
//!
//! The restyle agent should depend on this crate's two public items only:
//! `rasterize` and `paths::ICON_NAMES` / `paths::icon_path`.

pub mod paths;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use hodo_types::Color;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

/// LRU entry budget. At 24px a pixmap is 24 * 24 * 4 = 2,304 bytes, so 128
/// entries are about 294 KiB (roughly 300 KB) of pixel data. Larger `size`
/// arguments cost more, but the entry count stays bounded.
pub const CACHE_CAPACITY: usize = 128;

/// Canvas coordinate space of every generated path.
const ICON_CANVAS: f32 = 24.0;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    name: String,
    size: u32,
    /// `r | g << 8 | b << 16 | a << 24`. `Color` itself has no `Hash`.
    rgba: u32,
}

impl Key {
    fn new(name: &str, size: u32, color: Color) -> Self {
        Self {
            name: name.to_owned(),
            size,
            rgba: u32::from_le_bytes([color.r, color.g, color.b, color.a]),
        }
    }
}

struct Cache {
    map: HashMap<Key, (Pixmap, u64)>,
    clock: u64,
}

impl Cache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            clock: 0,
        }
    }
}

static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn global_cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| Mutex::new(Cache::new()))
}

/// Build a tiny-skia path from a generated path-d string.
///
/// The generated grammar is deliberately tiny: absolute `M`, `L`, `Q`, `C`
/// commands with the first number glued to the command letter, plus `Z`.
/// Anything else is rejected with `None` (the caller falls back to text).
fn build_path(d: &str) -> Option<tiny_skia::Path> {
    fn next<'a>(it: &mut impl Iterator<Item = &'a str>) -> Option<f32> {
        it.next()?.parse().ok()
    }

    let mut builder = PathBuilder::new();
    let mut it = d.split_whitespace();
    while let Some(tok) = it.next() {
        let (cmd, first) = tok.split_at(1);
        match cmd {
            "M" => {
                let x: f32 = first.parse().ok()?;
                let y = next(&mut it)?;
                builder.move_to(x, y);
            }
            "L" => {
                let x: f32 = first.parse().ok()?;
                let y = next(&mut it)?;
                builder.line_to(x, y);
            }
            "Q" => {
                let cx: f32 = first.parse().ok()?;
                let cy = next(&mut it)?;
                let x = next(&mut it)?;
                let y = next(&mut it)?;
                builder.quad_to(cx, cy, x, y);
            }
            "C" => {
                let c1x: f32 = first.parse().ok()?;
                let c1y = next(&mut it)?;
                let c2x = next(&mut it)?;
                let c2y = next(&mut it)?;
                let x = next(&mut it)?;
                let y = next(&mut it)?;
                builder.cubic_to(c1x, c1y, c2x, c2y, x, y);
            }
            "Z" if first.is_empty() => builder.close(),
            _ => return None,
        }
    }
    builder.finish()
}

fn render(d: &str, size: u32, color: Color) -> Option<Pixmap> {
    let path = build_path(d)?;
    let mut pixmap = Pixmap::new(size, size)?;

    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    paint.anti_alias = true;

    let scale = size as f32 / ICON_CANVAS;
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::from_scale(scale, scale),
        None,
    );
    Some(pixmap)
}

/// Same as [`rasterize`], but against an explicit cache so eviction can be
/// tested without racing the process-wide cache.
fn rasterize_in(cache: &Mutex<Cache>, name: &str, size: u32, color: Color) -> Option<Pixmap> {
    let d = paths::icon_path(name)?;
    if size == 0 {
        return None; // tiny-skia rejects zero-sized pixmaps too
    }

    let key = Key::new(name, size, color);
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache.clock += 1;
    let now = cache.clock;

    if let Some((pixmap, stamp)) = cache.map.get_mut(&key) {
        *stamp = now;
        return Some(pixmap.clone());
    }

    let pixmap = render(d, size, color)?;

    if cache.map.len() >= CACHE_CAPACITY {
        let victim = cache
            .map
            .iter()
            .min_by_key(|(_, (_, stamp))| *stamp)
            .map(|(key, _)| key.clone());
        if let Some(victim) = victim {
            cache.map.remove(&victim);
        }
    }
    cache.map.insert(key, (pixmap.clone(), now));
    Some(pixmap)
}

/// Rasterize an icon into an RGBA `Pixmap`.
///
/// Returns `None` for an unknown name, a zero size, or a malformed generated
/// path — callers should fall back to a text label in that case. The path-d
/// is parsed only on a cache miss; cache hits clone the cached pixmap.
pub fn rasterize(name: &str, size: u32, color: Color) -> Option<Pixmap> {
    rasterize_in(global_cache(), name, size, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_vendored_name_round_trips_at_16px() {
        assert_eq!(paths::ICON_NAMES.len(), 41);
        for &name in paths::ICON_NAMES {
            let d = paths::icon_path(name).expect("ICON_NAMES entry must resolve");
            assert!(!d.is_empty(), "{name}: empty path");
            let pm = rasterize(name, 16, Color::WHITE).expect("{name}: rasterize failed");
            assert_eq!((pm.width(), pm.height()), (16, 16), "{name}: wrong size");
            assert!(
                pm.data().as_chunks::<4>().0.iter().any(|px| px[3] != 0),
                "{name}: rendered fully transparent"
            );
        }
    }

    #[test]
    fn lru_cap_evicts_least_recently_used() {
        let cache = Mutex::new(Cache::new());

        for size in 16..(16 + CACHE_CAPACITY as u32) {
            assert!(rasterize_in(&cache, "home", size, Color::WHITE).is_some());
        }
        assert_eq!(cache.lock().unwrap().map.len(), CACHE_CAPACITY);

        // Touch the oldest entry; size 17 becomes the least recently used.
        assert!(rasterize_in(&cache, "home", 16, Color::WHITE).is_some());
        assert!(rasterize_in(&cache, "home", 1000, Color::WHITE).is_some());

        let guard = cache.lock().unwrap();
        assert_eq!(guard.map.len(), CACHE_CAPACITY);
        assert!(guard.map.contains_key(&Key::new("home", 16, Color::WHITE)));
        assert!(guard
            .map
            .contains_key(&Key::new("home", 1000, Color::WHITE)));
        assert!(!guard.map.contains_key(&Key::new("home", 17, Color::WHITE)));
    }

    #[test]
    fn color_tint_changes_pixels() {
        let red = rasterize("home", 24, Color::rgb(255, 0, 0)).unwrap();
        let blue = rasterize("home", 24, Color::rgb(0, 0, 255)).unwrap();
        let i = red
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .position(|px| px[3] != 0)
            .expect("home has ink")
            * 4;
        assert_ne!(red.data()[i], blue.data()[i], "red channel");
        assert_ne!(red.data()[i + 2], blue.data()[i + 2], "blue channel");
    }

    #[test]
    fn unknown_name_and_zero_size_return_none() {
        assert!(paths::icon_path("no_such_icon").is_none());
        assert!(rasterize("no_such_icon", 16, Color::WHITE).is_none());
        assert!(rasterize("home", 0, Color::WHITE).is_none());
    }

    #[test]
    fn cached_hit_is_pixel_identical() {
        let first = rasterize("star", 20, Color::rgb(1, 2, 3)).unwrap();
        let second = rasterize("star", 20, Color::rgb(1, 2, 3)).unwrap();
        assert_eq!(first.data(), second.data());
    }

    /// Regression: `star_border` used to be byte-identical to `star`, so the
    /// two rasterizations matched exactly. It must now be a hollow outline.
    #[test]
    fn star_border_differs_from_filled_star() {
        assert_ne!(
            paths::icon_path("star"),
            paths::icon_path("star_border"),
            "star_border must not reuse the filled star's path data"
        );
        const SIZE: usize = 48;
        let filled = rasterize("star", SIZE as u32, Color::WHITE).unwrap();
        let border = rasterize("star_border", SIZE as u32, Color::WHITE).unwrap();
        let differing = filled
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(border.data().as_chunks::<4>().0.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            differing >= SIZE * SIZE / 100,
            "star and star_border differ in only {differing} of {} pixels",
            SIZE * SIZE
        );
    }
}

//! Software rasterizer producing RGBA8 frames.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

use fontdue::{Font, FontSettings, Metrics};
use hodo_layout::{ControlKind, DisplayItem, DisplayList};
use hodo_types::{Color, Rect, Size};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

static FONT: OnceLock<Option<Font>> = OnceLock::new();
/// DejaVu Sans has no Han coverage; AOSP's DroidSansFallback (Apache-2.0)
/// supplies it. Parsed lazily, exactly like `FONT`.
static FONT_CJK: OnceLock<Option<Font>> = OnceLock::new();

/// Decoded-pixel budget: 4096 × 4096 = 16.7M px; at 4 B/px that is 64 MiB
/// worst case. Larger images are rejected before any pixel buffer is allocated
/// and fall back to the gray placeholder. This is separate from the net layer's
/// 2 MiB cap on compressed container bytes.
const MAX_DECODED_PIXELS: u64 = 4096 * 4096;

/// Shared budget gate for every decoder: `false` means reject, never allocate.
fn within_budget(w: u32, h: u32) -> bool {
    u64::from(w) * u64::from(h) <= MAX_DECODED_PIXELS
}

fn font() -> Option<&'static Font> {
    FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../../../assets/fonts/DejaVuSans.ttf").as_slice(),
            FontSettings::default(),
        )
        .ok()
    })
    .as_ref()
}

fn font_cjk() -> Option<&'static Font> {
    FONT_CJK
        .get_or_init(|| {
            Font::from_bytes(
                include_bytes!("../../../assets/fonts/DroidSansFallback.ttf").as_slice(),
                FontSettings::default(),
            )
            .ok()
        })
        .as_ref()
}

/// One rasterized glyph: an A8 coverage mask plus the fontdue metrics that
/// position it. A unique `(size, face, codepoint)` triple is rasterized once
/// per process in the common case (a rare first-touch race may duplicate one).
struct GlyphBitmap {
    metrics: Metrics,
    coverage: Vec<u8>,
    last_used: AtomicU64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    size_bits: u32,
    ch: char,
    /// Which face rasterized the mask; keeps DejaVu and Droid entries apart.
    cjk: bool,
}

struct AtlasState {
    map: HashMap<GlyphKey, Arc<GlyphBitmap>>,
    bytes: usize,
}

/// 1 MiB of A8 coverage masks. At 16 px DejaVu a glyph averages ~150 B, so
/// this holds several thousand unique glyphs — far more than a typical page.
const DEFAULT_ATLAS_CAP_BYTES: usize = 1024 * 1024;

/// Live cap, set from the persisted glyph-cache setting. Eviction re-reads it
/// on every insertion, so a change takes effect on the next frame's glyph work.
static ATLAS_CAP_BYTES: AtomicUsize = AtomicUsize::new(DEFAULT_ATLAS_CAP_BYTES);

/// Set the glyph atlas cap in MiB; applied on the next glyph insertion (frame).
/// Zero/oversized values fall back to the 1 MiB default.
pub fn set_glyph_cache_cap_mb(mb: f32) {
    let bytes = if mb.is_finite() && mb > 0.0 && mb <= 64.0 {
        (mb * 1024.0 * 1024.0) as usize
    } else {
        DEFAULT_ATLAS_CAP_BYTES
    };
    ATLAS_CAP_BYTES.store(bytes.max(1), Ordering::Relaxed);
}

/// Current atlas cap in bytes (test/telemetry accessor).
pub fn glyph_cache_cap_bytes() -> usize {
    ATLAS_CAP_BYTES.load(Ordering::Relaxed)
}

static GLYPH_ATLAS: OnceLock<RwLock<AtlasState>> = OnceLock::new();
static ATLAS_TICK: AtomicU64 = AtomicU64::new(1);
static ATLAS_HITS: AtomicU64 = AtomicU64::new(0);
static ATLAS_MISSES: AtomicU64 = AtomicU64::new(0);
/// How often the cmap fallback decision runs. Hot Latin hits never touch it.
static ATLAS_HAS_GLYPH_PROBES: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
thread_local! {
    /// Thread-local mirror for the direct-thread hot-path test's exact counts;
    /// parallel render tests would otherwise add noise to the global counter.
    static TEST_HAS_GLYPH_PROBES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// `has_glyph` is a cmap lookup; only call it from the atlas miss path. Tests
/// also observe the global counter via `glyph_atlas_stats().has_glyph_probes`
/// so probes made on band-worker threads are counted.
fn probe_has_glyph(font: &Font, ch: char) -> bool {
    #[cfg(test)]
    TEST_HAS_GLYPH_PROBES.with(|c| c.set(c.get() + 1));
    ATLAS_HAS_GLYPH_PROBES.fetch_add(1, Ordering::Relaxed);
    font.has_glyph(ch)
}

fn glyph_atlas() -> &'static RwLock<AtlasState> {
    GLYPH_ATLAS.get_or_init(|| {
        RwLock::new(AtlasState {
            map: HashMap::new(),
            bytes: 0,
        })
    })
}

/// Look up one atlas key, refreshing its LRU stamp on a hit.
fn atlas_lookup(key: &GlyphKey) -> Option<Arc<GlyphBitmap>> {
    let bitmap = glyph_atlas()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .map
        .get(key)
        .cloned()?;
    bitmap.last_used.store(
        ATLAS_TICK.fetch_add(1, Ordering::Relaxed),
        Ordering::Relaxed,
    );
    ATLAS_HITS.fetch_add(1, Ordering::Relaxed);
    Some(bitmap)
}

/// Return the cached coverage mask for `ch`, rasterizing once on a miss.
/// Codepoints DejaVu lacks use the CJK fallback; the face is part of the key so
/// the two masks never collide. The returned `Arc` keeps the mask alive even if
/// the atlas evicts it while the caller is blitting.
fn glyph_cached(font: &Font, ch: char, font_size: f32) -> Arc<GlyphBitmap> {
    let size_bits = font_size.to_bits();
    let primary = GlyphKey {
        size_bits,
        ch,
        cjk: false,
    };

    // Fast path: most glyphs are Latin and live in the primary face. Probe its
    // key first so the hot path never pays for a `has_glyph` cmap lookup.
    if let Some(bitmap) = atlas_lookup(&primary) {
        return bitmap;
    }

    // Miss only: decide whether the CJK fallback owns this codepoint. If
    // neither covers it, the primary face still rasterizes today's tofu box.
    let (font, cjk) = if probe_has_glyph(font, ch) {
        (font, false)
    } else {
        match font_cjk() {
            Some(fallback) if probe_has_glyph(fallback, ch) => (fallback, true),
            _ => (font, false),
        }
    };
    let key = GlyphKey { size_bits, ch, cjk };
    // A CJK glyph may already be cached under its own key; re-probe that key.
    if cjk {
        if let Some(bitmap) = atlas_lookup(&key) {
            return bitmap;
        }
    }

    ATLAS_MISSES.fetch_add(1, Ordering::Relaxed);
    // Rasterize outside the lock so threads never serialize on the slow path.
    // A duplicate rasterization on a race is harmless: identical coverage.
    let (metrics, coverage) = font.rasterize(ch, font_size);
    let bitmap = Arc::new(GlyphBitmap {
        metrics,
        coverage,
        last_used: AtomicU64::new(ATLAS_TICK.fetch_add(1, Ordering::Relaxed)),
    });
    let mut state = glyph_atlas().write().unwrap_or_else(|e| e.into_inner());
    state.bytes += bitmap.coverage.len();
    if let Some(previous) = state.map.insert(key, Arc::clone(&bitmap)) {
        state.bytes = state.bytes.saturating_sub(previous.coverage.len());
    }
    // Evict LRU entries until the byte cap fits. A single oversized glyph is
    // kept: dropping it mid-blit would be invalid. A working set larger than
    // the cap (e.g. a 2 MiB CJK page) thrashes by design — the cap is the RAM
    // promise, not a guarantee that every glyph stays resident.
    while state.bytes > ATLAS_CAP_BYTES.load(Ordering::Relaxed) && state.map.len() > 1 {
        let Some(lru) = state
            .map
            .iter()
            .min_by_key(|(_, b)| b.last_used.load(Ordering::Relaxed))
            .map(|(k, _)| *k)
        else {
            break;
        };
        if let Some(removed) = state.map.remove(&lru) {
            state.bytes = state.bytes.saturating_sub(removed.coverage.len());
        }
    }
    bitmap
}

/// Snapshot of glyph-atlas occupancy and cumulative hit counters.
#[derive(Debug, Clone, Copy)]
pub struct GlyphAtlasStats {
    pub entries: usize,
    pub bytes: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
    /// cmap lookups; stays flat on the hot Latin hit path.
    pub has_glyph_probes: u64,
}

/// Glyph atlas stats for performance reporting (cumulative since process start).
pub fn glyph_atlas_stats() -> GlyphAtlasStats {
    let state = glyph_atlas().read().unwrap_or_else(|e| e.into_inner());
    GlyphAtlasStats {
        entries: state.map.len(),
        bytes: state.bytes,
        capacity: glyph_cache_cap_bytes(),
        hits: ATLAS_HITS.load(Ordering::Relaxed),
        misses: ATLAS_MISSES.load(Ordering::Relaxed),
        has_glyph_probes: ATLAS_HAS_GLYPH_PROBES.load(Ordering::Relaxed),
    }
}

/// Integer-pixel rectangle (half-open `[x, x+w) × [y, y+h)`).
///
/// Used for damage regions and scroll bands so blitting and fresh rasterization
/// agree at the pixel level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IntRect {
    pub fn area(self) -> u64 {
        self.w.max(0) as u64 * self.h.max(0) as u64
    }

    pub fn right(self) -> i32 {
        self.x.saturating_add(self.w)
    }

    pub fn bottom(self) -> i32 {
        self.y.saturating_add(self.h)
    }

    pub fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            w: self.right().max(other.right()) - x,
            h: self.bottom().max(other.bottom()) - y,
        }
    }

    pub fn padded(self, pad: i32) -> Self {
        Self {
            x: self.x - pad,
            y: self.y - pad,
            w: self.w + 2 * pad,
            h: self.h + 2 * pad,
        }
    }

    /// Intersect with a `(0, 0, w, h)` surface; `None` when fully outside.
    pub fn clamp(self, w: i32, h: i32) -> Option<Self> {
        let x = self.x.max(0);
        let y = self.y.max(0);
        let right = self.right().min(w);
        let bottom = self.bottom().min(h);
        (x < right && y < bottom).then_some(Self {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    }
}

/// AA + fontdue glyph spill allowance for a changed item's bounding box.
pub const DAMAGE_PAD: i32 = 2;
/// Above this many disjoint damage rects, a full repaint is cheaper.
const MAX_DAMAGE_RECTS: usize = 32;

/// Outcome of diffing two composed display lists.
#[derive(Debug, Clone, PartialEq)]
pub enum Damage {
    None,
    Full,
    Rects(Vec<IntRect>),
}

fn item_rect(item: &DisplayItem) -> Option<&Rect> {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => Some(rect),
    }
}

fn item_rect_mut(item: &mut DisplayItem) -> Option<&mut Rect> {
    match item {
        DisplayItem::Rect { rect, .. }
        | DisplayItem::Text { rect, .. }
        | DisplayItem::Image { rect, .. }
        | DisplayItem::Control { rect, .. } => Some(rect),
    }
}

fn to_int_rect(rect: &Rect) -> IntRect {
    let x = rect.x.floor() as i32;
    let y = rect.y.floor() as i32;
    IntRect {
        x,
        y,
        w: ((rect.x + rect.w).ceil() as i32 - x).max(0),
        h: ((rect.y + rect.h).ceil() as i32 - y).max(0),
    }
}

fn item_bounds(item: &DisplayItem) -> Option<IntRect> {
    item_rect(item).map(to_int_rect)
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash = (*hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
    }
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash = (*hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
    }
}

fn hash_f32(hash: &mut u64, value: f32) {
    hash_u64(hash, value.to_bits() as u64);
}

fn hash_color(hash: &mut u64, color: Color) {
    hash_u64(
        hash,
        u32::from(color.r) as u64
            | ((u32::from(color.g) as u64) << 8)
            | ((u32::from(color.b) as u64) << 16)
            | ((u32::from(color.a) as u64) << 24),
    );
}

fn hash_rect(hash: &mut u64, rect: &Rect) {
    hash_f32(hash, rect.x);
    hash_f32(hash, rect.y);
    hash_f32(hash, rect.w);
    hash_f32(hash, rect.h);
}

/// FNV-1a over every field of a display item that can affect painted pixels.
/// A changed hash means the item must be repainted.
pub fn item_hash(item: &DisplayItem) -> u64 {
    let mut hash = FNV_OFFSET;
    match item {
        DisplayItem::Rect {
            rect,
            color,
            border_color,
            radius,
        } => {
            hash_u64(&mut hash, 0);
            hash_rect(&mut hash, rect);
            hash_color(&mut hash, *color);
            hash_u64(&mut hash, u64::from(border_color.is_some()));
            if let Some(border) = border_color {
                hash_color(&mut hash, *border);
            }
            hash_f32(&mut hash, *radius);
        }
        DisplayItem::Text {
            rect,
            text,
            color,
            font_size,
            weight,
            href,
            underline,
            line_through,
        } => {
            hash_u64(&mut hash, 1);
            hash_rect(&mut hash, rect);
            hash_bytes(&mut hash, text.as_bytes());
            hash_color(&mut hash, *color);
            hash_f32(&mut hash, *font_size);
            hash_u64(&mut hash, u64::from(*weight));
            hash_u64(&mut hash, u64::from(href.is_some()));
            if let Some(href) = href {
                hash_bytes(&mut hash, href.as_bytes());
            }
            hash_u64(&mut hash, u64::from(*underline));
            hash_u64(&mut hash, u64::from(*line_through));
        }
        DisplayItem::Image { rect, path } => {
            hash_u64(&mut hash, 2);
            hash_rect(&mut hash, rect);
            hash_bytes(&mut hash, path.as_bytes());
        }
        DisplayItem::Control {
            rect,
            kind,
            text,
            name,
            masked,
            disabled,
            readonly,
        } => {
            hash_u64(&mut hash, 3);
            hash_rect(&mut hash, rect);
            hash_u64(&mut hash, *kind as u64);
            hash_bytes(&mut hash, text.as_bytes());
            hash_bytes(&mut hash, name.as_bytes());
            hash_u64(&mut hash, u64::from(*masked));
            hash_u64(&mut hash, u64::from(*disabled));
            hash_u64(&mut hash, u64::from(*readonly));
        }
    }
    hash
}

/// Hash of a whole composed list, used to tell a scroll-only frame from a
/// content change without inspecting every item twice.
pub fn list_hash(list: &DisplayList) -> u64 {
    let mut hash = FNV_OFFSET;
    hash_u64(&mut hash, list.items.len() as u64);
    for item in &list.items {
        hash_u64(&mut hash, item_hash(item));
    }
    hash
}

fn merge_rects(rects: Vec<IntRect>) -> Vec<IntRect> {
    let mut merged: Vec<IntRect> = Vec::new();
    for rect in rects {
        if let Some(slot) = merged.iter_mut().find(|other| other.intersects(rect)) {
            *slot = slot.union(rect);
        } else {
            merged.push(rect);
        }
    }
    merged
}

/// Diff two composed display lists by index + payload hash. Same-index items
/// are compared; different lengths, too many rects or >50% repainted area
/// degrade to a full repaint.
pub fn diff_display_lists(old: &DisplayList, new: &DisplayList, viewport: Size) -> Damage {
    if old.items.len() != new.items.len() {
        return Damage::Full;
    }
    let (vw, vh) = (viewport.w.round() as i32, viewport.h.round() as i32);
    let viewport_area = (vw.max(0) as u64).saturating_mul(vh.max(0) as u64).max(1);
    let mut rects = Vec::new();
    for (before, after) in old.items.iter().zip(&new.items) {
        if item_hash(before) == item_hash(after) {
            continue;
        }
        // A moved item dirties both its old and its new location.
        let Some((old_bounds, new_bounds)) = item_bounds(before).zip(item_bounds(after)) else {
            continue;
        };
        let bounds = old_bounds.union(new_bounds);
        if let Some(clamped) = bounds.padded(DAMAGE_PAD).clamp(vw, vh) {
            rects.push(clamped);
        }
    }
    if rects.is_empty() {
        return Damage::None;
    }
    let rects = merge_rects(rects);
    if rects.len() > MAX_DAMAGE_RECTS {
        return Damage::Full;
    }
    let area: u64 = rects.iter().map(|rect| rect.area()).sum();
    if area.saturating_mul(2) > viewport_area {
        return Damage::Full;
    }
    Damage::Rects(rects)
}

/// Draw one display item into `pixmap`.
#[inline]
fn draw_item(pixmap: &mut Pixmap, item: &DisplayItem) {
    match item {
        DisplayItem::Rect { rect, color, .. } => {
            if let Some(r) = SkRect::from_xywh(rect.x, rect.y, rect.w, rect.h) {
                let path = PathBuilder::from_rect(r);
                let mut paint = Paint::default();
                paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                paint.anti_alias = true;
                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
        DisplayItem::Text {
            rect,
            text,
            color,
            font_size,
            underline,
            line_through,
            ..
        } => draw_text(
            pixmap,
            rect.x,
            rect.y,
            rect.h,
            text,
            *color,
            *font_size,
            *underline,
            *line_through,
        ),
        DisplayItem::Image { rect, path } => draw_image(pixmap, rect, path),
        // Form controls are layout-owned semantic items; rendering stays
        // intentionally simple (the forms agent only patches this arm).
        DisplayItem::Control {
            rect,
            kind,
            text,
            name,
            masked,
            disabled,
            readonly,
        } => draw_control(
            pixmap, rect, kind, text, name, *masked, *disabled, *readonly,
        ),
    }
}

fn shift_item_y(item: &DisplayItem, dy: f32) -> DisplayItem {
    let mut shifted = item.clone();
    if let Some(rect) = item_rect_mut(&mut shifted) {
        rect.y += dy;
    }
    shifted
}

/// Rasterize only `damage` (full-width row band) into the persistent premultiplied
/// pixmap. Intersecting items are drawn into a band-height scratch pixmap at
/// shifted coordinates, then copied over the band: this guarantees byte-identical
/// pixels to a full render without ever blending onto stale rows.
pub fn render_into(pixmap: &mut Pixmap, list: &DisplayList, damage: IntRect) {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    let Some(band) = damage.clamp(w, h) else {
        return;
    };
    if band.w <= 0 || band.h <= 0 {
        return;
    }
    // A full repaint reuses the persistent pixmap: stale pixels from the
    // previous frame must be erased before items with alpha are drawn again.
    // Partial bands are fully overwritten by the scratch copy, so no clear.
    if band.x == 0 && band.w == w && band.y == 0 && band.h == h {
        pixmap.data_mut().fill(0);
    }
    draw_into(pixmap, list, band, w);
}

fn draw_into(pixmap: &mut Pixmap, list: &DisplayList, band: IntRect, w: i32) {
    let h = pixmap.height() as i32;
    if band.x == 0 && band.w == w && band.y == 0 && band.h == h {
        draw_full(pixmap, list);
        return;
    }
    let Some(mut scratch) = Pixmap::new(w as u32, band.h as u32) else {
        return;
    };
    draw_band(&mut scratch, list, band);
    let row_bytes = w as usize * 4;
    let start = band.y as usize * row_bytes;
    let end = (band.y + band.h) as usize * row_bytes;
    pixmap.data_mut()[start..end].copy_from_slice(scratch.data());
}

/// Draw every item that can touch `band` into a zeroed, band-height scratch
/// pixmap, shifted by `-band.y`. This is the byte-identity contract shared by
/// damage bands and parallel full renders: a pixel's value depends only on the
/// items covering it, painted in order, so independent bands reproduce the
/// exact pixels of one full pass.
fn draw_band(scratch: &mut Pixmap, list: &DisplayList, band: IntRect) {
    for item in &list.items {
        let Some(bounds) = item_bounds(item) else {
            continue;
        };
        if !bounds.padded(DAMAGE_PAD).intersects(band) {
            continue;
        }
        draw_item(scratch, &shift_item_y(item, -band.y as f32));
    }
}

/// Lists below this size stay single-threaded: thread startup would dominate.
/// The 2 MB / 25k-item corpora are far above it; damage singles are far below.
const PARALLEL_ITEM_THRESHOLD: usize = 256;

/// Full-surface paint. Large lists are split into independent horizontal bands
/// rendered by `std::thread::scope`, each owning one reusable band scratch.
/// Bands are copied back only after all workers finish, so there is no shared
/// mutable state and the result is deterministic and byte-identical to a
/// single-threaded pass.
fn draw_full(pixmap: &mut Pixmap, list: &DisplayList) {
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8)
        .min(h.max(1) as usize);
    if threads <= 1 || list.items.len() < PARALLEL_ITEM_THRESHOLD {
        for item in &list.items {
            draw_item(pixmap, item);
        }
        return;
    }
    // Cold images are decoded/scaled up front (in parallel, <= 4 threads) so
    // band workers only hit the cache. Damage paths skip this entirely.
    warm_images(list, w, h);
    // One scratch per thread. Total peak = N * band_h * w * 4 B; because the
    // bands partition the height this is one surface's worth (~3.7 MB at
    // 1200x800), plus the <= 1 MiB glyph atlas.
    let band_h = (h as usize).div_ceil(threads);
    let row_bytes = w as usize * 4;
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);
        for i in 0..threads {
            let y = i * band_h;
            if y >= h as usize {
                break;
            }
            let band = IntRect {
                x: 0,
                y: y as i32,
                w,
                h: band_h.min(h as usize - y) as i32,
            };
            let list = &*list;
            handles.push(scope.spawn(move || {
                let mut scratch = Pixmap::new(w as u32, band.h as u32);
                if let Some(scratch) = scratch.as_mut() {
                    draw_band(scratch, list, band);
                }
                (band, scratch)
            }));
        }
        // Collect and copy in band order; nothing observes a partial frame.
        let mut bands: Vec<(IntRect, Option<Pixmap>)> = handles
            .into_iter()
            .map(|handle| handle.join().expect("band worker panicked"))
            .collect();
        bands.sort_unstable_by_key(|(band, _)| band.y);
        let data = pixmap.data_mut();
        for (band, scratch) in bands {
            let Some(scratch) = scratch else {
                continue;
            };
            let start = band.y as usize * row_bytes;
            let end = (band.y + band.h) as usize * row_bytes;
            if end <= data.len() && scratch.data().len() == end - start {
                data[start..end].copy_from_slice(scratch.data());
            }
        }
    });
}

/// Depremultiply rows `y0..y1` of a pixmap into straight RGBA8 bytes.
pub fn rgba_rows(pixmap: &Pixmap, y0: i32, y1: i32) -> Vec<u8> {
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let (top, bottom) = (y0.max(0), y1.min(h));
    if w <= 0 || top >= bottom {
        return Vec::new();
    }
    let data = pixmap.data();
    let mut out = Vec::with_capacity((bottom - top) as usize * w as usize * 4);
    for y in top..bottom {
        for x in 0..w {
            let i = (y as usize * w as usize + x as usize) * 4;
            let a = data[i + 3] as u32;
            if a == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                out.push(depremultiply(data[i], a));
                out.push(depremultiply(data[i + 1], a));
                out.push(depremultiply(data[i + 2], a));
                out.push(data[i + 3]);
            }
        }
    }
    out
}

/// Rasterize to RGBA8 (non-premultiplied), row-major, `w * h * 4` bytes.
pub fn render(list: &DisplayList, size: Size) -> Vec<u8> {
    let w = size.w.round() as i32;
    let h = size.h.round() as i32;
    if w <= 0 || h <= 0 {
        return Vec::new();
    }
    let Some(mut pixmap) = Pixmap::new(w as u32, h as u32) else {
        return Vec::new();
    };
    // The pixmap is fresh (zeroed), so no clear is needed before drawing.
    draw_into(&mut pixmap, list, IntRect { x: 0, y: 0, w, h }, w);
    rgba_rows(&pixmap, 0, h)
}

fn depremultiply(v: u8, a: u32) -> u8 {
    ((v as u32 * 255 + 127) / a).min(255) as u8
}

/// Solid fill of an axis-aligned box; out-of-bounds parts are clipped by tiny-skia.
fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
    if let Some(r) = SkRect::from_xywh(x, y, w, h) {
        let path = PathBuilder::from_rect(r);
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// Solid pill (two end caps + middle rect). Winding fill unions the subpaths.
fn fill_pill(pixmap: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, color: Color) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = h * 0.5;
    let mut path = PathBuilder::new();
    path.push_circle(x + r, y + r, r);
    path.push_circle(x + w - r, y + r, r);
    path.push_rect(SkRect::from_xywh(x + r, y, (w - h).max(0.0), h).unwrap());
    if let Some(path) = path.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn fill_circle(pixmap: &mut Pixmap, cx: f32, cy: f32, r: f32, color: Color) {
    if let Some(path) = PathBuilder::from_circle(cx, cy, r) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn ui_color(c: hodo_ui::Color) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}

/// M3 Expressive switch for settings controls. `text` carries "on"/"off" from
/// the form registry, which is repainted on the next tick after a click.
// shortcut: static switch visual — hover/press state layers and the
// spring::DEFAULT_SPATIAL thumb animation (tokens::MOTION_SWITCH_MS) need a
// per-frame animated control path; upgrade when the present path animates.
fn draw_switch(pixmap: &mut Pixmap, rect: &hodo_types::Rect, text: &str, disabled: bool) {
    use hodo_ui::tokens;

    let track_w = tokens::SWITCH_TRACK_WIDTH;
    let track_h = tokens::SWITCH_TRACK_HEIGHT;
    let thumb = tokens::SWITCH_THUMB;
    let tx = rect.x + (rect.w - track_w) * 0.5;
    let ty = rect.y + (rect.h - track_h) * 0.5;
    let on = text == "on";
    let track = if disabled {
        ui_color(tokens::LIGHT.surface_container_highest)
    } else if on {
        ui_color(tokens::LIGHT.primary)
    } else {
        ui_color(tokens::LIGHT.outline)
    };
    fill_pill(pixmap, tx, ty, track_w, track_h, track);
    let cx = if on {
        tx + track_w - thumb * 0.5 - 2.0
    } else {
        tx + thumb * 0.5 + 2.0
    };
    fill_circle(pixmap, cx, ty + track_h * 0.5, thumb * 0.5, Color::WHITE);
}

/// Draw one form control: a surface-tone box with a 1px border, a label, and a
/// glyph box for checkable kinds. `name` selects the settings-switch visual.
#[allow(clippy::too_many_arguments)]
fn draw_control(
    pixmap: &mut Pixmap,
    rect: &hodo_types::Rect,
    kind: &ControlKind,
    text: &str,
    name: &str,
    masked: bool,
    disabled: bool,
    readonly: bool,
) {
    if name.starts_with("setting:") && matches!(kind, ControlKind::Checkbox) {
        draw_switch(pixmap, rect, text, disabled);
        return;
    }
    let ink = if disabled {
        Color::rgb(140, 140, 140)
    } else {
        Color::BLACK
    };
    let surface = if disabled {
        Color::rgb(235, 235, 235)
    } else if readonly {
        Color::rgb(248, 248, 248)
    } else {
        Color::WHITE
    };
    let border = Color::rgb(120, 120, 120);

    if matches!(kind, ControlKind::Checkbox | ControlKind::Radio) {
        let side = rect.w.min(rect.h).clamp(1.0, 14.0);
        fill_rect(pixmap, rect.x, rect.y, side, side, surface);
        fill_rect(pixmap, rect.x, rect.y, side, 1.0, border);
        fill_rect(pixmap, rect.x, rect.y + side - 1.0, side, 1.0, border);
        fill_rect(pixmap, rect.x, rect.y, 1.0, side, border);
        fill_rect(pixmap, rect.x + side - 1.0, rect.y, 1.0, side, border);
        if matches!(kind, ControlKind::Radio) {
            // shortcut: circular glyphs await a path primitive; a center dot
            // distinguishes radio from checkbox until then.
            let dot = (side / 4.0).max(2.0);
            fill_rect(
                pixmap,
                rect.x + (side - dot) * 0.5,
                rect.y + (side - dot) * 0.5,
                dot,
                dot,
                ink,
            );
        }
        let label: String = text.chars().take(64).collect();
        let fs = (rect.h * 0.6).clamp(8.0, 16.0);
        draw_text(
            pixmap,
            rect.x + side + 4.0,
            rect.y,
            1.2 * fs,
            &label,
            ink,
            fs,
            false,
            false,
        );
        return;
    }

    fill_rect(pixmap, rect.x, rect.y, rect.w, rect.h, surface);
    fill_rect(pixmap, rect.x, rect.y, rect.w, 1.0, border);
    fill_rect(pixmap, rect.x, rect.y + rect.h - 1.0, rect.w, 1.0, border);
    fill_rect(pixmap, rect.x, rect.y, 1.0, rect.h, border);
    fill_rect(pixmap, rect.x + rect.w - 1.0, rect.y, 1.0, rect.h, border);

    let fs = (rect.h * 0.55).clamp(8.0, 20.0);
    let shown: String = if masked {
        "*".repeat(text.chars().take(64).count())
    } else {
        text.chars().take(64).collect()
    };
    let text_w = shown.chars().count() as f32 * 0.6 * fs;
    let (x, y) = match kind {
        ControlKind::Button | ControlKind::Select => (
            rect.x + ((rect.w - text_w) * 0.5).max(2.0),
            rect.y + ((rect.h - 1.2 * fs) * 0.5).max(0.0),
        ),
        _ => (rect.x + 3.0, rect.y + ((rect.h - 1.2 * fs) * 0.5).max(0.0)),
    };
    draw_text(pixmap, x, y, 1.2 * fs, &shown, ink, fs, false, false);
}

// ---- Image decode cache -----------------------------------------------------
// Before this cache, `draw_image` re-read and re-decoded the file on *every*
// blit — once per repeated frame, and once per band for images crossing a
// parallel band boundary. The cache decodes once per (path, mtime, rect size)
// and keeps only the already-downscaled buffer; the full-resolution decode is
// dropped immediately.

/// Maximum number of distinct image entries kept.
const IMAGE_CACHE_CAP_ENTRIES: usize = 64;
/// Maximum total bytes of scaled RGBA8 kept (16 MiB). A single entry larger
/// than this is still retained: it cannot be split and a live `Arc` may be
/// mid-blit, so the cap bounds steady-state multi-image pages, not one giant.
const IMAGE_CACHE_CAP_BYTES: usize = 16 * 1024 * 1024;
/// Upper bound on threads spawned to warm a page's images in parallel.
const IMAGE_WARM_THREADS: usize = 4;
/// At most this many full-resolution decodes run at once. Each holds an rgba
/// buffer up to 64 MiB plus a scaled copy; `IMAGE_CACHE_CAP_BYTES` counts only
/// the scaled copy, so the transient scratch is bounded here instead (four
/// parallel warm decodes would otherwise peak near 512 MiB unreported).
const MAX_CONCURRENT_DECODES: usize = 2;
static DECODE_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static DECODE_PEAK_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static DECODE_SCRATCH_PEAK_BYTES: AtomicU64 = AtomicU64::new(0);

/// Identity of one decoded image. `file_sig` is `(mtime_ns, byte_len)` for a
/// plain local file — both signals invalidate the key, because a coarse mtime
/// alone can miss a same-size rewrite. It is `None` for URL sources, which
/// bypass the cache entirely (see `image_cache_get`).
#[derive(Clone, PartialEq, Eq, Hash)]
struct ImageKey {
    path: String,
    file_sig: Option<(u128, u64)>,
    target_w: u32,
    target_h: u32,
}

/// A decoded image already scaled to (at most) its layout rect. Kept as
/// straight RGBA8 — exactly what `blit_scaled` consumes — so blitting needs no
/// second buffer and no premultiplied round-trip.
struct ScaledImage {
    w: u32,
    h: u32,
    rgba: Vec<u8>,
    last_used: AtomicU64,
}

struct ImageCache {
    map: HashMap<ImageKey, Arc<ScaledImage>>,
    bytes: usize,
}

impl ImageCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            bytes: 0,
        }
    }

    fn get(&self, key: &ImageKey) -> Option<Arc<ScaledImage>> {
        self.map.get(key).cloned()
    }

    fn insert(&mut self, key: ImageKey, image: Arc<ScaledImage>) {
        self.bytes += image.rgba.len();
        if let Some(old) = self.map.insert(key, image) {
            self.bytes = self.bytes.saturating_sub(old.rgba.len());
        }
        // Evict LRU until both caps hold. At least one entry survives so a
        // single oversized image still caches instead of re-decoding forever.
        while (self.map.len() > IMAGE_CACHE_CAP_ENTRIES || self.bytes > IMAGE_CACHE_CAP_BYTES)
            && self.map.len() > 1
        {
            let Some(lru) = self
                .map
                .iter()
                .min_by_key(|(_, image)| image.last_used.load(Ordering::Relaxed))
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(removed) = self.map.remove(&lru) {
                self.bytes = self.bytes.saturating_sub(removed.rgba.len());
            }
        }
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.map.clear();
        self.bytes = 0;
    }
}

static IMAGE_CACHE: OnceLock<RwLock<ImageCache>> = OnceLock::new();
static IMAGE_TICK: AtomicU64 = AtomicU64::new(1);
static IMAGE_HITS: AtomicU64 = AtomicU64::new(0);
static IMAGE_MISSES: AtomicU64 = AtomicU64::new(0);
static IMAGE_DECODES: AtomicU64 = AtomicU64::new(0);

fn image_cache() -> &'static RwLock<ImageCache> {
    IMAGE_CACHE.get_or_init(|| RwLock::new(ImageCache::new()))
}

fn image_tick() -> u64 {
    IMAGE_TICK.fetch_add(1, Ordering::Relaxed)
}

/// Image-cache occupancy and cumulative counters (perf reporting + tests).
#[derive(Debug, Clone, Copy)]
pub struct ImageCacheStats {
    pub entries: usize,
    pub bytes: usize,
    pub capacity_entries: usize,
    pub capacity_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    /// Actual decode attempts (one per successful cold load).
    pub decodes: u64,
    /// Peak number of full-resolution decodes in flight (bounded by the gate).
    pub peak_in_flight: usize,
    /// Peak transient scratch bytes (full-res rgba + scaled copy) in one decode.
    pub scratch_peak_bytes: u64,
}

/// Image cache stats: cumulative since process start, snapshot on call.
pub fn image_cache_stats() -> ImageCacheStats {
    let state = image_cache().read().unwrap_or_else(|e| e.into_inner());
    ImageCacheStats {
        entries: state.map.len(),
        bytes: state.bytes,
        capacity_entries: IMAGE_CACHE_CAP_ENTRIES,
        capacity_bytes: IMAGE_CACHE_CAP_BYTES,
        hits: IMAGE_HITS.load(Ordering::Relaxed),
        misses: IMAGE_MISSES.load(Ordering::Relaxed),
        decodes: IMAGE_DECODES.load(Ordering::Relaxed),
        peak_in_flight: DECODE_PEAK_IN_FLIGHT.load(Ordering::Relaxed),
        scratch_peak_bytes: DECODE_SCRATCH_PEAK_BYTES.load(Ordering::Relaxed),
    }
}

/// Peak transient decode scratch (full-resolution rgba + scaled copy) seen so
/// far, for observability. Not bounded by `IMAGE_CACHE_CAP_BYTES`; the in-flight
/// gate (`MAX_CONCURRENT_DECODES`) is what bounds the aggregate.
pub fn decode_scratch_peak() -> u64 {
    DECODE_SCRATCH_PEAK_BYTES.load(Ordering::Relaxed)
}

/// Resolve an image source to the local path used for both the cache key and
/// the mtime/size signature. `file:///C:/x.png` and `C:/x.png` name the same
/// file, so the slash left before a Windows drive letter is removed. A Unix
/// absolute path (`/home/...`) keeps its leading slash.
fn image_local_path(path: &str) -> &str {
    let local = path.strip_prefix("file://").unwrap_or(path);
    if local.starts_with('/')
        && local.as_bytes().get(1).is_some_and(u8::is_ascii_alphabetic)
        && local.as_bytes().get(2) == Some(&b':')
    {
        &local[1..]
    } else {
        local
    }
}

/// Content signal for a local file: `(mtime_ns, byte_len)`. Returns `None` for
/// URL sources (http/data) and anything we cannot stat. `file://` URLs resolve
/// to a local path, so they are stripped and treated as files. Two signals are
/// needed because a filesystem mtime can be too coarse to notice a same-size
/// rewrite; URL sources have no mtime at all and are never cached.
fn image_file_sig(path: &str) -> Option<(u128, u64)> {
    let local = image_local_path(path);
    if local.contains("://") || local.starts_with("data:") {
        return None;
    }
    let meta = std::fs::metadata(local).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((mtime, meta.len()))
}

fn image_key(path: &str, rect: &hodo_types::Rect) -> ImageKey {
    ImageKey {
        // Key by the canonical local path, not the raw spelling, so a file://
        // URL and a plain path share one cache entry.
        path: image_local_path(path).to_string(),
        file_sig: image_file_sig(path),
        // Identical integer transform to `blit_scaled`'s destination size.
        target_w: (rect.w.round() as i32).max(1) as u32,
        target_h: (rect.h.round() as i32).max(1) as u32,
    }
}

/// Cache lookup that refuses non-file sources. Without an mtime+size signal a
/// cached http/data image could never notice a server-side byte swap, so URL
/// sources are re-decoded on every load. Remote-image caching belongs in the
/// net layer's cache (a later feature); faking it here would serve stale pixels
/// forever. `file://` URLs carry a real signature and are cacheable.
fn image_cache_get(key: &ImageKey) -> Option<Arc<ScaledImage>> {
    // No signature means a URL source: there is no stable key, so bypass.
    key.file_sig?;
    image_cache()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(key)
}

/// Mirror of `image_cache_get`: never store a source we cannot invalidate.
fn image_cache_put(key: ImageKey, image: Arc<ScaledImage>) {
    if key.file_sig.is_none() {
        return;
    }
    image_cache()
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .insert(key, image);
}

/// Read + sniff + decode a file. The 4096² budget check stays inside each
/// decoder, so the full-resolution buffer is still rejected before allocation.
fn decode_image(path: &str) -> Option<(u32, u32, Vec<u8>)> {
    IMAGE_DECODES.fetch_add(1, Ordering::Relaxed);
    let hodo_net::Resource::Other { data, .. } = hodo_net::load(path).ok()? else {
        return None;
    };
    let decoded = if data.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        decode_png(&data)
    } else if data.starts_with(&[0xff, 0xd8]) {
        decode_jpeg(&data)
    } else if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        decode_webp(&data)
    } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        decode_gif(&data)
    } else {
        None
    };
    decoded.filter(|(w, h, rgba)| *w > 0 && *h > 0 && rgba.len() >= *w as usize * *h as usize * 4)
}

/// Downscale to at most the layout rect, clamped to the intrinsic size — the
/// stored buffer is never upscaled beyond the source (`min(target, source)`,
/// minimum 1 px). Sampling is nearest-neighbour with the exact integer map
/// `blit_scaled` uses (`sx = ox * iw / sw`): pre-scaling then blitting 1:1 is
/// byte-identical to the old decode-and-blit-in-one-step path. tiny-skia's
/// `draw_pixmap` would premultiply and resample with different filter/rounding
/// and change pixels, so it is deliberately not used here.
fn scale_to_target(
    iw: u32,
    ih: u32,
    rgba: &[u8],
    target_w: u32,
    target_h: u32,
) -> (u32, u32, Vec<u8>) {
    let sw = target_w.min(iw).max(1);
    let sh = target_h.min(ih).max(1);
    let mut out = vec![0u8; sw as usize * sh as usize * 4];
    for oy in 0..sh {
        let sy = (u64::from(oy) * u64::from(ih) / u64::from(sh)) as usize;
        for ox in 0..sw {
            let sx = (u64::from(ox) * u64::from(iw) / u64::from(sw)) as usize;
            let src = (sy * iw as usize + sx) * 4;
            if src + 3 >= rgba.len() {
                continue;
            }
            let dst = (oy as usize * sw as usize + ox as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    (sw, sh, out)
}

/// RAII slot in the global decode gate. Dropped after the full-resolution
/// buffer is released, so the in-flight count tracks live scratch memory.
struct DecodePermit;

impl Drop for DecodePermit {
    fn drop(&mut self) {
        DECODE_IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Take a decode slot, queueing while `MAX_CONCURRENT_DECODES` are in flight.
fn acquire_decode_permit() -> DecodePermit {
    loop {
        let mut current = DECODE_IN_FLIGHT.load(Ordering::Relaxed);
        loop {
            if current >= MAX_CONCURRENT_DECODES {
                break;
            }
            match DECODE_IN_FLIGHT.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    DECODE_PEAK_IN_FLIGHT.fetch_max(current + 1, Ordering::Relaxed);
                    return DecodePermit;
                }
                Err(actual) => current = actual,
            }
        }
        std::thread::yield_now();
    }
}

/// Decode and downscale in one step. The full-resolution `rgba` is owned here
/// and dropped on return — only the small scaled buffer survives. A permit is
/// held for the whole window in which that scratch buffer is live.
fn load_scaled_image(path: &str, key: &ImageKey) -> Option<Arc<ScaledImage>> {
    let _permit = acquire_decode_permit();
    let (iw, ih, rgba) = decode_image(path)?;
    let (w, h, scaled) = scale_to_target(iw, ih, &rgba, key.target_w, key.target_h);
    DECODE_SCRATCH_PEAK_BYTES.fetch_max((rgba.len() + scaled.len()) as u64, Ordering::Relaxed);
    Some(Arc::new(ScaledImage {
        w,
        h,
        rgba: scaled,
        last_used: AtomicU64::new(image_tick()),
    }))
}

/// Load, decode (once) and blit an <img> into its layout rect. Failure paints
/// a placeholder instead of propagating an error into the frame pipeline.
fn draw_image(pixmap: &mut Pixmap, rect: &hodo_types::Rect, path: &str) {
    let key = image_key(path, rect);
    let cached = image_cache_get(&key);
    let image = if let Some(image) = cached {
        IMAGE_HITS.fetch_add(1, Ordering::Relaxed);
        image
    } else {
        IMAGE_MISSES.fetch_add(1, Ordering::Relaxed);
        let Some(image) = load_scaled_image(path, &key) else {
            draw_placeholder(pixmap, rect);
            return;
        };
        image_cache_put(key, Arc::clone(&image));
        image
    };
    image.last_used.store(image_tick(), Ordering::Relaxed);
    blit_scaled(pixmap, rect, image.w, image.h, &image.rgba);
}

/// Decode+scale one missing image without blitting it. `warm_images` calls
/// this in parallel; `draw_image` uses `load_scaled_image` on its own cold path.
fn warm_one(path: &str, key: &ImageKey) {
    if image_cache_get(key).is_some() {
        return;
    }
    IMAGE_MISSES.fetch_add(1, Ordering::Relaxed);
    let Some(image) = load_scaled_image(path, key) else {
        return;
    };
    image_cache_put(key.clone(), image);
}

/// Warm every visible, not-yet-cached image on a page *before* band workers
/// start, so N threads never all miss the same file and serialize on decode.
/// Damage bands do not call this: they warm on demand, so a 1-rect damage pays
/// exactly one decode for the images it touches.
fn warm_images(list: &DisplayList, surface_w: i32, surface_h: i32) {
    let surface = IntRect {
        x: 0,
        y: 0,
        w: surface_w,
        h: surface_h,
    };
    let mut seen: HashSet<ImageKey> = HashSet::new();
    let mut missing: Vec<(String, ImageKey)> = Vec::new();
    {
        let cache = image_cache();
        let guard = cache.read().unwrap_or_else(|e| e.into_inner());
        for item in &list.items {
            let DisplayItem::Image { rect, path } = item else {
                continue;
            };
            if !item_bounds(item)
                .is_some_and(|bounds| bounds.padded(DAMAGE_PAD).intersects(surface))
            {
                continue;
            }
            let key = image_key(path, rect);
            // Non-file sources bypass the cache; warming them cannot be reused.
            if key.file_sig.is_none() {
                continue;
            }
            if seen.insert(key.clone()) && guard.get(&key).is_none() {
                missing.push((path.clone(), key));
            }
        }
    }
    if missing.is_empty() {
        return;
    }
    let threads = IMAGE_WARM_THREADS.min(missing.len());
    if threads == 1 {
        for (path, key) in &missing {
            warm_one(path, key);
        }
        return;
    }
    let chunk = missing.len().div_ceil(threads);
    std::thread::scope(|scope| {
        for part in missing.chunks(chunk) {
            scope.spawn(move || {
                for (path, key) in part {
                    warm_one(path, key);
                }
            });
        }
    });
}

/// PNG -> (width, height, straight RGBA8). EXPAND + STRIP_16 turns indexed,
/// grayscale and 16-bit inputs into the 8-bit RGB/RGBA we handle here.
fn decode_png(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let header = reader.info();
    if !within_budget(header.width, header.height) {
        return None;
    }
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        _ => return None,
    };
    Some((info.width, info.height, rgba))
}

/// JPEG -> (width, height, straight RGBA8).
fn decode_jpeg(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    decoder.read_info().ok()?;
    let header = decoder.info()?;
    if !within_budget(u32::from(header.width), u32::from(header.height)) {
        return None;
    }
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;
    let rgba = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::L8 => pixels.iter().flat_map(|&v| [v, v, v, 255]).collect(),
        jpeg_decoder::PixelFormat::L16 => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], 255])
            .collect(),
        jpeg_decoder::PixelFormat::CMYK32 => pixels
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let (c, m, y, k) = (p[0] as u32, p[1] as u32, p[2] as u32, p[3] as u32);
                [
                    ((255 - c) * (255 - k) / 255) as u8,
                    ((255 - m) * (255 - k) / 255) as u8,
                    ((255 - y) * (255 - k) / 255) as u8,
                    255,
                ]
            })
            .collect(),
    };
    Some((u32::from(info.width), u32::from(info.height), rgba))
}

/// WebP -> (width, height, straight RGBA8). `image-webp` decodes the first frame
/// of animated files; lossy/RGB output is expanded to RGBA8.
fn decode_webp(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = image_webp::WebPDecoder::new(std::io::Cursor::new(data)).ok()?;
    let (w, h) = decoder.dimensions();
    if !within_budget(w, h) {
        return None;
    }
    let mut buf = vec![0; decoder.output_buffer_size()?];
    decoder.read_image(&mut buf).ok()?;
    let rgba = if decoder.has_alpha() {
        buf
    } else {
        buf.as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect()
    };
    Some((w, h, rgba))
}

/// GIF -> (width, height, straight RGBA8); first frame only, no animation.
fn decode_gif(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(std::io::Cursor::new(data)).ok()?;
    if !within_budget(u32::from(decoder.width()), u32::from(decoder.height())) {
        return None;
    }
    let frame = decoder.read_next_frame().ok()??;
    let (w, h) = (u32::from(frame.width), u32::from(frame.height));
    if !within_budget(w, h) {
        return None;
    }
    Some((w, h, frame.buffer.to_vec()))
}

/// Nearest-neighbour scale into the destination rect, source-over blended
/// into tiny-skia's premultiplied buffer.
fn blit_scaled(pixmap: &mut Pixmap, rect: &hodo_types::Rect, iw: u32, ih: u32, rgba: &[u8]) {
    if iw == 0 || ih == 0 || rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let x0 = rect.x.round() as i32;
    let y0 = rect.y.round() as i32;
    let dw = (rect.w.round() as i32).max(1) as u32;
    let dh = (rect.h.round() as i32).max(1) as u32;
    let data = pixmap.data_mut();
    for oy in 0..dh {
        let sy = (oy * ih / dh) as usize;
        for ox in 0..dw {
            let sx = (ox * iw / dw) as usize;
            let px = x0 + ox as i32;
            let py = y0 + oy as i32;
            if px < 0 || py < 0 || px >= pw || py >= ph {
                continue;
            }
            let si = (sy * iw as usize + sx) * 4;
            if si + 3 >= rgba.len() {
                continue;
            }
            let a = rgba[si + 3] as u32;
            let pr = rgba[si] as u32 * a / 255;
            let pg = rgba[si + 1] as u32 * a / 255;
            let pb = rgba[si + 2] as u32 * a / 255;
            let inv = 255 - a;
            let di = (py as usize * pw as usize + px as usize) * 4;
            data[di] = (pr + data[di] as u32 * inv / 255).min(255) as u8;
            data[di + 1] = (pg + data[di + 1] as u32 * inv / 255).min(255) as u8;
            data[di + 2] = (pb + data[di + 2] as u32 * inv / 255).min(255) as u8;
            data[di + 3] = (a + data[di + 3] as u32 * inv / 255).min(255) as u8;
        }
    }
}

/// Light-gray box with a 1px darker border for missing/unsupported images.
fn draw_placeholder(pixmap: &mut Pixmap, rect: &hodo_types::Rect) {
    // shortcut: generic placeholder until an image-error UI exists; upgrade when
    // broken-image icons or alt-text rendering are specified.
    let pw = pixmap.width() as i32;
    let ph = pixmap.height() as i32;
    let x0 = rect.x.round() as i32;
    let y0 = rect.y.round() as i32;
    let w = (rect.w.round() as i32).max(1);
    let h = (rect.h.round() as i32).max(1);
    let data = pixmap.data_mut();
    for oy in 0..h {
        for ox in 0..w {
            let px = x0 + ox;
            let py = y0 + oy;
            if px < 0 || py < 0 || px >= pw || py >= ph {
                continue;
            }
            let border = ox == 0 || oy == 0 || ox == w - 1 || oy == h - 1;
            let c = if border { 100 } else { 200 };
            let i = (py as usize * pw as usize + px as usize) * 4;
            data[i] = c;
            data[i + 1] = c;
            data[i + 2] = c;
            data[i + 3] = 255;
        }
    }
}

/// Source-over blend of a coverage glyph pixel into premultiplied pixmap data.
fn blend_glyph(data: &mut [u8], w: i32, h: i32, x: i32, y: i32, cov: u8, color: Color) {
    if x < 0 || y < 0 || x >= w || y >= h || cov == 0 {
        return;
    }
    let i = (y as usize * w as usize + x as usize) * 4;
    let sa = (color.a as u32 * cov as u32 + 127) / 255;
    let inv = 255 - sa;
    data[i] = ((color.r as u32 * sa + 127) / 255 + data[i] as u32 * inv / 255).min(255) as u8;
    data[i + 1] =
        ((color.g as u32 * sa + 127) / 255 + data[i + 1] as u32 * inv / 255).min(255) as u8;
    data[i + 2] =
        ((color.b as u32 * sa + 127) / 255 + data[i + 2] as u32 * inv / 255).min(255) as u8;
    data[i + 3] = (sa + data[i + 3] as u32 * inv / 255).min(255) as u8;
}

/// Eight-pixel u64-word blit (rung 1). Measured neutral-to-slower: 14.69 vs
/// 14.96 ms on 25k-text LARGE, 6.745 vs 6.940 ms on hodo-perf LARGE (blits are
/// only ~20% of the frame). Scalar stays the default; A/B test forces both.
const GLYPH_CHUNKED_BLIT: bool = false;

#[cfg(test)]
/// 0 = production default, 1 = scalar, 2 = chunked; lets the A/B test select
/// either path regardless of the production const.
static TEST_BLIT_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn use_chunked_blit() -> bool {
    #[cfg(test)]
    match TEST_BLIT_MODE.load(Ordering::Relaxed) {
        1 => return false,
        2 => return true,
        _ => {}
    }
    GLYPH_CHUNKED_BLIT
}

/// Blend one premultiplied RGBA pixel packed in native (little-endian) u32
/// order. The scalar rounding is deliberately preserved: rounded source term,
/// truncating destination term, identical to `blend_glyph`.
#[inline]
fn blend_glyph_packed(pixel: u32, cov: u8, cr: u32, cg: u32, cb: u32, ca: u32) -> u32 {
    if cov == 0 {
        return pixel;
    }
    let sa = (ca * cov as u32 + 127) / 255;
    let inv = 255 - sa;
    let r = ((cr * sa + 127) / 255 + (pixel & 0xff) * inv / 255).min(255);
    let g = ((cg * sa + 127) / 255 + ((pixel >> 8) & 0xff) * inv / 255).min(255);
    let b = ((cb * sa + 127) / 255 + ((pixel >> 16) & 0xff) * inv / 255).min(255);
    let a = (sa + ((pixel >> 24) & 0xff) * inv / 255).min(255);
    r | (g << 8) | (b << 16) | (a << 24)
}

/// Blend two adjacent pixels held in one u64 word. Zero coverage leaves its
/// pixel untouched, exactly like the scalar path.
#[inline]
fn blend_glyph_word(data: &mut [u8], off: usize, cov_a: u8, cov_b: u8, color: Color) {
    let (cr, cg, cb, ca) = (
        color.r as u32,
        color.g as u32,
        color.b as u32,
        color.a as u32,
    );
    match (cov_a, cov_b) {
        (0, 0) => {}
        (0, cov) => {
            let dst = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap_or([0; 4]));
            let out = blend_glyph_packed(dst, cov, cr, cg, cb, ca);
            data[off + 4..off + 8].copy_from_slice(&out.to_le_bytes());
        }
        (cov, 0) => {
            let dst = u32::from_le_bytes(data[off..off + 4].try_into().unwrap_or([0; 4]));
            let out = blend_glyph_packed(dst, cov, cr, cg, cb, ca);
            data[off..off + 4].copy_from_slice(&out.to_le_bytes());
        }
        (a, b) => {
            let dst = u64::from_le_bytes(data[off..off + 8].try_into().unwrap_or([0; 8]));
            let lo = blend_glyph_packed(dst as u32, a, cr, cg, cb, ca);
            let hi = blend_glyph_packed((dst >> 32) as u32, b, cr, cg, cb, ca);
            let out = u64::from(lo) | (u64::from(hi) << 32);
            data[off..off + 8].copy_from_slice(&out.to_le_bytes());
        }
    }
}

/// Blend one glyph row: eight pixels per iteration read/written as four u64
/// words (two pixels each), a two-pixel u64 word for short runs, and the
/// original scalar blit for the last pixel. Clipping is resolved once per row.
fn blend_glyph_row(
    data: &mut [u8],
    w: i32,
    h: i32,
    origin_x: i32,
    y: i32,
    coverage_row: &[u8],
    color: Color,
) {
    if y < 0 || y >= h {
        return;
    }
    // Half-open visible column range [start, end) in coverage coordinates.
    let start = (-(origin_x as i64)).max(0);
    let end = (w as i64 - origin_x as i64).min(coverage_row.len() as i64);
    if start >= end {
        return;
    }
    let (start, end) = (start as usize, end as usize);
    let mut off = (y as usize * w as usize + (origin_x as i64 + start as i64) as usize) * 4;
    let mut col = start;
    while col + 8 <= end {
        let cov_word = u64::from_le_bytes(coverage_row[col..col + 8].try_into().unwrap_or([0; 8]));
        if cov_word != 0 {
            for lane in 0..4 {
                let shift = lane * 16;
                blend_glyph_word(
                    data,
                    off + lane * 8,
                    ((cov_word >> shift) & 0xff) as u8,
                    ((cov_word >> (shift + 8)) & 0xff) as u8,
                    color,
                );
            }
        }
        col += 8;
        off += 32;
    }
    while col + 2 <= end {
        blend_glyph_word(data, off, coverage_row[col], coverage_row[col + 1], color);
        col += 2;
        off += 8;
    }
    if col < end {
        blend_glyph(
            data,
            w,
            h,
            origin_x + col as i32,
            y,
            coverage_row[col],
            color,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    h: f32,
    text: &str,
    color: Color,
    font_size: f32,
    underline: bool,
    line_through: bool,
) {
    if !font_size.is_finite() || font_size <= 0.0 {
        return;
    }
    let w = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    // Layout emits Text rects at the line-box top with h = 1.2 * font_size,
    // so the baseline sits at ~0.8 * font_size below the top.
    let baseline = y + 0.8 * (h / 1.2);
    let decoration_h = 1.0f32.max(font_size / 16.0);

    let Some(font) = font() else {
        // shortcut: no usable vendored font, re-run the font vendor step; gray rect placeholders
        let mut pen_x = x;
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }
            let advance = 0.6 * font_size;
            if let Some(r) = SkRect::from_xywh(pen_x, y, advance * 0.7, font_size * 0.7) {
                let path = PathBuilder::from_rect(r);
                let mut paint = Paint::default();
                paint.set_color_rgba8(128, 128, 128, color.a);
                paint.anti_alias = true;
                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
            pen_x += advance;
        }
        draw_decorations(
            pixmap,
            x,
            baseline,
            pen_x - x,
            decoration_h,
            font_size,
            color,
            underline,
            line_through,
        );
        return;
    };

    let data = pixmap.data_mut();
    let chunked = use_chunked_blit();
    let mut pen_x = x;
    for ch in text.chars() {
        if ch.is_control() {
            continue;
        }
        let bitmap = glyph_cached(font, ch, font_size);
        let metrics = &bitmap.metrics;
        let coverage = &bitmap.coverage;
        let origin_x = pen_x as i32 + metrics.xmin;
        // fontdue's ymin is the glyph's lowest point in font units (Y-up,
        // baseline at 0): ~0 for glyphs resting on the baseline, negative for
        // descenders. Its bitmap is top-down, so the top row sits at
        // baseline - height - ymin; this puts '.' on the baseline and 'g' below it.
        // `floor` (not `as i32`, which truncates toward zero) keeps placement
        // invariant under the integer y-shift used by damage/parallel bands.
        let origin_y = (baseline - metrics.height as f32 - metrics.ymin as f32).floor() as i32;
        for row in 0..metrics.height {
            let row_start = row * metrics.width;
            let row_end = row_start + metrics.width;
            if chunked {
                blend_glyph_row(
                    data,
                    w,
                    height,
                    origin_x,
                    origin_y + row as i32,
                    &coverage[row_start..row_end],
                    color,
                );
            } else {
                for (col, &cov) in coverage[row_start..row_end].iter().enumerate() {
                    blend_glyph(
                        data,
                        w,
                        height,
                        origin_x + col as i32,
                        origin_y + row as i32,
                        cov,
                        color,
                    );
                }
            }
        }
        let advance = if metrics.advance_width > 0.0 {
            metrics.advance_width
        } else {
            0.6 * font_size
        };
        pen_x += advance;
    }
    draw_decorations(
        pixmap,
        x,
        baseline,
        pen_x - x,
        decoration_h,
        font_size,
        color,
        underline,
        line_through,
    );
}

/// Paint text-decoration lines in the text color: underline below the baseline
/// and line-through across it.
#[allow(clippy::too_many_arguments)]
fn draw_decorations(
    pixmap: &mut Pixmap,
    x: f32,
    baseline: f32,
    text_w: f32,
    h: f32,
    font_size: f32,
    color: Color,
    underline: bool,
    line_through: bool,
) {
    for y in [
        underline.then_some(baseline + 2.0),
        line_through.then_some(baseline - 0.35 * font_size),
    ]
    .into_iter()
    .flatten()
    {
        let Some(r) = SkRect::from_xywh(x, y, text_w, h) else {
            continue;
        };
        let path = PathBuilder::from_rect(r);
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, color.a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_layout::DisplayItem;
    use hodo_types::Rect;

    /// Image tests share one process-wide cache and decode counters; this
    /// serializes the ones that assert on those counters.
    static IMAGE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Glyph-atlas tests share process-wide state; serialize the ones that
    /// assert on its contents, clear it, or flood it.
    static GLYPH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn pixel(buf: &[u8], x: usize, y: usize, w: usize) -> [u8; 4] {
        let i = (y * w + x) * 4;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// FNV-1a full-frame hash: the A/B invariance shared by every perf rung.
    fn frame_hash(bytes: &[u8]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash
    }

    /// Text-heavy golden: mixed Latin + CJK, partial alpha, and rows that start
    /// off-screen so the chunked path's clip/tail branches are exercised.
    fn text_golden() -> (DisplayList, Size) {
        let size = Size { w: 360.0, h: 160.0 };
        let items = [
            (4.0, "The quick brown fox 0123 Ajg", Color::BLACK, 16.0),
            (
                32.0,
                "MMMMWWWWiiii....gggg",
                Color::rgba(210, 40, 90, 200),
                20.0,
            ),
            (64.0, "日本語 mix éè", Color::rgb(20, 90, 160), 18.0),
            (
                96.0,
                "aaaa bbbb cccc dddd",
                Color::rgba(30, 200, 40, 128),
                14.0,
            ),
            (126.0, "WWWWWWWW", Color::rgb(255, 255, 255), 24.0),
        ]
        .into_iter()
        .map(|(y, text, color, font_size)| DisplayItem::Text {
            rect: Rect {
                x: 12.0,
                y,
                w: 340.0,
                h: font_size * 1.2,
            },
            text: text.into(),
            color,
            font_size,
            weight: 400,
            href: None,
            underline: false,
            line_through: false,
        })
        .collect();
        (list(items), size)
    }

    #[test]
    fn solid_rect_draws_exact_color() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Rect {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 30.0,
                    h: 20.0,
                },
                color: Color::rgb(255, 0, 0),
                border_color: None,
                radius: 0.0,
            }],
        };
        let buf = render(&list, Size { w: 100.0, h: 100.0 });
        assert_eq!(buf.len(), 100 * 100 * 4);
        assert_eq!(pixel(&buf, 20, 20, 100), [255, 0, 0, 255]);
        assert_eq!(pixel(&buf, 5, 5, 100), [0, 0, 0, 0]);
    }

    #[test]
    fn out_of_bounds_rect_draws_visible_part_without_panic() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Rect {
                rect: Rect {
                    x: -10.0,
                    y: -10.0,
                    w: 30.0,
                    h: 30.0,
                },
                color: Color::rgb(0, 255, 0),
                border_color: None,
                radius: 0.0,
            }],
        };
        let buf = render(&list, Size { w: 50.0, h: 50.0 });
        assert_eq!(pixel(&buf, 5, 5, 50), [0, 255, 0, 255]);
        assert_eq!(pixel(&buf, 25, 25, 50), [0, 0, 0, 0]);
    }

    #[test]
    fn text_draws_visible_pixels() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Text {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 100.0,
                    h: 50.0,
                },
                text: "Hi".into(),
                color: Color::rgb(255, 255, 255),
                font_size: 32.0,
                weight: 400,
                href: None,
                underline: false,
                line_through: false,
            }],
        };
        let buf = render(&list, Size { w: 200.0, h: 100.0 });
        let visible = (10..90).any(|x| (10..60).any(|y| pixel(&buf, x, y, 200)[3] > 0));
        assert!(visible, "expected a non-transparent pixel in the text band");
    }

    /// Average y (screen rows, larger = lower) of visible pixels in an x band.
    fn avg_dark_y(buf: &[u8], w: usize, h: usize, x0: usize, x1: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0usize;
        for y in 0..h {
            for x in x0..x1 {
                if buf[(y * w + x) * 4 + 3] > 0 {
                    sum += y as f64;
                    count += 1;
                }
            }
        }
        (count > 0).then_some(sum / count as f64)
    }

    #[test]
    fn punctuation_sits_on_baseline_below_ascenders() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Text {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 100.0,
                    h: 1.2 * 32.0,
                },
                text: "l.".into(),
                color: Color::rgb(255, 255, 255),
                font_size: 32.0,
                weight: 400,
                href: None,
                underline: false,
                line_through: false,
            }],
        };
        let buf = render(&list, Size { w: 200.0, h: 100.0 });
        // 'l' rasterizes around x=13..16; '.' around x=22..25 at 32px.
        let ell = avg_dark_y(&buf, 200, 100, 10, 20).expect("'l' must rasterize");
        let dot = avg_dark_y(&buf, 200, 100, 20, 40).expect("'.' must rasterize");
        assert!(
            dot > ell,
            "the period must sit on the baseline below 'l': l={ell}, dot={dot}"
        );
    }

    #[test]
    fn later_rect_paints_on_top() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![
                DisplayItem::Rect {
                    rect: Rect {
                        x: 10.0,
                        y: 10.0,
                        w: 40.0,
                        h: 40.0,
                    },
                    color: Color::rgb(255, 0, 0),
                    border_color: None,
                    radius: 0.0,
                },
                DisplayItem::Rect {
                    rect: Rect {
                        x: 20.0,
                        y: 20.0,
                        w: 40.0,
                        h: 40.0,
                    },
                    color: Color::rgb(0, 0, 255),
                    border_color: None,
                    radius: 0.0,
                },
            ],
        };
        let buf = render(&list, Size { w: 100.0, h: 100.0 });
        assert_eq!(pixel(&buf, 15, 15, 100), [255, 0, 0, 255]);
        assert_eq!(pixel(&buf, 25, 25, 100), [0, 0, 255, 255]);
    }

    fn temp_dir() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("hodo-render-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_png(path: &std::path::Path, w: u32, h: u32, pixels: &[[u8; 4]]) {
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let flat: Vec<u8> = pixels.iter().flatten().copied().collect();
        writer.write_image_data(&flat).unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn image_draws_decoded_png_pixels() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("solid.png");
        let expected = [12, 34, 56, 255];
        let file = std::fs::File::create(&path).unwrap();
        let mut encoder = png::Encoder::new(file, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&[expected; 4].concat()).unwrap();
        writer.finish().unwrap();

        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Image {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 20.0,
                    h: 20.0,
                },
                path: path.to_string_lossy().into_owned(),
            }],
        };
        let buf = render(&list, Size { w: 100.0, h: 100.0 });
        assert_eq!(pixel(&buf, 15, 15, 100), expected, "decoded PNG color");
        assert_eq!(pixel(&buf, 29, 29, 100), expected, "scaled PNG color");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn control_button_draws_surface_border_and_label() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Control {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 80.0,
                    h: 30.0,
                },
                kind: ControlKind::Button,
                text: "OK".into(),
                name: "ok".into(),
                masked: false,
                disabled: false,
                readonly: false,
            }],
        };
        let buf = render(&list, Size { w: 100.0, h: 60.0 });
        assert_eq!(pixel(&buf, 15, 30, 100), [255, 255, 255, 255], "surface");
        assert!(pixel(&buf, 10, 10, 100)[3] > 0, "1px border must paint");
        let mut ink = 0;
        for x in 40..70 {
            for y in 14..38 {
                let p = pixel(&buf, x, y, 100);
                if p != [255, 255, 255, 255] && p[3] > 0 {
                    ink += 1;
                }
            }
        }
        assert!(ink > 0, "centered label must rasterize");
    }

    #[test]
    fn missing_image_draws_gray_placeholder_without_panic() {
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Image {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 10.0,
                    h: 10.0,
                },
                path: "hodo-no-such-image-42.png".to_string(),
            }],
        };
        let buf = render(&list, Size { w: 100.0, h: 100.0 });
        assert_eq!(pixel(&buf, 15, 15, 100), [200, 200, 200, 255]);
        assert_eq!(pixel(&buf, 10, 10, 100), [100, 100, 100, 255]);
    }

    // Tiny Pillow-generated fixtures (kept as text, never binary files).
    // 1x1 lossless WebP with alpha = 128, RGB(200,100,50).
    const WEBP_1X1_ALPHA: &str = "UklGRh4AAABXRUJQVlA4TBEAAAAvAAAAEAdQsiJXpoCBiOh/AAA=";
    // 4x4 lossless WebP without alpha, RGB(17,68,119).
    const WEBP_4X4_RGB: &str = "UklGRh4AAABXRUJQVlA4TBEAAAAvA8AAAAdQokb0rv+BiOh/AAA=";
    // 4x4 animated GIF: frame 0 = RGB(17,68,119), frame 1 = RGB(200,100,50).
    const GIF_2FRAME: &str = "R0lGODlhBAAEAIEAABFEd8hkMgAAAAAAACH/C05FVFNDQVBFMi4wAwEAAAAh+QQACgAAACwAAAAABAAEAAAICQABCBxIsCCAgAAh+QQACgAAACwAAAAABAAEAIERRHfIZDIAAAAAAAAICQADCBxIsGCAgAA7";
    // Minimal hand-built GIF87a: 1x1, palette[0] = RGB(17,68,119).
    const GIF87A_1X1: &str = "R0lGODdhAQABAIAAABFEd8hkMiwAAAAAAQABAAACAoQBADs=";

    fn b64(s: &str) -> Vec<u8> {
        let mut out = Vec::new();
        let (mut acc, mut bits) = (0u32, 0u32);
        for c in s.bytes() {
            let v = match c {
                b'A'..=b'Z' => c - b'A',
                b'a'..=b'z' => c - b'a' + 26,
                b'0'..=b'9' => c - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' => break,
                _ => panic!("bad base64 fixture"),
            };
            acc = (acc << 6) | u32::from(v);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
            }
        }
        out
    }

    fn render_image_bytes(name: &str, bytes: &[u8]) -> Vec<u8> {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        let list = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![DisplayItem::Image {
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 20.0,
                    h: 20.0,
                },
                path: path.to_string_lossy().into_owned(),
            }],
        };
        let buf = render(&list, Size { w: 100.0, h: 100.0 });
        std::fs::remove_dir_all(&dir).unwrap();
        buf
    }

    fn assert_placeholder(buf: &[u8]) {
        assert_eq!(pixel(buf, 15, 15, 100), [200, 200, 200, 255]);
        assert_eq!(pixel(buf, 10, 10, 100), [100, 100, 100, 255]);
    }

    #[test]
    fn webp_with_alpha_draws_decoded_pixels() {
        let buf = render_image_bytes("alpha.webp", &b64(WEBP_1X1_ALPHA));
        // round-trips through tiny-skia's premultiply + our depremultiply.
        assert_eq!(pixel(&buf, 15, 15, 100), [200, 100, 50, 128]);
    }

    #[test]
    fn webp_without_alpha_draws_scaled_pixels() {
        let buf = render_image_bytes("rgb.webp", &b64(WEBP_4X4_RGB));
        let expected = [17, 68, 119, 255];
        assert_eq!(pixel(&buf, 15, 15, 100), expected);
        assert_eq!(pixel(&buf, 29, 29, 100), expected, "scaled WebP color");
    }

    #[test]
    fn gif_draws_only_the_first_frame() {
        let buf = render_image_bytes("anim.gif", &b64(GIF_2FRAME));
        assert_eq!(pixel(&buf, 15, 15, 100), [17, 68, 119, 255]);
        // GIF87a must sniff identically to GIF89a.
        let buf = render_image_bytes("old.gif", &b64(GIF87A_1X1));
        assert_eq!(pixel(&buf, 15, 15, 100), [17, 68, 119, 255]);
    }

    #[test]
    fn riff_with_bad_webp_payload_draws_placeholder() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(b"WEBPnot-a-valid-payload");
        assert_placeholder(&render_image_bytes("bad.webp", &bytes));
    }

    #[test]
    fn oversized_decoded_dimensions_are_rejected() {
        // render_image_bytes below holds IMAGE_TEST_LOCK for the whole call.
        assert!(within_budget(4096, 4096), "budget boundary is inclusive");
        assert!(!within_budget(4097, 4096));
        assert!(!within_budget(4096, 4097));

        // Valid PNG header, no pixel data: the budget gate rejects the header
        // before any ~64 MiB output buffer is allocated.
        let bytes = {
            let mut bytes = Vec::new();
            {
                let mut encoder = png::Encoder::new(&mut bytes, 4097, 4096);
                encoder.set_color(png::ColorType::Rgb);
                encoder.set_depth(png::BitDepth::Eight);
                let _writer = encoder.write_header().unwrap();
            }
            bytes
        };
        assert_placeholder(&render_image_bytes("huge.png", &bytes));
    }

    // ---- M5.1 damage tracking ------------------------------------------------

    fn solid(x: f32, y: f32, w: f32, h: f32, color: Color) -> DisplayItem {
        DisplayItem::Rect {
            rect: Rect { x, y, w, h },
            color,
            border_color: None,
            radius: 0.0,
        }
    }

    fn label(y: f32, text: &str) -> DisplayItem {
        DisplayItem::Text {
            rect: Rect {
                x: 10.0,
                y,
                w: 80.0,
                h: 19.2,
            },
            text: text.into(),
            color: Color::BLACK,
            font_size: 16.0,
            weight: 400,
            href: None,
            underline: false,
            line_through: false,
        }
    }

    fn list(items: Vec<DisplayItem>) -> DisplayList {
        DisplayList {
            items,
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
        }
    }

    #[test]
    fn identical_lists_produce_no_damage() {
        let a = list(vec![solid(10.0, 10.0, 30.0, 20.0, Color::rgb(1, 2, 3))]);
        assert_eq!(
            diff_display_lists(&a, &a, Size { w: 100.0, h: 100.0 }),
            Damage::None
        );
    }

    #[test]
    fn single_color_change_produces_exact_padded_rect() {
        let old = list(vec![solid(10.0, 10.0, 30.0, 20.0, Color::rgb(1, 2, 3))]);
        let new = list(vec![solid(10.0, 10.0, 30.0, 20.0, Color::rgb(4, 5, 6))]);
        assert_eq!(
            diff_display_lists(&old, &new, Size { w: 100.0, h: 100.0 }),
            Damage::Rects(vec![IntRect {
                x: 8,
                y: 8,
                w: 34,
                h: 24,
            }])
        );
    }

    #[test]
    fn inserted_item_forces_full_repaint() {
        let old = list(vec![label(10.0, "a"), label(30.0, "b")]);
        let new = list(vec![
            label(10.0, "a"),
            label(20.0, "inserted"),
            label(30.0, "b"),
        ]);
        assert_eq!(
            diff_display_lists(&old, &new, Size { w: 100.0, h: 100.0 }),
            Damage::Full
        );
    }

    #[test]
    fn adjacent_changes_merge_into_one_union_rect() {
        let old = list(vec![
            solid(10.0, 10.0, 10.0, 10.0, Color::rgb(1, 2, 3)),
            solid(12.0, 10.0, 10.0, 10.0, Color::rgb(1, 2, 3)),
        ]);
        let new = list(vec![
            solid(10.0, 10.0, 10.0, 10.0, Color::rgb(9, 2, 3)),
            solid(12.0, 10.0, 10.0, 10.0, Color::rgb(8, 2, 3)),
        ]);
        assert_eq!(
            diff_display_lists(&old, &new, Size { w: 100.0, h: 100.0 }),
            Damage::Rects(vec![IntRect {
                x: 8,
                y: 8,
                w: 16,
                h: 14,
            }])
        );
    }

    #[test]
    fn too_many_changed_items_force_full_repaint() {
        let items = |value| {
            list(
                (0..40)
                    .map(|i| solid(i as f32 * 40.0, 5.0, 2.0, 2.0, Color::rgb(value, 0, 0)))
                    .collect(),
            )
        };
        assert_eq!(
            diff_display_lists(
                &items(1),
                &items(2),
                Size {
                    w: 2000.0,
                    h: 100.0
                }
            ),
            Damage::Full
        );
    }

    #[test]
    fn more_than_half_the_viewport_forces_full_repaint() {
        let old = list(vec![solid(0.0, 0.0, 80.0, 80.0, Color::rgb(1, 2, 3))]);
        let new = list(vec![solid(0.0, 0.0, 80.0, 80.0, Color::rgb(9, 2, 3))]);
        assert_eq!(
            diff_display_lists(&old, &new, Size { w: 100.0, h: 100.0 }),
            Damage::Full
        );
    }

    /// Apply a damage decision to a persistent pixmap + straight-RGBA buffer.
    fn apply_damage(
        pixmap: &mut Pixmap,
        buffer: &mut Vec<u8>,
        old: &DisplayList,
        new: &DisplayList,
        size: Size,
    ) -> Damage {
        let (w, h) = (size.w as i32, size.h as i32);
        let row_bytes = w as usize * 4;
        let damage = diff_display_lists(old, new, size);
        match &damage {
            Damage::Full => {
                render_into(pixmap, new, IntRect { x: 0, y: 0, w, h });
                *buffer = rgba_rows(pixmap, 0, h);
            }
            Damage::None => {}
            Damage::Rects(rects) => {
                for rect in rects {
                    let band = IntRect {
                        x: 0,
                        y: rect.y,
                        w,
                        h: rect.h,
                    };
                    render_into(pixmap, new, band);
                    let rows = rgba_rows(pixmap, rect.y, rect.y + rect.h);
                    let start = rect.y.max(0) as usize * row_bytes;
                    let end = (rect.y + rect.h).min(h) as usize * row_bytes;
                    if start < end && end <= buffer.len() {
                        buffer[start..end].copy_from_slice(&rows);
                    }
                }
            }
        }
        damage
    }

    /// `cargo test -p hodo-render --release -- --ignored --nocapture perf_report`
    /// Reports best-of-3 raster times for full / one-node damage / scroll.
    #[test]
    #[ignore]
    fn perf_report_incremental_paths() {
        use std::time::Instant;

        let size = Size {
            w: 1200.0,
            h: 800.0,
        };
        let (w, h) = (size.w as i32, size.h as i32);
        // Roughly the LARGE corpus: ~25k text items over a tall document.
        let page = list(
            (0..25_000)
                .map(|i| DisplayItem::Text {
                    rect: Rect {
                        x: 4.0,
                        y: (i / 60) as f32 * 20.0,
                        w: 120.0,
                        h: 19.2,
                    },
                    text: "The quick brown fox jumps over the lazy dog".into(),
                    color: Color::rgb((i % 200) as u8, 40, 90),
                    font_size: 16.0,
                    weight: 400,
                    href: None,
                    underline: false,
                    line_through: false,
                })
                .collect(),
        );
        let full_rect = IntRect { x: 0, y: 0, w, h };

        fn best(mut run: impl FnMut() -> (f64, u64)) -> (f64, u64) {
            let mut result = (f64::MAX, 0u64);
            for _ in 0..3 {
                let (ms, pixels) = run();
                if ms < result.0 {
                    result = (ms, pixels);
                }
            }
            result
        }

        let atlas_before = glyph_atlas_stats();
        let (full_ms, full_px) = best(|| {
            let start = Instant::now();
            let frame = render(&page, size);
            (
                start.elapsed().as_secs_f64() * 1000.0,
                (frame.len() / 4) as u64,
            )
        });
        let atlas_after = glyph_atlas_stats();

        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        render_into(&mut pixmap, &page, full_rect);
        let changed = {
            let mut next = page.clone();
            if let DisplayItem::Text { color, .. } = &mut next.items[100] {
                color.r = 7;
            }
            next
        };
        let (damage_ms, damage_px) = best(|| {
            let damage = diff_display_lists(&page, &changed, size);
            let mut pixels = 0u64;
            let start = Instant::now();
            if let Damage::Rects(rects) = damage {
                for rect in &rects {
                    let band = IntRect {
                        x: 0,
                        y: rect.y,
                        w,
                        h: rect.h,
                    };
                    render_into(&mut pixmap, &changed, band);
                    let rows = rgba_rows(&pixmap, rect.y, rect.y + rect.h);
                    pixels += rect.area();
                    std::hint::black_box(rows);
                }
            }
            (start.elapsed().as_secs_f64() * 1000.0, pixels)
        });

        let mut scroll_bench = |dy: i32| -> (f64, u64) {
            let row_bytes = w as usize * 4;
            best(|| {
                let start = Instant::now();
                if dy > 0 {
                    pixmap
                        .data_mut()
                        .copy_within(dy as usize * row_bytes..h as usize * row_bytes, 0);
                } else {
                    pixmap
                        .data_mut()
                        .copy_within(0..(h + dy) as usize * row_bytes, (-dy) as usize * row_bytes);
                }
                let strip = if dy > 0 {
                    IntRect {
                        x: 0,
                        y: h - dy,
                        w,
                        h: dy,
                    }
                } else {
                    IntRect {
                        x: 0,
                        y: 0,
                        w,
                        h: -dy,
                    }
                };
                render_into(&mut pixmap, &page, strip);
                let rows = rgba_rows(&pixmap, strip.y, strip.y + strip.h);
                std::hint::black_box(rows);
                // Only the exposed strip is rasterized; the rest is a blit.
                let pixels = dy.unsigned_abs() as u64 * w as u64;
                (start.elapsed().as_secs_f64() * 1000.0, pixels)
            })
        };
        let (line_ms, line_px) = scroll_bench(40);
        let (page_ms, page_px) = scroll_bench(400);

        let percent = |pixels: u64| pixels as f64 * 100.0 / (w as f64 * h as f64);
        println!("\n=== M5.1/M5.2 raster perf (best of 3, 1200x800) ===");
        println!(
            "full render:        {full_ms:8.3} ms  {} px ({:.1}%)",
            full_px,
            percent(full_px)
        );
        println!(
            "one-node damage:    {damage_ms:8.3} ms  {damage_px} px ({:.1}%)",
            percent(damage_px)
        );
        println!(
            "scroll by line:     {line_ms:8.3} ms  {line_px} px ({:.1}%)",
            percent(line_px)
        );
        println!(
            "scroll by page:     {page_ms:8.3} ms  {page_px} px ({:.1}%)",
            percent(page_px)
        );
        let hits = atlas_after.hits - atlas_before.hits;
        let misses = atlas_after.misses - atlas_before.misses;
        let rate = hits as f64 * 100.0 / (hits + misses).max(1) as f64;
        println!("\n=== glyph atlas (across the 3 full renders above) ===",);
        println!(
            "entries: {entries:6}  bytes: {used} / {cap} B (cap)",
            entries = atlas_after.entries,
            used = atlas_after.bytes,
            cap = atlas_after.capacity
        );
        println!(
            "hits:    {hits:6}  misses: {misses:4}  hit rate: {rate:.4}%  (misses = unique glyphs rasterized)"
        );
    }

    /// Rung 1 gate: chunked u64-word blit must be byte-for-byte the old scalar
    /// path. The scalar path stays reachable behind `GLYPH_CHUNKED_BLIT` plus
    /// the test-only override, so the same golden is rendered both ways.
    #[test]
    fn chunked_glyph_blit_matches_scalar_frame_hash() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (list, size) = text_golden();
        TEST_BLIT_MODE.store(1, Ordering::Relaxed);
        let scalar = render(&list, size);
        TEST_BLIT_MODE.store(2, Ordering::Relaxed);
        let chunked = render(&list, size);
        TEST_BLIT_MODE.store(0, Ordering::Relaxed);
        assert_eq!(
            frame_hash(&scalar),
            frame_hash(&chunked),
            "A/B frame hash diverged between scalar and chunked blit"
        );
        assert_eq!(scalar, chunked, "A/B frame bytes diverged");
    }

    /// `cargo test -p hodo-render --release -- --ignored --nocapture perf_report_glyph_blit`
    /// Text-heavy LARGE A/B: same corpus, scalar vs chunked, interleaved best of 5.
    #[test]
    #[ignore]
    fn perf_report_glyph_blit() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use std::time::Instant;
        let size = Size {
            w: 1200.0,
            h: 800.0,
        };
        let page = list(
            (0..25_000)
                .map(|i| DisplayItem::Text {
                    rect: Rect {
                        x: 4.0,
                        y: (i / 60) as f32 * 20.0,
                        w: 120.0,
                        h: 19.2,
                    },
                    text: "The quick brown fox jumps over the lazy dog".into(),
                    color: Color::rgb((i % 200) as u8, 40, 90),
                    font_size: 16.0,
                    weight: 400,
                    href: None,
                    underline: false,
                    line_through: false,
                })
                .collect(),
        );
        // Warm the atlas first so both arms measure blit, not rasterization.
        render(&page, size);
        let mut scalar_ms = f64::MAX;
        let mut chunked_ms = f64::MAX;
        for _ in 0..5 {
            for (mode, best) in [(1u8, &mut scalar_ms), (2u8, &mut chunked_ms)] {
                TEST_BLIT_MODE.store(mode, Ordering::Relaxed);
                let start = Instant::now();
                let frame = render(&page, size);
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                std::hint::black_box(frame);
                *best = (*best).min(ms);
            }
        }
        TEST_BLIT_MODE.store(0, Ordering::Relaxed);
        let delta_pct = (scalar_ms - chunked_ms) * 100.0 / scalar_ms;
        println!(
            "\n=== rung 1: text-heavy LARGE glyph blit (interleaved best of 5, 1200x800) ===\nscalar:  {scalar_ms:8.3} ms\nchunked: {chunked_ms:8.3} ms  ({delta_pct:+.1}%)"
        );
    }

    /// Rung 2 gate: `render_into` already clips to the damage band, so clean
    /// rows are neither re-rastered nor written. Poison them and prove they
    /// survive; then prove the damaged band reproduces the full frame hash.
    #[test]
    fn damage_path_skips_clean_bands_and_preserves_frame_hash() {
        let size = Size { w: 200.0, h: 120.0 };
        let (w, h) = (size.w as i32, size.h as i32);
        let baseline = list(vec![label(20.0, "alpha"), label(80.0, "beta")]);
        let mut changed = baseline.clone();
        if let DisplayItem::Text { text, .. } = &mut changed.items[1] {
            *text = "gamma".into();
        }
        let damage = diff_display_lists(&baseline, &changed, size);
        let Damage::Rects(rects) = &damage else {
            panic!("one-node edit must produce rect damage, got {damage:?}");
        };
        assert_eq!(rects.len(), 1, "one-node edit must produce one band");
        let band = IntRect {
            x: 0,
            y: rects[0].y,
            w,
            h: rects[0].h,
        };
        assert!(band.h > 0 && band.h < h, "damage must be a clipped band");
        let full = IntRect { x: 0, y: 0, w, h };
        let row_bytes = w as usize * 4;
        let y0 = band.y.max(0) as usize;
        let y1 = (band.y + band.h).min(h) as usize;

        // Poison every clean row, then repaint only the damaged band.
        let mut poisoned = Pixmap::new(w as u32, h as u32).unwrap();
        render_into(&mut poisoned, &changed, full);
        for (i, byte) in poisoned.data_mut().iter_mut().enumerate() {
            if !(y0..y1).contains(&(i / row_bytes)) {
                *byte = 0x7a;
            }
        }
        render_into(&mut poisoned, &changed, band);
        for (i, &byte) in poisoned.data().iter().enumerate() {
            if !(y0..y1).contains(&(i / row_bytes)) {
                assert_eq!(byte, 0x7a, "clean band row was re-rastered/written");
            }
        }

        // A/B invariance: same band path on an unpoisoned pixmap must equal the
        // full render, byte for byte and by frame hash.
        let mut band_pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        render_into(&mut band_pixmap, &changed, full);
        render_into(&mut band_pixmap, &changed, band);
        let band_frame = rgba_rows(&band_pixmap, 0, h);
        let full_frame = render(&changed, size);
        assert_eq!(frame_hash(&band_frame), frame_hash(&full_frame));
        assert_eq!(band_frame, full_frame);
    }

    /// Rung 3 gate: both faces are memoized in `OnceLock` (one parse per
    /// process, on first glyph use) and lazy parsing keeps frame bytes intact.
    #[test]
    fn fonts_are_lazy_memoized_and_frame_hash_stable() {
        let primary = font().expect("DejaVu font must parse");
        assert!(std::ptr::eq(
            primary,
            font().expect("DejaVu font must parse")
        ));
        let fallback = font_cjk().expect("Droid fallback font must parse");
        assert!(std::ptr::eq(
            fallback,
            font_cjk().expect("Droid fallback font must parse")
        ));
        let (list, size) = text_golden();
        let first = render(&list, size);
        let second = render(&list, size);
        assert_eq!(frame_hash(&first), frame_hash(&second));
        assert_eq!(first, second);
    }

    /// `cargo test -p hodo-render --release -- --ignored --nocapture perf_report_font_parse`
    /// One-time parse cost that used to sit at startup; `font()`/`font_cjk()`
    /// now pay it at first glyph use only. The first text frame additionally
    /// rasterizes its unique glyphs; every later frame reuses the atlas.
    #[test]
    #[ignore]
    fn perf_report_font_parse() {
        use std::time::Instant;
        let measure = |bytes: &'static [u8]| -> f64 {
            let mut best = f64::MAX;
            for _ in 0..3 {
                let start = Instant::now();
                let parsed = Font::from_bytes(bytes, FontSettings::default());
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                assert!(parsed.is_ok(), "vendored font must parse");
                std::hint::black_box(parsed.ok());
                best = best.min(ms);
            }
            best
        };
        let dejavu_ms = measure(include_bytes!("../../../assets/fonts/DejaVuSans.ttf"));
        let droid_ms = measure(include_bytes!(
            "../../../assets/fonts/DroidSansFallback.ttf"
        ));
        println!(
            "\n=== rung 3: one-time font parse deferred off startup (best of 3) ===\nDejaVu: {dejavu_ms:7.3} ms\nDroid:  {droid_ms:7.3} ms\nstartup deferred: {:.3} ms (sum)",
            dejavu_ms + droid_ms
        );

        let (list, size) = text_golden();
        {
            let mut atlas = glyph_atlas().write().unwrap_or_else(|e| e.into_inner());
            atlas.map.clear();
            atlas.bytes = 0;
        }
        let start = Instant::now();
        let first = render(&list, size);
        let first_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let second = render(&list, size);
        let second_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(frame_hash(&first), frame_hash(&second));
        println!(
            "first text frame (cold atlas): {first_ms:7.3} ms; second (warm): {second_ms:7.3} ms"
        );
    }

    #[test]
    fn repeated_full_repaint_into_same_pixmap_matches_fresh_render() {
        let size = Size { w: 100.0, h: 100.0 };
        let (w, h) = (size.w as i32, size.h as i32);
        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        let first = list(vec![solid(
            10.0,
            10.0,
            50.0,
            80.0,
            Color::rgba(255, 0, 0, 128),
        )]);
        render_into(&mut pixmap, &first, IntRect { x: 0, y: 0, w, h });
        let second = list(vec![solid(20.0, 20.0, 30.0, 30.0, Color::rgb(0, 0, 255))]);
        render_into(&mut pixmap, &second, IntRect { x: 0, y: 0, w, h });
        assert_eq!(rgba_rows(&pixmap, 0, h), render(&second, size));
    }

    #[test]
    fn damage_path_is_byte_identical_to_full_render() {
        let size = Size { w: 160.0, h: 120.0 };
        let (w, h) = (size.w as i32, size.h as i32);
        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        let mut current = list(vec![
            solid(4.0, 10.0, 60.0, 12.0, Color::rgb(30, 60, 90)),
            label(30.0, "alpha"),
            solid(4.0, 55.0, 60.0, 12.0, Color::rgb(90, 60, 30)),
            label(75.0, "beta"),
        ]);
        render_into(&mut pixmap, &current, IntRect { x: 0, y: 0, w, h });
        let mut buffer = rgba_rows(&pixmap, 0, h);

        // Deterministic LCG so the sequence is reproducible across platforms.
        let mut state = 0x1234_5678u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        };
        let mut scroll = 0i32;
        for step in 0..40 {
            let mut next_list = current.clone();
            match next() % 4 {
                0 => {
                    if let DisplayItem::Rect { color, .. } = &mut next_list.items[0] {
                        color.r = (next() % 200) as u8;
                    }
                }
                1 => {
                    if let DisplayItem::Text { text, .. } = &mut next_list.items[1] {
                        *text = format!("step-{step}");
                    }
                }
                2 => {
                    // Simulate a scroll: shift every item by an integer delta.
                    let target = (scroll + ((next() % 9) as i32) - 4).clamp(0, 40);
                    let delta = target - scroll;
                    scroll = target;
                    for item in &mut next_list.items {
                        if let Some(rect) = item_rect_mut(item) {
                            rect.y -= delta as f32;
                        }
                    }
                }
                _ => {
                    if let DisplayItem::Text { color, .. } = &mut next_list.items[3] {
                        color.g = (next() % 200) as u8;
                    }
                }
            }
            // The scroll simulation shifts the whole list, so compare against
            // the shifted list at the same integer offset.
            apply_damage(&mut pixmap, &mut buffer, &current, &next_list, size);
            let expected = render(&next_list, size);
            assert_eq!(
                buffer, expected,
                "damage path diverged from full render at step {step}"
            );
            current = next_list;
        }
    }

    /// Cached coverage bytes must be identical to a fresh fontdue rasterization:
    /// same metrics, same mask, same AA pixels.
    /// Cached coverage bytes must be identical to a fresh fontdue rasterization:
    /// same metrics, same mask, same AA pixels.
    #[test]
    fn glyph_atlas_matches_fresh_rasterization_byte_for_byte() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let size = Size { w: 220.0, h: 80.0 };
        let list = list(vec![label(30.0, "The quick brown fox 0123 Ajg")]);
        let cached = render(&list, size);
        {
            let mut atlas = glyph_atlas().write().unwrap();
            atlas.map.clear();
            atlas.bytes = 0;
        }
        let fresh = render(&list, size);
        assert_eq!(
            cached, fresh,
            "atlas coverage must equal fresh fontdue rasterization"
        );
    }

    /// Prove the fallback branch is live before relying on it for coverage.
    #[test]
    fn cjk_fallback_branch_is_live() {
        let dejavu = font().expect("DejaVu font must parse");
        let droid = font_cjk().expect("Droid fallback font must parse");
        assert!(
            !dejavu.has_glyph('日'),
            "DejaVu must lack 日 for the fallback branch to be exercised"
        );
        assert!(droid.has_glyph('日'), "Droid fallback must cover 日");
    }

    #[test]
    fn mixed_latin_cjk_text_rasterizes_both_faces() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // A font size no other test uses keeps these atlas keys unambiguous.
        let font_size: f32 = 37.0;
        let size_bits = font_size.to_bits();
        let list = list(vec![DisplayItem::Text {
            rect: Rect {
                x: 4.0,
                y: 4.0,
                w: 320.0,
                h: 1.2 * font_size,
            },
            text: "Hi 日本語".into(),
            color: Color::BLACK,
            font_size,
            weight: 400,
            href: None,
            underline: false,
            line_through: false,
        }]);
        let before = glyph_atlas_stats();
        let buf = render(&list, Size { w: 400.0, h: 80.0 });
        let after = glyph_atlas_stats();

        assert!(
            buf.len() == 400 * 80 * 4,
            "mixed line must render a full frame"
        );
        assert!(
            after.misses > before.misses,
            "rendering must rasterize new glyphs (atlas misses must grow)"
        );
        assert!(after.bytes > 0, "atlas occupancy must grow");

        let atlas = glyph_atlas().read().unwrap_or_else(|e| e.into_inner());
        let entries = |face: bool| {
            atlas
                .map
                .iter()
                .filter(move |(key, _)| key.size_bits == size_bits && key.cjk == face)
        };
        assert!(entries(false).count() >= 2, "Latin must use DejaVu entries");
        assert!(entries(true).count() >= 3, "日本語 must use Droid entries");
        for face in [false, true] {
            assert!(
                entries(face).any(|(_, bitmap)| bitmap.coverage.iter().any(|&v| v > 0)),
                "face {face} must rasterize non-zero coverage (no golden pixel diff)"
            );
        }
    }

    #[test]
    fn glyph_atlas_enforces_byte_cap_under_flood() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let primary = font().expect("DejaVu font must parse");
        let before = glyph_atlas_stats();
        // 128px glyphs are a few KiB each; this flood far overflows 1 MiB.
        // Each insert evicts back under the cap, so check counts, not `bytes > cap`.
        for cp in 0x4E00u32..0x4E00 + 8_000 {
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            glyph_cached(primary, ch, 128.0);
        }
        let stats = glyph_atlas_stats();
        let rasterized = stats.misses - before.misses;
        assert!(
            rasterized >= 1_000,
            "flood must rasterize many unique glyphs"
        );
        assert!(
            stats.bytes > 0 && stats.bytes <= stats.capacity,
            "LRU must enforce the cap: {} > {} B",
            stats.bytes,
            stats.capacity
        );
        assert!(
            (stats.entries as u64) < rasterized,
            "eviction must have dropped older entries ({} entries < {} rasterized)",
            stats.entries,
            rasterized
        );
    }

    /// The parallel full path must reproduce a single-threaded pass exactly.
    #[test]
    fn parallel_full_render_is_byte_identical_to_single_threaded() {
        let size = Size { w: 400.0, h: 300.0 };
        let (w, h) = (size.w as i32, size.h as i32);
        let items = (0..600)
            .map(|i| DisplayItem::Text {
                rect: Rect {
                    x: 2.0 + (i % 7) as f32 * 3.0,
                    // Past 300px some items are fully off-screen: the band
                    // path must skip them and still match the full pass.
                    y: (i / 7) as f32 * 5.0,
                    w: 120.0,
                    h: 19.2,
                },
                text: format!("row {i}: Aa g9"),
                color: Color::rgb((i % 200) as u8, 60, 90),
                font_size: 12.0 + (i % 3) as f32,
                weight: 400,
                href: None,
                underline: false,
                line_through: false,
            })
            .collect();
        let list = list(items);
        let parallel = render(&list, size);

        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        for item in &list.items {
            draw_item(&mut pixmap, item);
        }
        let single = rgba_rows(&pixmap, 0, h);
        assert_eq!(
            parallel, single,
            "parallel bands must match one single-threaded pass byte for byte"
        );
    }

    // ---- M5.x image pipeline: cache / downscale / warmup --------------------

    #[test]
    fn image_cache_hit_miss_and_rewrite_invalidation() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("cache.png");
        let c0 = [12, 34, 56, 255];
        let c1 = [200, 10, 20, 255];
        write_png(&path, 2, 2, &[c0; 4]);
        let path_str = path.to_string_lossy().into_owned();
        let list = list(vec![DisplayItem::Image {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 8.0,
                h: 8.0,
            },
            path: path_str.clone(),
        }]);
        let size = Size { w: 16.0, h: 16.0 };

        let base = image_cache_stats().decodes;
        let first = render(&list, size);
        assert_eq!(
            image_cache_stats().decodes,
            base + 1,
            "first render must decode exactly once"
        );
        let second = render(&list, size);
        assert_eq!(
            image_cache_stats().decodes,
            base + 1,
            "second render must be a cache hit"
        );
        assert_eq!(first, second);
        assert_eq!(pixel(&first, 3, 3, 16), c0);

        // Same path, new bytes with a DIFFERENT FILE SIZE -> new cache key even
        // when the filesystem mtime is too coarse to advance. This is the
        // signal the old (mtime-only) key missed.
        let old_len = image_file_sig(&path_str).unwrap().1;
        write_png(&path, 3, 3, &[c1; 9]);
        let new_len = image_file_sig(&path_str).unwrap().1;
        assert_ne!(old_len, new_len, "rewrite must change the file size");
        let third = render(&list, size);
        assert_eq!(
            image_cache_stats().decodes,
            base + 2,
            "size change must force a re-decode"
        );
        assert_eq!(pixel(&third, 3, 3, 16), c1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn file_url_and_plain_path_share_one_image_cache_entry() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("sig.png");
        write_png(&path, 2, 2, &[[12, 34, 56, 255]; 4]);
        let plain = path.to_string_lossy().replace('\\', "/");
        // Windows: `file:///C:/...` leaves a leading slash after the prefix.
        // Unix: the absolute path already starts with '/'.
        let file_url = if plain.starts_with('/') {
            format!("file://{plain}")
        } else {
            format!("file:///{plain}")
        };
        assert_eq!(image_local_path(&file_url), plain);
        assert_eq!(image_file_sig(&file_url), image_file_sig(&plain));
        assert!(image_file_sig(&file_url).is_some());

        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: 8.0,
            h: 8.0,
        };
        assert!(
            image_key(&file_url, &rect) == image_key(&plain, &rect),
            "the file:// URL and plain path must produce one cache key"
        );

        // Store under one spelling, hit with the other.
        image_cache_put(
            image_key(&plain, &rect),
            Arc::new(ScaledImage {
                w: 2,
                h: 2,
                rgba: vec![0; 16],
                last_used: AtomicU64::new(0),
            }),
        );
        assert!(image_cache_get(&image_key(&file_url, &rect)).is_some());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn non_file_image_sources_bypass_the_decode_cache() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: 8.0,
            h: 8.0,
        };
        let remote = image_key("https://example.test/a.png", &rect);
        let data = image_key("data:image/png;base64,AAAA", &rect);
        assert!(
            remote.file_sig.is_none(),
            "http URLs have no content signal"
        );
        assert!(data.file_sig.is_none(), "data URLs have no content signal");

        let dir = temp_dir();
        let path = dir.join("local.png");
        write_png(&path, 2, 2, &[[9, 8, 7, 255]; 4]);
        let local = image_key(&path.to_string_lossy(), &rect);
        let (mtime, len) = local.file_sig.expect("local file has a signature");
        assert!(mtime > 0, "local file carries an mtime");
        assert_eq!(len, std::fs::metadata(&path).unwrap().len(), "and a size");
        // A file:// URL resolves to the same local bytes and is cacheable.
        let file_url = image_key(&format!("file://{}", path.to_string_lossy()), &rect);
        assert_eq!(
            file_url.file_sig, local.file_sig,
            "file:// keys like a file"
        );

        let fake = Arc::new(ScaledImage {
            w: 1,
            h: 1,
            rgba: vec![1, 2, 3, 4],
            last_used: AtomicU64::new(1),
        });
        // Stores under URL keys are refused; lookups always miss.
        image_cache_put(remote.clone(), Arc::clone(&fake));
        assert!(
            image_cache_get(&remote).is_none(),
            "http source must never be cached"
        );
        image_cache_put(data.clone(), Arc::clone(&fake));
        assert!(
            image_cache_get(&data).is_none(),
            "data source must never be cached"
        );
        // The local key does cache.
        image_cache_put(local.clone(), Arc::clone(&fake));
        assert!(image_cache_get(&local).is_some(), "local file must cache");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn decode_gate_caps_concurrent_scratch_and_reports_peak() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let mut sources = Vec::new();
        for i in 0..4 {
            let path = dir.join(format!("gate{i}.png"));
            let pixels: Vec<[u8; 4]> = vec![[10 + i, 20, 30, 255]; 1024 * 1024];
            write_png(&path, 1024, 1024, &pixels);
            let rect = Rect {
                x: 0.0,
                y: 0.0,
                w: 16.0,
                h: 16.0,
            };
            let path_str = path.to_string_lossy().into_owned();
            let key = image_key(&path_str, &rect);
            sources.push((path_str, key));
        }

        // Barrier releases all four decodes at once; the gate must admit only
        // MAX_CONCURRENT_DECODES of them and queue the rest.
        let barrier = std::sync::Barrier::new(4);
        std::thread::scope(|scope| {
            for (path, key) in &sources {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    let _ = load_scaled_image(path, key);
                });
            }
        });

        let stats = image_cache_stats();
        assert_eq!(
            stats.peak_in_flight, MAX_CONCURRENT_DECODES,
            "in-flight decodes must cap at {}",
            MAX_CONCURRENT_DECODES
        );
        assert!(
            decode_scratch_peak() >= 1024 * 1024 * 4,
            "full-res rgba scratch must be reported"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn hot_latin_glyph_path_skips_cmap_lookup() {
        let _guard = GLYPH_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let font = font().expect("DejaVu font must parse");
        let font_size: f32 = 43.0; // a size no other test uses
        let ch = 'Q';
        {
            let mut atlas = glyph_atlas().write().unwrap_or_else(|e| e.into_inner());
            for cjk in [false, true] {
                atlas.map.remove(&GlyphKey {
                    size_bits: font_size.to_bits(),
                    ch,
                    cjk,
                });
            }
        }
        TEST_HAS_GLYPH_PROBES.with(|c| c.set(0));
        let _ = glyph_cached(font, ch, font_size);
        assert_eq!(
            TEST_HAS_GLYPH_PROBES.with(|c| c.get()),
            1,
            "a cold Latin glyph pays exactly one cmap lookup"
        );

        TEST_HAS_GLYPH_PROBES.with(|c| c.set(0));
        for _ in 0..1000 {
            let _ = glyph_cached(font, ch, font_size);
        }
        assert_eq!(
            TEST_HAS_GLYPH_PROBES.with(|c| c.get()),
            0,
            "1000 hot Latin lookups must do zero cmap lookups"
        );

        // The global counter must also see probes made on a worker thread;
        // a thread-local mirror would silently miss band workers.
        let before_worker = glyph_atlas_stats().has_glyph_probes;
        let worker = std::thread::spawn(move || {
            let _ = glyph_cached(font, 'Z', 43.5);
        });
        worker.join().unwrap();
        assert!(
            glyph_atlas_stats().has_glyph_probes > before_worker,
            "a cmap lookup on a worker thread must be counted globally"
        );
    }

    #[test]
    fn image_downscale_samples_target_pixels_and_clamps_storage() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("columns.png");
        // 4x4 source, four distinct column colors.
        let columns = [
            [10, 0, 0, 255],
            [20, 0, 0, 255],
            [30, 0, 0, 255],
            [40, 0, 0, 255],
        ];
        let pixels: Vec<[u8; 4]> = (0..16).map(|i| columns[i % 4]).collect();
        write_png(&path, 4, 4, &pixels);
        let flat: Vec<u8> = pixels.iter().flatten().copied().collect();

        // Rect 2x2 < source: stored at 2x2, sampled with blit_scaled's integer
        // map (sx = ox * 4 / 2), so column 0 -> 10 and column 2 -> 30.
        let (w, h, scaled) = scale_to_target(4, 4, &flat, 2, 2);
        assert_eq!((w, h), (2, 2));
        assert_eq!(scaled[0..4], [10, 0, 0, 255]);
        assert_eq!(scaled[4..8], [30, 0, 0, 255]);

        // Rect larger than source: storage is clamped to intrinsic size, never
        // upscaled (blit still stretches to the rect, byte-identically).
        let (uw, uh, _) = scale_to_target(2, 2, &[c4(); 4].concat(), 16, 16);
        assert_eq!((uw, uh), (2, 2), "no upscaling beyond source");

        // Rect with zero extent still stores a 1x1 buffer.
        let (mw, mh, _) = scale_to_target(2, 2, &[c4(); 4].concat(), 0, 0);
        assert_eq!((mw, mh), (1, 1), "1px minimum");

        // End-to-end: a solid colored source downscaled into a smaller rect
        // keeps the exact color at the target pixel center.
        let solid = [7, 99, 23, 255];
        write_png(&path, 8, 8, &[solid; 64]);
        let small = list(vec![DisplayItem::Image {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 4.0,
                h: 4.0,
            },
            path: path.to_string_lossy().into_owned(),
        }]);
        let buf = render(&small, Size { w: 8.0, h: 8.0 });
        assert_eq!(pixel(&buf, 2, 2, 8), solid, "downscaled center color");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn c4() -> [u8; 4] {
        [1, 2, 3, 255]
    }

    #[test]
    fn image_cache_evicts_by_entries_and_by_bytes() {
        // Entry cap: 74 inserts leave exactly 64 entries.
        let mut cache = ImageCache::new();
        for i in 0..IMAGE_CACHE_CAP_ENTRIES + 10 {
            cache.insert(
                ImageKey {
                    path: format!("img{i}.png"),
                    file_sig: None,
                    target_w: 1,
                    target_h: 1,
                },
                Arc::new(ScaledImage {
                    w: 1,
                    h: 1,
                    rgba: vec![i as u8; 4],
                    last_used: AtomicU64::new(i as u64),
                }),
            );
        }
        assert_eq!(cache.map.len(), IMAGE_CACHE_CAP_ENTRIES);
        assert!(cache.bytes <= IMAGE_CACHE_CAP_BYTES);

        // Byte cap: 20 x 1 MiB entries -> at most 16 survive (16 MiB cap).
        let mut cache = ImageCache::new();
        let entry_bytes = 1024 * 256 * 4;
        for i in 0..20 {
            cache.insert(
                ImageKey {
                    path: format!("big{i}.png"),
                    file_sig: None,
                    target_w: 1024,
                    target_h: 256,
                },
                Arc::new(ScaledImage {
                    w: 1024,
                    h: 256,
                    rgba: vec![0; entry_bytes],
                    last_used: AtomicU64::new(i as u64),
                }),
            );
        }
        assert!(cache.map.len() <= 16, "byte budget evicts LRU entries");
        assert!(cache.bytes <= IMAGE_CACHE_CAP_BYTES);
        assert!(cache.map.len() > 1, "an oversized image is still cached");
    }

    #[test]
    fn warm_images_prepopulates_cache_and_is_idempotent() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("warm.png");
        write_png(&path, 16, 16, &[[5, 6, 7, 255]; 256]);
        let path_str = path.to_string_lossy().into_owned();
        let items: Vec<DisplayItem> = (0..3)
            .map(|i| DisplayItem::Image {
                rect: Rect {
                    x: (i * 40) as f32,
                    y: 10.0,
                    w: 20.0 + i as f32 * 4.0,
                    h: 20.0 + i as f32 * 4.0,
                },
                path: path_str.clone(),
            })
            .collect();

        image_cache()
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        let before = image_cache_stats().decodes;
        warm_images(&list(items.clone()), 200, 100);
        assert_eq!(
            image_cache_stats().decodes,
            before + 3,
            "3 distinct image rects decoded up front"
        );
        let after = image_cache_stats().decodes;
        warm_images(&list(items), 200, 100);
        assert_eq!(
            image_cache_stats().decodes,
            after,
            "a second warm is a no-op"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parallel_warmup_matches_sequential_blits_byte_for_byte() {
        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();
        let path = dir.join("parallel.png");
        let color = [7, 99, 23, 255];
        write_png(&path, 16, 16, &[color; 256]);
        let path_str = path.to_string_lossy().into_owned();

        let mut items: Vec<DisplayItem> =
            (0..400).map(|i| label(i as f32 * 0.5, "Aa g9")).collect();
        for i in 0..4 {
            // Distinct rect sizes => distinct cache keys => 4 cold decodes.
            let side = 20.0 + i as f32 * 4.0;
            items.push(DisplayItem::Image {
                rect: Rect {
                    x: 5.0 + i as f32 * 40.0,
                    y: 10.0 + (i % 3) as f32 * 60.0,
                    w: side,
                    h: side,
                },
                path: path_str.clone(),
            });
        }
        let list = list(items);
        let size = Size { w: 200.0, h: 240.0 };
        let (w, h) = (size.w as i32, size.h as i32);

        // >= 256 items -> draw_full parallel band path (with image warmup).
        image_cache()
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        let parallel = render(&list, size);
        let parallel_decodes = image_cache_stats().decodes;

        // Reference: plain single-threaded blits in item order.
        image_cache()
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        for item in &list.items {
            draw_item(&mut pixmap, item);
        }
        let sequential = rgba_rows(&pixmap, 0, h);
        assert_eq!(
            parallel, sequential,
            "warmed parallel bands must match sequential blits byte for byte"
        );
        assert_eq!(
            image_cache_stats().decodes - parallel_decodes,
            4,
            "both paths decode each distinct image rect exactly once"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `cargo test -p hodo-render --release -- --ignored --nocapture image_perf_report`
    #[test]
    #[ignore]
    fn image_perf_report() {
        use std::time::Instant;

        let _guard = IMAGE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir();

        // Demo: 32x32 image, rect equals intrinsic size -> byte-identical.
        let demo_path = dir.join("demo32.png");
        write_png(&demo_path, 32, 32, &[[100, 150, 200, 255]; 1024]);
        let demo = list(vec![DisplayItem::Image {
            rect: Rect {
                x: 10.0,
                y: 10.0,
                w: 32.0,
                h: 32.0,
            },
            path: demo_path.to_string_lossy().into_owned(),
        }]);

        // Big doc: 25k text items + 16 distinct images (256x256 sources drawn
        // at 64-79px -> 16-25 KiB each in cache).
        let big_path = dir.join("big.png");
        write_png(&big_path, 256, 256, &[[30, 60, 90, 255]; 65536]);
        let mut big_items: Vec<DisplayItem> = (0..25_000)
            .map(|i| label((i / 60) as f32, "text"))
            .collect();
        for i in 0..16 {
            let side = 64.0 + i as f32;
            big_items.push(DisplayItem::Image {
                rect: Rect {
                    x: (i * 70) as f32 % 1100.0,
                    y: (i * 90) as f32 % 700.0,
                    w: side,
                    h: side,
                },
                path: big_path.to_string_lossy().into_owned(),
            });
        }
        let bigdoc = list(big_items);
        let size = Size {
            w: 1200.0,
            h: 800.0,
        };

        println!("\n=== image decode-cache perf (1200x800) ===");
        for (name, page) in [("demo", &demo), ("bigdoc", &bigdoc)] {
            image_cache()
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .clear();
            let before = image_cache_stats();
            let start = Instant::now();
            let _ = render(page, size);
            let first_ms = start.elapsed().as_secs_f64() * 1000.0;
            let cold = image_cache_stats();

            let start = Instant::now();
            for _ in 0..9 {
                let _ = render(page, size);
            }
            let warm_ms = start.elapsed().as_secs_f64() * 1000.0;
            let warm = image_cache_stats();
            println!(
                "{name:7} first={first_ms:8.3} ms ({} decodes), 9 warm renders={warm_ms:8.3} ms ({} extra decodes), cache={}/{} entries, {}/{} bytes",
                cold.decodes - before.decodes,
                warm.decodes - cold.decodes,
                warm.entries,
                warm.capacity_entries,
                warm.bytes,
                warm.capacity_bytes,
            );
        }
        println!("peak extra RAM = cache bytes above (bounded by 16 MiB cap)");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

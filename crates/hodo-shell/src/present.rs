//! Softbuffer draw path with M5.1 damage rects and M5.2 scroll blits.
//!
//! One persistent premultiplied pixmap and one ARGB8888 frame live in
//! [`RasterState`]. A redraw classifies the new composed display list as a full
//! repaint, a set of damage rects, or a pure scroll; only the affected rows are
//! re-rasterized, and only those rows are handed to softbuffer's damage-aware
//! present. Bars are never blitted.

use std::num::NonZeroU32;
use std::sync::OnceLock;

use hodo_layout::{DisplayItem, DisplayList};
use hodo_render::{diff_display_lists, render_into, rgba_rows, Damage, IntRect, DAMAGE_PAD};
use hodo_types::{Color, Rect, Size};
use hodo_ui::tokens::{self, Theme};
use softbuffer::Rect as SoftRect;
use tiny_skia::Pixmap;

use crate::app::{composite_over_white, BrowserApp};
use crate::popup::PopupState;
use crate::ui::{
    chrome_item_count, clamp_scroll_offset, compose_display_list_with_theme, current_theme,
    tab_counter, BOTTOM_BAR_H, TOP_BAR_H,
};

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn force_full_flag() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| env_flag("HODO_FORCE_FULL"))
}

fn debug_damage_flag() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| env_flag("HODO_DEBUG_DAMAGE"))
}

// BEGIN CHROME DRAW (no raw hex: every color comes from ThemeAccess)
//
// The page keeps hodo-render's DejaVu font. Chrome text uses the vendored
// Roboto statics, loaded once; built-in icons come from hodo-icons' LRU.
// Hover/press state layers are intentionally absent: `CursorMoved` only stores
// the cursor in app.rs and requests no redraw, so there is no hover plumbing
// to hang them on. The URL field does have real focus plumbing (`url_focused`),
// so it gets the M3 primary focus ring.
mod chrome {
    use super::*;
    use crate::ui::{
        render_color, toolbar_icon_rect, url_pill_rect, CHROME_ICONS, CHROME_ICON_SIZE,
    };
    use fontdue::{Font, FontSettings};
    use hodo_ui::{ColorRole, ThemeAccess};
    use std::sync::OnceLock;
    use tiny_skia::{FillRule, Paint, PathBuilder, PixmapPaint, Rect as SkRect, Transform};

    /// Draw-time chrome inputs that are not display-list items.
    #[derive(Clone, Copy)]
    pub(super) struct Style {
        pub(super) theme: Theme,
        pub(super) url_focused: bool,
    }

    static REGULAR: OnceLock<Option<Font>> = OnceLock::new();
    static MEDIUM: OnceLock<Option<Font>> = OnceLock::new();
    static BOLD: OnceLock<Option<Font>> = OnceLock::new();

    fn roboto(weight: u32) -> Option<&'static Font> {
        let font = match weight {
            500 => MEDIUM.get_or_init(|| {
                Font::from_bytes(
                    include_bytes!("../../../assets/fonts/roboto/Roboto-Medium.ttf").as_slice(),
                    FontSettings::default(),
                )
                .ok()
            }),
            700 => BOLD.get_or_init(|| {
                Font::from_bytes(
                    include_bytes!("../../../assets/fonts/roboto/Roboto-Bold.ttf").as_slice(),
                    FontSettings::default(),
                )
                .ok()
            }),
            _ => REGULAR.get_or_init(|| {
                Font::from_bytes(
                    include_bytes!("../../../assets/fonts/roboto/Roboto-Regular.ttf").as_slice(),
                    FontSettings::default(),
                )
                .ok()
            }),
        };
        font.as_ref()
    }

    fn fill(pixmap: &mut Pixmap, rect: Rect, color: Color) {
        let Some(sk) = SkRect::from_xywh(rect.x, rect.y, rect.w, rect.h) else {
            return;
        };
        let path = PathBuilder::from_rect(sk);
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

    fn fill_pill(pixmap: &mut Pixmap, rect: Rect, color: Color) {
        let Some(sk) = SkRect::from_xywh(rect.x, rect.y, rect.w, rect.h) else {
            return;
        };
        let path = hodo_ui::pill(sk);
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

    fn inset(rect: Rect, d: f32) -> Rect {
        Rect {
            x: rect.x + d,
            y: rect.y + d,
            w: (rect.w - 2.0 * d).max(1.0),
            h: (rect.h - 2.0 * d).max(1.0),
        }
    }

    /// Source-over blend of one coverage pixel into premultiplied pixmap data,
    /// clipped to the URL pill so a long URL cannot spill onto the buttons.
    #[allow(clippy::too_many_arguments)]
    fn blend(data: &mut [u8], w: i32, h: i32, clip: Rect, x: i32, y: i32, cov: u8, color: Color) {
        if cov == 0
            || x < clip.x as i32
            || y < clip.y as i32
            || x >= (clip.x + clip.w) as i32
            || y >= (clip.y + clip.h) as i32
            || x < 0
            || y < 0
            || x >= w
            || y >= h
        {
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

    fn draw_text(
        pixmap: &mut Pixmap,
        text: &str,
        rect: Rect,
        font_size: f32,
        weight: u16,
        color: Color,
        clip: Rect,
    ) {
        if !font_size.is_finite() || font_size <= 0.0 {
            return;
        }
        let Some(font) = roboto(weight as u32) else {
            return;
        };
        let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
        let baseline = rect.y + 0.8 * (rect.h.max(font_size * 1.2) / 1.2);
        let data = pixmap.data_mut();
        let mut pen_x = rect.x;
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }
            let (metrics, coverage) = font.rasterize(ch, font_size);
            let ox = pen_x as i32 + metrics.xmin;
            let oy = (baseline - metrics.height as f32 - metrics.ymin as f32).floor() as i32;
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    blend(
                        data,
                        w,
                        h,
                        clip,
                        ox + col as i32,
                        oy + row as i32,
                        coverage[row * metrics.width + col],
                        color,
                    );
                }
            }
            pen_x += if metrics.advance_width > 0.0 {
                metrics.advance_width
            } else {
                0.6 * font_size
            };
        }
    }

    /// Paint both bars, the built-in symbols, the URL pill and all chrome text
    /// over an already-rendered page. `items` is the display-list chrome suffix.
    pub(super) fn paint(pixmap: &mut Pixmap, items: &[DisplayItem], viewport: Size, style: Style) {
        let theme = &style.theme;
        let bar = render_color(theme.color(ColorRole::SurfaceContainer));
        fill(
            pixmap,
            Rect {
                x: 0.0,
                y: 0.0,
                w: viewport.w,
                h: TOP_BAR_H,
            },
            bar,
        );
        fill(
            pixmap,
            Rect {
                x: 0.0,
                y: viewport.h - BOTTOM_BAR_H,
                w: viewport.w,
                h: BOTTOM_BAR_H,
            },
            bar,
        );

        let icon_color = render_color(theme.color(ColorRole::OnSurface));
        for (index, name) in CHROME_ICONS.iter().enumerate() {
            let rect = toolbar_icon_rect(viewport, index);
            if let Some(pm) = hodo_icons::rasterize(name, CHROME_ICON_SIZE, icon_color) {
                pixmap.draw_pixmap(
                    rect.x as i32,
                    rect.y as i32,
                    pm.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }

        let pill = url_pill_rect(viewport);
        let pill_fill = render_color(theme.color(ColorRole::SurfaceContainerHighest));
        if style.url_focused {
            fill_pill(pixmap, pill, render_color(theme.color(ColorRole::Primary)));
            fill_pill(pixmap, inset(pill, 2.0), pill_fill);
        } else {
            fill_pill(pixmap, pill, pill_fill);
        }

        let full = Rect {
            x: 0.0,
            y: 0.0,
            w: viewport.w,
            h: viewport.h,
        };
        let mut iter = items.iter();
        let _top_bar = iter.next();
        if let Some(DisplayItem::Text {
            rect,
            text,
            color,
            font_size,
            weight,
            ..
        }) = iter.next()
        {
            draw_text(pixmap, text, *rect, *font_size, *weight, *color, pill);
        }
        if let Some(DisplayItem::Text {
            rect,
            text,
            color,
            font_size,
            weight,
            ..
        }) = iter.next()
        {
            draw_text(pixmap, text, *rect, *font_size, *weight, *color, full);
        }
        let _bottom_bar = iter.next();
        for item in iter {
            if let DisplayItem::Text {
                rect,
                text,
                color,
                font_size,
                weight,
                ..
            } = item
            {
                // Extension actions keep their own bitmaps; only their titles
                // are chrome-owned text.
                draw_text(pixmap, text, *rect, *font_size, *weight, *color, full);
            }
        }
    }
}
// END CHROME DRAW

/// Page + fixed items only: hodo-render keeps drawing the page, while the
/// chrome suffix above is painted by `chrome::paint`.
fn page_only(composed: &DisplayList, render_len: usize) -> DisplayList {
    DisplayList {
        items: composed.items[..render_len].to_vec(),
        node_map: composed.node_map.clone(),
        fixed_items: composed.fixed_items.clone(),
        fixed_node_map: composed.fixed_node_map.clone(),
    }
}

/// Identity of one painted frame. Metadata changes force a full repaint;
/// `page_hash`/`chrome_hash` changes select the damage path; a lone
/// `paint_offset` change selects the scroll-blit path.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FrameKey {
    viewport: (i32, i32),
    page_url: String,
    label: String,
    ext_titles: Vec<String>,
    page_hash: u64,
    chrome_hash: u64,
    paint_offset: i32,
    /// Index where the chrome suffix starts; the page rasterizer stops here.
    render_len: usize,
    /// Draw-time theme flag: a settings toggle changes this without a restart.
    theme_dark: bool,
    /// Focus ring is chrome-only, so a focus change must force a repaint.
    url_focused: bool,
}

/// Result of classifying a redraw.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Update {
    Full,
    Damage(Vec<IntRect>),
    Scroll { dy: i32, strip: IntRect },
}

fn items_hash(items: &[DisplayItem]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for item in items {
        hash = (hash ^ hodo_render::item_hash(item)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Integer bounds of a display item, using the same rounding as hodo-render's
/// damage pass so bands agree at the pixel level.
fn item_bounds(item: &DisplayItem) -> Option<IntRect> {
    let rect: &Rect = crate::ui::item_rect(item)?;
    let x = rect.x.floor() as i32;
    let y = rect.y.floor() as i32;
    Some(IntRect {
        x,
        y,
        w: ((rect.x + rect.w).ceil() as i32 - x).max(0),
        h: ((rect.y + rect.h).ceil() as i32 - y).max(0),
    })
}

/// Row bands that must be re-composed after a scroll blit. A fixed rect keeps
/// its viewport position, but the blit also drags its *previous* pixels to
/// `y - dy`, so both the new box and its shifted ghost need fresh page content
/// (and the box itself) painted back.
fn fixed_bands(
    composed: &DisplayList,
    top: i32,
    bottom: i32,
    w: i32,
    h: i32,
    dy: i32,
) -> Vec<(i32, i32)> {
    let mut bands = Vec::new();
    for item in &composed.fixed_items {
        let Some(bounds) = item_bounds(item).and_then(|rect| rect.clamp(w, h)) else {
            continue;
        };
        let shifted = IntRect {
            y: bounds.y - dy,
            ..bounds
        };
        let covered = bounds.union(shifted);
        let y0 = covered.y.max(top);
        let y1 = covered.bottom().min(bottom);
        if y1 > y0 {
            bands.push((y0, y1));
        }
    }
    merge_bands(bands)
}

/// A changed page item under a fixed box must also repaint the whole box, at
/// the same viewport position: otherwise the fixed pixels covering it could be
/// left stale. This keeps the damage path byte-identical to a full render.
fn expand_damage_for_fixed(
    composed: &DisplayList,
    mut rects: Vec<IntRect>,
    w: i32,
    h: i32,
) -> Vec<IntRect> {
    let fixed: Vec<IntRect> = composed
        .fixed_items
        .iter()
        .filter_map(item_bounds)
        .filter_map(|rect| rect.clamp(w, h))
        .collect();
    for box_ in fixed {
        if rects.iter().any(|rect| rect.intersects(box_)) {
            rects.push(box_.padded(DAMAGE_PAD).clamp(w, h).unwrap_or(box_));
        }
    }
    rects
}

/// Persistent raster state. Never stored in a `Tab`: inactive tabs stay
/// pixel-free.
pub(crate) struct RasterState {
    pixmap: Option<Pixmap>,
    pub(crate) frame: Vec<u32>,
    pub(crate) w: usize,
    pub(crate) h: usize,
    composed: Option<DisplayList>,
    key: Option<FrameKey>,
    /// Integer scroll offset actually used to compose the current frame.
    pub(crate) paint_offset: i32,
    render_len: usize,
    chrome_style: chrome::Style,
}

impl RasterState {
    pub(crate) fn new() -> Self {
        Self {
            pixmap: None,
            frame: Vec::new(),
            w: 0,
            h: 0,
            composed: None,
            key: None,
            paint_offset: 0,
            render_len: 0,
            chrome_style: chrome::Style {
                theme: tokens::LIGHT,
                url_focused: false,
            },
        }
    }

    fn ensure_pixmap(&mut self, size: Size) {
        let w = size.w.round().max(1.0) as u32;
        let h = size.h.round().max(1.0) as u32;
        if self
            .pixmap
            .as_ref()
            .is_some_and(|p| p.width() == w && p.height() == h)
        {
            return;
        }
        self.pixmap = Pixmap::new(w, h);
        self.w = w as usize;
        self.h = h as usize;
        self.frame.clear();
        self.frame.resize(self.w * self.h, 0xFF00_0000);
    }

    /// Classify and apply one frame. Returns the row bands that were rewritten.
    pub(crate) fn update(
        &mut self,
        size: Size,
        composed: &DisplayList,
        key: FrameKey,
        force_full: bool,
    ) -> (Update, Vec<(i32, i32)>) {
        self.ensure_pixmap(size);
        if self.pixmap.is_none() || self.w == 0 || self.h == 0 {
            return (Update::Full, Vec::new());
        }
        let force = force_full || force_full_flag();
        let update = if let (Some(previous), Some(old_list)) =
            (self.key.as_ref(), self.composed.as_ref())
        {
            let metadata_changed = previous.viewport != key.viewport
                || previous.page_url != key.page_url
                || previous.label != key.label
                || previous.ext_titles != key.ext_titles
                || previous.theme_dark != key.theme_dark
                || previous.url_focused != key.url_focused;
            if force || metadata_changed {
                Update::Full
            } else if previous.page_hash != key.page_hash || previous.chrome_hash != key.chrome_hash
            {
                let (vw, vh) = (size.w.round() as i32, size.h.round() as i32);
                match diff_display_lists(old_list, composed, size) {
                    Damage::Full => Update::Full,
                    Damage::Rects(rects) => {
                        Update::Damage(expand_damage_for_fixed(composed, rects, vw, vh))
                    }
                    Damage::None => Update::Damage(Vec::new()),
                }
            } else if previous.paint_offset != key.paint_offset {
                // Scroll-only: no style, layout or display-list rebuild needed.
                let dy = key.paint_offset - previous.paint_offset;
                let top = TOP_BAR_H as i32;
                let bottom = self.h as i32 - BOTTOM_BAR_H as i32;
                if bottom <= top || dy.abs() >= bottom - top {
                    Update::Full
                } else {
                    let strip = if dy > 0 {
                        IntRect {
                            x: 0,
                            y: bottom - dy,
                            w: self.w as i32,
                            h: dy,
                        }
                    } else {
                        IntRect {
                            x: 0,
                            y: top,
                            w: self.w as i32,
                            h: -dy,
                        }
                    };
                    Update::Scroll { dy, strip }
                }
            } else {
                Update::Damage(Vec::new())
            }
        } else {
            Update::Full
        };
        self.render_len = key.render_len;
        self.chrome_style = chrome::Style {
            theme: if key.theme_dark {
                tokens::DARK
            } else {
                tokens::LIGHT
            },
            url_focused: key.url_focused,
        };
        let bands = self.apply(&update, composed);
        self.composed = Some(composed.clone());
        self.paint_offset = key.paint_offset;
        self.key = Some(key);
        (update, bands)
    }

    fn apply(&mut self, update: &Update, composed: &DisplayList) -> Vec<(i32, i32)> {
        let render_len = self.render_len;
        let style = self.chrome_style;
        let Self {
            pixmap,
            frame,
            w,
            h,
            ..
        } = self;
        let Some(pixmap) = pixmap.as_mut() else {
            return Vec::new();
        };
        let (wi, hi) = (*w as i32, *h as i32);
        let size = Size {
            w: *w as f32,
            h: *h as f32,
        };
        let page = page_only(composed, render_len);
        let chrome_items = &composed.items[render_len..];
        match update {
            Update::Full => {
                render_into(
                    pixmap,
                    &page,
                    IntRect {
                        x: 0,
                        y: 0,
                        w: wi,
                        h: hi,
                    },
                );
                chrome::paint(pixmap, chrome_items, size, style);
                pack_rows(pixmap, frame, 0, hi);
                vec![(0, hi)]
            }
            Update::Damage(rects) => {
                let mut bands = Vec::new();
                for rect in rects {
                    render_into(
                        pixmap,
                        &page,
                        IntRect {
                            x: 0,
                            y: rect.y,
                            w: wi,
                            h: rect.h,
                        },
                    );
                    bands.push((rect.y, rect.y + rect.h));
                }
                // A theme/URL/focus change damages chrome rows; repaint the
                // whole chrome once before packing any of the affected rows.
                if chrome_rows_touched(&bands, hi) {
                    chrome::paint(pixmap, chrome_items, size, style);
                }
                for &(y0, y1) in &bands {
                    pack_rows(pixmap, frame, y0, y1);
                }
                merge_bands(bands)
            }
            Update::Scroll { dy, strip } => {
                let top = TOP_BAR_H as i32;
                let bottom = hi - BOTTOM_BAR_H as i32;
                if bottom > top {
                    blit_premul(pixmap.data_mut(), top, bottom, *dy, *w);
                    blit_argb(frame, top, bottom, *dy, *w);
                    render_into(pixmap, &page, *strip);
                    pack_rows(pixmap, frame, strip.y, strip.y + strip.h);
                    // The row blit shifts fixed rows together with the page.
                    // Re-compose each fixed band (new position plus the ghost
                    // left at `y - dy`) from the new list: the fixed box goes
                    // back to the same viewport pixel and the page beneath it
                    // is redrawn at the new scroll position.
                    for (y0, y1) in fixed_bands(composed, top, bottom, wi, hi, *dy) {
                        render_into(
                            pixmap,
                            &page,
                            IntRect {
                                x: 0,
                                y: y0,
                                w: wi,
                                h: y1 - y0,
                            },
                        );
                        pack_rows(pixmap, frame, y0, y1);
                    }
                }
                // The blit rewrote every page row, so present the whole band.
                vec![(top, bottom)]
            }
        }
    }

    /// Overlay the open popup on the single cached frame for one present and
    /// report the row band that must reach the swapchain. The panel is never
    /// baked into the page cache: [`Self::restore_rows`] puts clean page pixels
    /// back immediately, so a later scroll blit cannot smear popup pixels.
    fn overlay_popup(&mut self, popup: &PopupState, viewport: Size) -> Vec<(i32, i32)> {
        let y0 = popup.rect.y.round().max(0.0) as i32;
        let y1 = (popup.rect.y + popup.rect.h).round().min(viewport.h) as i32;
        if y1 <= y0 || y0 >= self.h as i32 {
            return Vec::new();
        }
        let band = (y0, y1.min(self.h as i32));
        popup.composite(&mut self.frame, viewport);
        vec![band]
    }

    /// Repack clean page rows from the pixmap after a temporary popup overlay.
    fn restore_rows(&mut self, y0: i32, y1: i32) {
        let Self { pixmap, frame, .. } = self;
        if let Some(pixmap) = pixmap {
            pack_rows(pixmap, frame, y0, y1);
        }
    }
}

/// Whether any rewritten row lies in a chrome bar. Scroll blits never touch
/// these rows, so they need no chrome overlay.
fn chrome_rows_touched(bands: &[(i32, i32)], h: i32) -> bool {
    let bottom_y = h - BOTTOM_BAR_H as i32;
    bands
        .iter()
        .any(|&(y0, y1)| y0 < TOP_BAR_H as i32 || y1 > bottom_y)
}

fn merge_bands(mut bands: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    bands.sort_unstable();
    let mut merged: Vec<(i32, i32)> = Vec::new();
    for (y0, y1) in bands {
        if let Some(last) = merged.last_mut() {
            if y0 <= last.1 {
                last.1 = last.1.max(y1);
                continue;
            }
        }
        merged.push((y0, y1));
    }
    merged
}

/// Shift a premultiplied RGBA row band inside a pixmap. `copy_within` is a
/// memmove, so overlapping source/destination ranges are safe.
fn blit_premul(data: &mut [u8], top: i32, bottom: i32, dy: i32, width: usize) {
    let row = width * 4;
    if dy > 0 {
        let source = (top + dy) as usize * row;
        let len = (bottom - top - dy) as usize * row;
        data.copy_within(source..source + len, top as usize * row);
    } else if dy < 0 {
        let source = top as usize * row;
        let len = (bottom - top + dy) as usize * row;
        data.copy_within(source..source + len, (top - dy) as usize * row);
    }
}

/// Shift an ARGB8888 row band inside the frame.
fn blit_argb(frame: &mut [u32], top: i32, bottom: i32, dy: i32, width: usize) {
    if dy > 0 {
        let source = (top + dy) as usize * width;
        let len = (bottom - top - dy) as usize * width;
        frame.copy_within(source..source + len, top as usize * width);
    } else if dy < 0 {
        let source = top as usize * width;
        let len = (bottom - top + dy) as usize * width;
        frame.copy_within(source..source + len, (top - dy) as usize * width);
    }
}

/// Depremultiply, composite over white and pack pixmap rows into the frame.
fn pack_rows(pixmap: &Pixmap, frame: &mut [u32], y0: i32, y1: i32) {
    let width = pixmap.width() as usize;
    let mut rgba = rgba_rows(pixmap, y0, y1);
    if rgba.is_empty() {
        return;
    }
    composite_over_white(&mut rgba);
    let base = y0.max(0) as usize;
    for (index, pixel) in rgba.as_chunks::<4>().0.iter().enumerate() {
        let slot = (base + index / width) * width + index % width;
        if let Some(dst) = frame.get_mut(slot) {
            *dst = 0xFF00_0000
                | ((pixel[0] as u32) << 16)
                | ((pixel[1] as u32) << 8)
                | pixel[2] as u32;
        }
    }
}

fn merge_stats_bands(bands: &[(i32, i32)], width: usize) -> u64 {
    bands
        .iter()
        .map(|&(y0, y1)| ((y1 - y0).max(0) as u64) * width as u64)
        .sum()
}

fn record_stats(update: &Update, bands: &[(i32, i32)], width: usize) {
    let (rects, area) = match update {
        Update::Full => (1usize, merge_stats_bands(bands, width)),
        Update::Damage(rects) => (rects.len(), rects.iter().map(|rect| rect.area()).sum()),
        Update::Scroll { .. } => (0, 0),
    };
    crate::about::record_repaint(
        matches!(update, Update::Full),
        matches!(update, Update::Scroll { .. }),
        rects,
        area,
        merge_stats_bands(bands, width),
    );
}

fn debug_outline(frame: &mut [u32], width: usize, height: usize, bands: &[(i32, i32)]) {
    let red = 0xFFFF_0000u32;
    for &(y0, y1) in bands {
        let (top, bottom) = (y0.max(0) as usize, y1.min(height as i32) as usize);
        if top >= bottom {
            continue;
        }
        for x in 0..width {
            if let Some(pixel) = frame.get_mut(top * width + x) {
                *pixel = red;
            }
            if let Some(pixel) = frame.get_mut((bottom - 1) * width + x) {
                *pixel = red;
            }
        }
        for y in top..bottom {
            if let Some(pixel) = frame.get_mut(y * width) {
                *pixel = red;
            }
            if let Some(pixel) = frame.get_mut(y * width + width - 1) {
                *pixel = red;
            }
        }
    }
}

impl BrowserApp {
    pub(crate) fn draw(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        let Some(width) = NonZeroU32::new(self.viewport.w.round().max(1.0) as u32) else {
            return;
        };
        let Some(height) = NonZeroU32::new(self.viewport.h.round().max(1.0) as u32) else {
            return;
        };
        if let Err(err) = surface.resize(width, height) {
            eprintln!("hodo: surface resize failed: {err}");
            return;
        }

        let viewport = self.viewport;
        // One theme snapshot per draw: the display-list colors and the overlay
        // must agree even if the settings toggle flips mid-frame.
        let theme = current_theme();
        let theme_dark = theme == tokens::DARK;
        let url_focused = self.state.url_focused;
        let mut bands = {
            let tab = &self.state.tabs[self.state.active];
            let label = tab_counter(self.state.active + 1, self.state.tabs.len());
            // Snap to integer pixels before composing: draw_text truncates glyph
            // origins to i32, so a fractional offset would desync blitted rows
            // from freshly rasterized ones and leave one-pixel seams.
            let offset =
                clamp_scroll_offset(tab.scroll_offset, tab.page_height, viewport.h).round() as i32;
            let composed = compose_display_list_with_theme(
                &tab.display_list,
                viewport,
                &self.state.url_bar,
                &self.ext_titles,
                offset as f32,
                &label,
                theme,
            );
            let render_len = composed.items.len() - chrome_item_count(viewport, &self.ext_titles);
            let key = FrameKey {
                viewport: (width.get() as i32, height.get() as i32),
                page_url: tab.url.clone(),
                label,
                ext_titles: self.ext_titles.clone(),
                page_hash: hodo_render::list_hash(&tab.display_list),
                chrome_hash: items_hash(&composed.items[render_len..]),
                paint_offset: offset,
                render_len,
                theme_dark,
                url_focused,
            };
            let force = self.force_full;
            self.force_full = false;
            let (update, bands) = self.raster.update(viewport, &composed, key, force);
            if !bands.is_empty() {
                record_stats(&update, &bands, self.raster.w);
            }
            bands
        };

        // One owner: the popup is composited onto the cached raster frame just
        // for this present, using the same integer-snapped scroll as the page.
        // A separate full-frame rebuild (and its second ARGB allocation) is
        // gone, so opening a popup cannot shift the page by half a pixel.
        let popup_band = if let Some(popup) = self.popup.as_ref() {
            self.raster.overlay_popup(popup, viewport)
        } else {
            Vec::new()
        };
        bands.extend(popup_band.iter().copied());

        let frame_width = self.raster.w;
        let frame_height = self.raster.h;
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        // age() == 0 is a fresh buffer with unspecified contents, and age() > 1
        // is older than the last presented frame; either way partial rows would
        // be stale, so fill every pixel and report full damage.
        if buffer.age() != 1 {
            bands = vec![(0, frame_height as i32)];
        }
        if bands.is_empty() {
            return;
        }
        if debug_damage_flag() {
            debug_outline(&mut self.raster.frame, frame_width, frame_height, &bands);
        }
        for &(y0, y1) in &bands {
            let start = y0.max(0) as usize * frame_width;
            let count = (y1 - y0).max(0) as usize * frame_width;
            if start + count <= buffer.len() && start + count <= self.raster.frame.len() {
                buffer[start..start + count]
                    .copy_from_slice(&self.raster.frame[start..start + count]);
            }
        }
        // The swapchain now has the overlay; the cache must not keep it.
        for &(y0, y1) in &popup_band {
            self.raster.restore_rows(y0, y1);
        }
        let damage: Vec<SoftRect> = bands
            .iter()
            .filter_map(|&(y0, y1)| {
                let (top, bottom) = (y0.max(0), y1.min(frame_height as i32));
                if top >= bottom {
                    return None;
                }
                NonZeroU32::new(frame_width as u32)
                    .zip(NonZeroU32::new((bottom - top) as u32))
                    .map(|(w, h)| SoftRect {
                        x: 0,
                        y: top as u32,
                        width: w,
                        height: h,
                    })
            })
            .collect();
        window.pre_present_notify();
        if let Err(err) = buffer.present_with_damage(&damage) {
            eprintln!("hodo: present failed: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_dom::NodeId;
    use hodo_types::{Color, Rect};

    fn solid(x: f32, y: f32, w: f32, h: f32, color: Color) -> DisplayItem {
        DisplayItem::Rect {
            rect: Rect { x, y, w, h },
            color,
            border_color: None,
            radius: 0.0,
        }
    }

    /// Test-local compose with a fixed light theme: the production path reads
    /// current_theme() at draw time, which would make these oracles flaky.
    fn compose(
        page: &DisplayList,
        viewport: Size,
        url: &str,
        ext_titles: &[String],
        scroll_offset: f32,
        tab_label: &str,
    ) -> DisplayList {
        compose_display_list_with_theme(
            page,
            viewport,
            url,
            ext_titles,
            scroll_offset,
            tab_label,
            tokens::LIGHT,
        )
    }

    fn text(y: f32, value: &str) -> DisplayItem {
        DisplayItem::Text {
            rect: Rect {
                x: 10.0,
                y,
                w: 100.0,
                h: 19.2,
            },
            text: value.into(),
            color: Color::BLACK,
            font_size: 16.0,
            weight: 400,
            href: None,
            underline: false,
            line_through: false,
        }
    }

    fn key(page: &DisplayList, viewport: Size, composed: &DisplayList, offset: i32) -> FrameKey {
        let render_len = composed.items.len() - chrome_item_count(viewport, &[]);
        FrameKey {
            viewport: (viewport.w as i32, viewport.h as i32),
            page_url: "about:hodo".into(),
            label: "tab 1/1".into(),
            ext_titles: Vec::new(),
            page_hash: hodo_render::list_hash(page),
            chrome_hash: items_hash(&composed.items[render_len..]),
            paint_offset: offset,
            render_len,
            theme_dark: false,
            url_focused: false,
        }
    }

    fn reference_frame(composed: &DisplayList, size: Size) -> Vec<u32> {
        render_frame(composed, size, false, false)
    }

    /// Full-render oracle: page through hodo-render, chrome through the same
    /// overlay the GUI uses. This is the golden the damage/blit paths must match.
    fn render_frame(
        composed: &DisplayList,
        size: Size,
        theme_dark: bool,
        url_focused: bool,
    ) -> Vec<u32> {
        let (w, h) = (size.w as i32, size.h as i32);
        let render_len = composed.items.len() - chrome_item_count(size, &[]);
        let mut pixmap = Pixmap::new(w as u32, h as u32).unwrap();
        render_into(
            &mut pixmap,
            &page_only(composed, render_len),
            IntRect { x: 0, y: 0, w, h },
        );
        chrome::paint(
            &mut pixmap,
            &composed.items[render_len..],
            size,
            chrome::Style {
                theme: if theme_dark {
                    tokens::DARK
                } else {
                    tokens::LIGHT
                },
                url_focused,
            },
        );
        let mut frame = vec![0u32; (w * h) as usize];
        pack_rows(&pixmap, &mut frame, 0, h);
        frame
    }

    fn pixels_in(frame: &[u32], width: usize, item: &DisplayItem) -> Vec<u32> {
        let rect = crate::ui::item_rect(item).unwrap();
        let (x0, y0) = (rect.x.floor() as usize, rect.y.floor() as usize);
        let (x1, y1) = (
            (rect.x + rect.w).ceil() as usize,
            (rect.y + rect.h).ceil() as usize,
        );
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| frame[y * width + x]))
            .collect()
    }

    #[test]
    fn scroll_blits_are_byte_identical_to_full_render_in_both_directions() {
        let size = Size { w: 120.0, h: 200.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..40)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 24.0,
                        60.0,
                        12.0,
                        Color::rgb(i as u8 * 5, 60, 120),
                    )
                })
                .chain(std::iter::once(text(100.0, "scroll me")))
                .collect(),
        };
        let mut raster = RasterState::new();
        // First frame is a full paint at offset 0.
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let (update, _) = raster.update(size, &composed0, key(&page, size, &composed0, 0), true);
        assert_eq!(update, Update::Full);
        assert_eq!(raster.frame, reference_frame(&composed0, size));

        // Down, down, up, up: every intermediate frame must match a full raster.
        for offset in [8i32, 21, 13, 0] {
            let composed = compose(&page, size, "", &[], offset as f32, "tab 1/1");
            let (update, _) =
                raster.update(size, &composed, key(&page, size, &composed, offset), false);
            assert!(
                matches!(update, Update::Scroll { .. }),
                "pure scroll must take the blit path, got {update:?}"
            );
            assert_eq!(
                raster.frame,
                reference_frame(&composed, size),
                "scroll to {offset} diverged from full render"
            );
        }
    }

    #[test]
    fn scroll_clamps_and_large_jumps_use_full_repaint() {
        let size = Size { w: 120.0, h: 200.0 };
        // Tall page so the clamp is actually exercised at both ends.
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..30)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 30.0,
                        60.0,
                        12.0,
                        Color::rgb(i as u8 * 8, 2, 3),
                    )
                })
                .collect(),
        };
        let mut raster = RasterState::new();
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        raster.update(size, &composed0, key(&page, size, &composed0, 0), true);
        let max = clamp_scroll_offset(f32::MAX, 900.0, 200.0).round() as i32;

        // Overshooting the top clamps to the current offset.
        let composed = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let (update, _) = raster.update(size, &composed, key(&page, size, &composed, 0), false);
        assert_eq!(update, Update::Damage(Vec::new()));

        // The effective jump exceeds the content band height: blit is unsafe.
        let jump = max;
        let composed = compose(&page, size, "", &[], jump as f32, "tab 1/1");
        let (update, _) = raster.update(size, &composed, key(&page, size, &composed, jump), false);
        assert_eq!(update, Update::Full);
        assert_eq!(raster.frame, reference_frame(&composed, size));

        // Overshooting the bottom clamps to the same offset.
        let raw = max as f32 + 500.0;
        let offset = clamp_scroll_offset(raw, 900.0, 200.0).round() as i32;
        let composed = compose(&page, size, "", &[], offset as f32, "tab 1/1");
        let (update, _) =
            raster.update(size, &composed, key(&page, size, &composed, offset), false);
        assert_eq!(update, Update::Damage(Vec::new()));
    }

    #[test]
    fn mixed_damage_and_scroll_sequence_matches_full_render() {
        let size = Size { w: 120.0, h: 200.0 };
        let mut page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..20)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 28.0,
                        60.0,
                        12.0,
                        Color::rgb(i as u8 * 10, 40, 20),
                    )
                })
                .chain(std::iter::once(text(60.0, "initial")))
                .collect(),
        };
        let mut raster = RasterState::new();
        let composed = compose(&page, size, "", &[], 0.0, "tab 1/1");
        raster.update(size, &composed, key(&page, size, &composed, 0), true);

        let mut offset = 0i32;
        for step in 0..16 {
            if step % 2 == 0 {
                if let DisplayItem::Text { text, .. } = &mut page.items[20] {
                    *text = format!("v{step}");
                }
            } else {
                offset = if offset == 0 { 12 } else { 0 };
            }
            let composed = compose(&page, size, "", &[], offset as f32, "tab 1/1");
            let (update, _) =
                raster.update(size, &composed, key(&page, size, &composed, offset), false);
            assert_ne!(
                update,
                Update::Damage(Vec::new()),
                "step {step} did nothing"
            );
            assert_eq!(
                raster.frame,
                reference_frame(&composed, size),
                "mixed step {step} diverged"
            );
        }
    }

    #[test]
    fn debug_outline_marks_only_band_edges() {
        let (w, h) = (40, 60);
        let mut frame = vec![0xFF00_0000u32; w * h];
        debug_outline(&mut frame, w, h, &[(10, 20)]);
        assert_eq!(frame[10 * w + 20], 0xFFFF_0000);
        assert_eq!(frame[19 * w + 20], 0xFFFF_0000);
        assert_eq!(frame[15 * w], 0xFFFF_0000);
        assert_eq!(frame[15 * w + w - 1], 0xFFFF_0000);
        assert_eq!(frame[15 * w + 20], 0xFF00_0000, "band interior untouched");
        assert_eq!(frame[9 * w + 20], 0xFF00_0000, "outside untouched");
    }

    #[test]
    fn explicit_force_full_bypasses_the_scroll_path() {
        let size = Size { w: 120.0, h: 200.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..20)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 28.0,
                        60.0,
                        12.0,
                        Color::rgb(i as u8 * 10, 2, 3),
                    )
                })
                .collect(),
        };
        let mut raster = RasterState::new();
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        raster.update(size, &composed0, key(&page, size, &composed0, 0), true);
        let composed = compose(&page, size, "", &[], 12.0, "tab 1/1");
        let (update, _) = raster.update(size, &composed, key(&page, size, &composed, 12), true);
        assert_eq!(update, Update::Full);
    }

    #[test]
    fn scroll_does_not_rebuild_the_page_list() {
        let size = Size { w: 120.0, h: 200.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![
                solid(5.0, 50.0, 60.0, 12.0, Color::rgb(1, 2, 3)),
                text(100.0, "stable"),
            ],
        };
        let mut raster = RasterState::new();
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let first_key = key(&page, size, &composed0, 0);
        raster.update(size, &composed0, first_key.clone(), true);
        let composed1 = compose(&page, size, "", &[], 8.0, "tab 1/1");
        let second_key = key(&page, size, &composed1, 8);
        assert_eq!(first_key.page_hash, second_key.page_hash);
        assert_eq!(first_key.chrome_hash, second_key.chrome_hash);
        let (update, _) = raster.update(size, &composed1, second_key, false);
        // RasterState::update never calls hodo_layout::layout; with identical
        // page/chrome hashes it can only blit.
        assert!(matches!(update, Update::Scroll { dy: 8, .. }));
    }

    #[test]
    fn tiny_viewport_paints_one_full_frame_and_never_clamps_negative() {
        let size = Size { w: 120.0, h: 30.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![text(10.0, "tiny")],
        };
        assert_eq!(clamp_scroll_offset(50.0, 500.0, size.h), 0.0);
        let composed = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let mut raster = RasterState::new();
        let (first, _) = raster.update(size, &composed, key(&page, size, &composed, 0), false);
        assert_eq!(first, Update::Full, "no visible band: one full repaint");
        let (second, _) = raster.update(size, &composed, key(&page, size, &composed, 0), false);
        assert_eq!(second, Update::Damage(Vec::new()), "no redundant repaint");
    }

    #[test]
    fn popup_overlay_uses_snapped_scroll_and_leaves_the_cache_clean() {
        let size = Size { w: 400.0, h: 600.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..40)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 24.0,
                        80.0,
                        12.0,
                        Color::rgb(i as u8 * 6, 3, 90),
                    )
                })
                .collect(),
        };
        let mut raster = RasterState::new();
        let offset = clamp_scroll_offset(10.4, 1000.0, size.h).round() as i32;
        let composed = compose(&page, size, "", &[], offset as f32, "tab 1/1");
        raster.update(size, &composed, key(&page, size, &composed, offset), true);
        let clean = raster.frame.clone();

        let popup = crate::popup::PopupState::open(
            &crate::popup::PopupAction {
                title: "Popup Demo".into(),
                root: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../extensions/popup-chrome"),
                popup_path: Some("popup.html".into()),
                icon_path: None,
            },
            0,
            size,
        )
        .unwrap();
        let band = raster.overlay_popup(&popup, size);
        let (y0, y1) = band[0];
        let width = size.w as usize;
        for y in 0..size.h as usize {
            if (y0 as usize..y1 as usize).contains(&y) {
                continue;
            }
            for x in 0..width {
                assert_eq!(
                    raster.frame[y * width + x],
                    clean[y * width + x],
                    "page outside the popup must not jump at ({x}, {y})"
                );
            }
        }
        raster.restore_rows(y0, y1);
        assert_eq!(raster.frame, clean, "the popup must not dirty the cache");

        // Popup closed, nothing else changed: no redundant Full repaint.
        let (update, _) =
            raster.update(size, &composed, key(&page, size, &composed, offset), false);
        assert_eq!(update, Update::Damage(Vec::new()));
    }

    #[test]
    fn scroll_line_update_keeps_the_fixed_rect_byte_stable() {
        let size = Size { w: 200.0, h: 300.0 };
        let page = DisplayList {
            items: (0..20)
                .map(|i| {
                    solid(
                        5.0,
                        i as f32 * 30.0,
                        80.0,
                        12.0,
                        Color::rgb(i as u8 * 7, 30, 90),
                    )
                })
                .collect(),
            node_map: Vec::new(),
            fixed_items: vec![solid(10.0, 120.0, 120.0, 40.0, Color::rgb(200, 40, 40))],
            fixed_node_map: vec![NodeId(7)],
        };
        let mut raster = RasterState::new();
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let (first, _) = raster.update(size, &composed0, key(&page, size, &composed0, 0), true);
        assert_eq!(first, Update::Full);
        let clean = pixels_in(&raster.frame, size.w as usize, &page.fixed_items[0]);

        let offset = 40; // SCROLL_LINE
        let composed1 = compose(&page, size, "", &[], offset as f32, "tab 1/1");
        let (update, bands) = raster.update(
            size,
            &composed1,
            key(&page, size, &composed1, offset),
            false,
        );
        assert!(
            matches!(update, Update::Scroll { dy: 40, .. }),
            "expected a pure scroll, got {update:?}"
        );
        let banner = item_bounds(&page.fixed_items[0]).unwrap();
        assert!(
            bands
                .iter()
                .any(|&(y0, y1)| y0 <= banner.y && y1 >= banner.bottom()),
            "the fixed band must be re-presented: {bands:?}"
        );
        assert_eq!(
            pixels_in(&raster.frame, size.w as usize, &page.fixed_items[0]),
            clean,
            "the fixed rect must not shift with the scroll"
        );
        assert_eq!(
            raster.frame,
            reference_frame(&composed1, size),
            "scroll must stay byte-identical to a full render with fixed items"
        );
    }

    #[test]
    fn damage_under_a_fixed_banner_repaints_the_banner_region_unchanged() {
        let size = Size { w: 200.0, h: 300.0 };
        let mut page = DisplayList {
            items: vec![
                solid(10.0, 130.0, 100.0, 20.0, Color::rgb(1, 2, 3)), // under the banner
                solid(10.0, 220.0, 100.0, 20.0, Color::rgb(9, 9, 9)),
            ],
            node_map: vec![NodeId(1), NodeId(2)],
            fixed_items: vec![solid(10.0, 120.0, 120.0, 40.0, Color::rgb(200, 40, 40))],
            fixed_node_map: vec![NodeId(7)],
        };
        let mut raster = RasterState::new();
        let composed0 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        raster.update(size, &composed0, key(&page, size, &composed0, 0), true);
        let clean_banner = pixels_in(&raster.frame, size.w as usize, &page.fixed_items[0]);

        if let DisplayItem::Rect { color, .. } = &mut page.items[0] {
            *color = Color::rgb(250, 250, 250);
        }
        let composed1 = compose(&page, size, "", &[], 0.0, "tab 1/1");
        let (update, bands) =
            raster.update(size, &composed1, key(&page, size, &composed1, 0), false);
        let Update::Damage(rects) = &update else {
            panic!("expected a damage update, got {update:?}");
        };
        let banner = item_bounds(&page.fixed_items[0]).unwrap();
        assert!(
            rects.iter().any(|rect| rect.intersects(banner)),
            "damage rects must cover the fixed banner region: {rects:?}"
        );
        assert!(
            bands
                .iter()
                .any(|&(y0, y1)| y0 <= banner.y && y1 >= banner.bottom()),
            "a band must cover the banner rows: {bands:?}"
        );
        assert_eq!(
            pixels_in(&raster.frame, size.w as usize, &page.fixed_items[0]),
            clean_banner,
            "the banner must re-rasterize at the same position"
        );
        assert_eq!(raster.frame, reference_frame(&composed1, size));
    }

    #[test]
    fn position_site_fixed_banner_is_pixel_identical_across_scroll_offsets() {
        let viewport = Size { w: 800.0, h: 600.0 };
        let index = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/position-site/index.html");
        let (_, page) = crate::app::load_page(&index.to_string_lossy(), viewport);
        let banner = page
            .fixed_items
            .iter()
            .find_map(|item| match item {
                DisplayItem::Rect { rect, .. } if rect.w >= 100.0 && rect.h >= 10.0 => Some(*rect),
                _ => None,
            })
            .expect("position-site must emit a fixed banner rect");
        assert!(
            banner.y >= TOP_BAR_H,
            "the banner must be visible below the top bar, got {banner:?}"
        );

        let mut raster = RasterState::new();
        let mut frame_for = |offset: i32| -> Vec<u32> {
            let composed = compose(&page, viewport, "", &[], offset as f32, "tab 1/1");
            raster.update(
                viewport,
                &composed,
                key(&page, viewport, &composed, offset),
                false,
            );
            raster.frame.clone()
        };
        let at_top = frame_for(0);
        let at_scrolled = frame_for(80);
        assert_ne!(
            at_top, at_scrolled,
            "the scroll must actually move page pixels"
        );

        let width = viewport.w as usize;
        let (x0, y0) = (banner.x.floor() as usize, banner.y.floor() as usize);
        let (x1, y1) = (
            (banner.x + banner.w).ceil() as usize,
            (banner.y + banner.h).ceil() as usize,
        );
        for y in y0..y1 {
            for x in x0..x1 {
                assert_eq!(
                    at_top[y * width + x],
                    at_scrolled[y * width + x],
                    "fixed banner pixel drifted at ({x}, {y})"
                );
            }
        }
    }

    /// The one pixel gate for this wave: restyling chrome must not move a page
    /// pixel. Page rendering is untouched; only the two bars may differ.
    #[test]
    fn chrome_restyle_leaves_page_pixels_byte_identical() {
        let size = Size { w: 400.0, h: 300.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: (0..12)
                .map(|i| {
                    solid(
                        10.0,
                        TOP_BAR_H + i as f32 * 18.0,
                        120.0,
                        12.0,
                        Color::rgb(i as u8 * 9, 40, 90),
                    )
                })
                .chain(std::iter::once(text(120.0, "page text")))
                .collect(),
        };
        let light = compose_display_list_with_theme(
            &page,
            size,
            "https://hodo.test",
            &[],
            0.0,
            "tab 1/1",
            tokens::LIGHT,
        );
        let dark = compose_display_list_with_theme(
            &page,
            size,
            "https://hodo.test",
            &[],
            0.0,
            "tab 1/1",
            tokens::DARK,
        );
        let light_frame = render_frame(&light, size, false, false);
        let dark_frame = render_frame(&dark, size, true, false);
        let width = size.w as usize;
        for y in TOP_BAR_H as usize..(size.h - BOTTOM_BAR_H) as usize {
            for x in 0..width {
                assert_eq!(
                    light_frame[y * width + x],
                    dark_frame[y * width + x],
                    "page pixel drifted at ({x}, {y})"
                );
            }
        }
        let chrome_pixels = TOP_BAR_H as usize * width;
        assert!(
            light_frame[..chrome_pixels]
                .iter()
                .zip(dark_frame[..chrome_pixels].iter())
                .any(|(a, b)| a != b),
            "the top bar must repaint in dark mode"
        );
    }

    #[test]
    fn theme_toggle_repaints_chrome_on_the_next_present() {
        let size = Size { w: 400.0, h: 300.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![solid(10.0, 60.0, 120.0, 20.0, Color::rgb(7, 8, 9))],
        };
        let light = compose_display_list_with_theme(
            &page,
            size,
            "https://hodo.test",
            &[],
            0.0,
            "tab 1/1",
            tokens::LIGHT,
        );
        let dark = compose_display_list_with_theme(
            &page,
            size,
            "https://hodo.test",
            &[],
            0.0,
            "tab 1/1",
            tokens::DARK,
        );
        let mut raster = RasterState::new();
        let mut frame_key = key(&page, size, &light, 0);
        raster.update(size, &light, frame_key.clone(), true);
        let before = raster.frame.clone();

        frame_key.theme_dark = true;
        let (update, bands) = raster.update(size, &dark, frame_key, false);
        assert_eq!(
            update,
            Update::Full,
            "a settings toggle is a metadata change"
        );
        assert_eq!(bands, vec![(0, size.h as i32)]);
        assert_eq!(raster.frame, render_frame(&dark, size, true, false));
        assert_ne!(
            raster.frame, before,
            "chrome must visibly follow the toggle"
        );

        let mut settled = key(&page, size, &dark, 0);
        settled.theme_dark = true;
        let (update, _) = raster.update(size, &dark, settled, false);
        assert_eq!(update, Update::Damage(Vec::new()), "no redundant repaint");
    }

    #[test]
    fn url_focus_ring_repaints_without_changing_page_pixels() {
        let size = Size { w: 400.0, h: 300.0 };
        let page = DisplayList {
            node_map: Vec::new(),
            fixed_items: Vec::new(),
            fixed_node_map: Vec::new(),
            items: vec![solid(10.0, 60.0, 120.0, 20.0, Color::rgb(5, 6, 7))],
        };
        let composed = compose(&page, size, "https://hodo.test", &[], 0.0, "tab 1/1");
        let mut raster = RasterState::new();
        raster.update(size, &composed, key(&page, size, &composed, 0), true);
        let before = raster.frame.clone();

        let mut focused = key(&page, size, &composed, 0);
        focused.url_focused = true;
        let (update, _) = raster.update(size, &composed, focused, false);
        assert_eq!(update, Update::Full, "focus has real plumbing, so repaint");
        assert_eq!(raster.frame, render_frame(&composed, size, false, true));
        let width = size.w as usize;
        for y in TOP_BAR_H as usize..(size.h - BOTTOM_BAR_H) as usize {
            for x in 0..width {
                assert_eq!(
                    before[y * width + x],
                    raster.frame[y * width + x],
                    "focus must not touch page pixels at ({x}, {y})"
                );
            }
        }
    }
}

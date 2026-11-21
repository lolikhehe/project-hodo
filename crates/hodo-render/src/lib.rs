//! Software rasterizer producing RGBA8 frames.

use std::sync::OnceLock;

use fontdue::{Font, FontSettings};
use hodo_layout::{DisplayItem, DisplayList};
use hodo_types::{Color, Size};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

static FONT: OnceLock<Option<Font>> = OnceLock::new();

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

/// Rasterize to RGBA8 (non-premultiplied), row-major, `w * h * 4` bytes.
pub fn render(list: &DisplayList, size: Size) -> Vec<u8> {
    let w = size.w.round() as i32;
    let h = size.h.round() as i32;
    if w <= 0 || h <= 0 {
        return Vec::new();
    }
    let mut pixmap = match Pixmap::new(w as u32, h as u32) {
        Some(p) => p,
        None => return Vec::new(),
    };

    for item in &list.items {
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
                ..
            } => draw_text(
                &mut pixmap,
                rect.x,
                rect.y,
                rect.h,
                text,
                *color,
                *font_size,
            ),
            DisplayItem::Image { rect, path } => draw_image(&mut pixmap, rect, path),
        }
    }

    // tiny-skia stores premultiplied alpha; convert to straight RGBA.
    let data = pixmap.data_mut();
    let (pixels, _) = data.as_chunks_mut::<4>();
    for px in pixels {
        let a = px[3] as u32;
        if a == 0 {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            continue;
        }
        px[0] = depremultiply(px[0], a);
        px[1] = depremultiply(px[1], a);
        px[2] = depremultiply(px[2], a);
    }
    data.to_vec()
}

fn depremultiply(v: u8, a: u32) -> u8 {
    ((v as u32 * 255 + 127) / a).min(255) as u8
}

/// Load, decode and blit an <img> into its layout rect. Failure paints a
/// placeholder instead of propagating an error into the frame pipeline.
fn draw_image(pixmap: &mut Pixmap, rect: &hodo_types::Rect, path: &str) {
    let decoded = (|| {
        let hodo_net::Resource::Other { data, .. } = hodo_net::load(path).ok()? else {
            return None;
        };
        if data.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
            decode_png(&data)
        } else if data.starts_with(&[0xff, 0xd8]) {
            decode_jpeg(&data)
        } else {
            None
        }
    })();
    match decoded {
        Some((w, h, rgba)) => blit_scaled(pixmap, rect, w, h, &rgba),
        None => draw_placeholder(pixmap, rect),
    }
}

/// PNG -> (width, height, straight RGBA8). EXPAND + STRIP_16 turns indexed,
/// grayscale and 16-bit inputs into the 8-bit RGB/RGBA we handle here.
fn decode_png(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
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
    Some((info.width as u32, info.height as u32, rgba))
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

fn draw_text(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    h: f32,
    text: &str,
    color: Color,
    font_size: f32,
) {
    if !font_size.is_finite() || font_size <= 0.0 {
        return;
    }
    let w = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    // Layout emits Text rects at the line-box top with h = 1.2 * font_size,
    // so the baseline sits at ~0.8 * font_size below the top.
    let baseline = y + 0.8 * (h / 1.2);

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
        return;
    };

    let data = pixmap.data_mut();
    let mut pen_x = x;
    for ch in text.chars() {
        if ch.is_control() {
            continue;
        }
        let (metrics, coverage) = font.rasterize(ch, font_size);
        let origin_x = pen_x as i32 + metrics.xmin;
        // fontdue's ymin is the glyph's lowest point in font units (Y-up,
        // baseline at 0): ~0 for glyphs resting on the baseline, negative for
        // descenders. Its bitmap is top-down, so the top row sits at
        // baseline - height - ymin; this puts '.' on the baseline and 'g' below it.
        let origin_y = (baseline - metrics.height as f32 - metrics.ymin as f32) as i32;
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let cov = coverage[row * metrics.width + col];
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
        let advance = if metrics.advance_width > 0.0 {
            metrics.advance_width
        } else {
            0.6 * font_size
        };
        pen_x += advance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_layout::DisplayItem;
    use hodo_types::Rect;

    fn pixel(buf: &[u8], x: usize, y: usize, w: usize) -> [u8; 4] {
        let i = (y * w + x) * 4;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn solid_rect_draws_exact_color() {
        let list = DisplayList {
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

    #[test]
    fn image_draws_decoded_png_pixels() {
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
    fn missing_image_draws_gray_placeholder_without_panic() {
        let list = DisplayList {
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
}

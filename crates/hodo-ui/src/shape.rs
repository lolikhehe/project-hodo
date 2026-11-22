use tiny_skia::{Path, PathBuilder, Rect};

/// Cubic-bezier approximation of a quarter circle.
const KAPPA: f32 = 0.552_284_8;

/// Resolve a corner step. A negative value is the `SHAPE_FULL` pill sentinel:
/// radius = half the shorter side.
pub fn resolve_radius(step: f32, rect: Rect) -> f32 {
    let w = rect.width();
    let h = rect.height();
    let half = if w.is_finite() && h.is_finite() {
        w.min(h).max(0.0) * 0.5
    } else {
        0.0
    };
    // NaN/infinite steps are malformed input; degrade to a sharp corner rather
    // than poisoning the path builder with NaN coordinates.
    if !step.is_finite() {
        return 0.0;
    }
    if step < 0.0 {
        half
    } else {
        step.clamp(0.0, half)
    }
}

/// Rounded rectangle with per-corner radii in the order [top-left, top-right,
/// bottom-right, bottom-left]. Corners use cubic arcs; radius 0 is a sharp
/// corner. Radii are clamped to half the shorter side.
pub fn rounded_rect_path(rect: Rect, radii: [f32; 4]) -> Path {
    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();
    let [tl, tr, br, bl] = radii.map(|r| resolve_radius(r, rect));
    let k = KAPPA;

    let mut b = PathBuilder::new();
    b.move_to(x + tl, y);
    b.line_to(x + w - tr, y);
    if tr > 0.0 {
        b.cubic_to(
            x + w - tr + tr * k,
            y,
            x + w,
            y + tr - tr * k,
            x + w,
            y + tr,
        );
    } else {
        b.line_to(x + w, y);
    }
    b.line_to(x + w, y + h - br);
    if br > 0.0 {
        b.cubic_to(
            x + w,
            y + h - br + br * k,
            x + w - br + br * k,
            y + h,
            x + w - br,
            y + h,
        );
    } else {
        b.line_to(x + w, y + h);
    }
    b.line_to(x + bl, y + h);
    if bl > 0.0 {
        b.cubic_to(
            x + bl - bl * k,
            y + h,
            x,
            y + h - bl + bl * k,
            x,
            y + h - bl,
        );
    } else {
        b.line_to(x, y + h);
    }
    b.line_to(x, y + tl);
    if tl > 0.0 {
        b.cubic_to(x, y + tl - tl * k, x + tl - tl * k, y, x + tl, y);
    } else {
        b.line_to(x + tl, y);
    }
    b.close();
    // Malformed rects/steps can yield an unbuildable path; degrade to a
    // degenerate (invisible) path instead of panicking in a frame painter.
    b.finish().unwrap_or_else(degenerate_path)
}

/// tiny-skia 0.12 has no `Path::empty()`: its builder rejects empty paths, so
/// the smallest accepted path is a zero-length move+line contour.
fn degenerate_path() -> Path {
    let mut b = PathBuilder::new();
    b.move_to(0.0, 0.0);
    b.line_to(0.0, 0.0);
    b.finish().expect("a finite move+line path is always valid")
}

/// Fully rounded rectangle (pill): radius = half the shorter side on all corners.
pub fn pill(rect: Rect) -> Path {
    let r = resolve_radius(-1.0, rect);
    rounded_rect_path(rect, [r; 4])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f32, h: f32) -> Rect {
        Rect::from_xywh(10.0, 20.0, w, h).unwrap()
    }

    #[test]
    fn pill_bounds_match_rect() {
        let r = rect(120.0, 40.0);
        let path = pill(r);
        let b = path.bounds();
        assert!((b.x() - r.x()).abs() < 0.01);
        assert!((b.y() - r.y()).abs() < 0.01);
        assert!((b.width() - r.width()).abs() < 0.01);
        assert!((b.height() - r.height()).abs() < 0.01);
    }

    #[test]
    fn radius_is_clamped_to_half_shorter_side() {
        let r = rect(40.0, 100.0);
        assert!((resolve_radius(999.0, r) - 20.0).abs() < f32::EPSILON);
        assert!((resolve_radius(-1.0, r) - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn square_corner_path_is_valid() {
        let _ = rounded_rect_path(rect(20.0, 20.0), [0.0; 4]);
    }

    #[test]
    fn nan_step_degrades_to_sharp_corners_without_panicking() {
        let r = rect(40.0, 40.0);
        assert_eq!(resolve_radius(f32::NAN, r), 0.0);
        assert_eq!(resolve_radius(f32::INFINITY, r), 0.0);
        let path = rounded_rect_path(r, [f32::NAN; 4]);
        let bounds = path.bounds();
        assert!((bounds.x() - r.x()).abs() < 0.01);
        assert!((bounds.width() - r.width()).abs() < 0.01);
    }
}

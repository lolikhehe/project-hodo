//! Display-list composition and hit-testing for the browser chrome.
//!
//! Every chrome color comes from `hodo-ui`'s [`ThemeAccess`] through
//! [`current_theme`], which re-reads `cache/design.json` on each call. Chrome
//! shapes, Roboto text and Material Symbols are rasterized in `present.rs`;
//! this module owns geometry, colors, the display-list schema and hit-testing.

use hodo_dom::NodeId;
use hodo_layout::{DisplayItem, DisplayList};
use hodo_types::{Color, Rect, Size};
use hodo_ui::{
    tokens::{self, Theme},
    ColorRole, ThemeAccess, TypeRole,
};

/// hodo-ui's token colors cross into the renderer's color type at the chrome
/// boundary; no color literal is created here.
pub(crate) fn render_color(color: hodo_ui::Color) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}

/// Height of the top URL bar strip. Shared with the scroll-blit content band.
pub(crate) const TOP_BAR_H: f32 = 40.0;
/// Height of the bottom extension toolbar strip.
pub(crate) const BOTTOM_BAR_H: f32 = 28.0;
/// Width of one toolbar action slot.
pub(crate) const ACTION_SLOT_W: f32 = 120.0;
/// Built-in actions rasterized by the shell. Extension actions keep their own
/// bitmaps and are deliberately absent from this list.
pub(crate) const CHROME_ICONS: [&str; 4] = ["back", "forward", "reload", "home"];
/// Material Symbol canvas size used by the toolbar buttons.
pub(crate) const CHROME_ICON_SIZE: u32 = 20;
const URL_PILL_H: f32 = 28.0;
const URL_PILL_PAD: f32 = 12.0;
const TAB_LABEL_W_EXPANDED: f32 = 108.0;
const TAB_LABEL_W_COMPACT: f32 = 80.0;

/// The chrome theme for this frame. `read_dark_enabled` is the landed
/// `cache/design.json` contract shared with `hodo://settings`; reading it here
/// (rather than caching at startup) makes a toggle visible on the next present.
pub fn current_theme() -> Theme {
    if crate::hpages::read_dark_enabled() {
        tokens::DARK
    } else {
        tokens::LIGHT
    }
}

/// Roomy vs tight chrome metrics from the landed window-class table.
/// Bar heights stay fixed; only padding/gaps and the URL pill cap change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChromeMetrics {
    pub(crate) name: &'static str,
    pub(crate) pad: f32,
    pub(crate) gap: f32,
    pub(crate) url_max: f32,
}

pub(crate) fn chrome_metrics(viewport: Size) -> ChromeMetrics {
    let w = viewport.w;
    let class = tokens::WINDOW_CLASSES
        .iter()
        .find(|class| w >= class.min_dp as f32 && (class.max_dp < 0 || w <= class.max_dp as f32))
        .copied()
        .unwrap_or(tokens::WINDOW_CLASSES[0]);
    // expanded / large / xl reuse the roomy layout; compact / medium share the
    // tight one (the token table itself only groups layout by url_max).
    let expanded = matches!(class.name, "expanded" | "large" | "xl");
    ChromeMetrics {
        name: class.name,
        pad: if expanded { 16.0 } else { 8.0 },
        gap: if expanded { 8.0 } else { 4.0 },
        url_max: class.url_max as f32,
    }
}

/// Slot rect of built-in toolbar icon `index` in the top bar.
pub(crate) fn toolbar_icon_rect(viewport: Size, index: usize) -> Rect {
    let m = chrome_metrics(viewport);
    Rect {
        x: m.pad + index as f32 * (CHROME_ICON_SIZE as f32 + m.gap),
        y: ((TOP_BAR_H - CHROME_ICON_SIZE as f32) * 0.5).round(),
        w: CHROME_ICON_SIZE as f32,
        h: CHROME_ICON_SIZE as f32,
    }
}

/// URL field pill. Its width is capped by the window class and by the space
/// left between the built-in icons and the tab counter.
pub(crate) fn url_pill_rect(viewport: Size) -> Rect {
    let m = chrome_metrics(viewport);
    let last = toolbar_icon_rect(viewport, CHROME_ICONS.len() - 1);
    let x = last.x + last.w + m.gap;
    let tab_w = tab_label_rect(viewport).w;
    let available = viewport.w - x - m.gap - tab_w - m.pad;
    Rect {
        x,
        y: (TOP_BAR_H - URL_PILL_H) * 0.5,
        w: m.url_max.min(available).max(1.0),
        h: URL_PILL_H,
    }
}

/// Text area inside the URL pill.
pub(crate) fn url_text_rect(viewport: Size) -> Rect {
    let pill = url_pill_rect(viewport);
    Rect {
        x: pill.x + URL_PILL_PAD,
        y: pill.y,
        w: (pill.w - 2.0 * URL_PILL_PAD).max(1.0),
        h: pill.h,
    }
}

fn tab_label_rect(viewport: Size) -> Rect {
    let m = chrome_metrics(viewport);
    let w = if m.name == "compact" || m.name == "medium" {
        TAB_LABEL_W_COMPACT
    } else {
        TAB_LABEL_W_EXPANDED
    };
    Rect {
        x: viewport.w - m.pad - w,
        y: (TOP_BAR_H - tokens::TYPE_TOOLBAR.line_height as f32) * 0.5,
        w,
        h: tokens::TYPE_TOOLBAR.line_height as f32,
    }
}

/// Number of chrome display items appended after the page + fixed prefix:
/// top bar, URL text, tab label, bottom bar, then one slot per visible action.
pub(crate) fn chrome_item_count(viewport: Size, ext_titles: &[String]) -> usize {
    let visible = visible_action_count(viewport, ext_titles.len());
    4 + visible + usize::from(visible < ext_titles.len())
}

/// Status text in the top strip, e.g. `tab 2/3`.
pub fn tab_counter(active: usize, total: usize) -> String {
    format!("tab {active}/{total}")
}

/// Clamp a scroll offset to the visible page range.
///
/// The URL/status bars overlay the top and bottom of the viewport, so only
/// `h - TOP_BAR_H - BOTTOM_BAR_H` pixels of the page are ever visible. A tiny
/// window leaves no band at all: pin the offset to 0 instead of returning a
/// negative maximum.
pub fn clamp_scroll_offset(offset: f32, page_height: f32, viewport_h: f32) -> f32 {
    let band = viewport_h - TOP_BAR_H - BOTTOM_BAR_H;
    if band <= 0.0 {
        return 0.0;
    }
    offset.clamp(0.0, (page_height - band).max(0.0))
}

/// Borrow the rect of a display item; every current variant has one.
pub(crate) fn item_rect(item: &DisplayItem) -> Option<&Rect> {
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

fn linked_href(item: &DisplayItem, x: f32, y: f32) -> Option<String> {
    match item {
        DisplayItem::Text {
            rect,
            href: Some(href),
            ..
        } if x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h => {
            Some(href.clone())
        }
        _ => None,
    }
}

fn item_contains(item: &DisplayItem, x: f32, y: f32) -> bool {
    item_rect(item).is_some_and(|rect| {
        x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h
    })
}

/// Topmost linked text under `(x, y)`, or `None`. Items later in the list are
/// painted on top, so they win; unlinked text is ignored.
pub fn hit_test_link(items: &[DisplayItem], x: f32, y: f32) -> Option<String> {
    items.iter().rev().find_map(|item| linked_href(item, x, y))
}

/// Resolve the topmost item under `(x, y)` into its link and DOM owner.
///
/// Layering contract: fixed items are painted above the scrolled page but
/// below chrome. The fixed slice is checked first through its index-parallel
/// `fixed_node_map`, so a fixed box wins over page items at the same pixel;
/// its link is the only one that can navigate there. Chrome has no DOM owner
/// and is never targetable. Generated pages carry no node map, so their links
/// fall back to the paint-order scan.
pub fn hit_test(composed: &DisplayList, x: f32, y: f32) -> (Option<String>, Option<NodeId>) {
    for (index, item) in composed.fixed_items.iter().enumerate().rev() {
        if item_contains(item, x, y) {
            return (
                linked_href(item, x, y),
                composed.fixed_node_map.get(index).copied(),
            );
        }
    }
    // `node_map` is index-parallel to the page + fixed prefix of `items`;
    // chrome is appended after it without an entry, so it is skipped here.
    for (index, &owner) in composed.node_map.iter().enumerate().rev() {
        if composed
            .items
            .get(index)
            .is_some_and(|item| item_contains(item, x, y))
        {
            return (linked_href(&composed.items[index], x, y), Some(owner));
        }
    }
    if composed.node_map.is_empty() && composed.fixed_items.is_empty() {
        return (hit_test_link(&composed.items, x, y), None);
    }
    (None, None)
}

/// Index of the topmost form control whose rect contains `(x, y)`.
/// Rectangles are in page space; the caller adds the scroll offset.
pub(crate) fn hit_test_control(rects: &[Rect], x: f32, y: f32) -> Option<usize> {
    rects
        .iter()
        .rposition(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
}

/// Left edge of action slot `index` in the bottom strip.
pub(crate) fn action_slot_x(index: usize) -> f32 {
    8.0 + index as f32 * ACTION_SLOT_W
}

/// How many action titles fit, leaving one slot for the `+N` overflow counter.
fn visible_action_count(viewport: Size, total: usize) -> usize {
    let capacity = ((viewport.w - 16.0).max(0.0) / ACTION_SLOT_W) as usize;
    if total > capacity {
        capacity.saturating_sub(1)
    } else {
        capacity
    }
    .min(total)
}

fn action_slot_rect(viewport: Size, index: usize) -> Rect {
    Rect {
        x: action_slot_x(index),
        y: viewport.h - BOTTOM_BAR_H + 3.0,
        w: ACTION_SLOT_W - 4.0,
        h: BOTTOM_BAR_H - 6.0,
    }
}

/// Ellipsize a title to the width of one slot (average Roboto advance).
fn slot_title(title: &str, font_size: f32) -> String {
    let max = ((ACTION_SLOT_W - 8.0) / (font_size * 0.55)) as usize;
    let chars: Vec<char> = title.chars().collect();
    if chars.len() <= max {
        return title.to_string();
    }
    let mut out: String = chars[..max.saturating_sub(1)].iter().collect();
    out.push('…');
    out
}

/// Action slot under `(x, y)`, if any. Checked before page link hit-testing.
///
/// Only actions with a `popup_path` get a click behavior at the app level;
/// the others are title-only here (a tooltip is deferred — wave 1 has no
/// tooltip rendering pass).
pub(crate) fn hit_test_action(viewport: Size, titles: &[String], x: f32, y: f32) -> Option<usize> {
    if y < viewport.h - BOTTOM_BAR_H || y >= viewport.h {
        return None;
    }
    (0..visible_action_count(viewport, titles.len())).find(|&index| {
        let rect = action_slot_rect(viewport, index);
        x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h
    })
}

/// Bottom edge of the page content, before any scroll offset is applied.
pub fn display_bottom(page: &DisplayList) -> f32 {
    page.items
        .iter()
        .filter_map(item_rect)
        .map(|rect| rect.y + rect.h)
        .fold(0.0, f32::max)
}

/// Page display list (shifted by the scroll offset) + the fixed UI strips,
/// rendered in a single pass.
pub fn compose_display_list(
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
        current_theme(),
    )
}

/// Same schema as [`compose_display_list`] with an explicit theme. The draw
/// path calls this with one [`current_theme`] snapshot so the display list and
/// the overlay cannot straddle a settings toggle; tests use it for light/dark.
pub(crate) fn compose_display_list_with_theme(
    page: &DisplayList,
    viewport: Size,
    url: &str,
    ext_titles: &[String],
    scroll_offset: f32,
    tab_label: &str,
    theme: Theme,
) -> DisplayList {
    let offset = clamp_scroll_offset(scroll_offset, display_bottom(page), viewport.h);
    let mut items: Vec<DisplayItem> = page
        .items
        .iter()
        .cloned()
        .map(|mut item| {
            if let Some(rect) = item_rect_mut(&mut item) {
                rect.y -= offset;
            }
            item
        })
        .collect();

    // Layering contract (page layer, bottom to top): scrolled page items, then
    // fixed items at their unshifted viewport positions, then the chrome bars.
    // `node_map` stays index-parallel to the page + fixed prefix of `items`;
    // chrome is appended unmapped. The fixed slice is also carried separately
    // so present.rs can damage/recompose its rows and hit-testing can consult
    // the index-parallel `fixed_node_map`.
    let mut node_map: Vec<NodeId> = page.node_map.clone();
    items.extend(page.fixed_items.iter().cloned());
    node_map.extend(page.fixed_node_map.iter().cloned());
    let fixed_items = page.fixed_items.clone();
    let fixed_node_map = page.fixed_node_map.clone();

    // Chrome suffix (kept index-stable for the shell hit/layout tests): top bar,
    // URL text, tab label, bottom bar, then extension slots. present.rs strips
    // this suffix before hodo-render runs and paints the real M3 chrome itself
    // (pill, Roboto text, Material Symbols); these items still drive the
    // frame-key hash and damage rects, so a theme or URL change repaints them.
    let bar_color = render_color(theme.color(ColorRole::SurfaceContainer));
    items.push(DisplayItem::Rect {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            w: viewport.w,
            h: TOP_BAR_H,
        },
        color: bar_color,
        border_color: None,
        radius: 0.0,
    });
    let url_style = theme.type_style(TypeRole::Url);
    items.push(DisplayItem::Text {
        rect: url_text_rect(viewport),
        text: url.to_string(),
        color: render_color(theme.color(ColorRole::OnSurface)),
        font_size: url_style.size as f32,
        weight: url_style.weight as u16,
        href: None,
        underline: false,
        line_through: false,
    });
    let tab_style = theme.type_style(TypeRole::Toolbar);
    items.push(DisplayItem::Text {
        rect: tab_label_rect(viewport),
        text: tab_label.to_string(),
        color: render_color(theme.color(ColorRole::OnSurface)),
        font_size: tab_style.size as f32,
        weight: tab_style.weight as u16,
        href: None,
        underline: false,
        line_through: false,
    });

    let bottom_y = viewport.h - BOTTOM_BAR_H;
    items.push(DisplayItem::Rect {
        rect: Rect {
            x: 0.0,
            y: bottom_y,
            w: viewport.w,
            h: BOTTOM_BAR_H,
        },
        color: bar_color,
        border_color: None,
        radius: 0.0,
    });
    // Extension action icons stay as-is (their own bitmaps); the shell only
    // paints the title. Titles are clickable when the extension declares a
    // popup; the app checks that before opening one.
    let strip_style = theme.type_style(TypeRole::Strip);
    let visible = visible_action_count(viewport, ext_titles.len());
    for (index, title) in ext_titles.iter().take(visible).enumerate() {
        items.push(DisplayItem::Text {
            rect: action_slot_rect(viewport, index),
            text: slot_title(title, strip_style.size as f32),
            color: render_color(theme.color(ColorRole::OnSurfaceVariant)),
            font_size: strip_style.size as f32,
            weight: strip_style.weight as u16,
            href: None,
            underline: false,
            line_through: false,
        });
    }
    if visible < ext_titles.len() {
        items.push(DisplayItem::Text {
            rect: action_slot_rect(viewport, visible),
            text: format!("+{}", ext_titles.len() - visible),
            color: render_color(theme.color(ColorRole::OnSurfaceVariant)),
            font_size: strip_style.size as f32,
            weight: strip_style.weight as u16,
            href: None,
            underline: false,
            line_through: false,
        });
    }

    DisplayList {
        items,
        node_map,
        fixed_items,
        fixed_node_map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_types::Color;
    use std::fs;
    use std::path::Path;

    #[test]
    fn scroll_clamp_uses_the_visible_band_and_never_goes_negative() {
        // 800px window: 40px top + 28px bottom bars leave 732px of page.
        assert_eq!(clamp_scroll_offset(500.0, 1000.0, 800.0), 268.0);
        assert_eq!(clamp_scroll_offset(-10.0, 1000.0, 800.0), 0.0);

        // Tiny windows have no visible band; clamping must not produce a
        // negative offset that would shift content down instead of up.
        for h in [0.0, 20.0, TOP_BAR_H + BOTTOM_BAR_H] {
            assert_eq!(clamp_scroll_offset(100.0, 500.0, h), 0.0, "h={h}");
        }
    }

    fn linked(x: f32, y: f32, w: f32, h: f32, href: &str) -> DisplayItem {
        DisplayItem::Text {
            rect: Rect { x, y, w, h },
            text: href.into(),
            color: Color::BLACK,
            font_size: 16.0,
            weight: 400,
            href: Some(href.into()),
            underline: false,
            line_through: false,
        }
    }

    #[test]
    fn hit_test_resolves_fixed_owner_and_fixed_wins_at_the_same_pixel() {
        let page = DisplayList {
            items: vec![
                linked(0.0, 100.0, 200.0, 20.0, "/under"),
                linked(0.0, 200.0, 200.0, 20.0, "/page-only"),
            ],
            node_map: vec![NodeId(11), NodeId(12)],
            fixed_items: vec![linked(0.0, 100.0, 200.0, 20.0, "/pinned")],
            fixed_node_map: vec![NodeId(42)],
        };
        let composed =
            compose_display_list(&page, Size { w: 400.0, h: 300.0 }, "", &[], 0.0, "tab 1/1");

        let (href, owner) = hit_test(&composed, 10.0, 110.0);
        assert_eq!(href.as_deref(), Some("/pinned"), "fixed layer wins");
        assert_eq!(
            owner,
            Some(NodeId(42)),
            "fixed_node_map must resolve the fixed element"
        );

        let (href, owner) = hit_test(&composed, 10.0, 210.0);
        assert_eq!(href.as_deref(), Some("/page-only"));
        assert_eq!(owner, Some(NodeId(12)), "page segment uses node_map");
    }

    #[test]
    fn current_theme_follows_the_design_json_flag() {
        let expected = if crate::hpages::read_dark_enabled() {
            tokens::DARK
        } else {
            tokens::LIGHT
        };
        assert_eq!(current_theme(), expected);
    }

    #[test]
    fn chrome_colors_and_type_flow_from_the_theme() {
        let size = Size {
            w: 1200.0,
            h: 800.0,
        };
        let ext = vec!["Ext".to_string()];
        for theme in [tokens::LIGHT, tokens::DARK] {
            let composed = compose_display_list_with_theme(
                &DisplayList::default(),
                size,
                "https://hodo.test",
                &ext,
                0.0,
                "tab 1/1",
                theme,
            );
            let chrome = &composed.items[composed.items.len() - chrome_item_count(size, &ext)..];
            let DisplayItem::Rect { color: bar, .. } = chrome[0] else {
                panic!("top bar must be a rect");
            };
            assert_eq!(bar, render_color(theme.color(ColorRole::SurfaceContainer)));
            let DisplayItem::Text {
                color: url,
                font_size,
                weight,
                ..
            } = &chrome[1]
            else {
                panic!("URL must be a text item");
            };
            let url_style = theme.type_style(TypeRole::Url);
            assert_eq!(*url, render_color(theme.color(ColorRole::OnSurface)));
            assert_eq!(
                (*font_size, *weight),
                (url_style.size as f32, url_style.weight as u16)
            );
            let DisplayItem::Rect { color: bottom, .. } = chrome[3] else {
                panic!("bottom bar must be a rect");
            };
            assert_eq!(
                bottom,
                render_color(theme.color(ColorRole::SurfaceContainer))
            );
            let DisplayItem::Text {
                color: ext_color, ..
            } = chrome[4]
            else {
                panic!("extension title must be a text item");
            };
            assert_eq!(
                ext_color,
                render_color(theme.color(ColorRole::OnSurfaceVariant))
            );
        }
    }

    #[test]
    fn window_class_drives_spacing_and_pill_width_but_not_bar_heights() {
        let large = chrome_metrics(Size {
            w: 1200.0,
            h: 800.0,
        });
        assert_eq!(large.name, "large");
        assert_eq!((large.pad, large.gap, large.url_max), (16.0, 8.0, 720.0));
        let compact = chrome_metrics(Size { w: 500.0, h: 400.0 });
        assert_eq!(compact.name, "compact");
        assert_eq!(
            (compact.pad, compact.gap, compact.url_max),
            (8.0, 4.0, 480.0)
        );
        assert!(
            url_pill_rect(Size {
                w: 1200.0,
                h: 800.0
            })
            .w <= large.url_max
        );
        assert!(url_pill_rect(Size { w: 500.0, h: 400.0 }).w <= compact.url_max);
        assert_eq!((TOP_BAR_H, BOTTOM_BAR_H), (40.0, 28.0));
    }

    #[test]
    fn chrome_item_count_matches_the_composed_suffix() {
        let size = Size { w: 400.0, h: 300.0 };
        let ext = vec!["a".into(), "b".into(), "c".into()];
        let composed =
            compose_display_list(&DisplayList::default(), size, "", &ext, 0.0, "tab 1/1");
        assert_eq!(chrome_item_count(size, &ext), composed.items.len());
    }

    // Mirror of `hodo-ui/tests/no_raw_hex.rs`, scoped to the shell chrome code:
    // present.rs has unrelated ARGB constants in the frame packer, so only the
    // explicitly delimited chrome draw region is checked there.
    fn contains_hex(s: &str) -> bool {
        let b = s.as_bytes();
        for i in 0..b.len() {
            let digits_at = if b[i] == b'#' {
                i + 1
            } else if b[i] == b'0' && i + 1 < b.len() && (b[i + 1] == b'x' || b[i + 1] == b'X') {
                i + 2
            } else {
                continue;
            };
            if digits_at + 6 <= b.len()
                && b[digits_at..digits_at + 6]
                    .iter()
                    .all(|c| c.is_ascii_hexdigit())
            {
                return true;
            }
        }
        false
    }

    #[test]
    fn no_raw_hex_in_chrome_drawing() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ui_src = fs::read_to_string(root.join("src/ui.rs")).expect("read ui.rs");
        let present_src = fs::read_to_string(root.join("src/present.rs")).expect("read present.rs");
        let begin = present_src
            .find("// BEGIN CHROME DRAW")
            .expect("chrome draw start marker");
        let end = present_src
            .find("// END CHROME DRAW")
            .expect("chrome draw end marker");
        assert!(begin < end, "chrome draw markers out of order");
        let chrome_src = &present_src[begin..end];
        assert!(
            !contains_hex(&ui_src),
            "raw hex in ui.rs chrome composition"
        );
        assert!(
            !contains_hex(chrome_src),
            "raw hex in present.rs chrome drawing"
        );
    }
}

//! M7.3 form control layout.
//!
//! Controls are atomic replaced elements: one `DisplayItem::Control` per
//! control, never fragmented across line boxes. Geometry comes from the
//! computed style with wave-1 defaults where CSS provides nothing; semantic
//! state (`name`, `masked`, `disabled`, `readonly`) is carried straight through
//! to the renderer. The shell keeps the live value/caret state — layout only
//! reads the initial value from the DOM.

use super::*;

/// Tags that emit a `DisplayItem::Control` instead of flowing their children.
pub(crate) fn is_form_control(tag: &str) -> bool {
    tag.eq_ignore_ascii_case("input")
        || tag.eq_ignore_ascii_case("button")
        || tag.eq_ignore_ascii_case("select")
        || tag.eq_ignore_ascii_case("textarea")
}

/// Semantic kind of one control element. Duplicated in the shell for its
/// registry: this crate deliberately exposes only the display-list contract.
fn control_kind(dom: &Dom, id: NodeId) -> Option<ControlKind> {
    let tag = dom.tag(id)?;
    if tag.eq_ignore_ascii_case("textarea") {
        return Some(ControlKind::Textarea);
    }
    if tag.eq_ignore_ascii_case("select") {
        return Some(ControlKind::Select);
    }
    if tag.eq_ignore_ascii_case("button") {
        return Some(ControlKind::Button);
    }
    if tag.eq_ignore_ascii_case("option") {
        return Some(ControlKind::Option);
    }
    if !tag.eq_ignore_ascii_case("input") {
        return None;
    }
    let ty = dom.attr(id, "type").unwrap_or("").to_ascii_lowercase();
    Some(match ty.as_str() {
        "checkbox" => ControlKind::Checkbox,
        "radio" => ControlKind::Radio,
        "submit" | "button" | "reset" | "image" => ControlKind::Button,
        _ => ControlKind::TextInput,
    })
}

/// Concatenated text children, used for `<button>` and `<textarea>` values.
fn text_content(dom: &Dom, id: NodeId) -> String {
    let mut out = String::new();
    for &child in dom.render_children(id) {
        if let Some(text) = dom.text(child) {
            out.push_str(text);
        }
    }
    out
}

/// Wave 1: the first `selected` option's label, else the first option's label.
fn select_label(dom: &Dom, id: NodeId) -> String {
    let mut first = None;
    for &child in dom.render_children(id) {
        if !dom
            .tag(child)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("option"))
        {
            continue;
        }
        let label = text_content(dom, child);
        if dom.attr(child, "selected").is_some() {
            return label;
        }
        if first.is_none() {
            first = Some(label);
        }
    }
    first.unwrap_or_default()
}

/// Initial text carried by the control item: `value` for inputs/buttons,
/// text children for textarea/button, the chosen option label for select.
fn initial_text(dom: &Dom, id: NodeId, kind: ControlKind) -> String {
    match kind {
        ControlKind::Textarea | ControlKind::Button | ControlKind::Option => {
            let body = text_content(dom, id);
            if body.is_empty() {
                dom.attr(id, "value").unwrap_or("").to_string()
            } else {
                body
            }
        }
        ControlKind::Select => select_label(dom, id),
        _ => dom.attr(id, "value").unwrap_or("").to_string(),
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

/// Lay out one form control into a single atomic slot. Mirrors
/// `inline_block_atom`'s call shape; the caller flushes the parent line first
/// and uses the returned vertical space to advance its flow cursor.
pub(crate) fn layout_control_atom(
    ctx: &mut Ctx<'_>,
    style: &ComputedStyle,
    id: NodeId,
    avail: Avail,
    _link: Option<String>,
) -> f32 {
    let Some(kind) = control_kind(ctx.dom, id) else {
        return 0.0;
    };
    let checkable = matches!(kind, ControlKind::Checkbox | ControlKind::Radio);
    let fs = finite_or(style.font_size, 16.0).clamp(1.0, 200.0);
    let bw = finite_or(style.border_width, 0.0).clamp(0.0, 8.0);
    let m = &style.margin;
    let p = &style.padding;

    // shortcut: entirely zero padding means "author said nothing" for controls;
    // an explicit `padding:0` cannot be distinguished until the style map
    // carries declaration origin. Upgrade with UA-sheet origin tracking if a
    // page needs a flat control.
    let (pad_v, pad_h) = match kind {
        ControlKind::Button | ControlKind::Select => (8.0, 16.0),
        ControlKind::Checkbox | ControlKind::Radio => (0.0, 0.0),
        _ => (2.0, 3.0),
    };
    let custom_padding = p.top != 0.0 || p.right != 0.0 || p.bottom != 0.0 || p.left != 0.0;
    let (pad_top, pad_right, pad_bottom, pad_left) = if custom_padding {
        (
            p.top.max(0.0),
            p.right.max(0.0),
            p.bottom.max(0.0),
            p.left.max(0.0),
        )
    } else {
        (pad_v, pad_h, pad_v, pad_h)
    };

    let text = initial_text(ctx.dom, id, kind);
    let text_w = text.chars().count() as f32 * 0.6 * fs;
    let rows = ctx
        .dom
        .attr(id, "rows")
        .and_then(|v| v.parse::<f32>().ok())
        .map_or(4.0, |r| finite_or(r, 4.0).clamp(1.0, 24.0));
    // Defaults are *content* sizes; padding and border are added below, just
    // as block layout adds them around `style.height`.
    let input_h = 1.2 * fs;
    let (default_w, default_h) = match kind {
        ControlKind::Checkbox | ControlKind::Radio => (13.0, 13.0),
        ControlKind::Textarea => (12.0 * fs, rows * 1.2 * fs),
        ControlKind::Button | ControlKind::Select => {
            (text_w + pad_left + pad_right + 2.0 * bw + 12.0, input_h)
        }
        _ => (8.0 * fs, input_h),
    };

    let box_w = finite_or(style.width.unwrap_or(default_w), default_w)
        .max(style.min_size.w.max(0.0))
        .min(style.max_size.w.max(0.0))
        .max(0.0);
    // `style.height` is the content height here, matching block layout: the
    // control's padding and border are added outside it.
    let content_h = finite_or(style.height.unwrap_or(default_h), default_h)
        .max(style.min_size.h.max(0.0))
        .min(style.max_size.h.max(0.0))
        .max(0.0);
    let box_h = if checkable {
        // Checkboxes/radios stay square-ish regardless of inherited padding.
        content_h.max(1.0)
    } else {
        content_h + pad_top + pad_bottom + 2.0 * bw
    };

    let ty = ctx.dom.attr(id, "type").unwrap_or("").to_ascii_lowercase();
    let masked = ty == "password";
    ctx.list.push(
        id,
        DisplayItem::Control {
            rect: Rect {
                x: avail.x + m.left,
                y: avail.y + m.top,
                w: box_w,
                h: box_h,
            },
            kind,
            text,
            name: ctx.dom.attr(id, "name").unwrap_or("").to_string(),
            masked,
            disabled: ctx.dom.attr(id, "disabled").is_some(),
            readonly: ctx.dom.attr(id, "readonly").is_some(),
        },
    );

    m.top + box_h + m.bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_css::{Declaration, Selector, StyleRule, Stylesheet};
    use hodo_dom::NodeKind;
    use hodo_style::compute_styles;

    fn elem(dom: &mut Dom, parent: NodeId, tag: &str, attrs: &[(&str, &str)]) -> NodeId {
        let id = dom.create(NodeKind::Element {
            tag: tag.to_string(),
            attrs: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        dom.append(parent, id);
        id
    }

    fn text_node(dom: &mut Dom, parent: NodeId, value: &str) {
        let id = dom.create(NodeKind::Text(value.to_string()));
        dom.append(parent, id);
    }

    fn control(
        dom: &Dom,
        rules: Vec<StyleRule>,
    ) -> Vec<(ControlKind, String, String, Rect, bool, bool)> {
        let styles = compute_styles(dom, &[Stylesheet { rules }]);
        layout(dom, &styles, Size { w: 800.0, h: 600.0 })
            .items
            .into_iter()
            .filter_map(|item| match item {
                DisplayItem::Control {
                    rect,
                    kind,
                    text,
                    name,
                    masked,
                    disabled,
                    ..
                } => Some((kind, text, name, rect, masked, disabled)),
                _ => None,
            })
            .collect()
    }

    fn rule(tag: &str, decls: &[(&str, &str)]) -> StyleRule {
        StyleRule {
            selector: Selector {
                tag: Some(tag.to_string()),
                id: None,
                classes: Vec::new(),
                universal: false,
            },
            declarations: decls
                .iter()
                .map(|(p, v)| Declaration {
                    property: p.to_string(),
                    value: v.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn text_input_uses_value_attr_and_carries_semantics() {
        let mut dom = Dom::new();
        let root = dom.root();
        elem(
            &mut dom,
            root,
            "input",
            &[
                ("type", "password"),
                ("value", "secret"),
                ("name", "pass"),
                ("disabled", ""),
            ],
        );
        let items = control(&dom, vec![]);
        assert_eq!(items.len(), 1, "one control item per input");
        let (kind, text, name, rect, masked, disabled) = &items[0];
        assert_eq!(*kind, ControlKind::TextInput);
        assert_eq!(text, "secret");
        assert_eq!(name, "pass");
        assert!(*masked, "type=password masks its glyphs");
        assert!(*disabled);
        assert!(rect.w > 0.0 && rect.h > 0.0, "sane default box: {rect:?}");
    }

    #[test]
    fn checkbox_and_radio_are_small_squares() {
        let mut dom = Dom::new();
        let root = dom.root();
        elem(
            &mut dom,
            root,
            "input",
            &[("type", "checkbox"), ("checked", "")],
        );
        elem(
            &mut dom,
            root,
            "input",
            &[("type", "radio"), ("name", "size"), ("value", "s")],
        );
        let items = control(&dom, vec![]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, ControlKind::Checkbox);
        assert_eq!(items[1].0, ControlKind::Radio);
        assert!(items[0].3.w <= 14.0 && items[0].3.h <= 14.0);
        assert!((items[1].3.w - 13.0).abs() < 0.01);
        assert_eq!(items[1].2, "size");
        assert_eq!(items[1].1, "s");
    }

    #[test]
    fn button_gets_padding_and_submit_value() {
        let mut dom = Dom::new();
        let root = dom.root();
        elem(
            &mut dom,
            root,
            "input",
            &[("type", "submit"), ("value", "Send")],
        );
        let plain = elem(&mut dom, root, "button", &[("type", "button")]);
        text_node(&mut dom, plain, "Plain button");
        let items = control(&dom, vec![]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, ControlKind::Button);
        assert_eq!(items[0].1, "Send");
        assert_eq!(items[1].1, "Plain button");
        // 1.2 * 16 content + 8px padding on both sides.
        assert!(
            (items[0].3.h - 35.2).abs() < 0.1,
            "button height: {:?}",
            items[0].3
        );
        assert!(
            items[0].3.w > 32.0,
            "button width includes 16px side padding"
        );
    }

    #[test]
    fn select_shows_one_selected_label_and_suppresses_options() {
        let mut dom = Dom::new();
        let root = dom.root();
        let select = elem(&mut dom, root, "select", &[("name", "pick")]);
        let red = elem(&mut dom, select, "option", &[]);
        text_node(&mut dom, red, "Red");
        let green = elem(&mut dom, select, "option", &[("selected", "")]);
        text_node(&mut dom, green, "Green");
        let blue = elem(&mut dom, select, "option", &[]);
        text_node(&mut dom, blue, "Blue");
        let items = control(&dom, vec![]);
        assert_eq!(items.len(), 1, "options never emit their own control item");
        assert_eq!(items[0].0, ControlKind::Select);
        assert_eq!(items[0].1, "Green");
        assert_eq!(items[0].2, "pick");
    }

    #[test]
    fn textarea_uses_text_children_and_rows() {
        let mut dom = Dom::new();
        let root = dom.root();
        let area = elem(&mut dom, root, "textarea", &[("rows", "2")]);
        text_node(&mut dom, area, "hello world");
        let items = control(&dom, vec![]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0, ControlKind::Textarea);
        assert_eq!(items[0].1, "hello world");
        // 2 rows * 1.2 * 16 content + 2px top/bottom padding.
        assert!(
            (items[0].3.h - 42.4).abs() < 0.1,
            "rows=2: {:?}",
            items[0].3
        );
    }

    #[test]
    fn style_padding_border_and_size_apply_to_control() {
        let mut dom = Dom::new();
        let root = dom.root();
        elem(&mut dom, root, "input", &[("type", "text"), ("value", "x")]);
        let items = control(
            &dom,
            vec![rule(
                "input",
                &[
                    ("width", "100px"),
                    ("height", "20px"),
                    ("padding", "0 5px"),
                    ("border", "1px solid #000000"),
                    ("margin-left", "7px"),
                ],
            )],
        );
        let rect = items[0].3;
        assert!((rect.w - 100.0).abs() < 0.01, "width from style: {rect:?}");
        assert!(
            (rect.h - 22.0).abs() < 0.01,
            "content height + borders: {rect:?}"
        );
        assert!(
            (rect.x - 7.0).abs() < 0.01,
            "margin shifts the slot: {rect:?}"
        );
    }
}

//! M7.3 form interaction state for the active page.
//!
//! The shell's low-RAM contract drops the DOM after each page load, so the
//! registry keeps only flat control state (value, caret, checked, focus). Layout
//! still owns geometry and the initial value from the DOM; this module owns
//! everything that changes while the user interacts with the page.
//!
//! Wave 1 simplifications (documented for the integrator):
//! - no select popup list (the Select item shows one label);
//! - no reset button behaviour;
//! - no caret/focus ring in hodo-render — focus is registry-level only;
//! - POST responses render as a navigate-by-response with no history entry;
//! - a live `FormPage` re-parses the page once on load, only when the page's
//!   display list actually contains a Control item (zero cost on other pages).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use hodo_dom::{Dom, NodeId};
use hodo_layout::{ControlKind, DisplayItem, DisplayList};
use hodo_net::Resource;
use hodo_types::{Rect, Size};

/// One editable/focusable control, in display-list order.
#[derive(Debug, Clone)]
pub(crate) struct ControlState {
    pub(crate) rect: Rect,
    pub(crate) kind: ControlKind,
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) checked: bool,
    pub(crate) submit: bool,
    /// Character index into `value` (not a byte index).
    pub(crate) caret: usize,
    pub(crate) disabled: bool,
    pub(crate) readonly: bool,
}

impl ControlState {
    fn editable(&self) -> bool {
        matches!(self.kind, ControlKind::TextInput | ControlKind::Textarea)
            && !self.disabled
            && !self.readonly
    }
}

/// Metadata that layout does not carry in `DisplayItem::Control`.
#[derive(Debug, Clone, Default)]
struct ControlMeta {
    name: String,
    checked: bool,
    submit: bool,
}

/// Flat interaction state of one page's controls.
#[derive(Debug, Clone, Default)]
pub(crate) struct ControlRegistry {
    pub(crate) controls: Vec<ControlState>,
    pub(crate) focus: Option<usize>,
    form_action: String,
    form_method: String,
}

fn byte_at(value: &str, caret: usize) -> usize {
    value
        .char_indices()
        .nth(caret)
        .map_or(value.len(), |(index, _)| index)
}

impl ControlRegistry {
    fn from_list(list: &DisplayList, meta: &[ControlMeta], action: &str, method: &str) -> Self {
        let mut controls = Vec::new();
        for item in &list.items {
            let DisplayItem::Control {
                rect,
                kind,
                text,
                name,
                masked: _,
                disabled,
                readonly,
            } = item
            else {
                continue;
            };
            let extra = meta.get(controls.len());
            controls.push(ControlState {
                rect: *rect,
                kind: *kind,
                // `id` is a wave-1 fallback so unnamed demo fields still
                // produce useful GET/POST pairs.
                name: if name.is_empty() {
                    extra.map_or_else(String::new, |m| m.name.clone())
                } else {
                    name.clone()
                },
                value: text.clone(),
                checked: extra.is_some_and(|m| m.checked),
                submit: extra.is_some_and(|m| m.submit),
                caret: text.chars().count(),
                disabled: *disabled,
                readonly: *readonly,
            });
        }
        Self {
            controls,
            focus: None,
            form_action: action.to_string(),
            form_method: method.to_string(),
        }
    }

    /// Cheap change probe, mirroring the popup agent's DOM fingerprint: when
    /// this changes the page is re-styled, re-laid out and re-rendered.
    pub(crate) fn forms_state_fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.focus.hash(&mut hasher);
        for control in &self.controls {
            control.name.hash(&mut hasher);
            control.value.hash(&mut hasher);
            control.checked.hash(&mut hasher);
            control.caret.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Focus the topmost control under the point. Returns true when anything
    /// changed (including a checkbox/radio toggle).
    pub(crate) fn click(&mut self, index: usize) -> bool {
        let Some(control) = self.controls.get(index) else {
            return false;
        };
        if control.disabled {
            return false;
        }
        let (kind, name, checked) = (control.kind, control.name.clone(), control.checked);
        let mut changed = self.focus != Some(index);
        self.focus = Some(index);
        match kind {
            ControlKind::Checkbox => {
                self.controls[index].checked = !checked;
                changed = true;
            }
            ControlKind::Radio if !checked => {
                // Unnamed radios have no group to be exclusive in: HTML treats
                // each as its own group, so skip the peer deselect loop.
                if !name.is_empty() {
                    for other in &mut self.controls {
                        if other.kind == ControlKind::Radio && other.name == name {
                            other.checked = false;
                        }
                    }
                }
                self.controls[index].checked = true;
                changed = true;
            }
            _ => {}
        }
        changed
    }

    /// Move focus to the next enabled control, wrapping at the end.
    pub(crate) fn tab(&mut self) -> bool {
        if self.controls.is_empty() {
            return false;
        }
        let len = self.controls.len();
        let start = self.focus.unwrap_or(len - 1);
        for step in 1..=len {
            let index = (start + step) % len;
            if !self.controls[index].disabled {
                if self.focus == Some(index) {
                    return false;
                }
                self.focus = Some(index);
                return true;
            }
        }
        false
    }

    fn focused(&mut self) -> Option<&mut ControlState> {
        self.focus.and_then(|index| self.controls.get_mut(index))
    }

    /// Insert text at the caret of the focused text field.
    pub(crate) fn insert(&mut self, text: &str) -> bool {
        let Some(control) = self.focused() else {
            return false;
        };
        if !control.editable() {
            return false;
        }
        control.caret = control.caret.min(control.value.chars().count());
        let at = byte_at(&control.value, control.caret);
        control.value.insert_str(at, text);
        control.caret += text.chars().count();
        true
    }

    /// Delete the character before the caret of the focused text field.
    pub(crate) fn backspace(&mut self) -> bool {
        let Some(control) = self.focused() else {
            return false;
        };
        if !control.editable() || control.caret == 0 {
            return false;
        }
        control.caret = control.caret.min(control.value.chars().count());
        let at = byte_at(&control.value, control.caret);
        let prev = control.value[..at]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index);
        control.value.replace_range(prev..at, "");
        control.caret -= 1;
        true
    }

    /// True when Enter in the focused field should submit the form.
    pub(crate) fn enter_submits(&self) -> bool {
        self.focus.is_some_and(|index| {
            self.controls
                .get(index)
                .is_some_and(|control| control.kind == ControlKind::TextInput && !control.disabled)
        })
    }

    fn urlencode(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char)
                }
                b' ' => out.push('+'),
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
        out
    }

    /// `application/x-www-form-urlencoded` pairs for successful controls.
    fn encoded_body(&self) -> String {
        let mut pairs = Vec::new();
        for control in &self.controls {
            if control.disabled || control.name.is_empty() {
                continue;
            }
            let include = match control.kind {
                ControlKind::Checkbox | ControlKind::Radio => control.checked,
                ControlKind::Button | ControlKind::Option => false,
                _ => true,
            };
            if include {
                pairs.push(format!(
                    "{}={}",
                    Self::urlencode(&control.name),
                    Self::urlencode(&control.value)
                ));
            }
        }
        pairs.join("&")
    }
}

/// Result of activating a submit control.
pub(crate) enum FormAction {
    /// GET: a normal navigation with the query string attached.
    Navigate(String),
    /// POST: the response body already run through the page pipeline.
    Response {
        title: String,
        list: DisplayList,
        /// Live registry rebuilt from the response HTML when it has controls.
        /// Boxed to keep `FormAction` small — a `FormPage` owns a whole DOM.
        form: Option<Box<FormPage>>,
    },
}

/// `join_url` is document-relative; a form action beginning with `/` is
/// root-relative and must keep only the scheme and authority of the page URL.
fn resolve_action(base: &str, action: &str) -> String {
    if action.starts_with('/') && (base.starts_with("http://") || base.starts_with("https://")) {
        let scheme_end = base.find("://").map_or(0, |index| index + 3);
        let path_start = base[scheme_end..]
            .find('/')
            .map_or(base.len(), |index| scheme_end + index);
        return format!("{}{}", &base[..path_start], action);
    }
    crate::app::join_url(base, action)
}

fn text_content(dom: &Dom, id: NodeId) -> String {
    let mut out = String::new();
    for &child in dom.children(id) {
        if let Some(text) = dom.text(child) {
            out.push_str(text);
        }
    }
    out
}

/// Control metadata the frozen display-list contract does not carry.
fn collect_meta(dom: &Dom) -> Vec<ControlMeta> {
    let mut out = Vec::new();
    dom.walk(|id, _| {
        // Interned DOMs keep no tag bytes in `Node`, so always use the accessor.
        let Some(tag) = dom.tag(id) else {
            return;
        };
        if !["input", "button", "select", "textarea"]
            .iter()
            .any(|name| tag.eq_ignore_ascii_case(name))
        {
            return;
        }
        let ty = dom.attr(id, "type").unwrap_or("");
        out.push(ControlMeta {
            name: dom
                .attr(id, "name")
                .filter(|name| !name.is_empty())
                .or_else(|| dom.attr(id, "id"))
                .unwrap_or("")
                .to_string(),
            checked: dom.attr(id, "checked").is_some() || dom.attr(id, "selected").is_some(),
            submit: ty.eq_ignore_ascii_case("submit")
                || (tag.eq_ignore_ascii_case("button")
                    && (ty.is_empty() || ty.eq_ignore_ascii_case("submit"))),
        });
    });
    out
}

/// First `<form>`'s action and method, read from the live DOM at load time.
fn collect_form(dom: &Dom) -> (String, String) {
    let mut action = String::new();
    let mut method = String::new();
    dom.walk(|id, _| {
        if action.is_empty()
            && dom
                .tag(id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("form"))
        {
            action = dom.attr(id, "action").unwrap_or("").to_string();
            method = dom.attr(id, "method").unwrap_or("get").to_string();
        }
    });
    (action, method)
}

/// Run the static page pipeline over an already-fetched HTML body.
fn render_html(html: &str, url: &str, viewport: Size) -> (String, DisplayList) {
    let dom = hodo_html::parse_document(html);
    let stylesheet = crate::app::collect_styles(&dom, url, viewport);
    let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
    let list = hodo_layout::layout(&dom, &styles, viewport);
    let mut title = String::new();
    let mut found = false;
    dom.walk(|id, _| {
        if !found
            && dom
                .tag(id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("title"))
        {
            found = true;
            title = text_content(&dom, id);
        }
    });
    (title, list)
}

/// Live page + control registry. Rebuilt on navigation, dropped on tab close.
pub(crate) struct FormPage {
    dom: Dom,
    base_url: String,
    viewport: Size,
    pub(crate) registry: ControlRegistry,
    fingerprint: u64,
    list: DisplayList,
}

impl FormPage {
    /// Load a page that actually has controls; `None` for every other page.
    pub(crate) fn load(url: &str, viewport: Size) -> Option<Self> {
        let html = match crate::app::load_resource(url) {
            Ok(Resource::Html(text)) => text,
            Ok(Resource::Other { data, .. }) => String::from_utf8_lossy(&data).into_owned(),
            Err(_) => return None,
        };
        Self::from_html(&html, url, viewport)
    }

    /// Same pipeline as [`Self::load`] for an HTML body we already have, e.g. a
    /// POST response. Keeps the DOM so the registry survives past submit.
    pub(crate) fn from_html(html: &str, url: &str, viewport: Size) -> Option<Self> {
        let mut dom = hodo_html::parse_document(html);
        // Same relative-src fix as app::load_page (layout has no base URL).
        let mut relative_images = Vec::new();
        dom.walk(|id, _| {
            if dom
                .tag(id)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("img"))
            {
                if let Some(src) = dom.attr(id, "src") {
                    if !src.contains("://") && !src.starts_with("data:") {
                        relative_images.push((id, crate::app::join_url(url, src)));
                    }
                }
            }
        });
        for (id, src) in relative_images {
            dom.set_attr(id, "src", src);
        }

        let stylesheet = crate::app::collect_styles(&dom, url, viewport);
        let styles = hodo_style::compute_styles(&dom, &[stylesheet]);
        let list = hodo_layout::layout(&dom, &styles, viewport);
        if !list
            .items
            .iter()
            .any(|item| matches!(item, DisplayItem::Control { .. }))
        {
            return None;
        }
        let meta = collect_meta(&dom);
        let (action, method) = collect_form(&dom);
        let registry = ControlRegistry::from_list(&list, &meta, &action, &method);
        let fingerprint = registry.forms_state_fingerprint();
        Some(Self {
            dom,
            base_url: url.to_string(),
            viewport,
            registry,
            fingerprint,
            list,
        })
    }

    pub(crate) fn display_list(&self) -> &DisplayList {
        &self.list
    }

    pub(crate) fn set_viewport(&mut self, viewport: Size) {
        if self.viewport == viewport {
            return;
        }
        self.viewport = viewport;
        self.relayout();
    }

    /// Re-run style/layout over the retained DOM and repaint registry values on
    /// the fresh control items. Rendering itself is the shell's normal compose
    /// path, driven by the next redraw.
    fn relayout(&mut self) {
        let stylesheet = crate::app::collect_styles(&self.dom, &self.base_url, self.viewport);
        let styles = hodo_style::compute_styles(&self.dom, &[stylesheet]);
        self.list = hodo_layout::layout(&self.dom, &styles, self.viewport);
        // Values live in the registry, not the DOM: paint the current text on
        // top of the freshly laid out control items. Geometry lives in layout,
        // so copy each fresh rect back too, or clicks keep hitting the old
        // positions after a resize.
        let before: Vec<(ControlKind, String)> = self
            .registry
            .controls
            .iter()
            .map(|control| (control.kind, control.name.clone()))
            .collect();
        let focused = self
            .registry
            .focus
            .and_then(|index| before.get(index).cloned());
        let mut after: Vec<(ControlKind, String)> = Vec::new();
        for item in &mut self.list.items {
            if let DisplayItem::Control {
                rect,
                kind,
                text,
                name,
                ..
            } = item
            {
                after.push((*kind, name.clone()));
                let index = after.len() - 1;
                if let Some(control) = self.registry.controls.get_mut(index) {
                    control.rect = *rect;
                    *text = control.value.clone();
                }
            }
        }
        if before != after {
            // Only a changed control set/order invalidates the focus index.
            let action = self.registry.form_action.clone();
            let method = self.registry.form_method.clone();
            let meta = collect_meta(&self.dom);
            let mut fresh = ControlRegistry::from_list(&self.list, &meta, &action, &method);
            fresh.focus = focused.and_then(|(kind, name)| {
                fresh
                    .controls
                    .iter()
                    .position(|control| control.kind == kind && control.name == name)
            });
            self.registry = fresh;
        }
    }

    /// Re-run style/layout when interaction state changed. Returns true when
    /// the display list was replaced (so the caller can redraw).
    pub(crate) fn tick(&mut self) -> bool {
        let fingerprint = self.registry.forms_state_fingerprint();
        if fingerprint == self.fingerprint {
            return false;
        }
        self.fingerprint = fingerprint;
        self.relayout();
        true
    }

    /// Whether a control occupies this point, in page coordinates.
    pub(crate) fn hit(&self, x: f32, y: f32, scroll: f32) -> bool {
        let rects: Vec<Rect> = self.registry.controls.iter().map(|c| c.rect).collect();
        crate::ui::hit_test_control(&rects, x, y + scroll).is_some()
    }

    /// Left-click at viewport coordinates. `scroll` is the page scroll offset.
    /// Returns a submit action when a submit control was activated.
    pub(crate) fn click(&mut self, x: f32, y: f32, scroll: f32) -> Option<FormAction> {
        let rects: Vec<Rect> = self.registry.controls.iter().map(|c| c.rect).collect();
        let index = crate::ui::hit_test_control(&rects, x, y + scroll)?;
        let submit = self.registry.controls.get(index).is_some_and(|c| c.submit);
        let _ = self.registry.click(index);
        if submit {
            self.submit()
        } else {
            None
        }
    }

    pub(crate) fn tab(&mut self) -> bool {
        self.registry.tab()
    }

    pub(crate) fn insert(&mut self, text: &str) -> bool {
        self.registry.insert(text)
    }

    pub(crate) fn backspace(&mut self) -> bool {
        self.registry.backspace()
    }

    pub(crate) fn enter(&mut self) -> Option<FormAction> {
        if self.registry.enter_submits() {
            self.submit()
        } else {
            None
        }
    }

    fn submit(&self) -> Option<FormAction> {
        let body = self.registry.encoded_body();
        let action = if self.registry.form_action.is_empty() {
            self.base_url.clone()
        } else {
            resolve_action(&self.base_url, &self.registry.form_action)
        };
        if self.registry.form_method.eq_ignore_ascii_case("post") {
            let resource = match hodo_net::post(
                &action,
                body.as_bytes(),
                "application/x-www-form-urlencoded",
            ) {
                Ok(resource) => resource,
                Err(err) => {
                    eprintln!("hodo: form submit failed: {err}");
                    return None;
                }
            };
            let html = match resource {
                Resource::Html(text) => text,
                Resource::Other { data, .. } => String::from_utf8_lossy(&data).into_owned(),
            };
            let (title, list) = render_html(&html, &action, self.viewport);
            // The response may itself contain controls: keep a live registry so
            // the shell does not orphan form state on every POST.
            let form = FormPage::from_html(&html, &action, self.viewport).map(Box::new);
            Some(FormAction::Response { title, list, form })
        } else {
            let url = if body.is_empty() {
                action
            } else {
                let sep = if action.contains('?') { '&' } else { '?' };
                format!("{action}{sep}{body}")
            };
            Some(FormAction::Navigate(url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn control(kind: ControlKind, name: &str, value: &str, rect: Rect) -> DisplayItem {
        DisplayItem::Control {
            rect,
            kind,
            text: value.to_string(),
            name: name.to_string(),
            masked: false,
            disabled: false,
            readonly: false,
        }
    }

    fn rect(x: f32) -> Rect {
        Rect {
            x,
            y: 0.0,
            w: 20.0,
            h: 20.0,
        }
    }

    fn registry(items: Vec<DisplayItem>, meta: Vec<ControlMeta>) -> ControlRegistry {
        ControlRegistry::from_list(
            &DisplayList {
                items,
                node_map: Vec::new(),
                fixed_items: Vec::new(),
                fixed_node_map: Vec::new(),
            },
            &meta,
            "/submit",
            "get",
        )
    }

    #[test]
    fn tab_cycles_focus_and_skips_disabled() {
        let mut items = vec![
            control(ControlKind::TextInput, "a", "one", rect(0.0)),
            control(ControlKind::Button, "b", "two", rect(30.0)),
            control(ControlKind::TextInput, "c", "three", rect(60.0)),
        ];
        if let DisplayItem::Control { disabled, .. } = &mut items[1] {
            *disabled = true;
        }
        let mut reg = registry(items, vec![]);
        assert_eq!(reg.focus, None);
        assert!(reg.tab());
        assert_eq!(reg.focus, Some(0));
        assert!(reg.tab());
        assert_eq!(reg.focus, Some(2), "disabled control must be skipped");
        assert!(reg.tab());
        assert_eq!(reg.focus, Some(0), "focus wraps at the end");
    }

    #[test]
    fn radio_group_is_exclusive_and_checkbox_toggles() {
        let items = vec![
            control(ControlKind::Checkbox, "remember", "on", rect(0.0)),
            control(ControlKind::Radio, "size", "s", rect(30.0)),
            control(ControlKind::Radio, "size", "l", rect(60.0)),
        ];
        let meta = vec![
            ControlMeta {
                checked: true,
                ..ControlMeta::default()
            },
            ControlMeta {
                checked: true,
                ..ControlMeta::default()
            },
            ControlMeta::default(),
        ];
        let mut reg = registry(items, meta.clone());
        assert!(reg.controls[0].checked && reg.controls[1].checked);
        assert!(reg.click(2), "clicking an unchecked radio changes state");
        assert!(!reg.controls[1].checked, "same-name radio is deselected");
        assert!(reg.controls[2].checked);
        assert!(reg.controls[0].checked, "other groups are untouched");
        assert!(!reg.click(2), "re-clicking a checked radio is a no-op");
        assert!(reg.click(0));
        assert!(!reg.controls[0].checked, "checkbox toggles off");
    }

    #[test]
    fn unnamed_radios_are_independent_groups() {
        let items = vec![
            control(ControlKind::Radio, "", "", rect(0.0)),
            control(ControlKind::Radio, "", "", rect(30.0)),
        ];
        let mut reg = registry(items, vec![]);
        assert!(reg.controls.iter().all(|control| control.name.is_empty()));
        assert!(reg.click(0));
        assert!(reg.click(1));
        assert!(
            reg.controls.iter().all(|control| control.checked),
            "unnamed radios must not deselect each other"
        );
    }

    #[test]
    fn relayout_refreshes_hit_rects_after_viewport_resize() {
        let html = "<html><body><form><input id='wide' name='q' value='x'></form></body></html>";
        let mut page =
            FormPage::from_html(html, "http://example.test/", Size { w: 400.0, h: 300.0 })
                .expect("page with one control");
        // Simulate the bug: the registry kept a pre-resize rect while the
        // display list is re-laid out at the new viewport.
        page.registry.controls[0].rect = Rect {
            x: 9999.0,
            y: 9999.0,
            w: 20.0,
            h: 20.0,
        };
        let new_size = Size {
            w: 1200.0,
            h: 800.0,
        };
        page.set_viewport(new_size);
        let new_rect = page.registry.controls[0].rect;
        let list_rect = page
            .display_list()
            .items
            .iter()
            .find_map(|item| match item {
                DisplayItem::Control { rect, .. } => Some(*rect),
                _ => None,
            })
            .expect("fresh control item");
        assert_eq!(new_rect, list_rect, "registry must track the new layout");
        assert_ne!(new_rect.x, 9999.0, "the stale rect must be replaced");
        assert!(
            page.hit(new_rect.x + 1.0, new_rect.y + 1.0, 0.0),
            "a click at the NEW position must land"
        );
        assert!(
            !page.hit(9999.0 + 1.0, 9999.0 + 1.0, 0.0),
            "the stale rect must no longer be a hit target"
        );
    }

    #[test]
    fn caret_edit_and_fingerprint_track_dirty_state() {
        let items = vec![control(ControlKind::TextInput, "q", "ab", rect(0.0))];
        let mut reg = registry(items, vec![]);
        let clean = reg.forms_state_fingerprint();
        assert!(reg.tab());
        assert!(reg.insert("X"));
        assert_eq!(reg.controls[0].value, "abX");
        assert_eq!(reg.controls[0].caret, 3);
        assert_ne!(clean, reg.forms_state_fingerprint(), "value edit is dirty");
        assert!(reg.backspace());
        assert_eq!(reg.controls[0].value, "ab");
        assert_eq!(reg.controls[0].caret, 2);
        let edited = reg.forms_state_fingerprint();
        assert_eq!(
            edited,
            reg.forms_state_fingerprint(),
            "stable between edits"
        );
    }

    #[test]
    fn get_submit_encodes_values_and_uses_form_action() {
        let items = vec![
            control(ControlKind::TextInput, "q", "a b&c", rect(0.0)),
            control(ControlKind::Checkbox, "ok", "on", rect(30.0)),
            control(ControlKind::Checkbox, "off", "on", rect(60.0)),
        ];
        let meta = vec![
            ControlMeta::default(),
            ControlMeta {
                checked: true,
                ..ControlMeta::default()
            },
            ControlMeta::default(),
        ];
        let reg = registry(items, meta);
        assert_eq!(reg.encoded_body(), "q=a+b%26c&ok=on");
        let page = FormPage {
            dom: Dom::new(),
            base_url: "http://example.test/page.html".to_string(),
            viewport: crate::app::DEFAULT_VIEWPORT,
            registry: reg,
            fingerprint: 0,
            list: DisplayList::default(),
        };
        let Some(FormAction::Navigate(url)) = page.submit() else {
            panic!("GET must navigate");
        };
        assert_eq!(url, "http://example.test/submit?q=a+b%26c&ok=on");
    }

    fn fixture_html() -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/forms-site/index.html");
        std::fs::read_to_string(path).expect("forms-site fixture exists")
    }

    #[test]
    fn forms_site_fixture_renders_all_controls() {
        let html = fixture_html();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/forms-site/index.html");
        let url = path.to_string_lossy().into_owned();
        let (_, list) = render_html(&html, &url, crate::app::DEFAULT_VIEWPORT);
        let controls: Vec<_> = list
            .items
            .iter()
            .filter_map(|item| match item {
                DisplayItem::Control {
                    rect, kind, text, ..
                } => Some((*kind, text.as_str(), *rect)),
                _ => None,
            })
            .collect();
        assert_eq!(controls.len(), 11, "fixture has 11 controls: {controls:?}");
        assert!(controls
            .iter()
            .any(|c| c.0 == ControlKind::TextInput && c.1 == "alice"));
        assert!(controls
            .iter()
            .any(|c| c.0 == ControlKind::TextInput && c.1 == "secret"));
        assert!(controls
            .iter()
            .any(|c| c.0 == ControlKind::Textarea && c.1 == "hello world"));
        assert!(controls
            .iter()
            .any(|c| c.0 == ControlKind::Select && c.1 == "Red"));
        assert!(controls
            .iter()
            .any(|c| c.0 == ControlKind::Button && c.1 == "Send"));
        assert_eq!(
            controls
                .iter()
                .filter(|(kind, _, _)| *kind == ControlKind::Radio)
                .count(),
            2
        );

        // Pixel check: every control box has opaque pixels and a visible
        // border/glyph, not just the surrounding prose.
        let frame = hodo_render::render(&list, crate::app::DEFAULT_VIEWPORT);
        assert_eq!(frame.len(), 1200 * 800 * 4, "frame matches viewport");
        for (_, _, r) in &controls {
            let mut opaque = 0;
            let mut ink = 0;
            for y in (r.y as usize)..((r.y + r.h) as usize).min(800) {
                for x in (r.x as usize)..((r.x + r.w) as usize).min(1200) {
                    let pixel = &frame[(y * 1200 + x) * 4..][..4];
                    if pixel[3] > 0 {
                        opaque += 1;
                        if pixel[..3] != [255, 255, 255] {
                            ink += 1;
                        }
                    }
                }
            }
            assert!(opaque >= 4, "control box must be painted: {r:?}");
            assert!(ink >= 2, "control border/glyph must be visible: {r:?}");
        }
    }

    // ---- loopback submit e2e ------------------------------------------------

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Recorded {
        method: String,
        path: String,
        content_type: Option<String>,
        body: String,
    }

    fn serve(script: Vec<(String, String)>) -> (String, Arc<Mutex<Vec<Recorded>>>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind");
        let port = listener.local_addr().expect("local addr").port();
        let url = format!("http://127.0.0.1:{port}/form");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&seen);
        let join = std::thread::spawn(move || {
            for (content_type, body) in script {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let header_end = loop {
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        break pos + 4;
                    }
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => return,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                };
                let head = String::from_utf8_lossy(&buf[..header_end]).into_owned();
                let mut lines = head.split("\r\n");
                let request_line = lines.next().unwrap_or_default();
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or_default().to_string();
                let path = parts.next().unwrap_or_default().to_string();
                let mut content_length = 0usize;
                let mut req_type = None;
                for line in lines {
                    let Some((name, value)) = line.split_once(':') else {
                        continue;
                    };
                    match name.trim().to_ascii_lowercase().as_str() {
                        "content-length" => {
                            content_length = value.trim().parse().unwrap_or(0);
                        }
                        "content-type" => req_type = Some(value.trim().to_string()),
                        _ => {}
                    }
                }
                while buf.len() < header_end + content_length {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                }
                let end = (header_end + content_length).min(buf.len());
                recorded.lock().expect("mutex").push(Recorded {
                    method,
                    path,
                    content_type: req_type,
                    body: String::from_utf8_lossy(&buf[header_end..end]).into_owned(),
                });
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        (url, seen, join)
    }

    const FORM_HTML: &str = "<html><head><title>form</title></head><body>\
        <form action=\"/submit\" method=\"post\">\
        <input name=\"user\" value=\"alice\">\
        <input name=\"pass\" type=\"password\" value=\"secret\">\
        <input type=\"submit\" value=\"Go\">\
        </form></body></html>";

    #[test]
    fn post_submit_sends_body_and_renders_response() {
        let (url, seen, join) = serve(vec![
            ("text/html".to_string(), FORM_HTML.to_string()),
            (
                "text/html".to_string(),
                "<html><title>done</title><p>posted ok</p>\
                 <form action=\"/again\"><input name=\"again\" value=\"again\"></form></html>"
                    .to_string(),
            ),
        ]);
        let mut page = FormPage::load(&url, crate::app::DEFAULT_VIEWPORT).expect("form page");
        assert_eq!(page.registry.controls.len(), 3);
        assert!(page.tab(), "Tab focuses the first text field");
        let Some(FormAction::Response { title, list, form }) = page.enter() else {
            panic!("POST must render a response");
        };
        join.join().expect("server thread");
        assert_eq!(title, "done");
        assert!(
            list.items.iter().any(|item| match item {
                DisplayItem::Text { text, .. } => text == "posted ok",
                _ => false,
            }),
            "response body must go through the page pipeline"
        );
        let response_form = form.expect("POST response controls keep a live form page");
        assert_eq!(response_form.registry.controls.len(), 1);
        assert_eq!(response_form.registry.controls[0].name, "again");
        let rect = response_form.registry.controls[0].rect;
        assert!(
            response_form.hit(rect.x + 1.0, rect.y + 1.0, 0.0),
            "the response registry's rect must be clickable"
        );
        let requests = seen.lock().expect("mutex");
        assert_eq!(requests.len(), 2, "first request loads the page");
        let post = requests.last().expect("POST recorded");
        assert_eq!(post.method, "POST");
        assert_eq!(post.path, "/submit");
        assert_eq!(
            post.content_type.as_deref(),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(post.body, "user=alice&pass=secret");
    }

    #[test]
    fn get_submit_builds_query_url() {
        let html = FORM_HTML.replace("method=\"post\"", "method=\"get\"");
        let (url, seen, join) = serve(vec![("text/html".to_string(), html)]);
        let mut page = FormPage::load(&url, crate::app::DEFAULT_VIEWPORT).expect("form page");
        assert!(page.tab(), "Tab focuses the first control");
        let Some(FormAction::Navigate(target)) = page.enter() else {
            panic!("GET must navigate");
        };
        join.join().expect("server thread");
        assert!(target.starts_with("http://127.0.0.1:"), "{target}");
        assert!(
            target.contains("/submit?user=alice&pass=secret"),
            "{target}"
        );
        assert_eq!(seen.lock().expect("mutex").len(), 1);
    }
}

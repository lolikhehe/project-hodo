//! Toolbar action metadata and the in-process popup overlay.
//!
//! A popup is *not* a second window. It is one offscreen ARGB8888 buffer kept
//! in `BrowserApp` while open, rendered through the same parse → styles →
//! layout → render pipeline as a page, and composited over the main frame.
//!
//! Wave-1 limits (deliberate, see the M2.2 brief):
//!   * toolbar icons are title-only text slots — `hodo_render::draw_image` and
//!     `decode_png` are private to the render crate, so icon pixels are deferred;
//!   * popup clicks hit-test `DisplayItem::Text` by geometry only. There is no
//!     layout item→node map yet, so events are synthesized on the DOM root
//!     (see the `TODO(page-JS)` in [`PopupState::click`]).

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use hodo_dom::{Dom, NodeKind};
use hodo_js::ExtScriptRuntime;
use hodo_layout::{DisplayItem, DisplayList};
use hodo_types::{Rect, Size};
use serde_json::{json, Value};

/// Default popup viewport. The buffer is always this size; only the
/// composited rectangle is clamped to the host window.
pub(crate) const POPUP_W: f32 = 360.0;
pub(crate) const POPUP_H: f32 = 480.0;
const POPUP_VIEWPORT: Size = Size {
    w: POPUP_W,
    h: POPUP_H,
};

/// Upper bound on buffered `sendResponse` pairs while no ext-host broker is
/// draining them. Oldest pairs are dropped first; `dropped_replies()` exposes
/// the count for diagnostics/tests.
const MAX_REPLIES: usize = 256;
static DROPPED_REPLIES: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn dropped_replies() -> usize {
    DROPPED_REPLIES.load(Ordering::Relaxed)
}

/// Attribute values a popup's style/layout/JS pipeline can actually consult.
/// Interned DOMs have no public attribute iterator, so enumerate the names
/// instead of hashing the raw `NodeKind::Element.attrs` vector. All attributes
/// read downstream (style, layout, forms and the relative-URL fixups) are here;
/// unknown `data-*` attributes are deliberately not part of the fingerprint.
const FINGERPRINT_ATTRS: &[&str] = &[
    "id",
    "class",
    "style",
    "href",
    "src",
    "rel",
    "action",
    "method",
    "name",
    "value",
    "type",
    "checked",
    "selected",
    "disabled",
    "readonly",
    "placeholder",
    "maxlength",
    "rows",
    "cols",
    "width",
    "height",
    "colspan",
    "rowspan",
];

/// One toolbar action, captured at startup from the extension manifest.
#[derive(Clone)]
pub(crate) struct PopupAction {
    pub(crate) title: String,
    pub(crate) root: PathBuf,
    pub(crate) popup_path: Option<String>,
    /// Largest `action.default_icon` entry. Unused until `hodo-render` exposes
    /// `decode_png` / `draw_image`; the slot shows the title instead.
    #[allow(dead_code)]
    pub(crate) icon_path: Option<String>,
}

/// Load unpacked extensions from `dir`, in host order.
pub(crate) fn load_actions_from(dir: &Path) -> Vec<PopupAction> {
    let mut host = hodo_ext_host::Host::new();
    // `dir` may itself be an unpacked extension (the --popup-screenshot CLI
    // passes one directly) or a container like ./extensions.
    let _ = host.load_dir(dir);
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = host.load_dir(&path);
            }
        }
    }
    host.extensions()
        .iter()
        .map(|ext| PopupAction {
            title: ext
                .manifest
                .action_title
                .clone()
                .unwrap_or_else(|| ext.manifest.name.clone()),
            root: ext.root.clone(),
            popup_path: ext.manifest.popup_path.clone(),
            icon_path: ext.manifest.popup_icon.as_ref().map(|p| p.1.clone()),
        })
        .collect()
}

/// CWD-relative extension directory used by the GUI path.
pub(crate) fn load_actions() -> Vec<PopupAction> {
    load_actions_from(Path::new("./extensions"))
}

/// Clamped rectangle of the popup panel, anchored at the clicked slot.
pub(crate) fn popup_rect(viewport: Size, anchor_x: f32) -> Rect {
    let vw = viewport.w.max(1.0);
    let vh = viewport.h.max(1.0);
    let w = POPUP_W.min(vw);
    let h = POPUP_H.min(vh);
    Rect {
        x: (anchor_x - 8.0).clamp(0.0, vw - w),
        y: (vh - crate::ui::BOTTOM_BAR_H - h).clamp(0.0, vh - h),
        w,
        h,
    }
}

/// Cheap change probe over the reachable DOM. A popup is tiny, so walking it
/// once per tick is cheaper than threading a mutation flag through hodo-js.
fn dom_fingerprint(dom: &Dom) -> u64 {
    let mut hasher = DefaultHasher::new();
    dom.walk(|id, node| {
        id.0.hash(&mut hasher);
        match &node.kind {
            NodeKind::Document => "document".hash(&mut hasher),
            NodeKind::Element { .. } => {
                // `dom.tag`/`dom.attr` work in both Dom::new() and interning
                // mode; the raw element fields do not.
                dom.tag(id).hash(&mut hasher);
                for name in FINGERPRINT_ATTRS {
                    dom.attr(id, name).hash(&mut hasher);
                }
            }
            NodeKind::Text(_) => dom.text(id).hash(&mut hasher),
        }
    });
    hasher.finish()
}

/// Inline and `src=` scripts, in document order.
fn collect_scripts(dom: &Dom, base_url: &str) -> Vec<String> {
    let mut scripts = Vec::new();
    dom.walk(|id, _| {
        if dom.tag(id) != Some("script") {
            return;
        }
        if let Some(src) = dom.attr(id, "src") {
            let url = crate::app::join_url(base_url, src);
            if let Ok(resource) = crate::app::load_resource(&url) {
                match resource {
                    hodo_net::Resource::Html(text) => scripts.push(text),
                    hodo_net::Resource::Other { data, .. } => {
                        scripts.push(String::from_utf8_lossy(&data).into_owned())
                    }
                }
            }
            return;
        }
        let mut source = String::new();
        for child in dom.children(id) {
            if let Some(text) = dom.text(*child) {
                source.push_str(text);
            }
        }
        scripts.push(source);
    });
    scripts
}

/// Run the static pipeline for the popup document and make an opaque frame.
fn render_popup(dom: &Dom, base_url: &str) -> (DisplayList, Vec<u32>) {
    let stylesheet = crate::app::collect_styles(dom, base_url);
    let styles = hodo_style::compute_styles(dom, &[stylesheet]);
    let list = hodo_layout::layout(dom, &styles, POPUP_VIEWPORT);
    let mut rgba = hodo_render::render(&list, POPUP_VIEWPORT);
    crate::app::composite_over_white(&mut rgba);
    let frame = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| (255u32 << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32)
        .collect();
    (list, frame)
}

/// Live state of the open popup. Dropped on close, which also drops the JS
/// context and the DOM handles it owns.
pub(crate) struct PopupState {
    pub(crate) dom: Rc<RefCell<Dom>>,
    pub(crate) runtime: ExtScriptRuntime,
    pub(crate) display_list: DisplayList,
    /// ARGB8888, `POPUP_W × POPUP_H`, opaque white where unpainted.
    pub(crate) frame: Vec<u32>,
    pub(crate) rect: Rect,
    /// Toolbar slot that opened the panel; kept so resize can re-anchor it.
    anchor_x: f32,
    base_url: String,
    fingerprint: u64,
    /// `(message, response)` pairs from `sendResponse`, drained by the broker.
    replies: Vec<(Value, Value)>,
}

impl PopupState {
    /// Load `popup_path` from the extension root, run its scripts and do one
    /// style/layout/render pass. Scripts run in document order, then
    /// `DOMContentLoaded` and `focus` are synthesized on the document root.
    pub(crate) fn open(
        action: &PopupAction,
        action_index: usize,
        viewport: Size,
    ) -> Result<Self, String> {
        let relative = action
            .popup_path
            .as_deref()
            .ok_or_else(|| "action has no popup".to_string())?;
        let path = action.root.join(relative);
        let cap = hodo_types::limits::MAX_MANIFEST_BYTES as u64 + 1;
        let mut bytes = Vec::new();
        let read = fs::File::open(&path)
            .map_err(|err| format!("popup {}: {err}", path.display()))?
            .take(cap)
            .read_to_end(&mut bytes)
            .map_err(|err| format!("popup {}: {err}", path.display()))?;
        if read > cap as usize - 1 {
            return Err(format!("popup {} too large", path.display()));
        }
        let html = String::from_utf8(bytes).map_err(|err| format!("popup is not UTF-8: {err}"))?;
        let base_url = path.to_string_lossy().into_owned();
        let mut dom = hodo_html::parse_document(&html);

        // Same relative-URL fix as the page loader: layout has no base URL.
        let mut relative_images = Vec::new();
        dom.walk(|id, _| {
            if dom.tag(id).is_some_and(|t| t.eq_ignore_ascii_case("img")) {
                if let Some(src) = dom.attr(id, "src") {
                    if !src.contains("://") && !src.starts_with("data:") {
                        relative_images.push((id, crate::app::join_url(&base_url, src)));
                    }
                }
            }
        });
        for (id, src) in relative_images {
            dom.set_attr(id, "src", src);
        }

        let scripts = collect_scripts(&dom, &base_url);
        let root = dom.root();
        let dom = Rc::new(RefCell::new(dom));
        let mut runtime = ExtScriptRuntime::new_page(Rc::clone(&dom));
        // The host owns reply correlation; hodo-js only produces the pairs.
        runtime.set_message_reply_hook(Some(Rc::new(|_: &Value, _: &Value| {})));
        for script in scripts {
            if let Err(err) = runtime.eval(&script) {
                eprintln!("hodo: popup script: {err}");
            }
        }
        runtime.dispatch_page_event(root, "DOMContentLoaded", json!({}));
        runtime.dispatch_page_event(root, "focus", json!({}));
        runtime.tick();
        let replies = runtime.take_message_replies();

        let (display_list, frame) = render_popup(&dom.borrow(), &base_url);
        let fingerprint = dom_fingerprint(&dom.borrow());
        let anchor_x = crate::ui::action_slot_x(action_index);
        let mut popup = Self {
            dom,
            runtime,
            display_list,
            frame,
            rect: popup_rect(viewport, anchor_x),
            anchor_x,
            base_url,
            fingerprint,
            replies: Vec::new(),
        };
        popup.push_replies(replies);
        Ok(popup)
    }

    /// Re-anchor the panel after a host resize without re-running its scripts.
    pub(crate) fn set_viewport(&mut self, viewport: Size) {
        self.rect = popup_rect(viewport, self.anchor_x);
    }

    /// Buffer replies with a hard cap: a popup with a chatty timer must not
    /// grow the queue without bound while no broker drains it.
    fn push_replies(&mut self, batch: Vec<(Value, Value)>) {
        self.replies.extend(batch);
        let excess = self.replies.len().saturating_sub(MAX_REPLIES);
        if excess != 0 {
            self.replies.drain(..excess);
            DROPPED_REPLIES.fetch_add(excess, Ordering::Relaxed);
            eprintln!("hodo: popup reply queue full; dropped {excess} oldest replies");
        }
    }

    /// Advance the popup's JS context; re-run style/layout/render when a
    /// script mutated the DOM through the page natives. Returns true when the
    /// painted frame changed (so the caller can request one redraw).
    pub(crate) fn tick(&mut self) -> bool {
        self.runtime.tick();
        let batch = self.runtime.take_message_replies();
        self.push_replies(batch);
        let fingerprint = dom_fingerprint(&self.dom.borrow());
        if fingerprint == self.fingerprint {
            return false;
        }
        self.fingerprint = fingerprint;
        let (list, frame) = render_popup(&self.dom.borrow(), &self.base_url);
        self.display_list = list;
        self.frame = frame;
        true
    }

    /// Take queued `sendResponse` pairs for the message broker.
    #[allow(dead_code)]
    pub(crate) fn take_replies(&mut self) -> Vec<(Value, Value)> {
        std::mem::take(&mut self.replies)
    }

    /// A left-click inside the panel. Returns true when the click was consumed
    /// (i.e. landed inside the panel), so the caller can skip page hit-testing.
    pub(crate) fn click(&mut self, x: f32, y: f32) -> bool {
        let local_x = x - self.rect.x;
        let local_y = y - self.rect.y;
        if local_x < 0.0 || local_y < 0.0 || local_x >= self.rect.w || local_y >= self.rect.h {
            return false;
        }
        let hit = self.display_list.items.iter().rev().any(|item| match item {
            DisplayItem::Text { rect, .. } => {
                local_x >= rect.x
                    && local_x < rect.x + rect.w
                    && local_y >= rect.y
                    && local_y < rect.y + rect.h
            }
            _ => false,
        });
        if hit {
            // TODO(page-JS): route via layout node_map once it lands
            let root = self.dom.borrow().root();
            self.runtime
                .dispatch_page_event(root, "click", json!({ "x": local_x, "y": local_y }));
            self.runtime.tick();
            let batch = self.runtime.take_message_replies();
            self.push_replies(batch);
        }
        true
    }

    /// Source-over composite the panel into a full-window ARGB8888 frame.
    pub(crate) fn composite(&self, frame: &mut [u32], viewport: Size) {
        let fw = viewport.w.round().max(1.0) as usize;
        let fh = viewport.h.round().max(1.0) as usize;
        let pw = POPUP_W as usize;
        let ph = POPUP_H as usize;
        let ox = self.rect.x.round() as i32;
        let oy = self.rect.y.round() as i32;
        for py in 0..ph.min(self.rect.h.round() as usize) {
            for px in 0..pw.min(self.rect.w.round() as usize) {
                let (dx, dy) = (ox + px as i32, oy + py as i32);
                if dx < 0 || dy < 0 || dx >= fw as i32 || dy >= fh as i32 {
                    continue;
                }
                let src = self.frame[py * pw + px];
                let dst_index = dy as usize * fw + dx as usize;
                if let Some(dst) = frame.get_mut(dst_index) {
                    let alpha = (src >> 24) & 0xFF;
                    if alpha == 255 {
                        *dst = src;
                    } else if alpha != 0 {
                        let inv = 255 - alpha;
                        let channel = |shift: u32| -> u32 {
                            let s = (src >> shift) & 0xFF;
                            let d = (*dst >> shift) & 0xFF;
                            ((s * alpha + d * inv + 127) / 255).min(255)
                        };
                        *dst =
                            (255u32 << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_popup(html: &str) -> PopupAction {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("hodo-popup-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("popup.html"), html).unwrap();
        PopupAction {
            title: "test".into(),
            root,
            popup_path: Some("popup.html".into()),
            icon_path: None,
        }
    }

    #[test]
    fn popup_rect_clamps_to_the_window() {
        let normal = popup_rect(
            Size {
                w: 1200.0,
                h: 800.0,
            },
            900.0,
        );
        assert_eq!(normal.w, POPUP_W);
        assert_eq!(normal.h, POPUP_H);
        assert!(normal.x + normal.w <= 1200.0);
        assert!(normal.y + normal.h <= 800.0 - crate::ui::BOTTOM_BAR_H);

        let tiny = popup_rect(Size { w: 200.0, h: 300.0 }, 150.0);
        assert_eq!((tiny.x, tiny.y, tiny.w, tiny.h), (0.0, 0.0, 200.0, 300.0));
    }

    #[test]
    fn fixture_popup_html_renders_headless() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions/popup-chrome");
        let action = PopupAction {
            title: "Popup Demo".into(),
            root,
            popup_path: Some("popup.html".into()),
            icon_path: None,
        };
        let popup = PopupState::open(
            &action,
            0,
            Size {
                w: 1200.0,
                h: 800.0,
            },
        )
        .unwrap();
        assert!(
            popup.display_list.items.iter().any(|item| matches!(
                item,
                DisplayItem::Text { text, .. } if text.contains("Popup Demo")
            )),
            "the fixture's <h1> must reach the display list"
        );
        assert_eq!(popup.frame.len(), POPUP_W as usize * POPUP_H as usize);
        assert!(
            popup.frame.iter().any(|&pixel| pixel != 0xFFFF_FFFF),
            "the fixture must paint something over the opaque white background"
        );
    }

    #[test]
    fn mutation_relayouts_and_replies_are_drained() {
        let action = temp_popup(
            "<div id='host'>before</div><script>\
             document.addEventListener('DOMContentLoaded', function () { \
             __hodo_message_reply('ping', 'pong'); });</script>",
        );
        let mut popup = PopupState::open(
            &action,
            0,
            Size {
                w: 1200.0,
                h: 800.0,
            },
        )
        .unwrap();
        assert_eq!(popup.replies.len(), 1, "DOMContentLoaded listener replied");
        assert_eq!(popup.take_replies().len(), 1);
        assert!(popup.replies.is_empty());

        popup
            .runtime
            .eval("document.getElementById('host').textContent = 'after'")
            .unwrap();
        assert!(
            !popup.display_list.items.iter().any(|item| matches!(
                item,
                DisplayItem::Text { text, .. } if text.contains("after")
            )),
            "eval alone must not relayout; the per-frame tick does"
        );
        popup.tick();
        assert!(
            popup.display_list.items.iter().any(|item| matches!(
                item,
                DisplayItem::Text { text, .. } if text.contains("after")
            )),
            "a DOM mutation must trigger a re-layout on the next tick"
        );
        let _ = fs::remove_dir_all(&action.root);
    }

    #[test]
    fn popup_rect_follows_host_resize() {
        let action = temp_popup("<p>hi</p>");
        let mut popup = PopupState::open(
            &action,
            0,
            Size {
                w: 1200.0,
                h: 800.0,
            },
        )
        .unwrap();
        let viewport = Size { w: 600.0, h: 600.0 };
        popup.set_viewport(viewport);
        assert_eq!(
            popup.rect,
            popup_rect(viewport, crate::ui::action_slot_x(0))
        );
        assert!(popup.rect.x >= 0.0 && popup.rect.x + popup.rect.w <= viewport.w);
        assert!(popup.rect.y >= 0.0 && popup.rect.y + popup.rect.h <= viewport.h);
        let inside = (
            popup.rect.x + popup.rect.w / 2.0,
            popup.rect.y + popup.rect.h / 2.0,
        );
        assert!(
            popup.click(inside.0, inside.1),
            "center still hits the panel"
        );
        assert!(
            !popup.click(viewport.w - 1.0, 0.0),
            "outside the resized panel must not be consumed"
        );
        let _ = fs::remove_dir_all(&action.root);
    }

    #[test]
    fn timer_driven_replies_stay_bounded() {
        let action = temp_popup(
            "<script>document.addEventListener('DOMContentLoaded', function () { \
             for (var i = 0; i < 400; i++) __hodo_message_reply('ping' + i, i); \
             });</script>",
        );
        let before = dropped_replies();
        let mut popup = PopupState::open(
            &action,
            0,
            Size {
                w: 1200.0,
                h: 800.0,
            },
        )
        .unwrap();
        assert_eq!(popup.replies.len(), MAX_REPLIES, "queue capped at open");
        for _ in 0..100 {
            popup.tick();
        }
        assert!(
            popup.replies.len() <= MAX_REPLIES,
            "queue must stay bounded"
        );
        assert!(
            dropped_replies() > before,
            "dropping oldest replies must be counted"
        );
        let _ = fs::remove_dir_all(&action.root);
    }

    #[test]
    fn fingerprint_matches_interned_and_plain_dom() {
        use hodo_dom::NodeKind;
        fn sample(intern: bool) -> Dom {
            let mut dom = if intern {
                Dom::with_interning()
            } else {
                Dom::new()
            };
            let html = dom.create(NodeKind::Element {
                tag: "html".into(),
                attrs: Vec::new(),
            });
            let input = dom.create(NodeKind::Element {
                tag: "input".into(),
                attrs: vec![
                    ("id".into(), "q".into()),
                    ("name".into(), "n".into()),
                    ("value".into(), "v".into()),
                ],
            });
            let text = dom.create(NodeKind::Text("hello".into()));
            dom.append(dom.root(), html);
            dom.append(html, input);
            dom.append(input, text);
            dom
        }
        assert_eq!(
            dom_fingerprint(&sample(false)),
            dom_fingerprint(&sample(true))
        );
    }
}

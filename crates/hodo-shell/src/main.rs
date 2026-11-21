//! hodo browser shell.
//!
//! Two modes:
//!   * `hodo-shell [<url>]` — GUI window (winit 0.30 + softbuffer 0.4).
//!   * `hodo-shell --screenshot <url> <out.png>` — pure headless render, no
//!     window, no GPU/display required. Used for CI verification.
//!
//! At GUI startup, unpacked extensions are loaded from `./extensions/`
//! (relative to the current working directory); load failures are ignored.

mod app;

use std::fs;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

/// Load unpacked extensions from `./extensions/` (CWD-relative) and return
/// their toolbar titles: `action_title` with the manifest `name` as fallback.
fn extension_titles() -> Vec<String> {
    let mut host = hodo_ext_host::Host::new();
    if let Ok(entries) = fs::read_dir("./extensions") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = host.load_dir(&path);
            }
        }
    }
    host.extensions()
        .iter()
        .map(|ext| {
            ext.manifest
                .action_title
                .clone()
                .unwrap_or_else(|| ext.manifest.name.clone())
        })
        .collect()
}

/// Prepare a headless frame for PNG encoding: composite over opaque white
/// (like the GUI path) before premultiplying for tiny-skia.
fn screenshot_frame(
    display_list: &hodo_layout::DisplayList,
    viewport: hodo_types::Size,
) -> Vec<u8> {
    let mut rgba = hodo_render::render(display_list, viewport);
    app::composite_over_white(&mut rgba);
    app::premultiply_rgba(&rgba)
}

/// Headless screenshot: run the page pipeline (no UI overlay), premultiply
/// the straight-alpha frame for tiny-skia, encode and write the PNG.
fn screenshot(url: &str, out: &str) {
    let (_, display_list) = app::load_page(url, app::DEFAULT_VIEWPORT);
    let premultiplied = screenshot_frame(&display_list, app::DEFAULT_VIEWPORT);
    let size = tiny_skia::IntSize::from_wh(1200, 800).expect("fixed screenshot size is valid");
    let pixmap = tiny_skia::Pixmap::from_vec(premultiplied, size).expect("frame matches viewport");
    let png = pixmap.encode_png().expect("PNG encoding failed");
    fs::write(out, &png).expect("failed to write screenshot");
    println!("screenshot written: {out} ({} bytes)", png.len());
}

/// State owned by the winit event loop. Graphics objects live here; the tab
/// model (`app::App`) stays window-agnostic and unit-testable.
struct BrowserApp {
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    state: app::App,
    modifiers: ModifiersState,
    ext_titles: Vec<String>,
    viewport: hodo_types::Size,
}

impl BrowserApp {
    fn new(initial_url: Option<&str>, ext_titles: Vec<String>) -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            state: app::App::new(initial_url),
            modifiers: ModifiersState::default(),
            ext_titles,
            viewport: app::DEFAULT_VIEWPORT,
        }
    }

    fn set_window_title(&self, title: &str) {
        if let Some(window) = &self.window {
            window.set_title(if title.is_empty() { "hodo" } else { title });
        }
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn navigate(&mut self, url: String) {
        let title = self.state.navigate(&url, self.viewport);
        self.set_window_title(&title);
        self.request_redraw();
    }

    fn on_key(&mut self, event: KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }
        let ctrl = self.modifiers.control_key();
        match event.logical_key.as_ref() {
            Key::Named(NamedKey::Enter) => {
                if self.state.url_focused {
                    let url = self.state.url_bar.clone();
                    self.navigate(url);
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if self.state.url_focused && self.state.url_bar.pop().is_some() {
                    self.request_redraw();
                }
            }
            Key::Named(NamedKey::F5) => {
                let title = self.state.reload(self.viewport);
                if !title.is_empty() {
                    self.set_window_title(&title);
                }
                self.request_redraw();
            }
            Key::Character(text) if ctrl => match text.to_ascii_lowercase().as_str() {
                "l" => {
                    self.state.url_focused = true;
                    self.state.url_bar.clear();
                    self.request_redraw();
                }
                "t" => {
                    self.state.new_tab();
                    self.set_window_title("");
                    self.request_redraw();
                }
                "w" => {
                    self.state.close_active();
                    let title = self.state.active_tab().title.clone();
                    self.set_window_title(&title);
                    self.request_redraw();
                }
                _ => {}
            },
            Key::Character(text) if self.state.url_focused => {
                self.state.url_bar.push_str(text);
                self.request_redraw();
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
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
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        let frame = self.state.frame(self.viewport, &self.ext_titles);
        let count = frame.len().min(buffer.len());
        buffer[..count].copy_from_slice(&frame[..count]);
        window.pre_present_notify();
        if let Err(err) = buffer.present() {
            eprintln!("hodo: present failed: {err}");
        }
    }
}

impl ApplicationHandler for BrowserApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("hodo")
            .with_inner_size(PhysicalSize::new(1200u32, 800u32));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                eprintln!("hodo: cannot create window: {err}");
                event_loop.exit();
                return;
            }
        };
        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(err) => {
                eprintln!("hodo: cannot create softbuffer context: {err}");
                event_loop.exit();
                return;
            }
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                eprintln!("hodo: cannot create softbuffer surface: {err}");
                event_loop.exit();
                return;
            }
        };
        let inner = window.inner_size();
        self.viewport = hodo_types::Size {
            w: inner.width as f32,
            h: inner.height as f32,
        };
        let title = self.state.active_tab().title.clone();
        window.set_title(if title.is_empty() { "hodo" } else { &title });
        self.window = Some(window.clone());
        self.context = Some(context);
        self.surface = Some(surface);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                self.viewport = hodo_types::Size {
                    w: size.width as f32,
                    h: size.height as f32,
                };
                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                {
                    if let Some(surface) = self.surface.as_mut() {
                        if let Err(err) = surface.resize(width, height) {
                            eprintln!("hodo: surface resize failed: {err}");
                        }
                    }
                }
                self.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event),
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            // Re-render at ~30 fps; only the active tab has a pixel buffer.
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(33),
            ));
        }
    }
}

fn run_gui(initial_url: Option<&str>) {
    let ext_titles = extension_titles();
    let mut app = BrowserApp::new(initial_url, ext_titles);
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            eprintln!("hodo: cannot create event loop: {err}");
            return;
        }
    };
    event_loop.set_control_flow(ControlFlow::Wait);
    if let Err(err) = event_loop.run_app(&mut app) {
        eprintln!("hodo: event loop error: {err}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, mode, url, out] if mode.as_str() == "--screenshot" => screenshot(url, out),
        [_] => run_gui(None),
        [_, url] => run_gui(Some(url.as_str())),
        _ => {
            eprintln!("usage: hodo-shell [--screenshot <url> <out.png>] [<url>]");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_frame_is_fully_opaque_white_where_unpainted() {
        let frame = screenshot_frame(&hodo_layout::DisplayList::default(), app::DEFAULT_VIEWPORT);
        assert_eq!(frame.len(), 1200 * 800 * 4);
        assert!(
            frame
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p == &[255, 255, 255, 255]),
            "unpainted headless pixels must be opaque white"
        );
    }
}

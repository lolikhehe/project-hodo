//! hodo browser shell.
//!
//! Two modes:
//!   * `hodo-shell [<url>]` — GUI window (winit 0.30 + softbuffer 0.4).
//!   * `hodo-shell --screenshot <url> <out.png>` — pure headless render, no
//!     window, no GPU/display required. Used for CI verification.
//!   * `hodo-shell --popup-screenshot <ext-dir> <out.png>` — headless render of
//!     an unpacked extension's popup page.
//!
//! At GUI startup, unpacked extensions are loaded from `./extensions/`
//! (relative to the current working directory); load failures are ignored.

mod about;
mod app;
mod forms;
mod hpages;
mod popup;
mod present;
mod sessions;
mod tabs;
mod ui;

use std::fs;

use winit::event_loop::{ControlFlow, EventLoop};

/// Headless popup screenshot: parse, run scripts, style/layout/render the
/// extension's popup page and encode its offscreen frame as PNG.
fn popup_screenshot(ext_dir: &str, out: &str) {
    let actions = popup::load_actions_from(std::path::Path::new(ext_dir));
    let action = actions
        .into_iter()
        .find(|action| action.popup_path.is_some())
        .expect("extension must declare a popup");
    let popup =
        popup::PopupState::open(&action, 0, app::DEFAULT_VIEWPORT).expect("popup page must load");
    let mut rgba = Vec::with_capacity(popup.frame.len() * 4);
    for pixel in &popup.frame {
        rgba.extend_from_slice(&[
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
            ((pixel >> 24) & 0xFF) as u8,
        ]);
    }
    let premultiplied = app::premultiply_rgba(&rgba);
    let size = tiny_skia::IntSize::from_wh(popup::POPUP_W as u32, popup::POPUP_H as u32)
        .expect("popup size is valid");
    let pixmap =
        tiny_skia::Pixmap::from_vec(premultiplied, size).expect("frame matches popup size");
    let png = pixmap.encode_png().expect("PNG encoding failed");
    fs::write(out, &png).expect("failed to write popup screenshot");
    println!("popup screenshot written: {out} ({} bytes)", png.len());
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
    let (title, display_list) = app::load_page(url, app::DEFAULT_VIEWPORT);
    println!("title: {title}");
    let premultiplied = screenshot_frame(&display_list, app::DEFAULT_VIEWPORT);
    let size = tiny_skia::IntSize::from_wh(1200, 800).expect("fixed screenshot size is valid");
    let pixmap = tiny_skia::Pixmap::from_vec(premultiplied, size).expect("frame matches viewport");
    let png = pixmap.encode_png().expect("PNG encoding failed");
    fs::write(out, &png).expect("failed to write screenshot");
    println!("screenshot written: {out} ({} bytes)", png.len());
}

fn run_gui(initial_url: Option<&str>) {
    let actions = popup::load_actions();
    let ext_titles = actions.iter().map(|action| action.title.clone()).collect();
    let mut app = app::BrowserApp::new(initial_url, ext_titles, actions);
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
        [_, mode, dir, out] if mode.as_str() == "--popup-screenshot" => popup_screenshot(dir, out),
        [_, mode, url, out] if mode.as_str() == "--screenshot" => screenshot(url, out),
        [_, flag] if flag.as_str() == "--private" => run_gui(None),
        [_, flag, url] if flag.as_str() == "--private" => run_gui(Some(url.as_str())),
        [_, url, flag] if flag.as_str() == "--private" => run_gui(Some(url.as_str())),
        [_] => run_gui(None),
        [_, url] => run_gui(Some(url.as_str())),
        _ => {
            eprintln!(
                "usage: hodo-shell [--private] [--screenshot <url> <out.png>] [--popup-screenshot <ext-dir> <out.png>] [<url>] [--private]"
            );
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

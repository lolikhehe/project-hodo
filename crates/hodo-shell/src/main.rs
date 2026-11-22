//! hodo browser shell.
//!
//! Two modes:
//!   * `hodo-shell [<url>]` — GUI window (winit 0.30 + softbuffer 0.4).
//!   * `hodo-shell --screenshot <url> <out.png> [--viewport WxH]` — pure headless
//!     render, no window, no GPU/display required. Used for CI verification.
//!   * `hodo-shell --popup-screenshot <ext-dir> <out.png>` — headless render of
//!     an unpacked extension's popup page.
//!
//! At GUI startup, unpacked extensions are loaded from `./extensions/`
//! (relative to the current working directory); load failures are ignored.

mod about;
mod app;
mod forms;
mod hpages;
mod modules;
mod popup;
mod present;
mod sessions;
mod tabs;
mod ui;

use std::fs;
use std::time::Duration;

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
fn screenshot(url: &str, out: &str, viewport: hodo_types::Size, wait_ms: u64) {
    // Same first-load priming as the GUI path; headless runs on this thread.
    hodo_js::prime_page_prelude();
    let (title, display_list) =
        app::load_page_with_wait(url, viewport, Duration::from_millis(wait_ms));
    println!("title: {title}");
    let premultiplied = screenshot_frame(&display_list, viewport);
    let size = tiny_skia::IntSize::from_wh(viewport.w as u32, viewport.h as u32)
        .expect("viewport dimensions are validated by the CLI parser");
    let pixmap = tiny_skia::Pixmap::from_vec(premultiplied, size).expect("frame matches viewport");
    let png = pixmap.encode_png().expect("PNG encoding failed");
    fs::write(out, &png).expect("failed to write screenshot");
    println!("screenshot written: {out} ({} bytes)", png.len());
    // No further navigation in headless mode: drop the one-shot primed realm on
    // this thread instead of leaving a boa Context for TLS teardown to finalise.
    hodo_js::drop_primed_page_prelude();
}

fn run_gui(initial_url: Option<&str>) {
    // Prime the STATIC page-prelude parse ahead of the first navigation so it
    // overlaps the host's network fetch. GUI thread only: boa Context is !Send.
    hodo_js::prime_page_prelude();
    // Startup pick from cache/design.json. The same accessor is called again on
    // every draw, so toggling dark mode on hodo://settings restyles the chrome
    // on the next present without a restart.
    let _ = ui::current_theme();
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
    // GUI shutdown: no next navigation will check out the primed realm.
    hodo_js::drop_primed_page_prelude();
}

const USAGE: &str = "usage: hodo-shell [--private] [--screenshot <url> <out.png> [--viewport WxH] [--wait-ms N (0=no settle loop)]] [--popup-screenshot <ext-dir> <out.png>] [<url>] [--private]";

/// Default screenshot settle time. Module graphs (and their dynamic imports)
/// need more wall time than a classic-script page to reach a stable paint.
/// `--wait-ms 0` runs no settle loop: only load-time eval is observed.
const DEFAULT_WAIT_MS: u64 = 250;
const MAX_WAIT_MS: u64 = 60_000;

/// House CLI error pattern: one message line, usage, exit code 2.
fn cli_error(message: &str) -> ! {
    eprintln!("hodo: {message}");
    eprintln!("{USAGE}");
    std::process::exit(2);
}

/// Parse `WxH`, rejecting malformed specs and dimensions outside 200..=4096.
fn parse_viewport(spec: &str) -> Result<hodo_types::Size, String> {
    let (width, height) = spec
        .split_once('x')
        .ok_or_else(|| format!("invalid --viewport `{spec}`: expected WxH"))?;
    let parse = |part: &str| {
        part.parse::<u32>()
            .map_err(|_| format!("invalid --viewport `{spec}`: expected integer dimensions"))
    };
    let (width, height) = (parse(width)?, parse(height)?);
    if !(200..=4096).contains(&width) || !(200..=4096).contains(&height) {
        return Err(format!(
            "invalid --viewport `{spec}`: width and height must be between 200 and 4096"
        ));
    }
    Ok(hodo_types::Size {
        w: width as f32,
        h: height as f32,
    })
}

/// Parse and validate `--wait-ms N` (0..=60000, the same one-line error pattern
/// as `--viewport`).
fn parse_wait_ms(spec: &str) -> Result<u64, String> {
    let value = spec
        .parse::<u64>()
        .map_err(|_| format!("invalid --wait-ms `{spec}`: expected an integer"))?;
    if value > MAX_WAIT_MS {
        return Err(format!(
            "invalid --wait-ms `{spec}`: must be between 0 and {MAX_WAIT_MS}"
        ));
    }
    Ok(value)
}

/// Parsed argv, kept separate from `main` so the flags can be unit-tested.
#[derive(Debug)]
enum Action {
    Gui(Option<String>),
    Screenshot {
        url: String,
        out: String,
        viewport: hodo_types::Size,
        wait_ms: u64,
    },
    PopupScreenshot {
        dir: String,
        out: String,
    },
}

fn parse_args(args: &[String]) -> Result<Action, String> {
    let mut rest = args.iter().skip(1).peekable();
    let mut viewport: Option<hodo_types::Size> = None;
    let mut wait_ms = DEFAULT_WAIT_MS;

    // Leading `--viewport` / `--wait-ms` pairs (before the mode), mirroring the
    // historic `--viewport WxH --screenshot ...` spelling.
    while matches!(
        rest.peek().map(|item| item.as_str()),
        Some("--viewport") | Some("--wait-ms")
    ) {
        let flag = rest.next().expect("peeked flag").clone();
        let value = rest.next().ok_or("missing flag value")?;
        if flag == "--viewport" {
            viewport = Some(parse_viewport(value)?);
        } else {
            wait_ms = parse_wait_ms(value)?;
        }
    }

    let first = rest.next().map(String::as_str).unwrap_or("");
    match first {
        "--screenshot" => {
            let url = rest.next().ok_or("missing screenshot url")?;
            let out = rest.next().ok_or("missing screenshot output path")?;
            // Trailing pairs in any order.
            while let Some(flag) = rest.next() {
                let value = rest.next().ok_or("missing flag value")?;
                match flag.as_str() {
                    "--viewport" => viewport = Some(parse_viewport(value)?),
                    "--wait-ms" => wait_ms = parse_wait_ms(value)?,
                    _ => return Err(USAGE.to_owned()),
                }
            }
            Ok(Action::Screenshot {
                url: url.clone(),
                out: out.clone(),
                viewport: viewport.unwrap_or(app::DEFAULT_VIEWPORT),
                wait_ms,
            })
        }
        "--popup-screenshot" => {
            let dir = rest.next().ok_or("missing extension directory")?;
            let out = rest.next().ok_or("missing screenshot output path")?;
            if rest.next().is_some() {
                return Err(USAGE.to_owned());
            }
            Ok(Action::PopupScreenshot {
                dir: dir.clone(),
                out: out.clone(),
            })
        }
        "--private" => match (rest.next(), rest.next()) {
            (None, None) => Ok(Action::Gui(None)),
            (Some(url), None) => Ok(Action::Gui(Some(url.clone()))),
            _ => Err(USAGE.to_owned()),
        },
        "" => Ok(Action::Gui(None)),
        url => match (rest.next().map(String::as_str), rest.next()) {
            (None, None) => Ok(Action::Gui(Some(url.to_owned()))),
            (Some("--private"), None) => Ok(Action::Gui(Some(url.to_owned()))),
            _ => Err(USAGE.to_owned()),
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match parse_args(&args).unwrap_or_else(|err| cli_error(&err)) {
        Action::Screenshot {
            url,
            out,
            viewport,
            wait_ms,
        } => screenshot(&url, &out, viewport, wait_ms),
        Action::PopupScreenshot { dir, out } => popup_screenshot(&dir, &out),
        Action::Gui(url) => run_gui(url.as_deref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_types::Size;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn viewport_flag_parses_and_validates_bounds() {
        assert_eq!(parse_viewport("800x600"), Ok(Size { w: 800.0, h: 600.0 }));
        assert_eq!(parse_viewport("200x200"), Ok(Size { w: 200.0, h: 200.0 }));
        assert_eq!(
            parse_viewport("4096x4096"),
            Ok(Size {
                w: 4096.0,
                h: 4096.0
            })
        );
        for bad in [
            "800",
            "800x",
            "800X600",
            "axb",
            "800x600x1",
            "199x600",
            "800x4097",
            "0x0",
        ] {
            assert!(parse_viewport(bad).is_err(), "`{bad}` must be rejected");
        }
        assert!(parse_viewport("100x100")
            .unwrap_err()
            .contains("between 200 and 4096"));
    }

    #[test]
    fn viewport_flag_threads_through_screenshot_args() {
        let action = parse_args(&argv(&[
            "hodo-shell",
            "--screenshot",
            "file:///tmp/a.html",
            "out.png",
            "--viewport",
            "800x600",
        ]))
        .expect("screenshot args parse");
        assert!(matches!(
            action,
            Action::Screenshot { viewport, .. }
                if viewport == Size { w: 800.0, h: 600.0 }
        ));

        let action = parse_args(&argv(&[
            "hodo-shell",
            "--viewport",
            "800x600",
            "--screenshot",
            "file:///tmp/a.html",
            "out.png",
        ]))
        .expect("leading viewport flag parses");
        assert!(matches!(
            action,
            Action::Screenshot { viewport, .. }
                if viewport == Size { w: 800.0, h: 600.0 }
        ));

        let err = parse_args(&argv(&[
            "hodo-shell",
            "--screenshot",
            "u",
            "o",
            "--viewport",
            "100x100",
        ]))
        .unwrap_err();
        assert!(err.contains("between 200 and 4096"), "{err}");
    }

    #[test]
    fn wait_ms_flag_defaults_and_validates() {
        assert_eq!(parse_wait_ms("0"), Ok(0));
        assert_eq!(parse_wait_ms("1500"), Ok(1500));
        assert_eq!(parse_wait_ms("60000"), Ok(60000));
        assert!(parse_wait_ms("-1").is_err());
        assert!(parse_wait_ms("60k").is_err());
        assert!(parse_wait_ms("60001")
            .unwrap_err()
            .contains("between 0 and 60000"));
    }

    #[test]
    fn wait_ms_threads_through_screenshot_args_in_either_order() {
        let action = parse_args(&argv(&[
            "hodo-shell",
            "--screenshot",
            "file:///tmp/a.html",
            "out.png",
            "--wait-ms",
            "1500",
        ]))
        .expect("trailing wait flag parses");
        assert!(matches!(action, Action::Screenshot { wait_ms: 1500, .. }));

        let action = parse_args(&argv(&[
            "hodo-shell",
            "--wait-ms",
            "500",
            "--viewport",
            "800x600",
            "--screenshot",
            "file:///tmp/a.html",
            "out.png",
        ]))
        .expect("leading flags parse in any order");
        assert!(matches!(
            action,
            Action::Screenshot { wait_ms: 500, viewport, .. }
                if viewport == Size { w: 800.0, h: 600.0 }
        ));

        let default = parse_args(&argv(&["hodo-shell", "--screenshot", "u", "o"]))
            .expect("default wait parses");
        assert!(
            matches!(default, Action::Screenshot { wait_ms, .. } if wait_ms == DEFAULT_WAIT_MS)
        );
    }

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

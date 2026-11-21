//! Pipeline benchmark for the hodo browser engine.
//!
//! Runs load -> parse -> style -> layout -> render three times and reports the
//! best (minimum) time for each stage. A stage that is still `todo!()` is caught
//! and reported as pending instead of aborting the whole run.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const RUNS: usize = 3;
const PENDING: &str = "stage not yet implemented (pending wave-2 agent)";

#[derive(Debug, Default, Clone, Copy)]
struct Times {
    load_html: Option<Duration>,
    load_css: Option<Duration>,
    parse_html: Option<Duration>,
    parse_css: Option<Duration>,
    styles: Option<Duration>,
    layout: Option<Duration>,
    render: Option<Duration>,
}

impl Times {
    fn merge_best(&mut self, other: &Times) {
        fn min(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
            match (a, b) {
                (Some(x), Some(y)) => Some(x.min(y)),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            }
        }
        self.load_html = min(self.load_html, other.load_html);
        self.load_css = min(self.load_css, other.load_css);
        self.parse_html = min(self.parse_html, other.parse_html);
        self.parse_css = min(self.parse_css, other.parse_css);
        self.styles = min(self.styles, other.styles);
        self.layout = min(self.layout, other.layout);
        self.render = min(self.render, other.render);
    }
}

fn demo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("demo-site")
}

fn read_direct(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => {
            println!("    direct read failed ({}): {e}", path.display());
            String::new()
        }
    }
}

/// Time one stage, turning a panic into a reported "pending" result.
fn time_stage<T>(name: &str, f: impl FnOnce() -> T) -> (Option<Duration>, Option<T>) {
    let started = Instant::now();
    let outcome = catch_unwind(AssertUnwindSafe(f));
    let elapsed = started.elapsed();
    match outcome {
        Ok(value) => {
            println!("  {name}: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
            (Some(elapsed), Some(value))
        }
        Err(_) => {
            println!("  {name}: {PENDING}");
            (None, None)
        }
    }
}

/// Load one file through `hodo_net`, falling back to a direct read so the
/// remaining stages can still be timed when the loader is not implemented.
fn load_text(path: &Path, name: &str) -> (Option<Duration>, String) {
    let url = path.to_string_lossy().into_owned();
    let (dur, outcome) = time_stage(name, || hodo_net::load(&url));
    let text = match outcome {
        Some(Ok(hodo_net::Resource::Html(text))) => text,
        Some(Ok(hodo_net::Resource::Other { data, .. })) => {
            String::from_utf8_lossy(&data).into_owned()
        }
        Some(Err(e)) => {
            println!("    hodo_net::load error: {e}; using direct file read");
            read_direct(path)
        }
        None => read_direct(path),
    };
    (dur, text)
}

fn print_best(best: &Times) {
    println!("\n=== best of {RUNS} runs ===");
    macro_rules! show {
        ($label:expr, $value:expr) => {
            match $value {
                Some(d) => println!("  {}: {:.3} ms", $label, d.as_secs_f64() * 1000.0),
                None => println!("  {}: {PENDING}", $label),
            }
        };
    }
    show!("load html", best.load_html);
    show!("load css", best.load_css);
    show!("parse html", best.parse_html);
    show!("parse css", best.parse_css);
    show!("compute styles", best.styles);
    show!("layout", best.layout);
    show!("render", best.render);
}

fn main() {
    let dir = demo_dir();
    let html_path = dir.join("index.html");
    let css_path = dir.join("style.css");
    let viewport = hodo_types::Size {
        w: 1200.0,
        h: 800.0,
    };

    println!("hodo pipeline benchmark ({RUNS} runs, viewport 1200x800)");
    println!("demo site: {}", dir.display());

    let mut best = Times::default();

    for run in 1..=RUNS {
        println!("\n--- run {run}/{RUNS} ---");
        let mut t = Times::default();

        let (d, html) = load_text(&html_path, "load html");
        t.load_html = d;
        let (d, css) = load_text(&css_path, "load css");
        t.load_css = d;

        let (d, dom) = time_stage("parse html", || hodo_html::parse_document(&html));
        t.parse_html = d;

        let (d, sheet) = time_stage("parse css", || hodo_css::parse_stylesheet(&css));
        t.parse_css = d;
        let sheet = sheet.unwrap_or(hodo_css::Stylesheet { rules: Vec::new() });

        let styles = match dom.as_ref() {
            Some(dom) => {
                let (d, s) = time_stage("compute styles", || {
                    hodo_style::compute_styles(dom, &[sheet])
                });
                t.styles = d;
                s
            }
            None => {
                println!("  compute styles: {PENDING} (no parsed DOM)");
                None
            }
        };

        let display_list = match (dom.as_ref(), styles.as_ref()) {
            (Some(dom), Some(styles)) => {
                let (d, list) = time_stage("layout", || hodo_layout::layout(dom, styles, viewport));
                t.layout = d;
                list
            }
            _ => {
                println!("  layout: {PENDING} (DOM or styles unavailable)");
                None
            }
        };
        let list = display_list.unwrap_or_default();
        println!("  display-list items: {}", list.items.len());

        let (d, buffer) = time_stage("render", || hodo_render::render(&list, viewport));
        t.render = d;
        if let Some(buffer) = buffer {
            println!("  frame buffer: {} bytes", buffer.len());
        }

        best.merge_best(&t);
    }

    print_best(&best);
}

//! Pipeline benchmark for the hodo browser engine.
//!
//! Runs load -> parse -> style -> layout -> render three times and reports the
//! single run with the lowest total time (per-stage minima from different runs
//! would describe a run that never happened). Every run's stage detail is still
//! printed. A stage that is still `todo!()` is caught and reported as pending
//! instead of aborting the whole run.

mod bigdoc;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(crate) const RUNS: usize = 3;
const PENDING: &str = "stage not yet implemented (pending wave-2 agent)";

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Times {
    load_html: Option<Duration>,
    load_css: Option<Duration>,
    pub(crate) parse_html: Option<Duration>,
    pub(crate) parse_css: Option<Duration>,
    pub(crate) styles: Option<Duration>,
    pub(crate) layout: Option<Duration>,
    pub(crate) render: Option<Duration>,
}

impl Times {
    /// Sum of the measured stages — the quantity used to pick the best run.
    pub(crate) fn total(&self) -> Duration {
        [
            self.load_html,
            self.load_css,
            self.parse_html,
            self.parse_css,
            self.styles,
            self.layout,
            self.render,
        ]
        .into_iter()
        .flatten()
        .sum()
    }
}

/// Keep one whole run — the one with the lowest total — rather than mixing
/// per-stage minima from different runs into a timings row that never occurred.
pub(crate) fn better_run(
    current: Option<(usize, Times)>,
    run: usize,
    candidate: Times,
) -> Option<(usize, Times)> {
    match current {
        None => Some((run, candidate)),
        Some((_, best)) if candidate.total() < best.total() => Some((run, candidate)),
        Some(existing) => Some(existing),
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
pub(crate) fn time_stage<T>(name: &str, f: impl FnOnce() -> T) -> (Option<Duration>, Option<T>) {
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

fn print_best(run: usize, best: &Times) {
    println!("\n=== best single run {run}/{RUNS} (lowest total) ===");
    println!("  total: {:.3} ms", best.total().as_secs_f64() * 1000.0);
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
    print_best_stages(best);
}

/// Shared stage summary used by both the demo pipeline and the big-doc corpus.
pub(crate) fn print_best_stages(best: &Times) {
    macro_rules! show {
        ($label:expr, $value:expr) => {
            match $value {
                Some(d) => println!("  {}: {:.3} ms", $label, d.as_secs_f64() * 1000.0),
                None => println!("  {}: {PENDING}", $label),
            }
        };
    }
    show!("parse html", best.parse_html);
    show!("parse css", best.parse_css);
    show!("compute styles", best.styles);
    show!("layout", best.layout);
    show!("render", best.render);
}

fn main() {
    if std::env::args().skip(1).any(|arg| arg == "--big-doc") {
        // The DEEP corpus recurses ~200 levels deep through parse/style/layout.
        // A debug build's frames overflow the 1 MiB Windows main-thread stack,
        // so run the whole corpus on a thread with a 16 MiB stack instead.
        std::thread::Builder::new()
            .name("big-doc".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn(bigdoc::run)
            .expect("spawn big-doc thread")
            .join()
            .expect("big-doc thread panicked");
        return;
    }

    let dir = demo_dir();
    let html_path = dir.join("index.html");
    let css_path = dir.join("style.css");
    let viewport = hodo_types::Size {
        w: 1200.0,
        h: 800.0,
    };

    println!("hodo pipeline benchmark ({RUNS} runs, viewport 1200x800)");
    println!("demo site: {}", dir.display());

    let mut best: Option<(usize, Times)> = None;

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

        best = better_run(best, run, t);
    }

    let (run, best) = best.expect("at least one run");
    print_best(run, &best);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Option<Duration> {
        Some(Duration::from_millis(millis))
    }

    /// Per-stage minima would bolt run A's fast parse onto run B's fast layout
    /// and report a 30 ms run that never happened. The chosen run must be real.
    #[test]
    fn best_run_is_lowest_total_not_per_stage_minima() {
        let a = Times {
            parse_html: ms(10),
            layout: ms(100),
            ..Times::default()
        };
        let b = Times {
            parse_html: ms(50),
            layout: ms(20),
            ..Times::default()
        };
        let c = Times {
            parse_html: ms(30),
            layout: ms(60),
            ..Times::default()
        };
        let best = better_run(None, 1, a)
            .and_then(|cur| better_run(Some(cur), 2, b))
            .and_then(|cur| better_run(Some(cur), 3, c))
            .expect("a best run");
        assert_eq!(best.0, 2, "run 2 has the lowest total (70 ms)");
        assert_eq!(best.1.parse_html, ms(50));
        assert_eq!(best.1.layout, ms(20));
        assert_eq!(best.1.total(), Duration::from_millis(70));
    }
}

//! Robustness harness: every public parser must return normally (or a normal
//! `Result`) for malformed inputs, and must never unwind.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use hodo_css::Stylesheet;
use hodo_dom::Dom;
use hodo_ext_manifest::Manifest;
use hodo_fuzz::{corpus, gen_string, DEFAULT_SEED};
use hodo_net::FetchError;

const ITERATIONS: u64 = 2_000;
const MAX_LEN: usize = 256;
const DEEP_DEPTH: usize = 2_000;

fn seed_for(i: u64) -> u64 {
    DEFAULT_SEED ^ i.wrapping_mul(DEFAULT_SEED)
}

fn preview(input: &str) -> String {
    let mut out = String::new();
    for (i, ch) in input.chars().enumerate() {
        if i == 120 {
            out.push_str("…(truncated)");
            break;
        }
        out.push(ch);
    }
    format!("{out:?}")
}

fn must_not_panic(label: &str, input: &str, seed: Option<u64>, f: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(f)).is_err() {
        panic!(
            "{label} PANICKED — seed={seed:?} input_len={} input={}",
            input.len(),
            preview(input)
        );
    }
}

#[test]
fn html_never_panics() {
    for input in corpus() {
        must_not_panic("hodo_html::parse_document/corpus", input, None, || {
            let _: Dom = hodo_html::parse_document(input);
        });
    }
    for i in 0..ITERATIONS {
        let seed = seed_for(i);
        let input = gen_string(seed, MAX_LEN);
        must_not_panic(
            "hodo_html::parse_document/random",
            &input,
            Some(seed),
            || {
                let _: Dom = hodo_html::parse_document(&input);
            },
        );
    }
}

#[test]
fn css_never_panics() {
    for input in corpus() {
        must_not_panic("hodo_css::parse_stylesheet/corpus", input, None, || {
            let _: Stylesheet = hodo_css::parse_stylesheet(input);
        });
    }
    for i in 0..ITERATIONS {
        let seed = seed_for(i);
        let input = gen_string(seed, MAX_LEN);
        must_not_panic(
            "hodo_css::parse_stylesheet/random",
            &input,
            Some(seed),
            || {
                let _: Stylesheet = hodo_css::parse_stylesheet(&input);
            },
        );
    }
}

#[test]
fn manifest_never_panics() {
    for input in corpus() {
        must_not_panic("hodo_ext_manifest::parse/corpus", input, None, || {
            let _: Result<Manifest, _> = hodo_ext_manifest::parse(input);
        });
    }
    for i in 0..ITERATIONS {
        let seed = seed_for(i);
        let input = gen_string(seed, MAX_LEN);
        must_not_panic(
            "hodo_ext_manifest::parse/random",
            &input,
            Some(seed),
            || {
                let _: Result<Manifest, _> = hodo_ext_manifest::parse(&input);
            },
        );
    }
}

#[test]
fn deeply_nested_html_completes() {
    let html = format!(
        "{}x{}",
        "<div>".repeat(DEEP_DEPTH),
        "</div>".repeat(DEEP_DEPTH)
    );
    let started = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let dom: Dom = hodo_html::parse_document(&html);
        dom.len()
    }));
    let elapsed = started.elapsed();

    match result {
        Ok(nodes) => {
            assert!(
                nodes >= DEEP_DEPTH,
                "expected >= {DEEP_DEPTH} nodes, got {nodes}"
            );
            assert!(
                elapsed < Duration::from_secs(10),
                "deep parse took too long: {elapsed:?}"
            );
        }
        Err(_) => panic!("parse_document panicked at nesting depth {DEEP_DEPTH}"),
    }
}

#[test]
fn net_edge_inputs_return_err_without_panicking() {
    let urls: &[&str] = &["", "file://", "file:///tmp/hodo\0nul", "C:\\hodo\0nul"];
    for url in urls {
        let result = catch_unwind(AssertUnwindSafe(|| hodo_net::load(url)));
        match result {
            Ok(Err(error)) => {
                let _: &FetchError = &error;
            }
            Ok(Ok(resource)) => panic!("load({url:?}) unexpectedly succeeded: {resource:?}"),
            Err(_) => panic!("load({url:?}) PANICKED"),
        }
    }

    assert!(matches!(hodo_net::load(""), Err(FetchError::InvalidUrl)));
    assert!(hodo_net::load("file://").is_err());
}

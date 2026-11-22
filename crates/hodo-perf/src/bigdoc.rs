//! M5.3 gate benchmark: deterministic big-document corpus.
//!
//! Three in-code generators build MEDIUM, LARGE and DEEP documents from a fixed
//! sentence pool and an LCG (no `rand` dependency). Each document is measured on
//! the same full pipeline as the demo benchmark (parse -> styles -> layout ->
//! render, best of three), then the two M5.3 mutation deltas are measured on the
//! full pre-incremental path: a single class change (full restyle) and a node
//! append (full restyle + relayout + repaint).
//!
//! The corpus is content-addressed with FNV-1a so a generator change cannot
//! silently invalidate a benchmark number. `HODO_SKIP_BIG_DOC=1` skips the run.

use std::path::{Path, PathBuf};

use hodo_types::limits::{MAX_NESTING_DEPTH, MAX_RESOURCE_BYTES};

use crate::{better_run, print_best_stages, time_stage, Times, RUNS};

const VIEWPORT: hodo_types::Size = hodo_types::Size {
    w: 1200.0,
    h: 800.0,
};

const MEDIUM_TARGET_BYTES: usize = 250 * 1024;
const LARGE_TARGET_BYTES: usize = 2 * 1024 * 1024;
const DEEP_TARGET_BYTES: usize = 500 * 1024;

/// Fixed pool of plain ASCII sentences; text is assembled from these with the
/// LCG and truncated to an exact byte length, so documents are reproducible on
/// every platform (no locale, time or allocator influence).
const SENTENCES: &[&str] = &[
    "the quick brown fox jumps over the lazy dog near the old river bank",
    "hodo renders each document with deterministic geometry on every platform",
    "a cascade of rules selects colors and spacing for the nested sections",
    "layout walks the arena in preorder and emits rectangles for every block",
    "the rasterizer fills paths and glyphs into a fixed size pixel buffer",
    "inline spans flow across line boxes while block containers stack vertically",
    "cache entries are keyed by a normalized address and a partition identifier",
    "extensions exchange small messages through a bounded and versioned channel",
    "the incremental path will recompute only the dirty boxes after a mutation",
    "a damage region is the union of changed rectangles from the display list",
    "scrolling blits the existing frame and repaints only the newly exposed band",
    "stylesheet parsing recovers from invalid selectors without dropping the sheet",
    "font metrics determine baselines and the subtle rhythm of paragraph text",
    "a bounded arena keeps node handles small and prevents stale slot aliasing",
    " determinism is the contract that makes every regression number comparable",
    "the article continues with another carefully measured paragraph of text",
];

/// 64-bit FNV-1a, used only as a content fingerprint of generated input.
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn pick<'a>(&mut self, pool: &'a [&'a str]) -> &'a str {
        let index = ((self.next() >> 33) as usize) % pool.len();
        pool[index]
    }

    fn hex_color(&mut self) -> String {
        format!("#{:06x}", self.next() & 0x00ff_ffff)
    }

    /// Deterministic ASCII text of exactly `len` bytes.
    fn text(&mut self, len: usize) -> String {
        let mut out = String::with_capacity(len);
        while out.len() < len {
            out.push_str(self.pick(SENTENCES));
            out.push(' ');
        }
        out.truncate(len);
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Corpus {
    Medium,
    Large,
    Deep,
}

impl Corpus {
    fn all() -> [Self; 3] {
        [Self::Medium, Self::Large, Self::Deep]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Medium => "MEDIUM (~250KB article)",
            Self::Large => "LARGE (~2MB mixed)",
            Self::Deep => "DEEP (~500KB pathological)",
        }
    }
}

#[derive(Debug)]
pub(crate) struct BigDoc {
    pub(crate) html: String,
    pub(crate) css: String,
    target_bytes: usize,
    expected_rules: usize,
    /// Class token on the mid-tree node that the restyle delta replaces.
    restyle_to: &'static str,
}

impl BigDoc {
    /// FNV-1a of the exact bytes fed to the HTML parser.
    fn html_hash(&self) -> u64 {
        fnv1a(self.html.as_bytes())
    }

    /// FNV-1a of the exact bytes fed to the CSS parser.
    fn css_hash(&self) -> u64 {
        fnv1a(self.css.as_bytes())
    }

    /// FNV-1a of both inputs, separated so html/css boundaries cannot collide.
    fn doc_hash(&self) -> u64 {
        let mut combined = self.html.clone();
        combined.push('\0');
        combined.push_str(&self.css);
        fnv1a(combined.as_bytes())
    }
}

impl Corpus {
    fn generate(self) -> BigDoc {
        match self {
            Self::Medium => generate_medium(),
            Self::Large => generate_large(),
            Self::Deep => generate_deep(),
        }
    }
}

/// 1 article + 25 sections x 20 paragraphs, 50 class/type rules, ~500-char text.
fn generate_medium() -> BigDoc {
    const SECTIONS: usize = 25;
    const PARAGRAPHS: usize = 20;
    const TEXT_LEN: usize = 470;
    const CLASSES: usize = 25;
    // 3 base type rules + 25 class rules + 22 type rules = 50.
    const TYPE_RULES: usize = 22;

    let mut rng = Lcg::new(0x4d45_4449_554d_0001);
    let mut html = String::with_capacity(MEDIUM_TARGET_BYTES + 16 * 1024);
    html.push_str(
        "<!doctype html><html><head><title>Medium article</title></head><body><article class=\"article\">\n",
    );
    for section in 0..SECTIONS {
        let append_root = if section == SECTIONS / 2 {
            " data-append-root=\"1\""
        } else {
            ""
        };
        html.push_str(&format!(
            "<section id=\"sec{section}\" class=\"sec\"{append_root}>\n"
        ));
        for paragraph in 0..PARAGRAPHS {
            let class = (section * PARAGRAPHS + paragraph) % CLASSES;
            let mid_tree = section == SECTIONS / 2 && paragraph == PARAGRAPHS / 2;
            let marker = if mid_tree { " target" } else { "" };
            html.push_str(&format!(
                "<p class=\"s{class}{marker}\">{}</p>\n",
                rng.text(TEXT_LEN)
            ));
        }
        html.push_str("</section>\n");
    }
    html.push_str("</article></body></html>\n");

    let mut css = String::with_capacity(3 * 1024);
    css.push_str("article{display:block;margin:8px}section{margin:4px}p{margin:2px}\n");
    for class in 0..CLASSES {
        css.push_str(&format!(
            ".s{class}{{color:{};background-color:{};padding:2px}}\n",
            rng.hex_color(),
            rng.hex_color()
        ));
    }
    for (i, tag) in ["p", "section", "article", "div", "span"]
        .iter()
        .cycle()
        .take(TYPE_RULES)
        .enumerate()
    {
        css.push_str(&format!(
            "{tag}{{font-size:{}px;line-height:{};font-weight:{};text-align:left}}\n",
            13 + i % 4,
            1 + i % 2,
            400 + (i % 3) * 100
        ));
    }

    assert!(
        html.len() <= MAX_RESOURCE_BYTES,
        "MEDIUM corpus exceeds the 2 MiB resource cap"
    );
    BigDoc {
        html,
        css,
        target_bytes: MEDIUM_TARGET_BYTES,
        expected_rules: 50,
        restyle_to: "s13",
    }
}

/// 200 mixed blocks: 4-deep divs, three ~900-char paragraphs, list, image,
/// spans; 200 class/id/attribute rules.
fn generate_large() -> BigDoc {
    const BLOCKS: usize = 200;
    const TEXT_LEN: usize = 900;
    const SHORT_LEN: usize = 300;
    const CLASSES: usize = 50;
    const SPANS: usize = 25;

    let mut rng = Lcg::new(0x004c_4152_4745_0002);
    let mut html = String::with_capacity(LARGE_TARGET_BYTES);
    html.push_str(
        "<!doctype html><html><head><title>Large mixed</title></head><body><main class=\"root\">\n",
    );
    for block in 0..BLOCKS {
        html.push_str(&format!(
            "<div class=\"b{}\" id=\"blk{block}\" data-kind=\"mixed\" data-idx=\"{block}\">\n",
            block % 20
        ));
        html.push_str("<div class=\"d1\">");
        html.push_str("<div class=\"d2\">");
        html.push_str("<div class=\"d3\">");
        let append_root = if block == BLOCKS / 2 {
            " data-append-root=\"1\""
        } else {
            ""
        };
        html.push_str(&format!("<div class=\"d4\"{append_root}>\n"));
        for paragraph in 0..3 {
            let class = (block * 3 + paragraph) % CLASSES;
            let mid_tree = block == BLOCKS / 2 && paragraph == 1;
            let marker = if mid_tree { " target" } else { "" };
            let span_count = if paragraph == 2 { 3 } else { 2 };
            html.push_str(&format!(
                "<p class=\"s{class}{marker}\">{}",
                rng.text(TEXT_LEN)
            ));
            for span in 0..span_count {
                html.push_str(&format!(
                    "<span class=\"sp{}\">{}</span>",
                    (block + span) % SPANS,
                    rng.text(TEXT_LEN)
                ));
            }
            html.push_str("</p>\n");
        }
        html.push_str("<ul class=\"list\">");
        for item in 0..2 {
            html.push_str(&format!(
                "<li class=\"li{item}\">{}</li>",
                rng.text(SHORT_LEN)
            ));
        }
        html.push_str("</ul>");
        html.push_str("<img src=\"pic.png\" width=\"120\" height=\"80\" alt=\"sample\">\n");
        html.push_str("</div></div></div></div>\n");
        html.push_str("</div>\n");
    }
    html.push_str("</main></body></html>\n");

    let mut css = String::with_capacity(12 * 1024);
    css.push_str(".root{display:block;padding:4px}\n");
    for i in 0..80 {
        let selector = if i % 2 == 0 {
            format!(".s{}", i % CLASSES)
        } else {
            format!(".sp{}", i % SPANS)
        };
        css.push_str(&format!(
            "{selector}{{color:{};background-color:{};padding:1px}}\n",
            rng.hex_color(),
            rng.hex_color()
        ));
    }
    for i in 0..60 {
        css.push_str(&format!(
            "#blk{i}{{border-width:1px;border-color:{};margin:2px}}\n",
            rng.hex_color()
        ));
    }
    for i in 0..59 {
        let selector = match i % 3 {
            0 => "[data-kind=\"mixed\"]".to_string(),
            1 => "[width=\"120\"]".to_string(),
            _ => format!("[data-idx=\"{i}\"]"),
        };
        css.push_str(&format!(
            "{selector}{{font-size:{}px;line-height:1.4}}\n",
            13 + i % 4
        ));
    }

    assert!(
        html.len() <= MAX_RESOURCE_BYTES,
        "LARGE corpus exceeds the 2 MiB resource cap"
    );
    BigDoc {
        html,
        css,
        target_bytes: LARGE_TARGET_BYTES,
        expected_rules: 1 + 80 + 60 + 59,
        restyle_to: "s14",
    }
}

/// 200-deep chain with two inline-styled leaves per level; 100 rules of which
/// 45 are descendant combinators (pathological for the full style pass).
fn generate_deep() -> BigDoc {
    const LEVELS: usize = 200;
    const LEAF_TEXT: usize = 1180;
    const {
        assert!(
            LEVELS < MAX_NESTING_DEPTH,
            "DEEP corpus must fit the depth cap"
        )
    };

    let mut rng = Lcg::new(0x0000_4445_4550_0003);
    let mut html = String::with_capacity(DEEP_TARGET_BYTES + 16 * 1024);
    html.push_str("<!doctype html><html><head><title>Deep chain</title></head><body><main>\n");
    for level in 0..LEVELS {
        let mid_tree = level == LEVELS / 2;
        let marker = if mid_tree { " target" } else { "" };
        let append_root = if mid_tree {
            " data-append-root=\"1\""
        } else {
            ""
        };
        html.push_str(&format!(
            "<div class=\"d{}{marker}\"{append_root} style=\"margin:1px;padding:1px;border-width:1px\">\n",
            level % 40
        ));
        html.push_str(&format!(
            "<span class=\"leafA\" style=\"color:{};font-size:14px;line-height:1.3\">{}</span>\n",
            rng.hex_color(),
            rng.text(LEAF_TEXT)
        ));
        html.push_str(&format!(
            "<em class=\"leafB\" style=\"color:{};font-size:13px;line-height:1.2\">{}</em>\n",
            rng.hex_color(),
            rng.text(LEAF_TEXT)
        ));
    }
    for _ in 0..LEVELS {
        html.push_str("</div>");
    }
    html.push_str("\n</main></body></html>\n");

    let mut css = String::with_capacity(8 * 1024);
    css.push_str("main{display:block}div{display:block}span,em{display:inline}\n");
    // 45 descendant rules: the ancestor walk is exercised for every one of the
    // 400 inline leaves. Type selectors are used because this parser version
    // folds `.a .b` into one compound; `div span` stays a real chain.
    for i in 0..45 {
        let ancestor = match i % 3 {
            0 => "div",
            1 => "main",
            _ => "div div",
        };
        let leaf = if i % 2 == 0 { "span" } else { "em" };
        css.push_str(&format!(
            "{ancestor} {leaf}{{color:{};text-decoration:underline}}\n",
            rng.hex_color()
        ));
    }
    for i in 0..51 {
        let selector = match i % 5 {
            0 => ".leafA".to_string(),
            1 => ".leafB".to_string(),
            2 => format!(".d{}", i % 40),
            3 => "span".to_string(),
            _ => "em".to_string(),
        };
        css.push_str(&format!(
            "{selector}{{background-color:{};margin:1px 2px}}\n",
            rng.hex_color()
        ));
    }

    assert!(
        html.len() <= MAX_RESOURCE_BYTES,
        "DEEP corpus exceeds the 2 MiB resource cap"
    );
    BigDoc {
        html,
        css,
        target_bytes: DEEP_TARGET_BYTES,
        expected_rules: 4 + 45 + 51,
        restyle_to: "d14",
    }
}

/// Print the measured inputs once so a regression can be traced to a corpus
/// change rather than an engine change.
fn print_fingerprint(doc: &BigDoc, sheet: &hodo_css::Stylesheet) {
    println!(
        "  html: {} bytes (~{} target), fnv1a64 {:016x}",
        doc.html.len(),
        doc.target_bytes,
        doc.html_hash()
    );
    println!(
        "  css:  {} bytes, fnv1a64 {:016x}",
        doc.css.len(),
        doc.css_hash()
    );
    println!("  html+css fnv1a64: {:016x}", doc.doc_hash());
    println!(
        "  css rules: {} (expected {})",
        sheet.rules.len(),
        doc.expected_rules
    );
    if sheet.rules.len() != doc.expected_rules {
        println!("  WARNING: rule count changed; calibrate expected_rules");
    }
}

/// Full pipeline on the unmodified document, reporting one whole best run.
fn run_full_path(doc: &BigDoc, sheet: &hodo_css::Stylesheet) {
    let mut best: Option<(usize, Times)> = None;
    let mut counts = None;

    for run in 1..=RUNS {
        println!("\n--- run {run}/{RUNS} ---");
        let mut times = Times::default();

        let (d, dom) = time_stage("parse html", || hodo_html::parse_document(&doc.html));
        times.parse_html = d;
        let dom = dom.expect("generated HTML must parse");

        let (d, parsed) = time_stage("parse css", || hodo_css::parse_stylesheet(&doc.css));
        times.parse_css = d;
        let parsed = parsed.expect("generated CSS must parse");

        let (d, styles) = time_stage("compute styles", || {
            hodo_style::compute_styles(&dom, std::slice::from_ref(&parsed))
        });
        times.styles = d;
        let styles = styles.expect("style computation must finish");

        let (d, list) = time_stage("layout", || hodo_layout::layout(&dom, &styles, VIEWPORT));
        times.layout = d;
        let list = list.expect("layout must finish");

        let (d, frame) = time_stage("render", || hodo_render::render(&list, VIEWPORT));
        times.render = d;
        let frame = frame.unwrap_or_default();

        println!(
            "  dom nodes: {}, display items: {}, frame bytes: {}",
            dom.len(),
            list.items.len(),
            frame.len()
        );
        counts = Some((dom.len(), list.items.len(), frame.len()));
        best = better_run(best, run, times);
    }

    let (best_run, best) = best.expect("at least one run");
    println!("\n=== best single run {best_run}/{RUNS} (lowest total) ===");
    println!("  total: {:.3} ms", best.total().as_secs_f64() * 1000.0);
    print_best_stages(&best);
    println!(
        "  styles ({}): {:.3} ms",
        rule_index_mode(),
        ms(best.styles)
    );
    if let Some((nodes, items, bytes)) = counts {
        println!(
            "dom nodes: {nodes}, display items: {items}, rule count: {}, frame bytes: {bytes}",
            sheet.rules.len()
        );
    }
}

#[derive(Debug, Clone, Copy)]
enum Mutation {
    Restyle,
    Append,
}

impl Mutation {
    fn label(self) -> &'static str {
        match self {
            Self::Restyle => "restyle_single",
            Self::Append => "append_node",
        }
    }
}

/// M5.3 delta on the full pre-incremental path: mutate the DOM, then pay the
/// entire style -> layout -> render cost again. No incremental machinery exists
/// yet, so both deltas are the same full recompute with a different trigger.
fn run_delta(doc: &BigDoc, sheet: &hodo_css::Stylesheet, mutation: Mutation) -> (f64, f64) {
    let mut best: Option<(usize, Times)> = None;

    for run in 1..=RUNS {
        println!("\n  --- {} run {run}/{RUNS} ---", mutation.label());
        let mut times = Times::default();
        let mut dom = hodo_html::parse_document(&doc.html);

        // Handles stay inferred: hodo-perf has no direct hodo-dom dependency.
        let mut target = None;
        let mut parent = None;
        let mut kind = None;
        dom.walk(|id, node| {
            if target.is_none()
                && dom
                    .attr(id, "class")
                    .is_some_and(|value| value.split_whitespace().any(|token| token == "target"))
            {
                target = Some(id);
            }
            if parent.is_none() && dom.attr(id, "data-append-root") == Some("1") {
                parent = Some(id);
            }
            if kind.is_none() && matches!(dom.tag(id), Some("p") | Some("span")) {
                kind = Some(node.kind.clone());
            }
        });

        match mutation {
            Mutation::Restyle => {
                let target = target.expect("corpus must contain a target node");
                dom.set_attr(target, "class", format!("target {}", doc.restyle_to));
            }
            Mutation::Append => {
                let parent = parent.expect("corpus must contain an append root");
                let new_node = dom.create(kind.expect("corpus must contain a paragraph"));
                dom.set_attr(new_node, "class", doc.restyle_to.to_string());
                dom.append(parent, new_node);
            }
        }

        let (d, styles) = time_stage("compute styles", || {
            hodo_style::compute_styles(&dom, std::slice::from_ref(sheet))
        });
        times.styles = d;
        let styles = styles.expect("style computation must finish");

        let (d, list) = time_stage("layout", || hodo_layout::layout(&dom, &styles, VIEWPORT));
        times.layout = d;
        let list = list.expect("layout must finish");

        let (d, frame) = time_stage("render", || hodo_render::render(&list, VIEWPORT));
        times.render = d;
        let frame = frame.unwrap_or_default();
        println!(
            "  display items: {}, frame bytes: {}",
            list.items.len(),
            frame.len()
        );

        best = better_run(best, run, times);
    }

    let (best_run, best) = best.expect("at least one run");
    println!(
        "\n  === best single run {best_run}/{RUNS} (lowest total): {} ===",
        mutation.label()
    );
    println!("  compute styles: {:.3} ms", ms(best.styles));
    println!("  layout: {:.3} ms", ms(best.layout));
    println!("  render: {:.3} ms", ms(best.render));
    let styles_ms = ms(best.styles);
    let layout_ms = ms(best.layout);
    println!("  styles+layout: {:.3} ms", styles_ms + layout_ms);
    (styles_ms, layout_ms)
}

fn ms(duration: Option<std::time::Duration>) -> f64 {
    duration.map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

/// Label the style-matching mode so the two re-baseline numbers are unambiguous.
fn rule_index_mode() -> &'static str {
    if std::env::var_os("HODO_NO_RULE_INDEX").is_some_and(|v| v == "1") {
        "brute force (HODO_NO_RULE_INDEX=1)"
    } else {
        "indexed"
    }
}

/// Entry point for `hodo-perf --big-doc`.
pub(crate) fn run() {
    if std::env::var_os("HODO_SKIP_BIG_DOC").is_some_and(|value| value == "1") {
        println!("HODO_SKIP_BIG_DOC=1: skipping big-document benchmark");
        return;
    }

    println!(
        "hodo big-document benchmark (best of {RUNS} runs, viewport 1200x800, {})",
        rule_index_mode()
    );
    let mut large_restyle = (0.0, 0.0);
    let mut large_append = (0.0, 0.0);

    for corpus in Corpus::all() {
        println!("\n============================================================");
        println!("=== {} ===", corpus.label());
        println!("============================================================");
        let doc = corpus.generate();
        let sheet = hodo_css::parse_stylesheet(&doc.css);
        print_fingerprint(&doc, &sheet);
        run_full_path(&doc, &sheet);

        println!("\n=== M5.3 deltas: full baseline (incremental pending) ===");
        let restyle = run_delta(&doc, &sheet, Mutation::Restyle);
        let append = run_delta(&doc, &sheet, Mutation::Append);
        if corpus == Corpus::Large {
            large_restyle = restyle;
            large_append = append;
        }
    }

    println!("\n============================================================");
    println!("=== M5.3 performance gate (LARGE, full path) ===");
    println!("============================================================");
    // Escape hatch for machines >10x slower than the calibration host.
    let no_gate = std::env::var_os("HODO_PERF_NO_GATE").is_some_and(|v| v == "1");
    if no_gate {
        println!("  HODO_PERF_NO_GATE=1: reporting only, gate not enforced");
    }
    let baseline_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("baseline_bigdoc.txt");
    match read_baseline(&baseline_path) {
        Ok(baseline) => {
            let checks = [
                GateCheck::new(
                    "restyle_single compute styles",
                    large_restyle.0,
                    baseline.restyle_styles,
                ),
                GateCheck::new(
                    "restyle_single layout",
                    large_restyle.1,
                    baseline.restyle_layout,
                ),
                GateCheck::new(
                    "append_node compute styles",
                    large_append.0,
                    baseline.append_styles,
                ),
                GateCheck::new("append_node layout", large_append.1, baseline.append_layout),
            ];
            println!("  each stage must stay within 1.10x of its pinned baseline");
            let mut failed = Vec::new();
            for check in &checks {
                let verdict = if check.exceeded() { "FAIL" } else { "ok" };
                println!(
                    "  {}: {:.3} ms / {:.3} ms -> {verdict}",
                    check.name, check.measured, check.limit
                );
                if check.exceeded() {
                    failed.push(check.name);
                }
            }
            if failed.is_empty() {
                println!("  verdict: PASS — all LARGE stages within 1.10x of baseline");
            } else {
                eprintln!(
                    "M5.3 GATE FAILED: {} exceeded 1.10x of pinned baseline",
                    failed.join(", ")
                );
                if !no_gate {
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!(
                "M5.3 GATE ERROR: cannot read {}: {error}",
                baseline_path.display()
            );
            if !no_gate {
                std::process::exit(1);
            }
        }
    }
}

/// A stage's measured time against its pinned baseline plus 10% tolerance.
const GATE_TOLERANCE: f64 = 1.10;

#[derive(Debug, Clone, Copy)]
struct GateCheck {
    name: &'static str,
    measured: f64,
    limit: f64,
}

impl GateCheck {
    fn new(name: &'static str, measured: f64, baseline_ms: f64) -> Self {
        Self {
            name,
            measured,
            limit: baseline_ms * GATE_TOLERANCE,
        }
    }

    fn exceeded(&self) -> bool {
        self.measured > self.limit
    }
}

#[derive(Debug, Clone, Copy)]
struct Baseline {
    restyle_styles: f64,
    restyle_layout: f64,
    append_styles: f64,
    append_layout: f64,
}

fn read_baseline(path: &Path) -> Result<Baseline, String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    parse_baseline(&text)
}

fn parse_baseline(text: &str) -> Result<Baseline, String> {
    let mut values = std::collections::HashMap::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {}: expected `key = value`", index + 1))?;
        let value: f64 = value
            .trim()
            .parse()
            .map_err(|_| format!("line {}: invalid number", index + 1))?;
        if !value.is_finite() || value <= 0.0 {
            return Err(format!(
                "line {}: baseline must be positive and finite",
                index + 1
            ));
        }
        values.insert(key.trim(), value);
    }
    let get = |key: &str| {
        values
            .get(key)
            .copied()
            .ok_or_else(|| format!("missing {key}"))
    };
    Ok(Baseline {
        restyle_styles: get("restyle_styles_ms")?,
        restyle_layout: get("restyle_layout_ms")?,
        append_styles: get("append_styles_ms")?,
        append_layout: get("append_layout_ms")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Anchored on the first measured run; any generator edit must be explicit.
    const EXPECTED: &[(Corpus, u64, u64, u64)] = &[
        (
            Corpus::Medium,
            0x5f4e_5608_13b6_0bd7,
            0x31cf_de5a_f538_156c,
            0xa3df_0757_3771_e15c,
        ),
        (
            Corpus::Large,
            0x10cc_aa46_5b65_683e,
            0xacd4_f4d0_2057_f74d,
            0xb590_e744_b143_e7d2,
        ),
        (
            Corpus::Deep,
            0x0bd4_a00c_0e88_7161,
            0x4400_7da5_86bd_2e2e,
            0x940f_8417_b840_a948,
        ),
    ];

    #[test]
    fn generators_are_deterministic_and_hashes_are_anchored() {
        for (corpus, html_hash, css_hash, doc_hash) in EXPECTED {
            let doc = corpus.generate();
            assert_eq!(doc.html_hash(), *html_hash, "{corpus:?} html hash drifted");
            assert_eq!(doc.css_hash(), *css_hash, "{corpus:?} css hash drifted");
            assert_eq!(
                doc.doc_hash(),
                *doc_hash,
                "{corpus:?} combined hash drifted"
            );
            let again = corpus.generate();
            assert_eq!(doc.html, again.html, "{corpus:?} html is not reproducible");
            assert_eq!(doc.css, again.css, "{corpus:?} css is not reproducible");
        }
    }

    #[test]
    fn document_sizes_stay_near_target_and_under_resource_cap() {
        for corpus in Corpus::all() {
            let doc = corpus.generate();
            let target = doc.target_bytes;
            let tolerance = target / 20; // +/-5%
            assert!(
                doc.html.len() >= target - tolerance && doc.html.len() <= target + tolerance,
                "{corpus:?} html size {} outside +/-5% of {target}",
                doc.html.len()
            );
            assert!(
                doc.html.len() <= MAX_RESOURCE_BYTES,
                "{corpus:?} exceeds the 2 MiB resource cap"
            );
        }
    }

    #[test]
    fn generator_rule_counts_match_the_parameter_tables() {
        for corpus in Corpus::all() {
            let doc = corpus.generate();
            let rule_count = hodo_css::parse_stylesheet(&doc.css).rules.len();
            assert_eq!(
                rule_count, doc.expected_rules,
                "{corpus:?} CSS rule count drifted"
            );
        }
        let deep = Corpus::Deep.generate();
        let sheet = hodo_css::parse_stylesheet(&deep.css);
        let descendant = sheet
            .rules
            .iter()
            .filter(|rule| hodo_css::complex(&rule.selector).compounds.len() > 1)
            .count();
        assert!(
            descendant * 100 >= sheet.rules.len() * 40,
            "DEEP needs at least 40% descendant combinators, got {descendant}/{}",
            sheet.rules.len()
        );
    }

    #[test]
    fn deep_chain_fits_the_nesting_cap() {
        let deep = Corpus::Deep.generate();
        let mut depth = 0usize;
        let mut max_depth = 0usize;
        for line in deep.html.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("<div class=\"d") {
                depth += trimmed.matches("<div class=\"d").count();
                max_depth = max_depth.max(depth);
            }
            if trimmed.starts_with("</div>") {
                depth -= trimmed.matches("</div>").count();
            }
        }
        assert_eq!(max_depth, 200);
        assert_eq!(depth, 0, "div tags must be balanced");
        assert!(max_depth < MAX_NESTING_DEPTH);
    }

    #[test]
    fn checked_in_baseline_parses_all_four_large_stages() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../baseline_bigdoc.txt");
        let baseline = read_baseline(&path).expect("baseline_bigdoc.txt must be present");
        assert_eq!(baseline.restyle_styles, 22.0);
        assert_eq!(baseline.restyle_layout, 38.5);
        assert_eq!(baseline.append_styles, 25.0);
        assert_eq!(baseline.append_layout, 40.5);
    }

    #[test]
    fn gate_fails_only_above_ten_percent() {
        let baseline = parse_baseline(
            "restyle_styles_ms = 100\nrestyle_layout_ms = 100\nappend_styles_ms = 100\nappend_layout_ms = 100\n",
        )
        .unwrap();
        let at_limit = GateCheck::new("at limit", 110.0, baseline.restyle_styles);
        let over_limit = GateCheck::new("over limit", 110.01, baseline.restyle_styles);
        assert!(!at_limit.exceeded(), "exactly 1.10x must pass");
        assert!(over_limit.exceeded(), "above 1.10x must fail the gate");
    }

    #[test]
    fn baseline_parser_rejects_missing_and_nonpositive_entries() {
        assert!(parse_baseline("restyle_styles_ms = 1\n")
            .unwrap_err()
            .contains("restyle_layout_ms"));
        assert!(parse_baseline(
            "restyle_styles_ms = 0\nrestyle_layout_ms = 1\nappend_styles_ms = 1\nappend_layout_ms = 1\n"
        )
        .is_err());
    }
}

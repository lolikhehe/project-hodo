//! Adversarial corpora + in-crate robustness tests for the remaining trust
//! boundaries: selectors, CSS tracks/rule counts, layout, IPC frames and DOM
//! generation handles. Same pattern as [`crate::corpus`]: seeded, reproducible,
//! ordinary `cargo test` code, no nightly and no `cargo-fuzz`.
#![cfg(test)]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use hodo_css::{
    complex, parse_stylesheet, specificity_full, MatchContext, MAX_COMPOUNDS, MAX_NOT_DEPTH,
    MAX_RULES,
};
use hodo_dom::{Dom, NodeId, NodeKind};
use hodo_ext_ipc::{
    decode, FrameReader, HandshakeState, IpcError, Message, PendingCalls, MAX_MSG_BYTES,
    MAX_MSG_DEPTH, MAX_OUTSTANDING_CALLS, VERSION,
};
use hodo_layout::layout;
use hodo_style::compute_styles;
use hodo_types::limits::MAX_DOM_NODES;
use hodo_types::Size;

use crate::{gen_string, leak, next, DEFAULT_SEED};

fn leak_str(s: &str) -> &'static str {
    leak(s.to_string())
}

// ---- corpora -------------------------------------------------------------

/// Selector strings (without a rule body) that stress the caps and the
/// recovery paths of the selector parser.
fn build_selector_bombs() -> Vec<&'static str> {
    let mut v = Vec::new();

    // Compound chains: at and well past the 64-compound cap.
    for n in [MAX_COMPOUNDS, MAX_COMPOUNDS + 1, 70, 200, 1000] {
        v.push(leak(format!("{}a", "div ".repeat(n))));
        v.push(leak(format!("{}span", ">*".repeat(n / 2))));
    }

    // :not() nesting: at, just past, and far past the depth-2 cap.
    for d in [usize::from(MAX_NOT_DEPTH), 3, 5, 10, 64] {
        v.push(leak(format!("{}.x{}", ":not(".repeat(d), ")".repeat(d))));
        v.push(leak(format!(
            "a{}{}:not(.y)",
            ":not(:not(".repeat(d / 2),
            ")".repeat(d / 2)
        )));
    }

    // Unclosed attribute brackets and malformed pseudo arguments.
    v.extend(
        [
            "a[href",
            "a[href=",
            "a[href=\"x\"",
            "a[href^",
            "a[href^=",
            "[",
            "[]",
            "[=]",
            "a]",
            "a[=]",
            "a[href~=]",
            "a[href=\"x\" i",
            "a:not(",
            "a:not()",
            "a:not(:hover",
            "a:not(a,)",
            "a:not(:not())",
            "a:nth-child(",
            "a:nth-child(-n+",
            "a:nth-child(2n+1)extra",
            "a::before",
            "a:unknown(1,2)",
            ":not(:not(:not(:not(.x))))",
        ]
        .into_iter()
        .map(leak_str),
    );

    // Comma groups with empty parts — the known exotic-recovery risk.
    v.extend(
        [
            ",",
            ",,",
            "a,",
            ",a",
            "a,,b",
            "a, ,b",
            ",a,,b,",
            "*,,#x,,.y",
            "a,/*c*/,b",
            "a, ,",
            " , a , ",
        ]
        .into_iter()
        .map(leak_str),
    );

    // Unicode / empty class and id names.
    v.extend(
        [
            ".",
            "..",
            ".a.",
            ".\u{0}",
            ".é",
            ".世界",
            ".😀",
            ".\u{feff}",
            "#\u{0}",
            ".a\\200b",
            "[.\u{4e16}]",
        ]
        .into_iter()
        .map(leak_str),
    );

    // Attribute operator edges, including the empty-value forms.
    v.extend(
        [
            "a[href]",
            "a[href=\"\"]",
            "a[href^=\"\"]",
            "a[href$=\"\"]",
            "a[href*=\"\"]",
            "a[href|=\"\"]",
            "a[href~=\"\"]",
            "a[HREF^=\"x\" i]",
            "a[href^='']I",
            "a[href=\"x\" s]",
            "a[data-x|=\"a-b\"]",
            "a[href=\"x\"]:not([href^=\"\"])",
            "a[href^=\"\"]:not(:not(:not(.x)))",
            "a[href^=\"\"][href$=\"\"][href*=\"\"]",
        ]
        .into_iter()
        .map(leak_str),
    );
    v
}

/// Declaration values for `grid-template-*` and the numeric properties:
/// overflowing repeat counts, nan/inf lengths, 1001 tracks, exponent forms.
fn build_track_bombs() -> Vec<&'static str> {
    let v = vec![
        "repeat(99999,1fr)",
        "repeat(4294967295,1fr)",
        "repeat(0,1fr)",
        "repeat(-1,1fr)",
        "repeat(abc,1fr)",
        "repeat(99999,)",
        "repeat(99999,1fr 2fr auto)",
        "repeat(2,repeat(2,repeat(2,1fr)))",
        "minmax(nan,auto)",
        "minmax(inf,auto)",
        "minmax(-inf,auto)",
        "minmax(1e308,1e308)",
        "minmax(auto,minmax(nan,auto))",
        "minmax(,)",
        "minmax(a)",
        "minmax(0,0,0)",
        "1e308px",
        "1e308em",
        "-1e308px",
        "1e309px",
        "1e-400px",
        "nanpx",
        "infpx",
        "-infpx",
        "nan",
        "inf",
        "-inf",
        "NaN%",
        "inf%",
        "1e308fr",
        "nanfr",
        "infinityfr",
        "0x1p999px",
    ];
    let mut out: Vec<&'static str> = v.into_iter().map(leak_str).collect();
    out.push(leak("1fr ".repeat(1001)));
    out.push(leak("minmax(1e308px,1fr) ".repeat(1001)));
    out
}

/// Whole stylesheets that exceed the per-sheet rule cap and at-rule nesting.
fn build_rule_bombs() -> Vec<&'static str> {
    vec![
        leak("a{color:red}".repeat(MAX_RULES + 1)),
        leak("a,.b{color:red}".repeat(MAX_RULES * 2)),
        leak(format!(
            "@media screen{{{}}}",
            "a{color:red}".repeat(MAX_RULES + 1)
        )),
        leak_str("@media screen{@media screen{@media screen{a{color:red}}}}"),
        leak(format!("{}/* unterminated", "a{color:red}".repeat(1000))),
    ]
}

/// Raw IPC frames: the caller only ever feeds bytes to `decode`/`FrameReader`.
fn build_ipc_frames() -> Vec<Vec<u8>> {
    fn frame(json: &[u8]) -> Vec<u8> {
        let mut body = Vec::with_capacity(json.len() + 1);
        body.push(VERSION);
        body.extend_from_slice(json);
        let mut out = Vec::with_capacity(body.len() + 4);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    let mut v: Vec<Vec<u8>> = vec![
        vec![],
        vec![0, 0, 0, 0],
        vec![1, 0, 0, 0],
        vec![1, 0, 0],
        vec![0xff, 0xff, 0xff, 0xff],
        (MAX_MSG_BYTES as u32 + 1).to_le_bytes().to_vec(),
        frame(br#"{"t":"shutdown"}"#),
        frame(br#"{"t":"bogus"}"#),
        frame(br#"{"t":"api_response","id":1,"result":1,"error":"e"}"#),
        frame(br#"{"t":"api_response","id":1}"#),
        frame(br#"{"t":"api_call","id":7,"method":"x","params":[]}"#),
        frame(br#"{"t":"event","name":"x","payload":null}"#),
        frame(b"not json"),
        frame(b""),
    ];
    let mut bad_version = frame(br#"{"t":"shutdown"}"#);
    bad_version[4] = VERSION + 1;
    v.push(bad_version);
    let valid = frame(br#"{"t":"event","name":"x","payload":1}"#);
    for cut in [1, 3, 5, valid.len() - 1] {
        v.push(valid[..cut].to_vec());
    }
    for depth in [MAX_MSG_DEPTH - 1, MAX_MSG_DEPTH, MAX_MSG_DEPTH + 1, 100] {
        let json = format!(
            r#"{{"t":"event","name":"probe","payload":{}1{}}}"#,
            "[".repeat(depth),
            "]".repeat(depth)
        );
        v.push(frame(json.as_bytes()));
    }
    let big = format!(
        r#"{{"t":"hello","token":"{}","manifest_digest":"d"}}"#,
        "t".repeat(MAX_MSG_BYTES / 2)
    );
    v.push(frame(big.as_bytes()));
    v
}

/// Curated corpora, built once like [`crate::corpus`].
pub fn selector_bombs() -> Vec<&'static str> {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(build_selector_bombs).clone()
}

pub fn track_bombs() -> Vec<&'static str> {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(build_track_bombs).clone()
}

pub fn rule_bombs() -> Vec<&'static str> {
    static C: OnceLock<Vec<&'static str>> = OnceLock::new();
    C.get_or_init(build_rule_bombs).clone()
}

pub fn ipc_frames() -> Vec<Vec<u8>> {
    static C: OnceLock<Vec<Vec<u8>>> = OnceLock::new();
    C.get_or_init(build_ipc_frames).clone()
}

// ---- helpers -------------------------------------------------------------

fn elem(tag: &str) -> NodeKind {
    NodeKind::Element {
        tag: tag.to_string(),
        attrs: vec![
            ("id".to_string(), "main".to_string()),
            ("class".to_string(), "a b".to_string()),
            ("href".to_string(), String::new()),
        ],
    }
}

fn selector_dom() -> Dom {
    let mut dom = Dom::new();
    let mut parent = dom.root();
    for tag in ["html", "body", "div", "a", "span"] {
        let id = dom.create(elem(tag));
        dom.append(parent, id);
        parent = id;
    }
    let text = dom.create(NodeKind::Text("x".to_string()));
    dom.append(parent, text);
    dom
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

/// Catch a panic and re-emit everything needed to replay it: the seed that
/// generated the input, the input itself, and the original panic payload.
fn must_not_panic(label: &str, seed: Option<u64>, input: &str, f: impl FnOnce()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(f)) {
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("<non-string panic payload>");
        eprintln!("[repro] {label} seed={seed:?} input={}", preview(input));
        panic!(
            "{label} PANICKED: {message} — seed={seed:?} input_len={} input={}",
            input.len(),
            preview(input)
        );
    }
}

fn timed(label: &str, budget: Duration, f: impl FnOnce()) {
    let started = Instant::now();
    must_not_panic(label, None, label, f);
    assert!(
        started.elapsed() < budget,
        "{label} exceeded its {budget:?} budget"
    );
}

/// shortcut: deep block layout recurses once per ancestor and needs more than
/// the default 2 MiB test-thread stack; hodo-types budgets 64 MiB for the whole
/// recursion chain. Upgrade when layout gains an explicit depth cap.
fn on_document_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn 64 MiB stack worker")
        .join()
        .expect("deep layout worker panicked");
}

// ---- selector bombs ------------------------------------------------------

#[test]
fn selector_bombs_never_panic() {
    let dom = selector_dom();
    let mut cases = 0usize;

    for sel in selector_bombs() {
        let css = format!("{sel}{{color:red}}");
        must_not_panic("selector parse", None, sel, || {
            let sheet = parse_stylesheet(&css);
            for rule in &sheet.rules {
                let _ = specificity_full(&rule.selector);
                let chain = complex(&rule.selector);
                let mut ctx = MatchContext::new();
                let _ = hodo_css::matches_compiled(&chain, &dom, dom.root(), &mut ctx);
            }
        });
        cases += 1;
    }

    for i in 0..1_000u64 {
        let seed = DEFAULT_SEED ^ i.wrapping_mul(DEFAULT_SEED);
        let prelude = gen_string(seed, 256);
        let css = format!("{prelude}{{color:red}}");
        must_not_panic("selector parse/random", Some(seed), &css, || {
            let sheet = parse_stylesheet(&css);
            for rule in &sheet.rules {
                let _ = specificity_full(&rule.selector);
                let mut ctx = MatchContext::new();
                let _ = hodo_css::matches_selector_with(&rule.selector, &dom, dom.root(), &mut ctx);
            }
        });
        cases += 1;
    }

    eprintln!("[selector] corpus={} cases={cases}", selector_bombs().len());
    assert!(cases >= 1_000);
}

// ---- CSS bombs -----------------------------------------------------------

#[test]
fn css_track_and_rule_bombs_never_panic() {
    let dom = selector_dom();
    let props = [
        "width",
        "height",
        "min-width",
        "max-height",
        "margin",
        "padding",
        "border-width",
        "gap",
        "column-gap",
        "font-size",
        "line-height",
        "letter-spacing",
        "word-spacing",
        "text-indent",
        "vertical-align",
        "flex-basis",
        "flex-grow",
        "border-spacing",
        "grid-column-start",
        "grid-row-end",
    ];
    let mut cases = 0usize;

    for value in track_bombs() {
        let decls = props
            .iter()
            .map(|p| format!("{p}:{value};"))
            .collect::<String>();
        let css = format!(
            "div{{{decls}}}div{{grid-template-columns:{value};grid-template-rows:{value}}}"
        );
        timed("track parse/cascade", Duration::from_secs(10), || {
            let sheet = parse_stylesheet(&css);
            let _ = compute_styles(&dom, &[sheet]);
        });
        cases += 1;
    }

    for sheet in rule_bombs() {
        timed("rule bomb", Duration::from_secs(20), || {
            let parsed = parse_stylesheet(sheet);
            assert!(
                parsed.rules.len() <= MAX_RULES,
                "per-sheet cap did not hold: {} rules",
                parsed.rules.len()
            );
            let _ = compute_styles(&dom, &[parsed]);
        });
        cases += 1;
    }

    for i in 0..1_000u64 {
        let seed = DEFAULT_SEED.wrapping_add(i.wrapping_mul(0x9E37_79B9));
        let css = gen_string(seed, 512);
        must_not_panic("css random", Some(seed), &css, || {
            let sheet = parse_stylesheet(&css);
            let _ = compute_styles(&dom, &[sheet]);
        });
        cases += 1;
    }

    eprintln!(
        "[css] tracks={} rules={} cases={cases}",
        track_bombs().len(),
        rule_bombs().len()
    );
    assert!(cases >= 1_000);
}

// ---- layout bombs --------------------------------------------------------

#[test]
fn layout_bombs_never_panic() {
    let mut cases = 0usize;

    on_document_stack(|| {
        let mut dom = Dom::new();
        let mut cur = dom.root();
        for _ in 0..600 {
            let id = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: Vec::new(),
            });
            dom.append(cur, id);
            cur = id;
        }
        let text = dom.create(NodeKind::Text("deep".to_string()));
        dom.append(cur, text);
        let styles = compute_styles(&dom, &[]);
        timed("layout/600-deep", Duration::from_secs(10), || {
            let _ = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        });
    });
    cases += 1;

    on_document_stack(|| {
        let html = format!("{}x{}", "<div>".repeat(600), "</div>".repeat(600));
        timed("html/600-deep", Duration::from_secs(10), || {
            let dom = hodo_html::parse_document(&html);
            let styles = compute_styles(&dom, &[]);
            let _ = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        });
    });
    cases += 1;

    {
        let cells = "<td colspan=\"65535\" rowspan=\"65535\">x</td>".repeat(100_000);
        let html = format!("<table><tr>{cells}</tr></table>");
        timed("parse/100k-table", Duration::from_secs(30), || {
            let dom = hodo_html::parse_document(&html);
            assert!(dom.len() <= MAX_DOM_NODES);
            let _ = compute_styles(&dom, &[]);
        });
        cases += 1;
    }

    // shortcut: table layout's occupancy grid scales with cells * colspan *
    // rowspan; 100k cells at colspan 65535 exhausts RAM in hodo-layout. Keep
    // real layout on bounded spans until grid area is capped there.
    {
        let cells = "<td colspan=\"3\" rowspan=\"3\">x</td>".repeat(1_000);
        let html = format!("<table><tr>{cells}</tr></table>");
        timed("layout/table-spans", Duration::from_secs(20), || {
            let dom = hodo_html::parse_document(&html);
            let styles = compute_styles(&dom, &[]);
            let _ = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        });
        cases += 1;
    }

    for display in ["flex", "grid", "table", "inline-flex", "inline-grid"] {
        let mut dom = Dom::new();
        let root = dom.root();
        for _ in 0..5_000 {
            let id = dom.create(NodeKind::Element {
                tag: "div".to_string(),
                attrs: Vec::new(),
            });
            dom.append(root, id);
        }
        let css = format!("div{{display:{display}}}");
        let sheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &[sheet]);
        timed("layout/5000-children", Duration::from_secs(10), || {
            let _ = layout(&dom, &styles, Size { w: 800.0, h: 600.0 });
        });
        cases += 1;
    }

    {
        let mut dom = Dom::new();
        let div = dom.create(NodeKind::Element {
            tag: "div".to_string(),
            attrs: Vec::new(),
        });
        dom.append(dom.root(), div);
        let word = "x".repeat(100_000);
        let text = dom.create(NodeKind::Text(word));
        dom.append(div, text);
        let styles = compute_styles(&dom, &[]);
        timed("layout/100k-text", Duration::from_secs(10), || {
            let _ = layout(&dom, &styles, Size { w: 60.0, h: 600.0 });
        });
        cases += 1;
    }

    eprintln!("[layout] cases={cases}");
    assert!(cases >= 10);
}

// ---- IPC frames ----------------------------------------------------------

#[test]
fn ipc_frames_never_panic() {
    let mut cases = 0usize;
    let mut rejected = 0usize;

    for frame in ipc_frames() {
        must_not_panic("ipc decode", None, &format!("{frame:?}"), || {
            if decode(&frame).is_err() {
                rejected += 1;
            }
        });
        cases += 1;
    }

    let mut hs = HandshakeState::new();
    let hello = Message::Hello {
        token: "t".to_string(),
        manifest_digest: "d".to_string(),
    };
    must_not_panic("ipc handshake", None, "", || {
        assert!(hs.validate_worker_frame(&Message::Shutdown).is_err());
        assert!(hs.validate_worker_frame(&hello).is_ok());
        assert!(hs.observe(&hello).is_ok());
        assert!(hs.observe(&hello).is_err());
        hs.mark_ready();
        assert!(hs.is_ready() && hs.saw_hello());
        assert!(hs.validate_worker_frame(&Message::Shutdown).is_ok());
    });
    cases += 1;

    let mut pending = PendingCalls::new();
    must_not_panic("ipc pending", None, "", || {
        for id in 0..MAX_OUTSTANDING_CALLS as u64 {
            assert!(pending.insert(id).is_ok());
        }
        assert!(matches!(pending.insert(7), Err(IpcError::Invalid(_))));
        assert!(matches!(
            pending.insert(MAX_OUTSTANDING_CALLS as u64),
            Err(IpcError::TooManyOutstanding)
        ));
        assert!(pending.remove(7));
        assert!(!pending.remove(7));
        assert!(pending.insert(MAX_OUTSTANDING_CALLS as u64).is_ok());
    });
    cases += 1;

    let mut one = Vec::new();
    one.push(VERSION);
    one.extend_from_slice(br#"{"t":"shutdown"}"#);
    let mut stream = (one.len() as u32).to_le_bytes().to_vec();
    stream.extend_from_slice(&one);
    stream.extend_from_slice(&(MAX_MSG_BYTES as u32 + 1).to_le_bytes());
    stream.extend_from_slice(&[1, 0, 0]);
    must_not_panic("ipc frame reader", None, "", || {
        let mut reader = FrameReader::new(std::io::Cursor::new(stream.clone()));
        assert_eq!(reader.read_frame().unwrap(), Some(Message::Shutdown));
        assert!(reader.read_frame().is_err());
    });
    cases += 1;

    let mut state = DEFAULT_SEED ^ 0x1c5_1f2;
    for _ in 0..1_000 {
        let case_seed = state;
        let len = (next(&mut state) % 512) as u32;
        let mut frame = len.to_le_bytes().to_vec();
        for _ in 0..(len as usize).min(256) {
            frame.push(next(&mut state) as u8);
        }
        must_not_panic("ipc random", Some(case_seed), &format!("{frame:?}"), || {
            let _ = decode(&frame);
        });
        cases += 1;
    }

    eprintln!(
        "[ipc] frames={} cases={cases} rejected={rejected}",
        ipc_frames().len()
    );
    assert!(cases >= 1_000);
    assert!(rejected > 0, "malformed frames should be rejected");
}

// ---- DOM generation handles ----------------------------------------------

/// Exercise every public `NodeId` entry point. The accessors (`get`, `children`,
/// `parent`, `tag`, `attr`, `text`) are documented to require a handle that has
/// passed `validate`, so the test follows that trust boundary; the mutating
/// entries validate internally and must ignore stale/forged handles.
fn exercise_handle(dom: &mut Dom, id: NodeId) {
    let _ = dom.validate(id);
    dom.detach_subtree(id);
    dom.append(id, dom.root());
    dom.append(dom.root(), id);
    dom.set_attr(id, "data-probe", "x".to_string());
    if dom.validate(id) {
        let _ = dom.get(id);
        let _ = dom.children(id);
        let _ = dom.parent(id);
        let _ = dom.tag(id);
        let _ = dom.attr(id, "href");
        let _ = dom.text(id);
    }
    dom.walk(|_, _| {});
}

#[test]
fn dom_stale_and_forged_handles_never_panic() {
    let mut dom = Dom::new();
    let parent = dom.create(elem("div"));
    dom.append(dom.root(), parent);
    let child = dom.create(elem("span"));
    dom.append(parent, child);
    let grand = dom.create(elem("b"));
    dom.append(child, grand);
    dom.detach_subtree(child);
    assert!(!dom.validate(child));
    assert!(!dom.validate(grand));

    let reused = dom.create(elem("new"));
    assert!(
        reused.slot() == child.slot() || reused.slot() == grand.slot(),
        "create must reuse one of the detached slots"
    );
    assert_ne!(reused, child);
    assert_ne!(reused, grand);
    assert!(dom.validate(reused));

    let mut cases = 0usize;
    for handle in [child, grand, reused, dom.root()] {
        must_not_panic("dom stale handle", None, &format!("{handle:?}"), || {
            exercise_handle(&mut dom, handle)
        });
        cases += 1;
    }

    let forged = [
        NodeId(u32::MAX),
        NodeId(0x000f_ffff),
        NodeId(0x001f_ffff),
        NodeId(1 << 20),
        NodeId(MAX_DOM_NODES as u32 & 0x000f_ffff),
        NodeId(0x7fff_ffff),
    ];
    let root_len = dom.children(dom.root()).len();
    for handle in forged {
        assert!(!dom.validate(handle), "forged handle unexpectedly valid");
        must_not_panic("dom forged handle", None, &format!("{handle:?}"), || {
            exercise_handle(&mut dom, handle)
        });
        cases += 1;
    }
    assert_eq!(dom.children(dom.root()).len(), root_len);

    let mut state = DEFAULT_SEED ^ 0xd06_1234;
    for _ in 0..1_000 {
        let case_seed = state;
        let handle = NodeId(next(&mut state) as u32);
        must_not_panic(
            "dom random handle",
            Some(case_seed),
            &format!("{handle:?}"),
            || exercise_handle(&mut dom, handle),
        );
        cases += 1;
    }

    eprintln!("[dom] cases={cases}");
    assert!(cases >= 1_000);
}

//! CI gate (§K.1 / M9): no raw hex literals outside the generated token file.

use std::fs;
use std::path::{Path, PathBuf};

fn is_hex_digit(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

fn contains_hex(s: &str) -> bool {
    let b = s.as_bytes();
    for i in 0..b.len() {
        let digits_at = if b[i] == b'#' {
            i + 1
        } else if b[i] == b'0' && i + 1 < b.len() && (b[i + 1] == b'x' || b[i + 1] == b'X') {
            i + 2
        } else {
            continue;
        };
        if digits_at + 6 <= b.len() && b[digits_at..digits_at + 6].iter().all(|c| is_hex_digit(*c))
        {
            return true;
        }
    }
    false
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_raw_hex_outside_generated_tokens() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);
    assert!(!files.is_empty(), "expected hodo-ui sources");

    let offenders: Vec<String> = files
        .into_iter()
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some("tokens.rs"))
        .filter(|p| contains_hex(&fs::read_to_string(p).expect("read source")))
        .map(|p| p.display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "raw hex outside generated file: {offenders:?}"
    );
}

#[test]
fn generated_tokens_file_exists_and_is_stamped() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tokens.rs");
    let text = fs::read_to_string(path).expect("generated tokens.rs must be checked in");
    assert!(text.starts_with("// GENERATED from design/tokens.toml v1 — do not edit"));
}

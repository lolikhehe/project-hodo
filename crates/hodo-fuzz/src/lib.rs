//! Deterministic fuzz inputs for the parser trust boundaries.
//!
//! This is deliberately ordinary `cargo test` code: no nightly, no
//! `cargo-fuzz`. A seeded xorshift64 generator and a curated corpus of
//! malformed inputs are fed to the public parser APIs, which must never panic.

use std::sync::OnceLock;

/// Non-zero default seed: an all-zero xorshift64 state is a fixed point.
pub const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Advance an xorshift64 state one step. Callers must avoid a zero state;
/// [`gen_string`] normalizes its seed for you.
pub fn next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Characters the parsers are most likely to trip over: control bytes
/// (including NUL), the markup/CSS/JSON delimiters, and a few multibyte UTF-8
/// scalars.
fn alphabet() -> &'static [char] {
    static ALPHABET: OnceLock<Vec<char>> = OnceLock::new();
    ALPHABET.get_or_init(|| {
        let mut chars = Vec::with_capacity(160);
        chars.extend('\u{0}'..='\u{1f}');
        chars.push('\u{7f}');
        chars.extend(' '..='~');
        chars.extend([
            '\u{e9}',
            '\u{4e16}',
            '\u{1f30d}',
            '\u{2028}',
            '\u{2029}',
            '\u{feff}',
        ]);
        chars
    })
}

/// Generate a deterministic printable-and-control `String` of at most
/// `max_len` characters for the given seed. A zero seed is remapped to
/// [`DEFAULT_SEED`] so the generator always produces a real sequence.
pub fn gen_string(seed: u64, max_len: usize) -> String {
    let mut state = if seed == 0 { DEFAULT_SEED } else { seed };
    let len = (next(&mut state) as usize) % max_len.saturating_add(1);
    let chars = alphabet();
    let mut out = String::with_capacity(len.saturating_mul(4));
    for _ in 0..len {
        let index = (next(&mut state) as usize) % chars.len();
        out.push(chars[index]);
    }
    out
}

fn leak(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn build_corpus() -> Vec<&'static str> {
    let deep_html = leak(format!(
        "{}x{}",
        "<div>".repeat(5_000),
        "</div>".repeat(5_000)
    ));
    let huge_attr = leak(format!("<div data-x=\"{}\">x</div>", "a".repeat(100_000)));
    let huge_name = leak(format!(
        r#"{{"manifest_version":3,"name":"{}","version":"1"}}"#,
        "n".repeat(1024 * 1024)
    ));
    let deep_json = leak(format!("{}1{}", "[".repeat(200), "]".repeat(200)));

    vec![
        // --- malformed HTML: unclosed, misnested, valueless attrs, entities ---
        "",
        "<div><span><p><a><b><i>unclosed",
        "<div><span><p>one</b></span></div></table></foo>trailing",
        "<div a b c disabled checked data-x=\"\" x='' y=z>",
        "<input type=text readonly autofocus noshade>",
        "<p>&amp &lt &# &#x &#999999999; &notanentity; &AMP; &#0; &#x110000;</p>",
        "<p>&#x80;&#x9;&#x1F;&#x7F;</p>",
        "<script>function f(){ return '</script>' + unclosed",
        "<style>a{color:red} /* unterminated",
        "<!-- unclosed comment <div>text",
        "<!DOCTYPE html><html><head><title>x</title></head><body><table><tr><td>a<td>b</table>",
        "<svg><foreignObject><div>x</div></foreignObject></svg><math><mi>x</mi></math>",
        "<template><div>x</div></template><div>y</div>",
        "<div onclick=\"' onmouseover='\">x</div>",
        "<![CDATA[ not cdata in html ]]> <p>x</p>",
        deep_html,
        huge_attr,
        // --- malformed CSS: unterminated blocks/comments, bad @ rules, urls ---
        "a { color: red",
        "a { color: red; b { color: blue } c { color: green",
        "/* unterminated comment aaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "@media screen { a { color: red;",
        "@import ; @@; @font-face { src: url(",
        "a { background: url(\"unclosed); color: red } b { color: blue }",
        "a{background:url('x)}b{color:red}",
        "}{)(@@#",
        "a { color: ; ; ::: ; --x: }",
        "@charset \"UTF-8\"; @media (max-width:600px){.x{color:red}} p{color:green}",
        "* { margin: 0 } div.card#main, h1, .a.b { color: rgb(1,2,3) }",
        "a{color:rgb(0,0,0,0,0,0,0,0,0,0)}",
        "a { content: \"\\\" }",
        // --- manifest edge cases: wrong types, nulls, nesting, huge values ---
        "not json",
        "{",
        "null",
        "[]",
        r#"{"manifest_version":3,"name":123,"version":456}"#,
        r#"{"manifest_version":"2","name":"x","version":"1"}"#,
        r#"{"manifest_version":true,"name":"x","version":"1"}"#,
        r#"{"manifest_version":300,"name":"x","version":"1"}"#,
        r#"{"manifest_version":18446744073709551616,"name":"x","version":"1"}"#,
        r#"{"manifest_version":3,"name":"x","version":"1","permissions":null}"#,
        r#"{"manifest_version":3,"name":"x","version":"1","permissions":[["a"],{"b":1},[null,[[[1]]]]]}"#,
        r#"{"manifest_version":3,"name":"x","version":"1","icons":{"48":5,"128":null,"x":["a"]},"background":{"scripts":[1,null,"ok"],"service_worker":7},"action":{"default_title":42},"browser_specific_settings":{"gecko":{"id":9}},"key":[]}"#,
        r#"{"manifest_version":3,"name":"x","version":"1","permissions":["A","a","B","b","A"]}"#,
        huge_name,
        deep_json,
        // --- URL edge cases (harmless when parsed as the other formats) ---
        "file://",
        "file:///tmp/hodo\0nul",
        "data:text/html,hi",
        "\0",
    ]
}

/// Curated adversarial inputs. Built once, returned as a `Vec` of static
/// strings so callers can iterate it without owning any of the payloads.
pub fn corpus() -> Vec<&'static str> {
    static CORPUS: OnceLock<Vec<&'static str>> = OnceLock::new();
    CORPUS.get_or_init(build_corpus).clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_is_deterministic_and_hits_required_chars() {
        let a = gen_string(DEFAULT_SEED, 256);
        let b = gen_string(DEFAULT_SEED, 256);
        assert_eq!(a, b);
        assert_eq!(gen_string(0, 64), gen_string(DEFAULT_SEED, 64));
        assert!(a.is_empty() || a.len() <= 256 * 4);

        // Over many seeds every required delimiter and NUL must be produced.
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..500 {
            for ch in gen_string(DEFAULT_SEED.wrapping_add(i), 512).chars() {
                seen.insert(ch);
            }
        }
        for ch in ['<', '>', '{', '}', '"', '#', '@', '\\', '\0'] {
            assert!(seen.contains(&ch), "generator never produced {ch:?}");
        }
    }

    #[test]
    fn corpus_is_nonempty() {
        assert!(corpus().len() > 40);
    }
}

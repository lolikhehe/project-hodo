//! ESM module-graph loader — the Rust half of hodo's module contract.
//!
//! Rolldown emits a very regular shape: static imports hoisted to the top of a
//! chunk, a single trailing `export{...}`, and `import(...)` for code splits.
//! This module scans exactly that shape *textually* (no JS parser in this
//! crate), rewrites each chunk into one `__hodoRunModule(...)` call and orders
//! the graph so the JS runtime can evaluate it with a plain `eval`.
//!
//! Wave-1 limits, all reported with a debug note instead of guessing:
//!   * an `import` statement outside the top import block, or an `export`
//!     that is not the final statement → skip the module;
//!   * a dependency cycle → skip the whole entry graph;
//!   * `import(` inside a string literal is rewritten like any other (the
//!     paired JS agent owns the Promise side and verifies against real bundles).

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde_json::Value;

use crate::app::join_url;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// One `var <local>=__mods[<dep>].<export>;` assignment (`*` = whole namespace).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Alias {
    pub(crate) local: String,
    pub(crate) dep_index: usize,
    pub(crate) export: String,
}

/// One `{ exported: local }` entry of the factory's return literal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Export {
    pub(crate) name: String,
    pub(crate) local: String,
}

/// A scanned module, ready to be emitted as a single `__hodoRunModule` call.
#[derive(Clone, Debug)]
pub(crate) struct ParsedModule {
    pub(crate) url: String,
    /// Absolute URLs, deduplicated in first-mention order.
    pub(crate) deps: Vec<String>,
    pub(crate) assigns: Vec<Alias>,
    /// Import statements and the trailing export removed; `import(` rewritten.
    pub(crate) body: String,
    pub(crate) exports: Vec<Export>,
}

#[derive(Debug)]
pub(crate) enum GraphError {
    Fetch(String, String),
    Cycle(Vec<String>),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(url, err) => write!(f, "fetch {url}: {err}"),
            Self::Cycle(chain) => write!(f, "cycle: {}", chain.join(" -> ")),
        }
    }
}

/// A dynamic-import request decoded from the native reply channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DynamicModuleRequest {
    pub(crate) promise_id: u64,
    pub(crate) spec: String,
    pub(crate) base_url: String,
}

// ---------------------------------------------------------------------------
// Tiny tokenizer (comments and strings are skipped as single units so the word
// `import` inside a string or regex can never be mistaken for a statement)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Str(String),
    Punct(char),
}

#[derive(Clone, Debug)]
struct Tk {
    tok: Tok,
    start: usize,
    end: usize,
}

fn is_word(t: &Tk, word: &str) -> bool {
    matches!(&t.tok, Tok::Word(w) if w == word)
}

fn is_punct(t: &Tk, c: char) -> bool {
    matches!(t.tok, Tok::Punct(p) if p == c)
}

fn word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Minimal escape decoding — enough for URL specifiers and the rare escaped
/// quote; Rolldown chunk specifiers are plain ASCII paths.
fn decode_string(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw).into_owned();
    text.replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\'", "'")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn lex(src: &str) -> Vec<Tk> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // Comments.
        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        // Identifiers / keywords / numbers.
        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' || b.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && word_char(bytes[i]) {
                i += 1;
            }
            out.push(Tk {
                tok: Tok::Word(src[start..i].to_owned()),
                start,
                end: i,
            });
            continue;
        }
        // Quoted strings and template literals.
        if b == b'\'' || b == b'"' || b == b'`' {
            let quote = b;
            let start = i;
            i += 1;
            let content_start = i;
            let mut template_depth = 0u32;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                    continue;
                }
                if quote == b'`' && bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
                    template_depth += 1;
                    i += 2;
                    continue;
                }
                if quote == b'`' && template_depth > 0 && bytes[i] == b'}' {
                    template_depth -= 1;
                    i += 1;
                    continue;
                }
                if bytes[i] == quote && template_depth == 0 {
                    break;
                }
                i += 1;
            }
            let content_end = i.min(bytes.len());
            out.push(Tk {
                tok: Tok::Str(decode_string(&bytes[content_start..content_end])),
                start,
                end: (i + 1).min(bytes.len()),
            });
            i = (i + 1).min(bytes.len());
            continue;
        }
        // Everything else is punctuation we care about ( ; { } , * . ( ) = ).
        out.push(Tk {
            tok: Tok::Punct(b as char),
            start: i,
            end: i + 1,
        });
        i += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Import / export clause parsing
// ---------------------------------------------------------------------------

fn expect_as_name(toks: &[Tk], mut k: usize) -> Result<String, String> {
    if k >= toks.len() || !is_word(&toks[k], "as") {
        return Err("expected `as` in import clause".to_owned());
    }
    k += 1;
    match toks.get(k).map(|t| &t.tok) {
        Some(Tok::Word(name)) => Ok(name.clone()),
        _ => Err("expected identifier after `as`".to_owned()),
    }
}

/// Parse the tokens between `import` and `from` (or the bare specifier).
fn parse_clause(toks: &[Tk], dep: usize, out: &mut Vec<Alias>) -> Result<(), String> {
    // `import"./polyfill.js";` — the clause is just the quoted specifier.
    if toks.is_empty() || (toks.len() == 1 && matches!(toks[0].tok, Tok::Str(_))) {
        let _ = dep;
        return Ok(()); // side-effect-only import: dep slot, no alias
    }
    let mut k = 0;
    // Default import (`import X from` or `import X, {...} from`).
    if let Tok::Word(word) = &toks[k].tok {
        if word != "type" {
            out.push(Alias {
                local: word.clone(),
                dep_index: dep,
                export: "default".to_owned(),
            });
            k += 1;
            if k < toks.len() && is_punct(&toks[k], ',') {
                k += 1;
            }
        } else {
            k += 1; // `import type ...`
        }
    }
    if k >= toks.len() {
        return Ok(());
    }
    // Namespace import (`import * as ns from`).
    if is_punct(&toks[k], '*') {
        let local = expect_as_name(toks, k + 1)?;
        out.push(Alias {
            local,
            dep_index: dep,
            export: "*".to_owned(),
        });
        return Ok(());
    }
    // Named imports (`import {a, b as c} from`).
    if is_punct(&toks[k], '{') {
        k += 1;
        while k < toks.len() && !is_punct(&toks[k], '}') {
            let original = match &toks[k].tok {
                Tok::Word(w) => w.clone(),
                Tok::Str(s) => s.clone(),
                other => return Err(format!("unexpected import specifier {other:?}")),
            };
            k += 1;
            let mut local = original.clone();
            if k < toks.len() && is_word(&toks[k], "as") {
                local = expect_as_name(toks, k)?;
                k += 2;
            }
            out.push(Alias {
                local,
                dep_index: dep,
                export: original,
            });
            if k < toks.len() && is_punct(&toks[k], ',') {
                k += 1;
            }
        }
        if k >= toks.len() || !is_punct(&toks[k], '}') {
            return Err("unterminated named import list".to_owned());
        }
        return Ok(());
    }
    Err(format!(
        "unsupported import clause at byte {}",
        toks[k].start
    ))
}

/// Parse `a, b as c, d as default` between the braces of a trailing export.
fn parse_export_list_text(inner: &str) -> Result<Vec<Export>, String> {
    let mut out = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut words = part.split_whitespace();
        let local = words
            .next()
            .ok_or_else(|| "empty export specifier".to_owned())?;
        let name = match words.next() {
            Some("as") => words
                .next()
                .ok_or_else(|| "expected exported name after `as`".to_owned())?,
            _ => local,
        };
        out.push(Export {
            name: name.to_owned(),
            local: local.to_owned(),
        });
    }
    Ok(out)
}

/// A static-import-shaped token in the body is a shape we refuse to guess at.
fn looks_like_static_import(next: Option<&Tk>) -> bool {
    matches!(
        next.map(|t| &t.tok),
        Some(Tok::Punct('{')) | Some(Tok::Punct('*')) | Some(Tok::Word(_)) | Some(Tok::Str(_))
    )
}

// ---------------------------------------------------------------------------
// Public scanner
// ---------------------------------------------------------------------------

/// Scan one module's source. `Err(reason)` means "skip with a debug note".
pub(crate) fn parse_module(src: &str, url: &str) -> Result<ParsedModule, String> {
    let toks = lex(src);
    let mut i = 0;

    let mut deps: Vec<String> = Vec::new();
    let mut assigns = Vec::new();
    // Byte spans cut from the emitted body: directives and static imports.
    let mut removed: Vec<(usize, usize)> = Vec::new();

    // Directive prologue: `"use strict";` and friends.
    while i + 1 < toks.len() && matches!(toks[i].tok, Tok::Str(_)) && is_punct(&toks[i + 1], ';') {
        removed.push((toks[i].start, toks[i + 1].end));
        i += 2;
    }

    // ESM hoists static imports, but Vite/Rolldown emit a leading helper
    // (`const __vite__mapDeps=...;`) above the import block. Collect every
    // top-level import in source order and cut its span from the body.
    let mut depth = 0i32;
    while i < toks.len() {
        match toks[i].tok {
            Tok::Punct('{') | Tok::Punct('(') | Tok::Punct('[') => depth += 1,
            Tok::Punct('}') | Tok::Punct(')') | Tok::Punct(']') => depth -= 1,
            _ => {}
        }
        let is_import = depth == 0
            && is_word(&toks[i], "import")
            && !matches!(
                toks.get(i + 1).map(|t| &t.tok),
                Some(Tok::Punct('(')) | Some(Tok::Punct('.'))
            );
        if !is_import {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        let mut inner_depth = 0i32;
        let mut spec: Option<String> = None;
        let mut from: Option<usize> = None;
        while j < toks.len() {
            match &toks[j].tok {
                Tok::Punct('{') | Tok::Punct('(') | Tok::Punct('[') => inner_depth += 1,
                Tok::Punct('}') | Tok::Punct(')') | Tok::Punct(']') => inner_depth -= 1,
                Tok::Punct(';') if inner_depth == 0 => break,
                Tok::Str(s) => spec = Some(s.clone()),
                Tok::Word(w) if w == "from" && inner_depth == 0 => from = Some(j),
                _ => {}
            }
            j += 1;
        }
        if j >= toks.len() {
            return Err(format!("unterminated import at byte {}", toks[i].start));
        }
        let spec = spec.ok_or_else(|| "static import has no specifier".to_owned())?;
        let dep = match deps.iter().position(|d| d == &spec) {
            Some(pos) => pos,
            None => {
                deps.push(spec.clone());
                deps.len() - 1
            }
        };
        parse_clause(&toks[i + 1..from.unwrap_or(j)], dep, &mut assigns)?;
        removed.push((toks[i].start, toks[j].end));
        i = j + 1;
    }
    // Trailing `export{...}` — detected TEXTUALLY from EOF. A regex literal
    // such as `Be=/\"/g` can make a token scanner swallow the tail as a string;
    // the export list itself only holds identifiers, `as` and commas.
    let tail = src.trim_end();
    let tail = tail.strip_suffix(';').unwrap_or(tail).trim_end();
    let mut exports = Vec::new();
    let mut export_byte = src.len();
    if tail.ends_with('}') {
        let mut found: Option<(usize, usize)> = None;
        for pattern in ["export{", "export {"] {
            if let Some(pos) = tail.rfind(pattern) {
                if found.is_none_or(|(best, _)| pos > best) {
                    found = Some((pos, pattern.len()));
                }
            }
        }
        if let Some((pos, pattern_len)) = found {
            let open = pos + pattern_len - 1; // byte index of '{'
            let inner = &tail[open + 1..tail.len() - 1];
            if !inner.contains('{') && !inner.contains('}') {
                exports = parse_export_list_text(inner)?;
                export_byte = pos;
            }
        }
    }
    // `export default Ident;` at EOF (hand-written equivalent).
    if export_byte == src.len() {
        if let Some(pos) = tail.rfind("export default") {
            let ident = tail[pos + "export default".len()..]
                .trim()
                .trim_end_matches(';')
                .trim();
            if !ident.is_empty()
                && ident
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
            {
                exports.push(Export {
                    name: "default".to_owned(),
                    local: ident.to_owned(),
                });
                export_byte = pos;
            }
        }
    }

    // Rebuild the body from the raw source with all import/directive spans cut.
    let export_byte = export_byte.min(src.len());
    let mut body = String::new();
    let mut cursor = 0usize;
    for (start, span_end) in &removed {
        let start = (*start).min(export_byte);
        let span_end = (*span_end).min(export_byte);
        if start < cursor {
            continue;
        }
        body.push_str(&src[cursor..start]);
        cursor = span_end;
    }
    body.push_str(&src[cursor..export_byte]);
    let mut body = body.trim().to_owned();

    // Any `import`/`export` statement left in the body is a nested or unusual
    // shape we refuse to guess at; dynamic `import(` is fine (rewritten below).
    let body_toks = lex(&body);
    for (k, token) in body_toks.iter().enumerate() {
        if is_word(token, "export") {
            return Err(format!(
                "export statement outside final export block (byte {})",
                token.start
            ));
        }
        if is_word(token, "import") && looks_like_static_import(body_toks.get(k + 1)) {
            return Err(format!(
                "static import outside top import block (byte {})",
                token.start
            ));
        }
    }

    if !body.is_empty() {
        body = body
            .replace("import (", "__hodoDynImport(")
            .replace("import(", "__hodoDynImport(");
        if !body.ends_with(';') {
            body.push(';');
        }
    }

    // Resolve bare specifiers against the importing module's URL.
    let deps = deps.into_iter().map(|d| join_url(url, &d)).collect();

    Ok(ParsedModule {
        url: url.to_owned(),
        deps,
        assigns,
        body,
        exports,
    })
}

// ---------------------------------------------------------------------------
// Emission (the frozen contract)
// ---------------------------------------------------------------------------

fn quote_key(name: &str) -> String {
    let simple = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if simple {
        name.to_owned()
    } else {
        serde_json::to_string(name).expect("string serializes")
    }
}

impl ParsedModule {
    /// Canonical shell emit for one chunk:
    /// `__hodoModuleBaseUrl=<U>;__hodoRunModule(<U>, [<deps>], (function(__mods){ <body> return <exports>; }))`
    ///
    /// The base-URL assignment precedes every call so a factory's own
    /// synchronous `import()` resolves against that chunk's URL, not the page.
    pub(crate) fn emit(&self) -> String {
        let url = serde_json::to_string(&self.url).expect("string serializes");
        let deps = self
            .deps
            .iter()
            .map(|d| serde_json::to_string(d).expect("string serializes"))
            .collect::<Vec<_>>()
            .join(",");
        let assigns = self
            .assigns
            .iter()
            .map(|a| {
                let value = if a.export == "*" {
                    format!("__mods[{}]", a.dep_index)
                } else {
                    format!("__mods[{}].{}", a.dep_index, a.export)
                };
                format!("var {}={};", a.local, value)
            })
            .collect::<String>();
        let exports = self
            .exports
            .iter()
            .map(|e| format!("{}:{}", quote_key(&e.name), e.local))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "__hodoModuleBaseUrl={};__hodoRunModule({}, [{}], (function(__mods){{{}{}return {{{}}}; }}))",
            url, url, deps, assigns, self.body, exports
        )
    }
}

/// One script string for a whole topologically ordered graph.
///
/// Each chunk gets its own try/catch: one throwing module must not abort the
/// independent ones that follow it in the joined script.
pub(crate) fn emit_graph(modules: &[ParsedModule]) -> String {
    modules
        .iter()
        .map(|module| {
            format!(
                "try{{{}}}catch(e){{__hodoReportError('module {} failed: '+String(e&&(e.stack||e.message)||e));}}",
                module.emit(),
                module.url
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Graph resolution
// ---------------------------------------------------------------------------

fn visit<F>(
    url: &str,
    fetch: &mut F,
    parsed: &mut HashMap<String, ParsedModule>,
    visiting: &mut Vec<String>,
    done: &mut HashSet<String>,
    order: &mut Vec<ParsedModule>,
) -> Result<(), GraphError>
where
    F: FnMut(&str) -> Result<String, String>,
{
    if done.contains(url) {
        return Ok(());
    }
    if let Some(pos) = visiting.iter().position(|u| u == url) {
        let mut chain = visiting[pos..].to_vec();
        chain.push(url.to_owned());
        return Err(GraphError::Cycle(chain));
    }
    if !parsed.contains_key(url) {
        let source = fetch(url).map_err(|err| GraphError::Fetch(url.to_owned(), err))?;
        match parse_module(&source, url) {
            Ok(module) => {
                parsed.insert(url.to_owned(), module);
            }
            Err(reason) => {
                eprintln!("hodo: module {url}: skipped ({reason})");
                done.insert(url.to_owned());
                return Ok(());
            }
        }
    }
    visiting.push(url.to_owned());
    let deps = parsed[url].deps.clone();
    for dep in deps {
        visit(&dep, fetch, parsed, visiting, done, order)?;
    }
    visiting.pop();
    done.insert(url.to_owned());
    order.push(parsed[url].clone());
    Ok(())
}

/// Resolve a static import graph into dependency-first order.
///
/// Fetch errors and cycles are errors (the caller skips the whole entry with a
/// note); an unsupported module shape is skipped with its own debug note.
pub(crate) fn resolve_graph_with<F>(
    fetch: &mut F,
    entry: &str,
) -> Result<Vec<ParsedModule>, GraphError>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let mut parsed = HashMap::new();
    let mut visiting = Vec::new();
    let mut done = HashSet::new();
    let mut order = Vec::new();
    visit(
        entry,
        fetch,
        &mut parsed,
        &mut visiting,
        &mut done,
        &mut order,
    )?;
    Ok(order)
}

/// Production resolver: the entry's direct deps are fetched as one level
/// (`load_chunk_batch`: parallel when cold, per-chunk cached when warm), then
/// the existing dependency-first walk reuses those bodies and falls back to a
/// single privacy-aware load for deeper chunks.
pub(crate) fn resolve_graph(entry: &str) -> Result<Vec<ParsedModule>, GraphError> {
    let mut bodies: HashMap<String, String> = HashMap::new();
    let entry_body = crate::app::load_resource(entry)
        .map(crate::app::resource_text)
        .map_err(|err| GraphError::Fetch(entry.to_owned(), err.to_string()))?;
    if let Ok(module) = parse_module(&entry_body, entry) {
        let deps = crate::app::load_chunk_batch(&module.deps);
        bodies.extend(deps);
    }
    bodies.insert(entry.to_owned(), entry_body);
    let mut fetch = |url: &str| {
        if let Some(body) = bodies.get(url) {
            return Ok(body.clone());
        }
        crate::app::load_resource(url)
            .map(crate::app::resource_text)
            .map_err(|err| err.to_string())
    };
    resolve_graph_with(&mut fetch, entry)
}

// ---------------------------------------------------------------------------
// Dynamic-import reply-channel decoding
// ---------------------------------------------------------------------------

/// Decode a `__hodo_message_reply(...)` payload into a dynamic-module request.
///
/// The paired JS agent emits either an object
/// `{__hodoDynImport: 1, spec, base, id}` or its JSON-string form; both are
/// accepted so the native seam is not coupled to how JS serialises the call.
pub(crate) fn decode_dynamic_request(message: &Value) -> Option<DynamicModuleRequest> {
    let value = match message {
        Value::String(text) => serde_json::from_str::<Value>(text).ok()?,
        other => other.clone(),
    };
    let object = value.as_object()?;
    let marker = object
        .get("__hodoDynImport")
        .or_else(|| object.get("__hodo_module").filter(|v| *v == "dynamic"))?;
    if marker.is_null() || marker == false {
        return None;
    }
    let spec = object.get("spec")?.as_str()?.to_owned();
    let base_url = object
        .get("base")
        .or_else(|| object.get("base_url"))
        .and_then(Value::as_str)?
        .to_owned();
    let promise_id = match object.get("id").or_else(|| object.get("promise_id"))? {
        Value::Number(n) => n.as_u64()?,
        Value::String(s) => s.parse().ok()?,
        _ => return None,
    };
    Some(DynamicModuleRequest {
        promise_id,
        spec,
        base_url,
    })
}

// ---------------------------------------------------------------------------
// Tests — local string fixtures only, no network
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const BASE: &str = "http://hodo.test/";

    fn fixture(url: &str) -> String {
        match url {
            "http://hodo.test/a.js" => concat!(
                "import{n as e}from\"./rolldown-runtime.js\";\n",
                "import\"./polyfill.js\";\n",
                "import{a as g,n as ee}from\"./vendor-shared.js\";\n",
                "var x=e(1),U=g(x),L=ee(x);\n",
                "export{x as _,U as a,L as c};"
            )
            .to_owned(),
            // b = rolldown-runtime.js
            "http://hodo.test/rolldown-runtime.js" => {
                "function n(t){return t+1};export{n};".to_owned()
            }
            // c = vendor-shared.js, depends on b (c -> b)
            "http://hodo.test/vendor-shared.js" => concat!(
                "import{n as e}from\"./rolldown-runtime.js\";",
                "var a=e(1),n=e(2);export{a,n};"
            )
            .to_owned(),
            // d = polyfill.js, side-effect only
            "http://hodo.test/polyfill.js" => "var side=1;".to_owned(),
            _ => format!("var x={};", 1 + url.len()),
        }
    }

    fn fetcher() -> impl FnMut(&str) -> Result<String, String> {
        |url: &str| Ok(fixture(url))
    }

    #[test]
    fn contract_golden_string_is_exact() {
        let module = parse_module(&fixture("http://hodo.test/a.js"), "http://hodo.test/a.js")
            .expect("a.js parses");
        assert_eq!(
            module.deps,
            vec![
                "http://hodo.test/rolldown-runtime.js",
                "http://hodo.test/polyfill.js",
                "http://hodo.test/vendor-shared.js",
            ]
        );
        assert_eq!(
            module.emit(),
            concat!(
                "__hodoModuleBaseUrl=\"http://hodo.test/a.js\";",
                "__hodoRunModule(\"http://hodo.test/a.js\", ",
                "[\"http://hodo.test/rolldown-runtime.js\",",
                "\"http://hodo.test/polyfill.js\",",
                "\"http://hodo.test/vendor-shared.js\"], ",
                "(function(__mods){var e=__mods[0].n;",
                "var g=__mods[2].a;var ee=__mods[2].n;",
                "var x=e(1),U=g(x),L=ee(x);",
                "return {_:x,a:U,c:L}; }))"
            )
        );
    }

    #[test]
    fn side_effect_only_import_gets_a_dep_slot_without_an_alias() {
        let module = parse_module(&fixture("http://hodo.test/a.js"), "http://hodo.test/a.js")
            .expect("a.js parses");
        assert_eq!(module.deps.len(), 3);
        assert!(module.assigns.iter().all(|a| a.dep_index != 1));
    }

    #[test]
    fn duplicate_specifiers_dedupe_to_first_mention() {
        let src = "import{a as x}from\"./m.js\";import{b as y}from\"./m.js\";var z=x+y;export{z};";
        let module = parse_module(src, &format!("{BASE}entry.js")).expect("parses");
        assert_eq!(module.deps, vec![format!("{BASE}m.js")]);
        assert_eq!(
            module.assigns,
            vec![
                Alias {
                    local: "x".into(),
                    dep_index: 0,
                    export: "a".into()
                },
                Alias {
                    local: "y".into(),
                    dep_index: 0,
                    export: "b".into()
                },
            ]
        );
    }

    #[test]
    fn default_and_namespace_imports_emit_the_right_aliases() {
        let src = "import d,* as ns from\"./m.js\";var x=d+ns.a;export{x};";
        let module = parse_module(src, &format!("{BASE}entry.js")).expect("parses");
        assert_eq!(
            module.assigns,
            vec![
                Alias {
                    local: "d".into(),
                    dep_index: 0,
                    export: "default".into()
                },
                Alias {
                    local: "ns".into(),
                    dep_index: 0,
                    export: "*".into()
                },
            ]
        );
        assert!(module
            .emit()
            .contains("var d=__mods[0].default;var ns=__mods[0];"));
    }

    #[test]
    fn export_aliases_and_default_map_to_the_exports_literal() {
        let module = parse_module(
            "var a=1,c=2,d=3;export{a as b,c,d as default};",
            &format!("{BASE}entry.js"),
        )
        .expect("parses");
        assert_eq!(
            module.exports,
            vec![
                Export {
                    name: "b".into(),
                    local: "a".into()
                },
                Export {
                    name: "c".into(),
                    local: "c".into()
                },
                Export {
                    name: "default".into(),
                    local: "d".into()
                },
            ]
        );
        let emitted = module.emit();
        assert!(
            emitted.contains("return {b:a,c:c,default:d};"),
            "emitted: {emitted}"
        );
    }

    #[test]
    fn dynamic_import_is_rewritten_textually() {
        let module = parse_module(
            "var p=import(\"./chunk.js\");export{p};",
            &format!("{BASE}entry.js"),
        )
        .expect("parses");
        assert!(module.body.contains("__hodoDynImport(\"./chunk.js\")"));
        assert!(!module.body.contains("import("));
        assert!(module.deps.is_empty(), "dynamic import is not a static dep");
    }

    #[test]
    fn directive_prologue_is_skipped() {
        let module = parse_module(
            "\"use strict\";import{n}from\"./m.js\";var x=n(1);export{x};",
            &format!("{BASE}entry.js"),
        )
        .expect("parses");
        assert_eq!(module.deps, vec![format!("{BASE}m.js")]);
        assert!(module.body.starts_with("var x=n(1);"));
    }

    #[test]
    fn nested_import_is_skipped_with_a_note() {
        let err = parse_module(
            "var a=1;function f(){import{n}from\"./m.js\";}export{a};",
            &format!("{BASE}entry.js"),
        )
        .unwrap_err();
        assert!(err.contains("outside top import block"), "{err}");
    }

    #[test]
    fn vite_leading_helper_before_imports_is_supported() {
        // The real archive.org entry starts with a `const __vite__mapDeps=...`
        // helper *before* its import block; ESM hoists them, so do we.
        let src = concat!(
            "const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=[\"a.js\"])))=>i.map(i=>d[i]);",
            "import{n as e}from\"./rolldown-runtime.js\";",
            "import\"./polyfill.js\";",
            "import{a as g}from\"./vendor-shared.js\";",
            "var x=e(g(1));export{x};"
        );
        let module = parse_module(src, &format!("{BASE}entry.js")).expect("vite shape parses");
        assert_eq!(
            module.deps,
            vec![
                format!("{BASE}rolldown-runtime.js"),
                format!("{BASE}polyfill.js"),
                format!("{BASE}vendor-shared.js"),
            ]
        );
        assert!(module.body.contains("__vite__mapDeps"), "{}", module.body);
        assert!(!module.body.contains("import{"), "{}", module.body);
        assert!(module.emit().contains("__hodoModuleBaseUrl="));
    }

    #[test]
    fn non_final_export_is_skipped_with_a_note() {
        let err = parse_module("export var a=1;var b=2;", &format!("{BASE}entry.js")).unwrap_err();
        assert!(err.contains("outside final export block"), "{err}");
    }

    #[test]
    fn four_module_graph_is_topo_ordered() {
        let mut fetch = fetcher();
        let modules = resolve_graph_with(&mut fetch, "http://hodo.test/a.js").expect("graph");
        let urls: Vec<_> = modules.iter().map(|m| m.url.as_str()).collect();
        // b and d are leaves; c depends on b; a is the entry (always last).
        assert_eq!(
            urls,
            vec![
                "http://hodo.test/rolldown-runtime.js",
                "http://hodo.test/polyfill.js",
                "http://hodo.test/vendor-shared.js",
                "http://hodo.test/a.js",
            ]
        );
        assert_eq!(emit_graph(&modules).lines().count(), 4);
    }

    #[test]
    fn emit_graph_isolates_each_chunk_with_its_own_catch() {
        let bad = parse_module("throw new Error('bad');export{};", &format!("{BASE}bad.js"))
            .expect("bad.js parses");
        let good = parse_module("var ok=1;export{ok};", &format!("{BASE}good.js"))
            .expect("good.js parses");
        let graph = emit_graph(&[bad, good]);
        assert_eq!(graph.matches("try{__hodoModuleBaseUrl=").count(), 2);
        assert_eq!(graph.matches("catch(e){__hodoReportError(").count(), 2);
        assert!(graph.contains(&format!("module {BASE}bad.js failed")));
        // The good module's call is a sibling of the bad one's catch, not
        // swallowed inside it.
        let catch = graph
            .find(&format!("catch(e){{__hodoReportError('module {BASE}bad.js"))
            .expect("bad chunk has its own catch");
        assert!(graph[catch..].contains(&format!("__hodoRunModule(\"{BASE}good.js\"")));
    }

    #[test]
    fn cycle_is_rejected_with_the_chain() {
        let mut fetch = |url: &str| match url {
            "a" => Ok("import\"b\";".to_owned()),
            "b" => Ok("import\"a\";".to_owned()),
            other => Ok(format!("var x={other};")),
        };
        let err = resolve_graph_with(&mut fetch, "a").unwrap_err();
        match err {
            GraphError::Cycle(chain) => assert_eq!(chain, vec!["a", "b", "a"]),
            other => panic!("expected cycle, got {other}"),
        }
    }

    #[test]
    fn dynamic_request_decodes_object_and_string_shapes() {
        let object = json!({"__hodoDynImport": 1, "spec": "./chunk.js", "base": "http://hodo.test/a.js", "id": 7});
        let decoded = decode_dynamic_request(&object).expect("object decodes");
        assert_eq!(decoded.promise_id, 7);
        assert_eq!(decoded.spec, "./chunk.js");
        assert_eq!(decoded.base_url, "http://hodo.test/a.js");
        let string = Value::String(object.to_string());
        assert_eq!(decode_dynamic_request(&string), Some(decoded));
        assert!(decode_dynamic_request(&json!({"type": "unrelated"})).is_none());
    }

    /// The standing P0 hole: page/ESM/chunk fetches used to bypass the privacy
    /// layer. Prove the production module-graph path now routes through
    /// `hodo_net::fetch`, so an ad-listed chunk is blocked before any bytes load.
    #[test]
    fn adblock_listed_chunk_is_blocked_through_the_module_graph_path() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let _guard = crate::hpages::tests::page_lock();
        let enabled_before = hodo_adblock::adblock_enabled();
        hodo_adblock::set_adblock_enabled(true);
        hodo_net::revoke_site("doubleclick.net");

        // One loopback connection: the entry. Its only static dep is an
        // adblock-listed host, which must be blocked before a second request.
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback bind");
        let port = listener.local_addr().expect("local addr").port();
        let entry = format!("http://127.0.0.1:{port}/entry.js");
        let server = std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            let end = loop {
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break pos + 4;
                }
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                }
            };
            let _head = String::from_utf8_lossy(&buf[..end]).into_owned();
            let body = "import \"http://doubleclick.net/banner.js\";var x=1;export{x};";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        });

        let err = resolve_graph(&entry).expect_err("ad-listed chunk must abort the graph");
        let GraphError::Fetch(url, reason) = err else {
            panic!("expected a fetch error, got {err:?}");
        };
        assert_eq!(url, "http://doubleclick.net/banner.js");
        assert!(
            reason.to_ascii_lowercase().contains("blocked"),
            "reason: {reason}"
        );
        assert!(
            hodo_net::blocked_urls()
                .iter()
                .any(|u| u.contains("doubleclick.net")),
            "the block must be recorded by the privacy layer"
        );
        server.join().expect("server thread");
        hodo_adblock::set_adblock_enabled(enabled_before);
    }
}

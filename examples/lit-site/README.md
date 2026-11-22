# `lit-site` — local twin of the Lit/archive.org regression

A self-contained, network-free fixture that exercises the **whole JS/DOM
substrate chain** the archive.org prize screenshot depends on. No CDN, no
deployments, no live site: every byte comes from this directory over `file://`,
so the regression cannot rot when somebody ships a new bundle.

## Files

| File | Role |
| --- | --- |
| `index.html` | Two existing `<x-lit-card>` tags, a `<link rel=modulepreload>`, light-DOM styles, and `<script type=module src=entry.js>` |
| `entry.js` | ESM entry: two static imports + one `import()`, custom element, shadow root, template stamping, constructable stylesheet, probe writer |
| `chunk-model.js` | Static dep — CJK/Latin binding strings |
| `chunk-view.js` | Static dep — the ~40-line TreeWalker part filler (Lit-style) |
| `chunk-dynamic.js` | Dynamic dep — the item that appears only after the promise settles |

## Substrate chain exercised

- ESM module graph: entry + 2 static chunks (`chunk-model.js`, `chunk-view.js`),
  one dynamic `import('./chunk-dynamic.js')`, and a `modulepreload` for it.
- Hand-rolled custom element: `customElements.define('x-lit-card', …)` upgrades
  the two tags already in `index.html` (no append needed).
- `attachShadow({mode:'open'})` + `<template>`/`importNode`/`appendChild`
  stamping, with `<!--?lit$123$-->` marker comments.
- TreeWalker consumer: `createTreeWalker(root, 129)` finds markers, then re-walks
  the stamped tree at the same indices to insert each binding's text — the
  two-consumer pattern Lit uses.
- `new CSSStyleSheet()` + `replaceSync` + `shadowRoot.adoptedStyleSheets = [sheet]`.
- A real `<style>` element inside the shadow root (header background/color).
- CJK + Latin mixed text in bindings and in the dynamic chunk.
- Visible end state: two styled headers, six filled bindings, the dynamic
  chunk's item in the first card, and a green probe line.

## Standing gate (manual)

From the workspace root:

```sh
cargo run --release -p hodo-shell -- --wait-ms 3000 --screenshot "file://D:/project hodo/examples/lit-site/index.html" out.png
```

Expected stdout:

```
title: lit-site — Lit substrate regression
screenshot written: out.png (25392 bytes)
```

Expected visible probe line (bottom of the painted content):

```
PROBE markers=6 filled=6 dyn=OK
```

`markers=6` means the TreeWalker found all six `?lit$123$` comments across the
two upgraded cards; `filled=6` means all six bindings were stamped; `dyn=OK`
means the dynamic chunk's `ITEM` reached the DOM before capture. The first
card's third line reads `item:敦煌 · Dunhuang 卷轴`; the second card retains its
`…loading` placeholder, which also proves the dynamic write is targeted.

Determinism: repeated runs of the command above produced byte-identical PNGs
(SHA-256 `AF0208656195CA322F7F80F8F50ABFEEC17AFDF9C1928D925560A45A2C956C47`).
The hash is informational — assert on the probe line and the byte count, not on
the hash, because font/toolchain upgrades legitimately change PNG bytes.

### `file://` form on Windows

`hodo-net` passes the URL after `file://` straight to `Path::new` — no percent
decoding. Use `file://D:/…` (two slashes, literal spaces, forward slashes in the
path). `file:///D:/…` yields os error 123, and `file://D:/…%20…` looks for a
directory literally named `%20`. On POSIX the usual `file:///abs/path` works.

## `file://` load smoke

The command above *is* the load smoke: a wrong path prints an empty title and
paints `hodo cannot load this page` (os error 123). The correct load prints the
title above and paints the probe. For an interactive check:

```sh
cargo run -p hodo-shell -- "file://D:/project hodo/examples/lit-site/index.html"
```

Do not use the directory shorthand (`--screenshot examples/lit-site`) for this
fixture: relative dynamic-import specifiers resolve against a doubled base in
that mode and `chunk-dynamic.js` fails to fetch, leaving `dyn=MISS`.

## Why there is no Rust screenshot-diff test here

`crates/hodo-tests` has no `hodo-shell` dependency and no PNG/image-diff crate,
so a screenshot-diff test there cannot be added without crossing into another
crate's manifest. Per the fixture's zone restriction (`examples/lit-site/**`
only), no shell/net/js source or test manifest was touched. The gate above is
the standing regression check; upgrade it to an automated diff only when a
screenshot harness exists in `hodo-tests`.

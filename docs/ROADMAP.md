# hodo roadmap

Milestones for the hodo browser. "Done" here means the crate is implemented and
tested, with no `todo!()` stubs left in the tree. Crate names below are the
workspace members in `Cargo.toml`.

## M0 — Engine core (DONE)

The parse → style → layout → raster pipeline exists as a workspace of crates
with defined types: `hodo-html`, `hodo-css`, `hodo-style`, `hodo-layout`,
`hodo-render`, plus `hodo-dom`, `hodo-types`, and `hodo-net`. Alongside it, the
cross-dialect extension layer — `hodo-ext-manifest`, `hodo-ext-api`,
`hodo-ext-host` — defines the manifest model, canonical namespace mapping, and
permission gating contract. The implementation bodies are complete: the
pipeline parses, styles, lays out and rasterizes real input, and the extension
crates parse manifests, dispatch APIs and enforce permissions.

**Crates:** `hodo-types`, `hodo-dom`, `hodo-html`, `hodo-css`, `hodo-style`,
`hodo-layout`, `hodo-render`, `hodo-net`, `hodo-ext-manifest`, `hodo-ext-api`,
`hodo-ext-host`.

## M1 — Shell GUI (in progress)

Windowed browser shell using `winit` + `softbuffer`: a tab strip and a URL/address
bar wired to the engine. The GUI implementation is in progress (the M0 engine
stages behind it are done); headless screenshots (`--screenshot`) already work.
Goal is a usable CPU-rendered window that can drive the M0 pipeline.

**Crates:** `hodo-shell`, plus the `hodo-render` / `hodo-layout` outputs it
consumes.

## M2 — JavaScript execution (landed through `d63a90c`)

Embed `boa_engine` and inject `chrome.*` and `browser.*` globals so both map to
the single canonical dispatch table in `hodo-ext-api`. **Landed:** `hodo-js`
embeds boa 0.22, exposes `ExtScriptRuntime` with an event loop (microtasks after
each task), `setTimeout`/`setInterval`, Promise + MV2 callback styles, timers,
`alarms`, `fetch`, `Port` messaging and async event delivery. `hodo-ext-api`
gained `storage.session` under a 10 MiB quota, `StorageChanged`/`AlarmFired`
events, alarms and the fetch contract. `hodo-ext-host` now creates one runtime
per extension, filters namespaces by manifest permissions, validates CSP,
blocks script `eval`, runs `bg.js` / MV3 service workers and unloads/wakes idle
workers. **Remaining in this milestone:** shell GUI plumbing (M1) for a visible
browser window; content scripts are separate.

**Crates:** `hodo-js` (new), `hodo-ext-host`, `hodo-ext-api`, `hodo-ext-manifest`, `hodo-ext-i18n` (new, `4aec263`).

## M3 — Out-of-process sandboxed extension host

Move extension execution out of the browser process into a sandboxed child
process, replacing the current in-process `Host`. Communication via IPC with
the same `ApiCall` / `ApiResult` contract, keeping permission enforcement at the
trust boundary.

**Crates:** `hodo-ext-host` (and a new IPC/sandbox process target).

## M4 — HTTP cache + compression + streaming parse

Add an HTTP cache and `gzip`/`br` content decoding to `hodo-net`, and make HTML
parsing streaming rather than whole-document. Respects the existing 2 MiB
resource cap. Reduces latency and memory for real pages. **Landed:** the
`hodo-cache` crate provides a corruption-tolerant RFC 9111 disk cache with
freshness, ETag/Last-Modified revalidation, a `Vary` whitelist, offline mode and
LRU eviction, with payloads kept off RAM (`0c0d5cc`). `hodo-net` gained the
opt-in `load_with` / `FetchOptions` cached path and cache stats, and the shell
owns one global 128 MiB cache for page loads with graceful fallback plus
`about:hodo` statistics (`cd1b1ec`). `hodo-net` also transparently decompresses
gzip and hardens content sniffing, BOM stripping, the Content-Length guard and
the 5-redirect cap. **Pending:** `br` decoding and streaming HTML parse.

**Crates:** `hodo-cache` (new), `hodo-net`, `hodo-html`.

## M5 — Incremental layout + partial repaint

Reuse layout results across frames and repaint only damaged regions, instead of
recomputing the whole tree. Aimed at a RAM and CPU win, especially on the
low-resource targets hodo is designed for.

**Crates:** `hodo-layout`, `hodo-style`, `hodo-render`, `hodo-shell`.

## M6 — GPU compositing (feature-flagged)

Add a `wgpu` compositing backend behind a Cargo feature flag. CPU rasterization
remains the default to preserve the low-RAM profile; GPU is opt-in for machines
that can use it.

**Crates:** `hodo-render`, `hodo-shell` (feature-gated).

## M7 — Stretch: richer CSS

**Landed in Wave A:** the full own selectors engine (descendant/child/adjacent/
general-sibling combinators, attribute operators incl. `i`, 7 pseudo-classes,
specificity, rule index + Bloom filter, 100k rule cap, `!important` ladder with
inline styles) in `25d03fa`, and the inline box model (line boxes, per-fragment
borders/backgrounds, anonymous blocks, `<br>`, linear hard-break) in `f3e6d9d`.
**In flight via contracts:** flexbox (ТЗ M7.4), grid (M7.5), tables (M7.3) and
remaining text properties (M7.2); position/float/overflow are split into M8.
See [PROPERTY_STATUS.md](PROPERTY_STATUS.md).

**Crates:** `hodo-css`, `hodo-style`, `hodo-layout`.

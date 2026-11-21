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
stages behind it are done). Goal is a usable CPU-rendered window that can drive
the M0 pipeline.

**Crates:** `hodo-shell`, plus the `hodo-render` / `hodo-layout` outputs it
consumes.

## M2 — JavaScript execution

Embed `boa_engine` and inject `chrome.*` and `browser.*` globals so both map to
the single canonical dispatch table in `hodo-ext-api`. This finally executes
the `bg.js` files in `extensions/hello-chrome` and `extensions/hello-gecko`,
routing storage/tabs/runtime calls through the permission gate.

**Crates:** `hodo-ext-host`, `hodo-ext-api`, `hodo-ext-manifest`.

## M3 — Out-of-process sandboxed extension host

Move extension execution out of the browser process into a sandboxed child
process, replacing the current in-process `Host`. Communication via IPC with
the same `ApiCall` / `ApiResult` contract, keeping permission enforcement at the
trust boundary.

**Crates:** `hodo-ext-host` (and a new IPC/sandbox process target).

## M4 — HTTP cache + compression + streaming parse

Add an HTTP cache and `gzip`/`br` content decoding to `hodo-net`, and make HTML
parsing streaming rather than whole-document. Respects the existing 2 MiB
resource cap. Reduces latency and memory for real pages.

**Crates:** `hodo-net`, `hodo-html`.

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

Flexbox and grid layout, plus descendant selectors in the CSS matcher. Not
committed to a slot; picked up once M1–M6 are stable.

**Crates:** `hodo-css`, `hodo-style`, `hodo-layout`.

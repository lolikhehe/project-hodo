# Architecture

## Pipeline and data types

```text
net            String                 fetched bytes, hard 2 MiB cap (hodo-net)
html           String → Dom           parse_document (hodo-html)
dom            Dom                    arena Vec<Node>, stable NodeId(u32) handles (hodo-dom)
css/style      Stylesheet → StyleMap  compound selectors, UA + author cascade (hodo-css, hodo-style)
layout         StyleMap → DisplayList block flow, flat Rect/Text items (hodo-layout)
render         DisplayList → Vec<u8>  non-premultiplied RGBA8, w*h*4 bytes (hodo-render)
shell          Vec<u8> → surface      window presentation or headless PNG (hodo-shell)
```

The `Dom` owns nodes in one `Vec<Node>`. `NodeId` indexes that vector; parent/child links are `NodeId`s, not references. `StyleMap` is likewise a `Vec<Option<ComputedStyle>>` indexed by `NodeId.0`, so styling needs no borrow of the DOM. `DisplayItem` is an owned `Rect`/`Text` enum, keeping layout output independent of the tree it came from.

## Why an arena DOM

RAM is the primary constraint. One `Vec<Node>` allocation replaces a heap node per element; a `NodeId(u32)` handle is 4 bytes versus an 8-byte pointer on 64-bit targets, with no `Rc`/`RefCell` refcounts or borrow overhead per node. Handles are cheap to copy, and removal does not dangle other nodes (the workspace forbids `unsafe_code`). The same indexing trick is reused by `StyleMap`.

## Extension host

```text
manifest.json (Chrome OR Gecko dialect)
        │
        ▼
hodo-ext-manifest::parse ──► Manifest { version, permissions, gecko_id, chrome_key, ... }
        │
        ▼
Host::load_dir ──► Host { extensions: Vec<Extension>, registry: ApiRegistry }
        │
        ▼
permission gate ──► required_namespace(call) ⊆ manifest.permissions  ("runtime" always allowed)
        │
        ▼
ApiRegistry::dispatch ◄── canonical_namespace("chrome.*" | "browser.*")
        │
        ▼
ExtEvent queue ──► ApiRegistry::drain_events (host delivery wiring in progress)
```

`host_permissions` is a separate manifest field of URL patterns; it is never merged into `permissions` and never grants an API namespace. `chrome.*` and `browser.*` are aliases at the dispatch boundary: `canonical_namespace` strips either prefix and maps storage/tabs/runtime variants (plus `browser.extension` → `runtime` and `notifications` → `notifications`) to one canonical name, so one `ApiRegistry` serves both dialects. `flavor()` reports Gecko when `gecko_id` is present, Chrome when the top-level `chrome_key` is present, otherwise Generic. MV2 background scripts and MV3 service workers are both represented on `Manifest`. `ApiRegistry` also queues `ExtEvent`s (StorageChanged, MessageReceived, TabActivated, NotificationShown) drained via `drain_events`; host delivery is still being wired. The host is currently in-process; sandboxed child processes are roadmap. See [EXTENSIONS.md](EXTENSIONS.md).

## JavaScript execution (hodo-js)

`hodo-js` embeds `boa_engine` 0.22 behind the default-on feature `js`; `--no-default-features` leaves an `eval` stub. `ExtScriptRuntime` owns one boa `Context`, exposes `new` / `eval` / `registry`, and installs `chrome` and `browser` as the *same* JS object dispatching storage.local/sync, tabs.create/list, runtime.sendMessage and notifications.create into `ApiRegistry`. RAM note: a boa `Context` is single-threaded and `!Send`, so the runtime uses `Rc<RefCell<ApiRegistry>>` plus a thread-local handle table rather than a shared global. Host wiring (per-extension permission filtering and event delivery) is in progress; callbacks/promises are deferred.

## Disk cache (hodo-cache)

`DiskCache` is a size-capped LRU: `put` / `get` / `len` / `total_bytes`, `CacheEntry` and `CacheError` (Io, TooLarge, Corrupt). Keys are FNV-1a hashes used as filenames under `root/cache/`; reads verify size and checksum and prune corrupt entries. RAM note: payloads live in files — only small per-entry `EntryMeta` records (hash, content type, size, checksum, last use) stay resident. `hodo-net` integration is pending.

## Robustness corpus (hodo-fuzz)

`hodo-fuzz` feeds public parser APIs a deterministic xorshift64 stream (`DEFAULT_SEED`, `gen_string`) and a 49-input curated `corpus()` of malformed HTML/CSS/manifests and URL edges. Tests use `catch_unwind` under plain stable `cargo test` — no nightly and no `cargo-fuzz`. RAM note: the corpus is built once behind `OnceLock` and borrowed as `&'static str`; there is no persistent parser state. Zero panics over 6,000+ parses so far.

## Crates

- **hodo-types** — `Rect`, `Size`, `Color`, `Edges`; shared value types passed across every stage.
- **hodo-dom** — `Dom`, `Node`, `NodeId`, `NodeKind`; the arena and its append/detach operations.
- **hodo-html** — `parse_document(&str) -> Dom`; HTML5 input to the arena.
- **hodo-css** — `Stylesheet`, `Selector`, `Declaration`, `specificity`, `matches_element`; compound selectors only.
- **hodo-style** — `ComputedStyle`, `StyleMap`, `ua_stylesheet`; cascade by specificity and declaration order.
- **hodo-layout** — `DisplayItem`, `DisplayList`, `layout`; block flow with approximate text metrics.
- **hodo-render** — `render(&DisplayList, Size) -> Vec<u8>`; software RGBA8 rasterizer.
- **hodo-net** — `Resource`, `FetchError`, `MAX_BYTES = 2 MiB`, `load`; http(s), `file://`, local paths, content sniffing, BOM strip, 5-redirect cap.
- **hodo-cache** — `DiskCache`, `CacheEntry`, `CacheError`; on-disk LRU, payloads off RAM.
- **hodo-ext-manifest** — `Manifest`, `ManifestVersion` (V2/V3), `BrowserFlavor`, `parse`, `flavor`.
- **hodo-ext-api** — `ApiCall`, `ApiResult`, `ExtEvent`, `ApiRegistry` (`dispatch`, `drain_events`), `canonical_namespace`.
- **hodo-ext-host** — `Extension`, `Host`, `HostError`, `load_dir`, permission-gated `dispatch`.
- **hodo-js** — `ExtScriptRuntime`; boa bridge installing cross-dialect WebExtension globals.
- **hodo-shell** — binary crate that owns the window surface, event wiring and headless screenshots; GUI implementation in progress.
- **hodo-tests** — workspace-level integration and regression tests.
- **hodo-fuzz** — `corpus`, `gen_string`, `DEFAULT_SEED`; deterministic parser robustness harness.
- **hodo-perf** — pipeline benchmark over the demo site in `examples/demo-site/`.

## Current limitations (MVP)

- Selectors: compound selectors only (tag, `#id`, `.class`, universal, and their combinations) — no combinators or pseudo-classes.
- Text metrics are approximate: glyph width = `0.6 * font_size`, line height = `1.2 * font_size`; no real font shaping.
- The boa engine and JS bridge have landed; host/shell wiring, event delivery and async callbacks remain in progress.
- Margins do not collapse; block flow only, no floats or positioned layout.
- All stage entry points are implemented (no `todo!()` stubs remain); extension content scripts are not implemented yet.

# Architecture

## Pipeline and data types

```text
net            String                 fetched bytes, hard 2 MiB cap (hodo-net)
html           String → Dom           parse_document (hodo-html)
dom            Dom                    arena Vec<Node>, stable NodeId(u32) handles (hodo-dom)
css/style      Stylesheet → StyleMap  full selector matcher (combinators, attributes, pseudos) + UA/author/inline cascade (hodo-css, hodo-style)
layout         StyleMap → DisplayList block + inline formatting, line boxes, flat Rect/Text items (hodo-layout)
render         DisplayList → Vec<u8>  non-premultiplied RGBA8, w*h*4 bytes (hodo-render)
shell          Vec<u8> → surface      window presentation or headless PNG (hodo-shell)
```

The `Dom` owns nodes in one `Vec<Node>`. `NodeId` indexes that vector; parent/child links are `NodeId`s, not references. `StyleMap` is likewise a `Vec<Option<ComputedStyle>>` indexed by `NodeId::slot()` (not the raw handle, which now also carries a generation — see below), so styling needs no borrow of the DOM. `DisplayItem` is an owned `Rect`/`Text` enum, keeping layout output independent of the tree it came from.

## Why an arena DOM

RAM is the primary constraint. One `Vec<Node>` allocation replaces a heap node per element; a `NodeId(u32)` handle is 4 bytes versus an 8-byte pointer on 64-bit targets, with no `Rc`/`RefCell` refcounts or borrow overhead per node. Handles are cheap to copy, and removal does not dangle other nodes (the workspace forbids `unsafe_code`). The same indexing trick is reused by `StyleMap`.

### Generation handles (`d80dd84`)

A `NodeId` is now an opaque handle: the low **20 bits** hold the arena slot and the high **12 bits** a generation counter (`SLOT_BITS = 20`, `GEN_MOD = 1 << 12`). `Dom` keeps a parallel `generations: Vec<u32>` plus a `free: Vec<u32>` list populated only by `detach_subtree`; reclaiming a slot bumps its generation before reuse, so a recycled slot hands out a *different* handle. `Dom::is_live` accepts a handle only when its slot is in range, not trashed and its generation still matches — stale handles from JS, IPC or extensions are detected rather than silently indexing the wrong node. The 20-bit slot is why `hodo_types::limits::MAX_DOM_NODES` is 1 000 000, asserted at compile time. `NodeId::slot()` is provided for callers that keep parallel arrays keyed by handle (such as `StyleMap`).

### Central limits (`hodo-types::limits`, `d80dd84`)

The single source of truth for resource caps is `crates/hodo-types/src/limits.rs`, re-exported from `hodo_types`: `MAX_RESOURCE_BYTES`, `MAX_MANIFEST_BYTES` and `MAX_LOCALE_BYTES` (2 MiB each), `MAX_DOM_NODES` (1 000 000), `MAX_ATTR_LEN` (64 KiB), `MAX_CSS_RULES` (100 000), `MAX_NESTING_DEPTH` (512), `MAX_ACTIVE_TIMERS` (100), `MAX_MSG_BYTES` (1 MiB), `MAX_MSG_DEPTH` (64) and `MAX_EXT_CONTEXTS` (16). Parsers, caches, brokers and DOM mutation paths read these instead of redefining their own; the depth caps are mandatory because a Rust stack overflow aborts the whole browser rather than panicking. Tests assert the caps are non-zero, internally consistent and fit the 20-bit slot encoding.

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
ExtEvent queue ──► ApiRegistry::drain_events ──► runtime.tick() ──► JS listeners
```

`host_permissions` is a separate manifest field of URL patterns; it is never merged into `permissions` and never grants an API namespace. `chrome.*` and `browser.*` are aliases at the dispatch boundary: `canonical_namespace` strips either prefix and maps storage/tabs/runtime variants (plus `browser.extension` → `runtime` and `notifications` → `notifications`) to one canonical name, so one `ApiRegistry` serves both dialects. `flavor()` reports Gecko when `gecko_id` is present, Chrome when the top-level `chrome_key` is present, otherwise Generic. MV2 background scripts and MV3 service workers are both represented on `Manifest`. `ApiRegistry` also queues `ExtEvent`s (StorageChanged with area/oldValue/newValue, AlarmFired, MessageReceived, TabActivated, NotificationShown) drained via `drain_events` and delivered to JS listeners by each runtime's `tick()` (`d63a90c`). The host is currently in-process; sandboxed child processes are roadmap. See [EXTENSIONS.md](EXTENSIONS.md).

## JavaScript execution (hodo-js)

`hodo-js` embeds `boa_engine` 0.22 behind the default-on feature `js`; `--no-default-features` leaves an `eval` stub. `ExtScriptRuntime` owns one boa `Context`, exposes `new` / `eval` / `tick` / `registry`, and installs `chrome` and `browser` as the *same* JS object dispatching into `ApiRegistry`. Since `d63a90c` it also owns a browser-like event loop: a FIFO task queue for timers and drained `ExtEvent`s, with `HodoJobExecutor` draining boa promise jobs after **each** task (microtask-before-next-timer ordering). `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`, `alarms.create/clear`, `storage.*` (local/sync/session), `fetch`, `runtime.connect` (`Port`) and `runtime.onMessage` all return Promises or accept callbacks on the same table; `chrome.*` MV2 callback style sets `runtime.lastError`, while `browser.*` is always Promise-style. Script-driven `eval`/`Function` are blocked (`hooks.set_block_compile(true)`, "content security policy: eval is disabled"), and the host rejects manifest CSPs containing `'unsafe-eval'`/`'wasm-unsafe-eval'`. RAM note: a boa `Context` is single-threaded and `!Send`, so the runtime uses `Rc<RefCell<ApiRegistry>>` plus a thread-local handle table rather than a shared global. Host wiring landed in the same commit: one runtime per extension, namespace filtering by manifest permissions, drained-event delivery and MV3 idle unload/wake (`Host::run_background_scripts`, `tick`, `poll`, `emit_event`).

## Disk cache (hodo-cache + hodo-net + shell)

`DiskCache` is now an RFC 9111 cache (`0c0d5cc`): `put` / `get` / `len` / `total_bytes`, `CacheEntry` / `CacheKey` / `Clock`, freshness math against an injectable `SystemClock`, `ETag` / `Last-Modified` validators with 304 header merge (the stored body is never rewritten), an observable `Vary` whitelist (`accept-encoding`, `accept-language`, `accept`), `offline` cache-only mode and LRU byte-quota eviction. Keys are FNV-1a hashes used as filenames under a `cache/` directory; reads verify size and checksum and prune corrupt entries. RAM note: payloads live in files — only small per-entry `EntryMeta` records (hash, content type, size, checksum, last use, vary map) stay resident. The default cache quota is 256 MiB, but the shell opens it at **128 MiB**.

`hodo-net::load` stays frozen and uncached; `load_with(url, &FetchOptions)` is the opt-in cached path (`use_cache`, `offline`, `partition`). In `cd1b1ec` the shell made caching **default-on for page loads**: one process-wide `OnceLock<Mutex<DiskCache>>` single owner, `DiskCache::open(&root, 128 * 1024 * 1024)`, with graceful fallback to the frozen uncached loader if the cache cannot be opened. The generated `cache/` directory is `.gitignore`d, and `about:hodo` exposes `hodo_net::cache_stats()` (hits, misses, bytes, evictions) alongside the synthetic welcome page.

## Robustness corpus (hodo-fuzz)

`hodo-fuzz` feeds public parser APIs a deterministic xorshift64 stream (`DEFAULT_SEED`, `gen_string`) and a 49-input curated `corpus()` of malformed HTML/CSS/manifests and URL edges. Tests use `catch_unwind` under plain stable `cargo test` — no nightly and no `cargo-fuzz`. RAM note: the corpus is built once behind `OnceLock` and borrowed as `&'static str`; there is no persistent parser state. Zero panics over 6,000+ parses so far.

## Crates

- **hodo-types** — `Rect`, `Size`, `Color`, `Edges`; shared value types plus the central `limits` table (`d80dd84`).
- **hodo-dom** — `Dom`, `Node`, `NodeId`, `NodeKind`; the arena and its append/detach operations.
- **hodo-html** — `parse_document(&str) -> Dom`; HTML5 input to the arena.
- **hodo-css** — `Stylesheet`, `Selector`, `Declaration`, `specificity`, `matches_element`, `RuleIndex`, `AncestorBloom`; combinators, attribute ops (incl. `i`), 7 pseudo-classes, `!important` parsing.
- **hodo-style** — `ComputedStyle`, `StyleMap`, `ua_stylesheet`; cascade by origin, `!important`, specificity and order, with inline styles (`25d03fa`).
- **hodo-layout** — `DisplayItem`, `DisplayList`, `layout`; block flow + inline formatting (line boxes, fragment borders/backgrounds, anonymous blocks, `<br>`) with approximate text metrics (`f3e6d9d`).
- **hodo-render** — `render(&DisplayList, Size) -> Vec<u8>`; software RGBA8 rasterizer.
- **hodo-net** — `Resource`, `FetchError`, `MAX_BYTES = 2 MiB`, `load`, `load_with`/`FetchOptions`, `cache_stats`; http(s), `file://`, content sniffing, BOM strip, 5-redirect cap, RFC 9111 cached path (`0c0d5cc`).
- **hodo-cache** — `DiskCache`, `CacheEntry`, `CacheKey`, `Clock`, `CacheStats`; on-disk RFC 9111 cache with freshness, revalidation, Vary and LRU, payloads off RAM.
- **hodo-ext-manifest** — `Manifest`, `ManifestVersion` (V2/V3), `BrowserFlavor`, `parse`, `flavor`, `resolve_messages` (`__MSG_` substitution, `4aec263`).
- **hodo-ext-i18n** — `Messages`, `load_locales`, `substitute_messages`, `predefined_value`; locale fallback chain, `$name$`/`$1`–`$9` placeholders, `@@` predefines (`4aec263`).
- **hodo-ext-api** — `ApiCall`, `ApiResult`, `ExtEvent`, `ApiRegistry` (`dispatch`, `drain_events`), `canonical_namespace`.
- **hodo-ext-host** — `Extension`, `Host`, `HostError`, `load_dir`, permission-gated `dispatch`, CSP validation, per-extension `ExtScriptRuntime`s, MV3 idle lifecycle (`d63a90c`).
- **hodo-js** — `ExtScriptRuntime`, `HodoJobExecutor`; boa bridge with event loop, timers, Promises/callbacks and async event delivery (`d63a90c`).
- **hodo-shell** — binary crate that owns the window surface, event wiring, headless screenshots, the single-owner disk cache and `about:hodo` stats (`cd1b1ec`); GUI implementation in progress.
- **hodo-tests** — workspace-level integration and regression tests.
- **hodo-fuzz** — `corpus`, `gen_string`, `DEFAULT_SEED`; deterministic parser robustness harness.
- **hodo-perf** — pipeline benchmark over the demo site in `examples/demo-site/`.

## Current limitations (landed Wave A, `9df4f23`)

- Selectors are full for the supported set (combinators, 7 attribute operators incl. `i`, 7 pseudo-classes), but `:hover`/`:focus`, `:is()`/`:where()`, dynamic pseudos and newer selectors are not implemented.
- Text metrics are approximate: glyph width = `0.6 * font_size`, line height = `1.2 * font_size`; no real font shaping. `white-space` is `normal` only; long words are hard-broken linearly.
- The async extension runtime, host wiring, per-extension permission filtering and event delivery have landed (`d63a90c`); what remains is shell-level GUI plumbing (M1) and content scripts.
- Margins do not collapse; no floats, positioned layout, flex, grid or tables yet — see [PROPERTY_STATUS.md](PROPERTY_STATUS.md) and ТЗ M7.3/M7.4/M7.5/M8 (in flight).
- i18n resolves `__MSG_` in manifest fields and message placeholders, but CSS/HTML are not localized and all locales currently get LTR bidi values (`4aec263`).
- All stage entry points are implemented (no `todo!()` stubs remain); extension content scripts are not implemented yet.

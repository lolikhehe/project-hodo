# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Workspace scaffold with frozen crate contracts: the initial crate set, shared `hodo-types`, `unsafe_code = "forbid"`, and the end-to-end pipeline boundary `net → html → dom → css/style → layout → render → shell`.
- Arena DOM (`hodo-dom`): `u32` `NodeId` handles over a `Vec<Node>` arena, no `Rc`/`RefCell`, parent/child linking, attribute/text accessors and pre-order walk.
- HTML5 parsing (`hodo-html`): `html5ever`-backed document parsing into the arena DOM.
- Stylesheet parsing (`hodo-css`): `cssparser`-based CSS parsing with a compound-selector model (`tag`, `#id`, `.class`, universal), declarations, and `(ids, classes, tags)` specificity.
- Cascade/computed-style engine (`hodo-style`): built-in UA stylesheet plus author sheets, cascade by specificity and order, inheritance of `color` and `font_size`, `ComputedStyle` and `StyleMap`.
- Block-flow layout (`hodo-layout`): flat `DisplayList` of rect and text items with approximate font metrics.
- Software rasterizer (`hodo-render`): `tiny-skia` + `fontdue` producing row-major RGBA8 frames, with vendored DejaVu Sans and its license under `assets/fonts/`.
- Resource loader (`hodo-net`): `http`/`https`/`file` URLs behind a hard 2 MiB cap (`MAX_BYTES`).
- Cross-dialect WebExtension manifest parser (`hodo-ext-manifest`): one code path for Chrome MV3 and Gecko MV2, including flavor detection, permissions, background scripts vs. service worker, and `gecko_id`.
- API bridge (`hodo-ext-api`): `chrome.*` and `browser.*` normalized onto a single `ApiCall` / `ApiRegistry` dispatch table for tabs, storage and runtime.
- Extension host (`hodo-ext-host`): in-process host with manifest-based permission gating (`runtime` always allowed).
- Sample extensions under `extensions/` (8 total): `hello-chrome`, `hello-gecko`, `tabs-demo-chrome`, `storage-demo-gecko`, plus four new ones below.
- Four more sample extensions: `notify-chrome` (MV3 notifications), `messages-gecko` (MV2 `runtime.onMessage`), `popup-chrome` (MV3 `popup.html` with real 16/48px PNG icons) and `i18n-gecko` (MV2 `_locales` + `__MSG__` convention; substitution is roadmap).
- JavaScript execution engine (`hodo-js`, new crate): `boa_engine` 0.22 behind the default-on feature `js` (`--no-default-features` leaves an `eval` stub). `ExtScriptRuntime::new` / `eval` / `registry` install `chrome` and `browser` as the *same* JS object, exposing `storage.local` / `storage.sync`, `tabs.create` / `tabs.list`, `runtime.sendMessage` and `notifications.create` over the shared API table. Six tests, including a cross-dialect roundtrip; the context is single-threaded and `!Send` (documented). Shell/host wiring is in progress.
- Disk cache (`hodo-cache`, new crate): size-capped LRU keyed by FNV-1a, payloads stored as files with checksum-verified reads that prune corrupt or truncated entries. Five tests. RAM holds only small `EntryMeta` records.
- Robustness corpus (`hodo-fuzz`, new crate): 49 curated adversarial inputs plus 2,000 seeded random strings per parser on stable Rust — no nightly, no `cargo-fuzz`. Zero panics across 6,000+ parses of HTML, CSS and manifests, plus deep-nesting and URL edge cases.
- API bridge (`hodo-ext-api`) expansion: `ApiCall::{StorageSyncSet, StorageSyncGet, TabsList, TabsActivate, NotificationsCreate}`, `ExtEvent` and `ApiRegistry::drain_events` (StorageChanged, MessageReceived, TabActivated, NotificationShown), and the canonical `notifications` namespace.
- Resource loading (`hodo-net`) hardening: HTML content sniffing when no `Content-Type` is present, `Content-Length` fail-fast before streaming, UTF-8 BOM stripping, redirects capped at 5 with a clear error, and documented gzip transparency via ureq's default features.
- Project README and architecture notes (`docs/ARCHITECTURE.md`).
- CI workflow (`.github/workflows/ci.yml`): formatting check and cargo check, plus clippy (`-D warnings`), tests and a release shell build on Linux and Windows.
- Pipeline benchmark harness (`hodo-perf`): times load → parse → style → layout → render over `examples/demo-site/` and reports the best of three runs.
- Demo site (`examples/demo-site/`): sample `index.html` and `style.css` used by the perf harness.

The crate boundaries and public contracts are stable; all stage bodies are implemented (no `todo!()` stubs remain). The boa JS engine and bridge have landed; the shell/host wiring is in progress. The shell GUI is also in progress.

### Fixed

- Style cascade: the `border` shorthand now expands `<width> <style> <color>` tokens (in any order, including partial forms like `border: red`) into the border-width/color longhands in cascade order, so borders from author stylesheets actually rasterize. Non-solid styles currently degrade to solid.
- Rasterizer: glyphs are now placed from a computed line baseline using fontdue per-glyph metrics, so periods and commas sit on the baseline and descenders hang below it instead of every glyph being top-aligned.
- Headless screenshots: the rendered frame is composited over opaque white before PNG encoding, so screenshots no longer have transparent edge strips.
- Layout: text inside `<li>` elements now receives a single `• ` bullet prefix in its Text item.
- Extension host: `host_permissions` is now a separate `Manifest` field instead of being merged into `permissions`, so a host pattern like `"storage"` can no longer self-grant the storage API. Host-side permission gating has a regression test.
- Style cascade: per-side `margin-*`, `padding-*` and `border-*-width` longhands are now applied; shorthands expand into longhands in cascade order so both shorthand-then-longhand and longhand-then-shorthand resolve by source order. Border widths are exposed per side via the new `ComputedStyle::border_widths` field.
- API bridge: `browser.extension.*` (Gecko idiom) now maps to the canonical `runtime` namespace.
- Resource loading (`hodo-net`): `text/html` content types are matched case-insensitively, valid UTF-8 no longer incurs a full-buffer copy, and the HTTP agent is built once and shared across requests.
- Extension host: `manifest.json` reads are capped at 2 MiB and over-limit files fail with `HostError::Io("manifest too large")`.

---

Semantic versioning: releases follow `MAJOR.MINOR.PATCH` per [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html). Before 1.0, a minor version may contain breaking public-API changes; breaking changes are called out in the relevant entry.

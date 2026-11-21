# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Workspace scaffold with frozen crate contracts: 13 crates, shared `hodo-types`, `unsafe_code = "forbid"`, and the end-to-end pipeline boundary `net → html → dom → css/style → layout → render → shell`.
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
- Sample extensions under `extensions/`: `hello-chrome`, `hello-gecko`, `tabs-demo-chrome`, `storage-demo-gecko`.
- Project README and architecture notes (`docs/ARCHITECTURE.md`).
- CI workflow (`.github/workflows/ci.yml`): formatting check and cargo check, plus clippy (`-D warnings`), tests and a release shell build on Linux and Windows.
- Pipeline benchmark harness (`hodo-perf`): times load → parse → style → layout → render over `examples/demo-site/` and reports the best of three runs.
- Demo site (`examples/demo-site/`): sample `index.html` and `style.css` used by the perf harness.

The crate boundaries and public contracts are stable; all stage bodies are implemented (no `todo!()` stubs remain). The shell GUI is in progress, and JS execution is still on the roadmap.

### Fixed

- Extension host: `host_permissions` is now a separate `Manifest` field instead of being merged into `permissions`, so a host pattern like `"storage"` can no longer self-grant the storage API. Host-side permission gating has a regression test.
- Style cascade: per-side `margin-*`, `padding-*` and `border-*-width` longhands are now applied; shorthands expand into longhands in cascade order so both shorthand-then-longhand and longhand-then-shorthand resolve by source order. Border widths are exposed per side via the new `ComputedStyle::border_widths` field.
- API bridge: `browser.extension.*` (Gecko idiom) now maps to the canonical `runtime` namespace.
- Resource loading (`hodo-net`): `text/html` content types are matched case-insensitively, valid UTF-8 no longer incurs a full-buffer copy, and the HTTP agent is built once and shared across requests.
- Extension host: `manifest.json` reads are capped at 2 MiB and over-limit files fail with `HostError::Io("manifest too large")`.

---

Semantic versioning: releases follow `MAJOR.MINOR.PATCH` per [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html). Before 1.0, a minor version may contain breaking public-API changes; breaking changes are called out in the relevant entry.

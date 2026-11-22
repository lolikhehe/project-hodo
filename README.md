# hodo

A Rust web browser with its own engine, built for minimal RAM: arena DOM on `u32` handles (no `Rc`/`RefCell`), a hard 2 MiB cap per resource. Design is influenced by Firefox (WebExtension cross-compatibility) and Safari (minimalism).

## Build & run

```sh
cargo build --release -p hodo-shell
cargo run --release -p hodo-shell
cargo run -p hodo-shell -- --screenshot <url> out.png
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Pipeline

```text
net → html → dom → css/style → layout → render → shell
```

`String` (fetched bytes, capped) → `Dom` (arena `Vec<Node>`, `NodeId(u32)`) → `Stylesheet` → `StyleMap` → `DisplayList` → RGBA8 buffer → window surface. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Features

Wave A landed the items below (`git log --oneline d911e52..9df4f23`, ending at `9df4f23`); the commit in parentheses is the evidence. The last bullet records what is still in flight.

- **End-to-end engine pipeline:** HTML/CSS parsing, cascade, block + inline layout and software rasterization do real work — no `todo!()` stubs (`25d03fa`, `f3e6d9d`).
- **Own selectors engine (`25d03fa`):** descendant/child/adjacent/general-sibling combinators; 7 attribute operators (presence, `=`, `~=`, `|=`, `^=`, `$=`, `*=`) including the case-insensitive `i` flag; 7 pseudo-classes (`:first-child`, `:last-child`, `:nth-child(an+b)`, `:only-child`, `:not()`, `:root`, `:empty`); specificity; rule index + 64-bit ancestor Bloom filter for descendant rejection. Capped at `hodo_types::limits::MAX_CSS_RULES` = 100 000 rules.
- **Cascade (`25d03fa`):** full `!important` ladder — UA normal < author normal < inline normal < author `!important` < inline `!important` < UA `!important` — with `style=""` parsed as its own author-origin layer.
- **Inline box model (`f3e6d9d`):** real line boxes, inline backgrounds/borders painted per line fragment, anonymous block boxes for mixed inline/block content, `<br>` hard breaks and linear-time hard-breaking of long words.
- **Async extension runtime (`d63a90c`):** per-context event loop (boa microtasks drained after each task), Promises and MV2-style callbacks on one dispatch table, `setTimeout`/`setInterval`, `alarms` driven by an injectable `Clock`, messaging with `Port` (`runtime.connect`) and async `onMessage` replies, `storage.local`/`sync`/`session` with a shared 10 MiB quota and `storage.onChanged`, permission-gated `fetch`, MV3 idle-unload/wake, manifest CSP validation and script-level `eval`/`Function` blocked. See [docs/EXTENSIONS.md](docs/EXTENSIONS.md).
- **RFC 9111 disk cache (`0c0d5cc`, `cd1b1ec`):** freshness (`max-age`), `ETag`/`Last-Modified` revalidation with 304 header merge, a whitelist of observable `Vary` headers, offline cache-only mode, LRU eviction at the shell's 128 MiB quota; default-on for page loads in the shell with graceful fallback to the uncached loader, and `about:hodo` reports cache hits/misses/bytes/evictions.
- **i18n (`4aec263`):** new `hodo-ext-i18n` crate — `_locales/<locale>/messages.json` with an exact → language → default-locale fallback chain, single-pass `$name$` / `$1`–`$9` placeholder expansion, `@@` predefined messages and `__MSG_name__` substitution in manifest `name`/`description`/`action_title`.
- **Central limits + safe handles (`d80dd84`):** one `hodo-types::limits` table (2 MiB resource caps, 100k CSS rules, 512 nesting depth, 100 timers, 16 extension contexts, …); DOM `NodeId` encodes a 20-bit slot + 12-bit generation, with `detach_subtree` free-list reuse and stale-handle detection.
- **Shell (`cd1b1ec`):** relative `img[src]` resolved against the page URL, headless screenshots via `hodo-shell --screenshot <url> out.png` (opaque white composite).
- **In progress / roadmap:** shell GUI (M1), CSS feature contracts (flex/grid/table/position/float/text — see [docs/PROPERTY_STATUS.md](docs/PROPERTY_STATUS.md)), content scripts and out-of-process sandboxing. See [docs/ROADMAP.md](docs/ROADMAP.md).

Cross-dialect WebExtensions remain a first-class goal: MV2/MV3 manifests, `chrome.*` and `browser.*` on one dispatch table, host-side permission gating, 10 sample extensions (including the new `async-chrome/` and `promises-gecko/` demos from `d63a90c`). A deterministic parser robustness corpus (`hodo-fuzz`) stays under plain `cargo test`.

## Crates

| Crate | Role |
| --- | --- |
| `hodo-types` | Shared value types (`Rect`, `Size`, `Color`, `Edges`). |
| `hodo-dom` | Arena DOM: `u32` node handles, zero `Rc`/`RefCell`. |
| `hodo-html` | HTML5 document parsing into the arena DOM. |
| `hodo-css` | CSS parsing, own full selector matcher (combinators, attribute ops, 7 pseudo-classes) with rule index + Bloom filter. |
| `hodo-style` | UA + author + inline cascade (`!important` ladder) into `StyleMap` / `ComputedStyle`. |
| `hodo-layout` | Block + inline formatting layout producing a flat `DisplayList`. |
| `hodo-render` | Software rasterizer producing row-major RGBA8 frames. |
| `hodo-net` | Resource loading with a hard 2 MiB cap, sniffing/redirect cap, opt-in RFC 9111 cached path (`FetchOptions`) and cache stats. |
| `hodo-cache` | RFC 9111 disk cache: freshness, revalidation, Vary, LRU quota; payloads on disk, metadata in RAM. |
| `hodo-ext-manifest` | Chrome and Gecko WebExtension manifest parsing (MV2/MV3) plus `__MSG_` resolution. |
| `hodo-ext-api` | `chrome.*` / `browser.*` bridge over one dispatch table, alarms, storage areas, fetch and queued events. |
| `hodo-ext-host` | In-process extension host: permission gating, CSP validation, per-extension runtimes and MV3 idle lifecycle. |
| `hodo-ext-i18n` | Locale message resolver: fallback chain, placeholders, `@@` predefines (`4aec263`). |
| `hodo-js` | boa bridge with event loop: timers, Promises/callbacks, async events, script `eval` blocked. |
| `hodo-shell` | Binary entry point: window surface, headless screenshots, single-owner disk cache and `about:hodo` stats (GUI in progress). |
| `hodo-tests` | Workspace integration and regression tests. |
| `hodo-fuzz` | Deterministic seeded corpus that fuzzes the parsers under stable `cargo test`. |
| `hodo-perf` | Pipeline benchmark harness over the demo site in `examples/demo-site/`. |

## Extensions

Both Gecko and Chromium WebExtension dialects are accepted: MV2 and MV3 manifests, `chrome.*` (MV2 callback-style) and `browser.*` (Promise-style) namespaces mapped to a single dispatch table, and the Gecko `gecko_id` honored. The host gates calls on manifest permissions (`runtime` is always allowed; `fetch` uses `host_permissions`). Ten sample extensions ship in `extensions/`, including notification, messaging, popup, i18n and the new `async-chrome/` / `promises-gecko/` demos (`d63a90c`). See [docs/EXTENSIONS.md](docs/EXTENSIONS.md).

## License

Apache-2.0. See [LICENSE](LICENSE).

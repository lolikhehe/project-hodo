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

- End-to-end engine pipeline: HTML/CSS parsing, cascade, block layout and software rasterization do real work — no `todo!()` stubs.
- Cross-dialect WebExtensions: MV2/MV3 manifests, `chrome.*` and `browser.*` on one dispatch table, host-side permission gating, 8 sample extensions.
- JS execution engine: extension scripts execute via the boa bridge — shell wiring in progress. See [docs/EXTENSIONS.md](docs/EXTENSIONS.md).
- Low-RAM infrastructure: size-capped disk LRU cache (`hodo-cache`) and a deterministic parser robustness corpus (`hodo-fuzz`).
- Headless screenshots via `hodo-shell --screenshot <url> out.png` (opaque white composite).
- In progress: the shell GUI, per-extension JS host wiring. Roadmap: content scripts, out-of-process sandboxing, `__MSG__` i18n substitution.

## Crates

| Crate | Role |
| --- | --- |
| `hodo-types` | Shared value types (`Rect`, `Size`, `Color`, `Edges`). |
| `hodo-dom` | Arena DOM: `u32` node handles, zero `Rc`/`RefCell`. |
| `hodo-html` | HTML5 document parsing into the arena DOM. |
| `hodo-css` | CSS parsing, MVP compound selectors, specificity. |
| `hodo-style` | UA + author cascade into `StyleMap` / `ComputedStyle`. |
| `hodo-layout` | Block-flow layout producing a flat `DisplayList`. |
| `hodo-render` | Software rasterizer producing row-major RGBA8 frames. |
| `hodo-net` | Resource loading with a hard 2 MiB cap, content sniffing and redirect cap. |
| `hodo-cache` | Size-capped disk LRU cache; payloads on disk, metadata in RAM. |
| `hodo-ext-manifest` | Chrome and Gecko WebExtension manifest parsing (MV2/MV3). |
| `hodo-ext-api` | `chrome.*` / `browser.*` bridge over one dispatch table, plus queued events. |
| `hodo-ext-host` | In-process extension host with permission gating. |
| `hodo-js` | boa-based JS bridge executing extension scripts against the one API table. |
| `hodo-shell` | Binary entry point; window surface, event wiring and headless screenshots (GUI in progress). |
| `hodo-tests` | Workspace integration and regression tests. |
| `hodo-fuzz` | Deterministic seeded corpus that fuzzes the parsers under stable `cargo test`. |
| `hodo-perf` | Pipeline benchmark harness over the demo site in `examples/demo-site/`. |

## Extensions

Both Gecko and Chromium WebExtension dialects are accepted: MV2 and MV3 manifests, `chrome.*` and `browser.*` namespaces mapped to a single dispatch table, and the Gecko `gecko_id` honored. The host gates calls on manifest permissions (`runtime` is always allowed). Eight sample extensions ship in `extensions/`, including notification, messaging, popup and i18n demos. See [docs/EXTENSIONS.md](docs/EXTENSIONS.md).

## License

Apache-2.0. See [LICENSE](LICENSE).

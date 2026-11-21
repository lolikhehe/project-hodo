# hodo

A Rust web browser with its own engine, built for minimal RAM: arena DOM on `u32` handles (no `Rc`/`RefCell`), a hard 2 MiB cap per resource. Design is influenced by Firefox (WebExtension cross-compatibility) and Safari (minimalism).

## Build & run

```sh
cargo build --release -p hodo-shell
cargo run --release -p hodo-shell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Pipeline

```text
net → html → dom → css/style → layout → render → shell
```

`String` (fetched bytes, capped) → `Dom` (arena `Vec<Node>`, `NodeId(u32)`) → `Stylesheet` → `StyleMap` → `DisplayList` → RGBA8 buffer → window surface. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

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
| `hodo-net` | Resource loading with a hard 2 MiB cap. |
| `hodo-ext-manifest` | Chrome and Gecko WebExtension manifest parsing (MV2/MV3). |
| `hodo-ext-api` | `chrome.*` / `browser.*` bridge over one dispatch table. |
| `hodo-ext-host` | In-process extension host with permission gating. |
| `hodo-shell` | Binary entry point; window surface and event wiring (GUI implementation in progress). |
| `hodo-tests` | Workspace integration and regression tests. |
| `hodo-perf` | Pipeline benchmark harness over the demo site in `examples/demo-site/`. |

## Extensions

Both Gecko and Chromium WebExtension dialects are accepted: MV2 and MV3 manifests, `chrome.*` and `browser.*` namespaces mapped to a single dispatch table, and the Gecko `gecko_id` honored. The host gates calls on manifest permissions (`runtime` is always allowed). See [docs/EXTENSIONS.md](docs/EXTENSIONS.md).

## Status

The engine core pipeline is implemented end-to-end — HTML/CSS parsing, cascade, block layout and software rasterization all do real work, with no `todo!()` stubs in the tree. Manifest parsing, the canonical API bridge and host-side permission gating are implemented (`host_permissions` stays separate from API permissions). The shell's GUI is in progress. JS execution (boa), content scripts and sandboxed extension processes remain on the roadmap.

## License

Apache-2.0. See [LICENSE](LICENSE).

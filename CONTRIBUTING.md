# Contributing

## Prerequisites

- Stable Rust (latest stable via rustup). Nightly is not required.

## Commands

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

Run all four before opening a PR; CI enforces them.

## Rules

- **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
- **Clippy clean.** No new warnings; CI runs clippy with `-D warnings`.
- **Frozen public API.** Any change to a `pub` signature in `crates/*` requires a separate contract-update commit *first* (update the crate contract/docs and `CHANGELOG.md`), followed by the implementation change.
- **Conventional commits.** Use `feat:`, `fix:`, `docs:`, `ci:`, `refactor:`, `test:`, `chore:` (optional scope, e.g. `feat(dom): add node walk`).
- **Smallest-diff-wins.** Make the minimal change that fully solves the problem: no speculative abstractions, options, or unrelated rewrites.

## Pull requests

- Branch from `main` and open PRs against `main`; keep each PR focused on one change.
- Say what changed and why, and link any related issue.
- Make sure build, tests, formatting and clippy pass locally.

## Extensions

Demos are welcome: drop a new folder into `extensions/` with a `manifest.json`, a background script and a short `README.md`. Both Chrome MV3 and Gecko MV2 manifests are supported.

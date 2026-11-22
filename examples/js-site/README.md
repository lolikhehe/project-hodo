# `js-site` — page-JS fixture

A page whose inline script mutates the DOM before the first paint and registers
a click listener that increments a counter.

```html
<div id="out">JS-PENDING</div>   <!-- script sets textContent = 'JS-OK' -->
<div id="count">count: 0</div>
<button id="btn" type="button">Click me</button>
```

## Headless verification

From the workspace root:

```sh
cargo run -p hodo-shell -- --screenshot examples/js-site out.png
```

The render pipeline runs the inline script, fires `DOMContentLoaded`, then
computes styles/layout once. The PNG therefore contains **JS-OK** (not
`JS-PENDING`). The unit tests assert the same thing through the display list:

```sh
cargo test -p hodo-shell js_site -- --nocapture
```

## Interactive verification (GUI only)

```sh
cargo run -p hodo-shell -- examples/js-site/index.html
```

Click **Click me**: the hit-test resolves the `<button>` to its `NodeId`,
`dispatch_page_event(node, "click", …)` fires the listener, the DOM fingerprint
changes and styles/layout/render run on the next frame. The counter shows
`count: 1`, `count: 2`, … The automated test drives that same path directly:
it resolves the button's `NodeId` from `DisplayList::node_map` and calls the
same `PageContext::dispatch` the app routes to, because the winit mouse path
needs a real window.

## Kill switch

```sh
HODO_NO_PAGE_JS=1 cargo run -p hodo-shell -- examples/js-site/index.html
```

No page realm is created: `JS-PENDING` stays on screen and no listeners are
registered. Script-free pages take the identical code path either way.

## Limits

- `<script type="module">` is skipped; `src=` scripts are resolved with
  `join_url` and loaded through the shared cache.
- 16 scripts per document, 2 MiB per script, 50 ms total eval budget (plus
  hodo-js's per-script loop/recursion limits).
- Page scripts only see the page prelude (`document`, timers, console,
  `__hodoFetch`). No `chrome`, `browser`, `window` or host globals.

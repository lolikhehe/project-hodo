# CSS property / feature status (Wave A baseline, plus current-tree verification)

> **IN FLIGHT (uncommitted wave):** the tree moves under this doc. Active zones:
> style-parallel (CSS value/cascade work), tab-spill (`HODO_SPILL_THRESHOLD`
> display-list spill agent), net-mmap (mmap-backed resource path), js-memo
> (memoized JS bindings), render image-cache (decoded-image cache), and 6 bug
> indexers. Every status flip below was verified by reading current code, not
> `git log`; re-check before using this as a release gate. No commits in this wave.

Scope: the Wave A baseline (`git log --oneline d911e52..9df4f23`) **plus** code
verified in the current uncommitted tree for the rows flipped below. Each flip
cites `file:line`; rows not read this wave are labelled "not audited" and keep
their old status.
A generic `property: value` pair is always tokenized into a `Declaration` by
`hodo-css` — "parser only" below means **no `ComputedStyle` field and no layout
effect**, not that the source text is rejected.

## Legend

| Status | Meaning |
| --- | --- |
| ✓ parsed | Produces a `Declaration` (and, where relevant, a validated value) in `hodo-css`. |
| ✓ consumed | Read into `ComputedStyle` by `hodo-style` and used by `hodo-layout`. |
| ✓ painted | Produces a `DisplayItem` (Rect / Text / Image) in `hodo-layout` and is rasterized by `hodo-render`. |
| parser only, layout pending | The declaration parses, but nothing consumes it yet. In flight — see ТЗ M7.4/M7.5/M7.3/M8 sections (text properties also M7.2). |

## Landed and painted (✓ / ✓ / ✓)

| Property / feature | Parsed | Consumed | Painted | Evidence & limits |
| --- | --- | --- | --- | --- |
| `display: block` / `inline` / `none` | ✓ | ✓ | ✓ | `25d03fa`, `f3e6d9d`; `none` suppresses the box, `inline` participates in line boxes. |
| `color` | ✓ | ✓ | ✓ | `25d03fa`, `f3e6d9d`; sets `ComputedStyle::color` on `DisplayItem::Text`. |
| `background` / `background-color` | ✓ | ✓ | ✓ | `25d03fa`, `f3e6d9d`; painted as a rect behind content, once per inline line fragment. |
| `border` shorthand + `border-width`, `border-color` and per-side longhands | ✓ | ✓ | ✓ | `25d03fa` expands the shorthand; `f3e6d9d` paints edge rects, with left/right borders only on first/last fragment. |
| `margin` / `padding` + per-side longhands | ✓ | ✓ | ✓ | `25d03fa` expands shorthands; `f3e6d9d` consumes horizontal insets in line boxes. Margins **do not collapse**. |
| `font-size` (`px` / `em` / `rem` / `%`) | ✓ | ✓ | ✓ | `25d03fa`, `f3e6d9d`; glyph metrics are approximate (`0.6 × font-size`), not real font shaping. |
| Inline `style="…"` declarations | ✓ | ✓ | ✓ | `25d03fa`; `parse_style_attribute` feeds the cascade as an author-origin layer. |
| Selectors: combinators, 7 attribute operators incl. `i`, 7 pseudo-classes, specificity, rule index + Bloom filter | ✓ | ✓ | n/a | `25d03fa`; capped at `MAX_CSS_RULES = 100 000` and 10 000 rules per stylesheet. |
| `!important` (full origin ladder) | ✓ | ✓ | n/a | `25d03fa`; UA normal < author normal < inline normal < author `!important` < inline `!important` < UA `!important`. |
| Inline box model: line boxes, fragment backgrounds/borders, anonymous blocks, `<br>`, linear hard-break | ✓ | ✓ | ✓ | `f3e6d9d`; the "normal only, no `inline-block`" limit was lifted — `white-space` modes and `inline-block` are now consumed/painted (`crates/hodo-layout/src/inline.rs:626-640,702-714`), see the verified section below. |
| `<img src>` (relative URL resolved against the page directory) | ✓ | ✓ | ✓ | `cd1b1ec`; produces `DisplayItem::Image`. |

## Landed with a known limitation

| Property / feature | Status | Evidence & limit |
| --- | --- | --- |
| `border-style` (`solid`, `dashed`, …) | ✓ parsed, painter ignores the style | `25d03fa` accepts the token inside `border:` but `ComputedStyle` stores only widths and color; `f3e6d9d` paints all borders as solid rects. |
| Anonymous block generation from mixed inline/block children | ✓ consumed and painted | `f3e6d9d`; consecutive inline runs get their own flow, no line box spans a block sibling. |

## Formerly pending — verified in the current tree (code-read, not git log)

These rows were "parser only, layout pending" at `9df4f23`. Each flip below has
`file:line` evidence from a code read; limits are called out in the last column.
The M7/M8 ТЗ sections are kept only for traceability — these rows no longer wait
on layout.

| Property / feature | Parsed | Consumed | Painted | Evidence & limits |
| --- | --- | --- | --- | --- |
| Flexbox: `display: flex` / `inline-flex`, `flex-direction`, `flex-*` (grow/shrink/basis), `justify-content`, `align-items` / `align-self`, `gap`, `order` | ✓ | ✓ | ✓ | `crates/hodo-layout/src/flex.rs:153` (`layout_flex`), wired at `block.rs:520`; emits rect/text `DisplayItem`s (`flex.rs:198`). Limits: `flex-wrap` parsed but treated as nowrap (`flex.rs:266-267`); `align-content` has no `ComputedStyle` field (no hit in `crates/`); percentage gap/basis unresolved. |
| Grid: `display: grid` / `inline-grid`, `grid-template-columns/rows`, `gap`, auto-placement, `grid-column/row` | ✓ | ✓ | ✓ | `crates/hodo-layout/src/grid.rs:302` (`layout_grid`), wired at `block.rs:521`; items go through `layout_child` and emit `DisplayItem`s. Limit: percent/`fr` rows fall back to metric height (`grid.rs:394-395`). |
| Tables: `display: table*`, `border-spacing`, anonymous table boxes, `colspan`/`rowspan` | ✓ | ✓ | ✓ | `crates/hodo-layout/src/table.rs:274` (`layout_table`), wired at `block.rs:522`; cells emit rect/text items. Limit: `border-collapse` is parsed (`crates/hodo-style/src/lib.rs:1037`) but degrades to `separate` in layout (`table.rs:11,281-282`). |
| `position: static` / `relative` / `absolute` / `fixed` | ✓ | ✓ | ✓ except `fixed`: 🟡 | `relative` offsets `crates/hodo-layout/src/block.rs:391-395`; `absolute` box `block.rs:362,402-417`; `fixed` routes to `DisplayList::fixed_items` (`block.rs:298-322`, `lib.rs:108`) but shell painting is unwired (`lib.rs:99-107`, `TODO(integrator)`) — emitted, not painted. |
| `float` / `clear` | ✓ | ✓ | ✓ | Float boxes recorded in `crates/hodo-layout/src/block.rs:363-385,428-435`, `clear` at `block.rs:365-375`; line boxes wrap via `inline.rs:128-134,400-433`. Limit: one rect per side, no shrink-to-fit (`block.rs:273-275,378-379`). |
| `overflow: visible` / `hidden` / `clip` | ✓ | ✓ | ✓ | `clip_items` intersects rect/image items and drops whole text runs (`crates/hodo-layout/src/block.rs:133-170`); applied from `block.rs:389-390`. |
| `width` / `height`, `min-width` / `min-height` / `max-width` / `max-height` | ✓ | ✓ | ✓ | Parsed `crates/hodo-style/src/lib.rs:1081-1086`; clamped `crates/hodo-layout/src/block.rs:381-385,604-605,703-705`. Limit: widthless floats take the full content width (`block.rs:378-379`). |
| `white-space`, `line-height`, `text-align`, `text-indent`, `letter-spacing` / `word-spacing`, `text-decoration`, `overflow-wrap` | ✓ | ✓ | ✓ | Parsed `crates/hodo-style/src/lib.rs:965-968,1048-1060`; consumed in `crates/hodo-layout/src/inline.rs:603-645` (ws/indent/align/spacing), `inline.rs:144,616,797` (line-height), `inline.rs:448` (overflow-wrap); `line-through` painted at `crates/hodo-render/src/lib.rs:1379-1395`. `word-break`: no field/handler found in `crates/` — **not audited**, still pending. |
| `vertical-align`, `inline-block` | ✓ | ✓ | ✓ | Baseline shift `crates/hodo-layout/src/inline.rs:269-303,621`; `Display::InlineBlock` → `inline_block_atom` (`block.rs:525`, `inline.rs:702-714`), one atomic slot per line. |
| `z-index` / stacking, `box-sizing`, CSS variables `var()`, `calc()`, `@media`, `@font-face`, `background-image` / gradients | not audited | not audited | not audited | Unchanged from `9df4f23` (ТЗ M8 / post-M8 decision); not read this wave, so no status flip. |

## Non-CSS features with painted output

| Feature | Status | Evidence |
| --- | --- | --- |
| Headless screenshots (`hodo-shell --screenshot <url> out.png`, opaque white composite) | ✓ painted | landed before the wave; still current at `9df4f23` |
| `about:hodo` synthetic page with cache stats | ✓ painted | `cd1b1ec` |

# CSS property / feature status (Wave A baseline, plus current-tree verification)

> **WAVE 6 (2026-11-22) — property/layout seam truth-sync:** the shell now
> consumes the landed incremental/property APIs instead of leaving them idle.
> `PageContext::recompute_if_changed` diffs per-node fingerprints into
> `dirty_roots`, recomputes styles with the redirect-visited set, then classifies
> the change with `hodo_style::style_change_class`: `PaintOnly` takes the
> damage-only repaint path (geometry and item count unchanged), `LayoutAffecting`
> calls `hodo_layout::layout_from` on the dirty top-level subtrees. One
> `LayoutContext` lives per document; `reset()` on stylesheet swap / resize /
> cross-inline edits, and the headless screenshot path uses `layout_from` too.
> `:visited` is wired end-to-end (shell history → `compute_styles_with_visited`):
> link colour flips only for visited hrefs (`examples/js-site` stays
> byte-identical, SHA256 `09B8DCA9…8775D`). Module chunks take
> `load_many` (cold, parallel) or per-chunk cached loads (warm, 304 revalidation)
> through an adblock pre-check.
>
> Measured wave-6 perf (release, LARGE best-of-3): indexed styles 5.498 ms vs
> brute 20.215 ms (rule index ≈3.7×), layout 38.316 ms, render 6.406 ms,
> composite 50.22 ms (yesterday 68 ms); M5.3 gate PASS in both modes with new
> pins 22.0/38.5/25.0/40.5 ms; one-node incremental 4.37×; measure cache 3.88×;
> damage 0.3 % pixels. Parallel siblings remain negative (0.61×) — an honest
> skip, not a shipped win. `archive.org` paints 19,542 bytes (prize).
>
> **WAVE 4 LANDED:** the wave-4 tree was gated (fmt + clippy `-D warnings` +
> 837 workspace tests green; bigdoc perf PASS inside every 1.10× pin, no re-pin)
> and committed. Newly verified below: `float` now drops a float below an
> opposite-side float whose margin box it geometrically intersects (0.01px edge
> epsilon, 4-hop cap) — `clear_opposite_float` at
> `crates/hodo-layout/src/block.rs:281-302`, applied at `block.rs:500-531`; this
> puts EFF's third featured row below the first row's float band. Every status
> flip below was verified by reading current code (`file:line` evidence), not
> `git log`; re-check before using this as a release gate.

> **TRUTH-SYNC ROUND 2 (current uncommitted tree, NOT a commit):** re-read the
> CSS/layout-adjacent rows below against live code. Newly verified: descendant
> combinators are not folded (`.a .b` needs a real ancestor chain —
> `crates/hodo-css/src/lib.rs:3020-3050`, function
> `descendant_selectors_are_not_folded_into_one_compound`); a vendored
> DroidSansFallback face supplies CJK glyphs when DejaVu lacks them, behind a
> 1 MiB LRU atlas (`crates/hodo-render/src/lib.rs:13-15,39-49,73-75`,
> `glyph_cached` at `:128-160`, eviction at `:178-187`,
> tests `lib.rs:2400-2485`); and layout reads the flattened shadow-DOM child view
> via `Dom::render_children` (`crates/hodo-layout/src/block.rs:45-47`, also
> `flex.rs:174`, `grid.rs:94`, `forms.rs:51`, `table.rs:73`). Honest limits:
> glyphs neither face covers still tofu, the atlas is an A8-coverage cap (not a
> font-family shaping engine), and deeply nested HTML completes without a panic
> but the 512-level cap is still not enforced on the HTML side
> (`crates/hodo-fuzz/tests/robust.rs:108-146`, TODO at `110-113`). Rows not read
> this round keep their old status; no commits.

> **WAVE 3 LANDED:** the wave-3 tree was integrated, gated (fmt + clippy
> `-D warnings` + 826 workspace tests green) and committed. Newly verified below:
> `@layer` cascade semantics with `!important` order inversion, CSS custom
> properties `var()` with fallback + cycle guard, `calc()` evaluation, `@media`
> MQ4 ranges/hover/prefers-color-scheme, and percent width/margin/padding
> resolution against the containing block (plus float-height BFC and absolute
> auto-offset layout fixes). Every status flip below was verified by reading
> current code (`file:line` evidence), not `git log`; re-check before using this
> as a release gate.

> **WAVE 2 LANDED:** the uncommitted wave-2 tree was integrated, gated and
> committed. Newly verified below: `@media`/`@supports`/`@import`, document-order
> stylesheet merge / cascade order, link/interaction pseudos, `:where()`/`:is()`,
> `font-family` and viewport units. Every status flip below was verified by
> reading current code (`file:line` evidence), not `git log`; re-check before
> using this as a release gate.

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
| Selectors: combinators, 7 attribute operators incl. `i`, 7 pseudo-classes, specificity, rule index + Bloom filter | ✓ | ✓ | n/a | `25d03fa`; capped at `MAX_CSS_RULES = 100 000` and 10 000 rules per stylesheet. Round 2: descendant combinators are **not** folded into one compound — `.a .b` / `.a #b` / `.a *` / `.a [data-x]` / `.a :first-child` each need an ancestor chain (`crates/hodo-css/src/lib.rs:3020-3050`, `descendant_selectors_are_not_folded_into_one_compound`, `.a` cases near `:3043`). |
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
| `float` / `clear` | ✓ | ✓ | ✓ | Float boxes recorded in `crates/hodo-layout/src/block.rs:363-385,428-435`, `clear` at `block.rs:365-375`; line boxes wrap via `inline.rs:128-134,400-433`. Wave 4: cross-side collision — `clear_opposite_float` (`block.rs:281-302`, applied at `block.rs:500-531`) drops a float below an opposite-side float whose margin box it intersects (0.01px edge epsilon so touching edges still share a line, 4-hop cap); fixes EFF row 3. Limit: one rect per side, no shrink-to-fit (`block.rs:273-275,378-379`). |
| `overflow: visible` / `hidden` / `clip` | ✓ | ✓ | ✓ | `clip_items` intersects rect/image items and drops whole text runs (`crates/hodo-layout/src/block.rs:133-170`); applied from `block.rs:389-390`. |
| `width` / `height`, `min-width` / `min-height` / `max-width` / `max-height` | ✓ | ✓ | ✓ | Parsed `crates/hodo-style/src/lib.rs:1081-1086`; clamped `crates/hodo-layout/src/block.rs:381-385,604-605,703-705`. Limit: widthless floats take the full content width (`block.rs:378-379`). |
| `width` / `height` percentages + `margin` / `padding` percentages (floats & in-flow blocks) | ✓ | ✓ | ✓ | Wave 3: raw percentages kept at parse time in `width_percent`/`margin_percent`/`padding_percent` (`crates/hodo-style/src/lib.rs:261-265,1191-1216`; `parse_length_or_percent` at `1541-1552`) and resolved against the containing-block width in layout (`crates/hodo-layout/src/block.rs:388-402,688-706`). Floats form side-by-side columns (`block.rs:1598`); floats also raise a parent's auto-height (BFC fix, `block.rs:1731`). |
| `calc()` lengths | ✓ | ✓ | ✓ | Wave 3: recursive-descent evaluator in `hodo-css` resolves `px`/`em`/`rem`/`vw`/`vh` (nested parens ≤4) into parse-time px (`crates/hodo-css/src/lib.rs:950-1045`, tests `3360-3397`); consumed as ordinary lengths by style/layout. Limit: `%` inside `calc()` is dropped with one warning (no parse-time percentage basis), `em`/`rem` use a 16px fallback (TODO: thread element font size), `/0`/garbage drops the declaration. |
| CSS custom properties `var()` / `@layer` cascade layers | ✓ | ✓ | n/a | Wave 3: `var(--name[, fallback])` substitution with cycle guard and sharing-cache soundness (`crates/hodo-css/src/lib.rs:1349-1416`, tests `3580-3609`); `@layer` establishes first-seen order, later layers win and `!important` inverts layer order (`lib.rs:105-108,241-391`, tests `3539-3574`), rank carried on rules via `LAYER_MARKER`/`cascade_layer`. |
| `white-space`, `line-height`, `text-align`, `text-indent`, `letter-spacing` / `word-spacing`, `text-decoration`, `overflow-wrap` | ✓ | ✓ | ✓ | Parsed `crates/hodo-style/src/lib.rs:965-968,1048-1060`; consumed in `crates/hodo-layout/src/inline.rs:603-645` (ws/indent/align/spacing), `inline.rs:144,616,797` (line-height), `inline.rs:448` (overflow-wrap); `line-through` painted at `crates/hodo-render/src/lib.rs:1379-1395`. `word-break`: no field/handler found in `crates/` — **not audited**, still pending. |
| `vertical-align`, `inline-block` | ✓ | ✓ | ✓ | Baseline shift `crates/hodo-layout/src/inline.rs:269-303,621`; `Display::InlineBlock` → `inline_block_atom` (`block.rs:525`, `inline.rs:702-714`), one atomic slot per line. |
| Media queries: `@media` (`screen`/`print`, `min-`/`max-width`, `min-`/`max-height`, `orientation`, `hover`) + `@supports` + `@import` | ✓ | ✓ | n/a | Wave 2: `hodo_css::Viewport` (`crates/hodo-css/src/lib.rs:31-46`) plus `parse_stylesheet_with`, which evaluates conditions and drops non-matching blocks (`lib.rs:223-292,346-452`). The shell threads the real viewport and merges sheet bodies document-order (`crates/hodo-shell/src/app.rs:1101-1152`); `media="print"` links are skipped (`app.rs:1119-1127`). `@import`/`@layer` warn once and are skipped (`lib.rs:283-289`). CSS wave: `hover` (none matches, hover does not), `prefers-color-scheme` against `Viewport::dark` (shell threads `hpages::read_dark_enabled()`, default light), `not` inversion, top-level comma/`or` lists, MQ4 `width`/`height` ranges, `min-`/`max-resolution` and `-webkit-`/`-moz-` DPR aliases, reduced-motion/forced-colors/pointer; unknown conditions still warn once per sheet. |
| Link/interaction pseudos (`:link`, `:visited`, `:hover`, `:focus`, `:active`) + `:where()` / `:is()` | ✓ | ✓ | n/a | Wave 2: parsed/encoded in `hodo-css` (`crates/hodo-css/src/lib.rs:124-128,949-952,1002,1121`); `:link`/`:visited` both match `a[href]` and pointer/focus states never match in a static render (`lib.rs:1652-1657`); `:where()` is zero-specificity, `:is()` takes its best branch (`lib.rs:1426`). Consumed by the `hodo-style` matcher. |
| `font-family` | ✓ | ✓ | n/a | Wave 2: parsed as a declaration (`crates/hodo-css/src/lib.rs:1946-1958`), stored and inherited in `ComputedStyle` (`crates/hodo-style/src/lib.rs:212,925`) and seeded in the layout fallback style (`crates/hodo-layout/src/lib.rs:155`). Round 2: glyph selection now tries the primary face then the vendored DroidSansFallback (`crates/hodo-render/src/lib.rs:13-15,39-49`, `glyph_cached` at `:128-160`), with mixed Latin/CJK rasterization covered at `lib.rs:2416-2445` and a 1 MiB LRU A8 atlas (`lib.rs:73-75`, eviction at `:178-187`); still **no per-family shaping**, and codepoints neither face covers render the primary face's tofu (`lib.rs:142-150`). |
| Viewport units `vh` / `vw` / `vmin` / `vmax` | ✓ | ✓ | ✓ | Wave 2: resolved to px against the stylesheet `Viewport` at parse time (`crates/hodo-css/src/lib.rs:632-655,639-650`), so `hodo-style`/`hodo-layout` consume them as ordinary lengths; tests `lib.rs:2607-2632`. `vi`/`vb` stay unresolved. CSS wave: `calc()` resolves at the same parse-time point (`calc(2*(10px+5px))` → `30px`; `/0` or garbage drops the declaration). `em`/`rem` in `calc()` use a 16px fallback — there is no element font-size context at parse time (TODO: thread it). `%` in `calc()` is dropped with one warning: plain `%` has no percentage basis in this pipeline and widths resolve late. |
| Stylesheet merge order (cascade order) | ✓ | ✓ | n/a | Wave 2: `<style>`/`<link>` bodies are appended in document order so a later sheet wins a same-specificity tie (`crates/hodo-shell/src/app.rs:1107-1152`, test `app.rs:1235`); failed links warn and the page still renders (`app.rs:1132-1138`, test `app.rs:1300`). Wave 3: `@layer` now parses into real cascade layers (first-seen order, later layers win, `!important` inverts) — see the `var()`/`@layer` row above. |
| `z-index` / stacking, `box-sizing`, `@font-face`, `background-image` / gradients | not audited | not audited | not audited | Unchanged from `9df4f23` (ТЗ M8 / post-M8 decision); not read in truth-sync round 2 either, so no status flip. `var()`/`calc()`/`@layer` are no longer in this row — they landed in wave 3 (see rows above); `@media` landed in wave 2. |

## Non-CSS features with painted output

| Feature | Status | Evidence |
| --- | --- | --- |
| Headless screenshots (`hodo-shell --screenshot <url> out.png`, opaque white composite) | ✓ painted | landed before the wave; still current at `9df4f23` |
| `about:hodo` synthetic page with cache stats | ✓ painted | `cd1b1ec` |
| Shadow-DOM flattened render view | ✓ painted (substrate) | Round 2: `Dom::attach_shadow`/`render_children`/`SHADOW_ROOT_TAG` (`crates/hodo-dom/src/lib.rs:32,382-424`); layout reads the flattened children in `crates/hodo-layout/src/block.rs:45-47`, `flex.rs:174`, `grid.rs:94`, `forms.rs:51`, `table.rs:73`; JS-visible light view filters the root (`crates/hodo-js/src/dom.rs:200-213,676-704`). Limit: slots/assignment are not modelled; `adoptedStyleSheets` is storage-only (`crates/hodo-js/src/prelude.rs:1436-1447`). |
| Deeply nested HTML parse (crash class) | ✓ no panic; depth cap not enforced | `crates/hodo-fuzz/tests/robust.rs:108-146` — 600-deep `<div>` chain completes under 10 s and inside `MAX_DOM_NODES`; the TODO at `110-113` says the HTML layer still does not enforce `MAX_NESTING_DEPTH`, so the surplus nodes are retained rather than degraded. Not audited beyond this test. |

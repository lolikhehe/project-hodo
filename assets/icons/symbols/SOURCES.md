# Vendored icon sources

All SVGs in this directory are **Material Symbols, Outlined style, 24px** from
[`google/material-design-icons`](https://github.com/google/material-design-icons)
(`master` branch), licensed **Apache-2.0** — see [`LICENSE-apache.txt`](LICENSE-apache.txt).
They were fetched on 2026-11-22 and are vendored verbatim (byte-for-byte from
`raw.githubusercontent.com`). `xtask-icons` parses them at build time only; no
runtime code reads SVG.

## URL pattern

```
https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/<symbol>/materialsymbolsoutlined/<symbol>_24px.svg
```

Each file below is stored as `<API name>.svg`; the **API name** is the key passed to
`hodo_icons::rasterize` / `hodo_icons::paths::icon_path`.

## Name adaptations

The brief's names are UI-level names; three do not exist verbatim in the upstream
symbol set and were mapped as follows (all other names match upstream exactly):

| API name | Upstream symbol | Reason |
| --- | --- | --- |
| `back` | `arrow_back` | upstream prefixes navigation arrows |
| `forward` | `arrow_forward` | upstream prefixes navigation arrows |
| `reload` | `autorenew` | upstream has no `reload`; `autorenew` is the reload glyph (`refresh` is vendored separately) |
| `plus` | `add` | upstream name is `add` |
| `reader` | `chrome_reader_mode` | upstream reading-mode glyph |
| `copy` | `content_copy` | upstream prefixes copy with `content_` |
| `private` | `visibility_off` | upstream has no `private`; privacy is represented by hidden visibility |
| `star_border` | derived (see note) | upstream fetch bug: the vendored copy was byte-identical to `star.svg`, so the outline was derived locally instead |

## Exact source URLs

| File (API name) | Exact source URL |
| --- | --- |
| `back.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/arrow_back/materialsymbolsoutlined/arrow_back_24px.svg |
| `forward.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/arrow_forward/materialsymbolsoutlined/arrow_forward_24px.svg |
| `reload.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/autorenew/materialsymbolsoutlined/autorenew_24px.svg |
| `star.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/star/materialsymbolsoutlined/star_24px.svg |
| `star_border.svg` | derived outline (not fetched) — the vendored file was a duplicate of `star.svg` due to an upstream fetch bug, so its path was derived locally as an outer contour plus a counter-wound inner contour (a hollow star). Its `viewBox` is `0 0 24 24` so xtask-icons reproduces the path byte-for-byte. |
| `lock.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/lock/materialsymbolsoutlined/lock_24px.svg |
| `search.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/search/materialsymbolsoutlined/search_24px.svg |
| `close.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/close/materialsymbolsoutlined/close_24px.svg |
| `plus.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/add/materialsymbolsoutlined/add_24px.svg |
| `menu.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/menu/materialsymbolsoutlined/menu_24px.svg |
| `settings.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/settings/materialsymbolsoutlined/settings_24px.svg |
| `shield.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/shield/materialsymbolsoutlined/shield_24px.svg |
| `tabs.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/tabs/materialsymbolsoutlined/tabs_24px.svg |
| `download.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/download/materialsymbolsoutlined/download_24px.svg |
| `reader.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/chrome_reader_mode/materialsymbolsoutlined/chrome_reader_mode_24px.svg |
| `screenshot.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/screenshot/materialsymbolsoutlined/screenshot_24px.svg |
| `dark_mode.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/dark_mode/materialsymbolsoutlined/dark_mode_24px.svg |
| `light_mode.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/light_mode/materialsymbolsoutlined/light_mode_24px.svg |
| `bookmark.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/bookmark/materialsymbolsoutlined/bookmark_24px.svg |
| `extension.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/extension/materialsymbolsoutlined/extension_24px.svg |
| `history.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/history/materialsymbolsoutlined/history_24px.svg |
| `home.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/home/materialsymbolsoutlined/home_24px.svg |
| `more_vert.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/more_vert/materialsymbolsoutlined/more_vert_24px.svg |
| `check.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/check/materialsymbolsoutlined/check_24px.svg |
| `arrow_drop_down.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/arrow_drop_down/materialsymbolsoutlined/arrow_drop_down_24px.svg |
| `chevron_left.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/chevron_left/materialsymbolsoutlined/chevron_left_24px.svg |
| `chevron_right.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/chevron_right/materialsymbolsoutlined/chevron_right_24px.svg |
| `folder_open.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/folder_open/materialsymbolsoutlined/folder_open_24px.svg |
| `print.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/print/materialsymbolsoutlined/print_24px.svg |
| `zoom_in.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/zoom_in/materialsymbolsoutlined/zoom_in_24px.svg |
| `zoom_out.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/zoom_out/materialsymbolsoutlined/zoom_out_24px.svg |
| `fullscreen.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/fullscreen/materialsymbolsoutlined/fullscreen_24px.svg |
| `copy.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/content_copy/materialsymbolsoutlined/content_copy_24px.svg |
| `share.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/share/materialsymbolsoutlined/share_24px.svg |
| `link.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/link/materialsymbolsoutlined/link_24px.svg |
| `public.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/public/materialsymbolsoutlined/public_24px.svg |
| `visibility.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/visibility/materialsymbolsoutlined/visibility_24px.svg |
| `private.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/visibility_off/materialsymbolsoutlined/visibility_off_24px.svg |
| `refresh.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/refresh/materialsymbolsoutlined/refresh_24px.svg |
| `filter.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/filter/materialsymbolsoutlined/filter_24px.svg |
| `help.svg` | https://raw.githubusercontent.com/google/material-design-icons/master/symbols/web/help/materialsymbolsoutlined/help_24px.svg |

LICENSE source: https://raw.githubusercontent.com/google/material-design-icons/master/LICENSE

# Extensions in hodo (cross-browser compatibility)

hodo is built to run WebExtensions from **both** the Chrome/Chromium and the
Firefox/Gecko families. A single extension pipeline — manifest parsing, a
canonical API dispatch table, and an in-process host — accepts either dialect
so authors do not have to fork an extension per engine.

> **Status legend (read this first)**
>
> - **Implemented** — the actual Rust code is present and does real work today.
>   `Manifest::parse`, `ApiRegistry::dispatch`, and `Host::load_dir` /
>   `Host::dispatch` are all implemented; there are no `todo!()` stubs in the tree.
>
> The contracts below describe what the crates actually do. What is *not* yet
> implemented is the JavaScript layer that would call them from an extension's
> `bg.js` — that is milestone M2 (see `ROADMAP.md`).

Crates involved:

| Crate | Role |
| --- | --- |
| `hodo-ext-manifest` | Parse MV2/MV3 manifests in either dialect; detect browser flavor. |
| `hodo-ext-api` | Canonical namespace mapping and the `ApiCall` / `ApiResult` dispatch contract. |
| `hodo-ext-host` | Load unpacked extension directories, gate calls on declared permissions. |

## Compatibility matrix

The manifest model accepts both manifest versions in both dialects. The fields
that make this possible (`version`, `background_scripts`, `service_worker`,
`gecko_id`, `chrome_key`, `action_title`, `icons`) are all defined in
`Manifest` and populated by `parse` (**Implemented**).

| Manifest | Dialect | Background form | Identity / key | Toolbar field | Accepted? |
| --- | --- | --- | --- | --- | --- |
| MV2 | Gecko / Firefox | `background.scripts` → `background_scripts` | `browser_specific_settings.gecko.id` → `gecko_id` | `browser_action` → `action_title` | Yes |
| MV2 | Chrome / Edge | `background.scripts` → `background_scripts` | top-level `chrome_key` when present | `browser_action` → `action_title` | Yes |
| MV3 | Chrome / Chromium | `background.service_worker` → `service_worker` | top-level `chrome_key` when present | `action` → `action_title` | Yes |
| MV3 | Gecko | `background.service_worker` → `service_worker` | `gecko_id` when present | `action` → `action_title` | Yes |

Notes grounded in the source:

- `background.scripts` (the Gecko/Edge MV2 style) and `background.service_worker`
  (the Chrome MV3 style) are **both** represented, as `background_scripts:
  Vec<String>` and `service_worker: Option<String>` respectively.
- `browser_specific_settings.gecko.id` is honored as `gecko_id`.
- The top-level Chrome Web Store `"key"` is honored as `chrome_key`.
- Both `action` (Chrome MV3) and `browser_action` (Gecko MV2) feed the single
  `action_title` field.
- `flavor(&Manifest)` (**Implemented**) classifies an extension as `Gecko` when
  `gecko_id` is set, `Chrome` when `chrome_key` is set, otherwise `Generic`.
  Because flavor detection keys off those two fields, an MV3 manifest that
  omits `"key"` classifies as `Generic` — which is fine, since flavor does not
  gate dispatch. The shipped `hello-gecko` has a `gecko_id` and classifies as
  `Gecko`.

## One canonical namespace: `chrome.*` and `browser.*` are the same table

Both prefixes are stripped and folded into a single canonical namespace by
`canonical_namespace(&str)` (**Implemented**). There is no separate Chrome
table and Gecko table — only one dispatch table.

| Author writes | Canonical namespace |
| --- | --- |
| `chrome.storage.local` | `storage` |
| `browser.storage` | `storage` |
| `browser.storage.local` / `.sync` / `.managed` / `.session` | `storage` |
| `chrome.tabs` / `browser.tabs` | `tabs` |
| `chrome.runtime` / `browser.runtime` | `runtime` |
| `browser.extension` | `runtime` |

So `chrome.storage.local.get(...)` and `browser.storage.get(...)` resolve to the
same `StorageGet` call — the dialect prefix is irrelevant once canonicalized.
Unknown namespaces return `None` and are rejected.

## Permissions model

Dispatch is gated by the permissions declared in the extension's manifest. The
mapping from an `ApiCall` to the namespace it requires lives in
`Host::required_namespace(call)` (**Implemented**):

| `ApiCall` | Required namespace |
| --- | --- |
| `TabsCreate`, `TabsQueryActive` | `tabs` |
| `StorageSet`, `StorageGet` | `storage` |
| `SendMessage` | `runtime` (always allowed) |

A call is dispatched only if the manifest's `permissions` list contains the
required namespace; `runtime` is always allowed. A denied call returns
`ApiResult::Err(String)` with an error string instead of executing. `Host::dispatch`
enforces this at runtime. `host_permissions` is a separate field for URL patterns
and is **never** treated as API permission: an entry such as `"storage"` there
does not grant the storage API.

## Supported APIs (Rust-side dispatch contract)

These are the API variants defined in `hodo-ext-api`. `ApiRegistry::dispatch`
(**Implemented**) owns an in-memory `HashMap<String, serde_json::Value>` for
storage and returns the results below.

| API (both dialects) | `ApiCall` variant | Behavior |
| --- | --- | --- |
| `tabs.create({url})` | `TabsCreate { url }` | Returns a mock, incrementing `tabId`. |
| `tabs.queryActive()` | `TabsQueryActive` | Returns a mock active-tab result. |
| `storage.local.get(key)` | `StorageGet { key }` | Reads from the in-memory map; missing keys yield `null`. |
| `storage.local.set(key, value)` | `StorageSet { key, value }` | Writes to the in-memory map. |
| `runtime.sendMessage(msg)` | `SendMessage { message }` | Acknowledged but not delivered (no JS receiver yet). |

Results use `ApiResult::Ok(serde_json::Value)` on success and
`ApiResult::Err(String)` on a denied call or error.

## Installing an extension

Drop an unpacked extension directory (one containing `manifest.json`) into the
repo's `extensions/` folder. Two examples ship with hodo:

- `extensions/hello-chrome/` — MV3, Chrome dialect: `background.service_worker`,
  `action`, `chrome.storage.local` + `chrome.tabs` usage.
- `extensions/hello-gecko/` — MV2, Gecko dialect: `background.scripts`,
  `browser_specific_settings.gecko.id`, `browser_action`, `browser.*` usage.

`Host::load_dir(path)` (**Implemented**) reads `manifest.json` from such a
directory (capped at 2 MiB). The `bg.js` files are not executed yet, so the
examples demonstrate the manifest format and dispatch contract rather than
running end to end.

## Not yet (honest gaps)

- **No JavaScript execution.** The `bg.js` files in the examples are not run.
  Wiring a JS engine (`boa_engine`) with `chrome.*` / `browser.*` globals injected
  is **M2** on the roadmap.
- **No content scripts** injection or lifecycle.
- **No `web_accessible_resources`** handling.
- **No CSP** enforcement for extension pages.
- Out-of-process sandboxing of the extension host is a later milestone (**M3**).

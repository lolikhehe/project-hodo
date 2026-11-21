# Extensions in hodo (cross-browser compatibility)

hodo is built to run WebExtensions from **both** the Chrome/Chromium and the
Firefox/Gecko families. A single extension pipeline — manifest parsing, a
canonical API dispatch table, an in-process host and a boa JS bridge — accepts
either dialect so authors do not have to fork an extension per engine.

> **Status legend (read this first)**
>
> - **Implemented** — the actual Rust code is present and does real work today.
>   `Manifest::parse`, `ApiRegistry::dispatch`, `ApiRegistry::drain_events` and
>   `Host::load_dir` / `Host::dispatch` are implemented; there are no `todo!()`
>   stubs in the tree. `ExtScriptRuntime` in `hodo-js` evaluates JS and installs
>   the WebExtension globals.
> - **In progress** — wiring the JS runtime into the extension host: creating one
>   runtime per extension, filtering the exposed namespace by that extension's
>   manifest permissions, and delivering drained events to JS listeners.
>
> The contracts below describe what the crates actually do. What is *not* yet
> finished is the host-side plumbing that runs an unpacked extension's `bg.js`
> end to end (see `ROADMAP.md`, M2).

Crates involved:

| Crate | Role |
| --- | --- |
| `hodo-ext-manifest` | Parse MV2/MV3 manifests in either dialect; detect browser flavor. |
| `hodo-ext-api` | Canonical namespace mapping, the `ApiCall` / `ApiResult` dispatch contract and queued `ExtEvent`s. |
| `hodo-ext-host` | Load unpacked extension directories, gate calls on declared permissions. |
| `hodo-js` | boa 0.22 bridge that installs `chrome.*` / `browser.*` globals over that API table. |

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
  `action_title` field; the `icons` map is recorded as `HashMap<String, String>`.
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
| `chrome.notifications` / `browser.notifications` | `notifications` |

So `chrome.storage.local.get(...)` and `browser.storage.get(...)` resolve to the
same `StorageGet` call — the dialect prefix is irrelevant once canonicalized.
`storage.sync` uses a separate backing map from `storage.local` but shares the
`storage` permission. Unknown namespaces return `None` and are rejected.

## Permissions model

Dispatch is gated by the permissions declared in the extension's manifest. The
mapping from an `ApiCall` to the namespace it requires lives in
`Host::required_namespace(call)` (**Implemented**):

| `ApiCall` | Required namespace |
| --- | --- |
| `TabsCreate`, `TabsQueryActive`, `TabsList`, `TabsActivate` | `tabs` |
| `StorageSet`, `StorageGet`, `StorageSyncSet`, `StorageSyncGet` | `storage` |
| `SendMessage` | `runtime` (always allowed) |
| `NotificationsCreate` | `notifications` |

A call is dispatched only if the manifest's `permissions` list contains the
required namespace; `runtime` is always allowed. A denied call returns
`ApiResult::Err(String)` with an error string instead of executing. `Host::dispatch`
enforces this at runtime. `host_permissions` is a separate field for URL patterns
and is **never** treated as API permission: an entry such as `"storage"` there
does not grant the storage API. Note that using notifications requires
`"notifications"` in `permissions` (see `extensions/notify-chrome/`).

## Supported APIs (Rust-side dispatch contract)

These are the API variants defined in `hodo-ext-api`. `ApiRegistry::dispatch`
(**Implemented**) owns in-memory `HashMap`s for local and sync storage, a tab
list and a notification counter; storage and notification writes queue events.

| API (both dialects) | `ApiCall` variant | Behavior |
| --- | --- | --- |
| `tabs.create({url})` | `TabsCreate { url }` | Returns an incrementing `tabId` and `url`. |
| `tabs.queryActive()` | `TabsQueryActive` | Returns a mock active-tab result. |
| `tabs.list()` | `TabsList` | Returns all tabs created so far as `[{tabId, url}]`. |
| `tabs.activate(tabId)` | `TabsActivate { tab_id }` | Queues a `TabActivated` event; returns null. |
| `storage.local.get(key)` | `StorageGet { key }` | Reads the local map; missing keys yield `null`. |
| `storage.local.set(key, value)` | `StorageSet { key, value }` | Writes the local map; queues `StorageChanged`. |
| `storage.sync.get(key)` | `StorageSyncGet { key }` | Reads the separate sync map; missing keys yield `null`. |
| `storage.sync.set(key, value)` | `StorageSyncSet { key, value }` | Writes the sync map; queues `StorageChanged`. |
| `runtime.sendMessage(msg)` | `SendMessage { message }` | Queues a `MessageReceived` event; returns null (no JS receiver wired yet). |
| `notifications.create(title, message)` | `NotificationsCreate { title, message }` | Returns an incrementing `notificationId`; queues `NotificationShown`. |

Results use `ApiResult::Ok(serde_json::Value)` on success and
`ApiResult::Err(String)` on a denied call or error.

### Events

`ExtEvent` (**Implemented**) is the queue side of the bridge. Calls push events
in dispatch order; the host takes them all with `ApiRegistry::drain_events`,
which empties the queue.

| `ExtEvent` | Emitted by |
| --- | --- |
| `StorageChanged { key }` | `StorageSet`, `StorageSyncSet` |
| `MessageReceived { message }` | `SendMessage` |
| `TabActivated { tab_id }` | `TabsActivate` |
| `NotificationShown { title }` | `NotificationsCreate` |

Host-side delivery of these events to JS `onMessage` / `onChanged`-style
listeners is **in progress**.

## JavaScript execution

`hodo-js` embeds `boa_engine` 0.22 behind the default-on `js` feature
(`--no-default-features` leaves an `eval` stub). `ExtScriptRuntime::new` /
`eval` / `registry` create one boa `Context` and install `chrome` and `browser`
as the **same** JS object — so the two dialects are interchangeable inside a
script. The exposed surface is `storage.local` / `storage.sync` (get/set),
`tabs.create` / `tabs.list`, `runtime.sendMessage` and `notifications.create`.
Six tests cover evaluation, errors, sync/local aliasing and a cross-dialect
roundtrip. Caveats worth knowing:

- The runtime is single-threaded: boa's `Context` is `!Send`, so the bridge uses
  `Rc<RefCell<ApiRegistry>>` with a thread-local handle table.
- Per-extension namespace filtering by manifest permissions (and event
  delivery) is **in progress** in the host; the runtime itself does not gate
  calls yet. `Host::dispatch` still enforces permissions on the Rust side.
- In the JS bridge, `notifications.create` is currently acknowledged (returns
  null) rather than dispatched; the `NotificationsCreate` `ApiCall` and its
  event are already implemented on the Rust side. Callbacks/promises are
  deferred — the bridge is synchronous.
- `popup-chrome/` and `i18n-gecko/` demonstrate parsing only: `Manifest` records
  the `icons` map and `action_title`, but `action.default_popup`,
  `default_locale` and `_locales` are not typed fields. The manifests parse as
  given, `__MSG_appName__`-style `name` values stay literal, and `__MSG__`
  substitution against `_locales` is roadmap.

## Installing an extension

Drop an unpacked extension directory (one containing `manifest.json`) into the
repo's `extensions/` folder. Eight examples ship with hodo:

- `hello-chrome/` — MV3, Chrome dialect: `background.service_worker`, `action`, `chrome.storage.local` + `chrome.tabs` usage.
- `hello-gecko/` — MV2, Gecko dialect: `background.scripts`, `gecko_id`, `browser_action`, `browser.*` usage.
- `tabs-demo-chrome/` — MV3 tab APIs.
- `storage-demo-gecko/` — MV2 local storage.
- `notify-chrome/` — MV3 notifications with the `notifications` permission.
- `messages-gecko/` — MV2 `runtime.onMessage` style messaging.
- `popup-chrome/` — MV3 `popup.html` with real 16/48px PNG icons.
- `i18n-gecko/` — MV2 `_locales/en/messages.json` with `__MSG__` tokens.

`Host::load_dir(path)` (**Implemented**) reads `manifest.json` from such a
directory (capped at 2 MiB). The host does not yet create an `ExtScriptRuntime`
per extension, so `bg.js` files are not run end to end; `hodo-js` can evaluate
them in isolation today.

## Not yet (honest gaps)

- **Host wiring for JS execution.** The engine and bridge exist; running an
  unpacked extension's scripts with per-extension permission filtering and
  event delivery is milestone M2 (**in progress**).
- **Async WebExtension semantics.** Promises and event-listener callbacks are
  deferred; the bridge is synchronous.
- **`__MSG__` substitution** against `_locales` (names stay literal today).
- **Popup / toolbar UI** for `action.default_popup` pages.
- **Content scripts** injection or lifecycle.
- **`web_accessible_resources`** handling.
- **CSP** enforcement for extension pages.
- Out-of-process sandboxing of the extension host is a later milestone (**M3**).

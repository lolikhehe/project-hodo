# Extensions in hodo (cross-browser compatibility)

hodo is built to run WebExtensions from **both** the Chrome/Chromium and the
Firefox/Gecko families. A single extension pipeline — manifest parsing, a
canonical API dispatch table, an in-process host and a boa JS bridge — accepts
either dialect so authors do not have to fork an extension per engine.

> **Status legend (read this first)**
>
> - **Landed (Wave A)** — `Manifest::parse`, `ApiRegistry::dispatch`,
>   `ApiRegistry::drain_events`, `Host::load_dir` / `Host::dispatch` and, as of
>   `d63a90c`, one `ExtScriptRuntime` per extension with permission-filtered
>   namespaces, drained-event delivery and the MV3 idle lifecycle. `hodo-js`
>   owns an event loop with timers, Promises and callbacks. There are no
>   `todo!()` stubs in the tree.
> - **Still in progress / roadmap** — shell GUI (M1) and the remaining gaps
>   listed under "Not yet" below (content scripts, popup UI, sandboxing).
>   Everything marked **Landed** is in the git history through `9df4f23`.
>
> The contracts below describe what the crates actually do; each wave fact is
> tagged with the commit that landed it. See `ROADMAP.md` for milestone scope.
>
> **Truth-sync round 2 (current uncommitted tree, NOT a commit):** only the i18n
> fixture below changed in this zone — `ru` messages were added to
> `extensions/i18n-gecko/_locales/ru/messages.json:1-8` and are resolved by the
> new test at `crates/hodo-ext-i18n/src/lib.rs:640-654`. No new extension API
> namespace, permission or manifest field landed in this arc (the large uncommitted
> work is page-JS/shell/render). The claimed ext P1×3 bug-cycle fixes have no
> ledger or matching code read in these crates, so they are **not audited** and
> nothing below was flipped for them.

Crates involved:

| Crate | Role |
| --- | --- |
| `hodo-ext-manifest` | Parse MV2/MV3 manifests in either dialect; detect browser flavor. |
| `hodo-ext-api` | Canonical namespace mapping, the `ApiCall` / `ApiResult` dispatch contract and queued `ExtEvent`s. |
| `hodo-ext-host` | Load unpacked extension directories, gate calls on declared permissions, validate CSP, own per-extension runtimes and the MV3 idle lifecycle. |
| `hodo-ext-i18n` | `_locales` message resolution: fallback chain, placeholders, `@@` predefines and `__MSG_` substitution (`4aec263`). |
| `hodo-js` | boa 0.22 bridge with an event loop: installs `chrome.*` / `browser.*` globals over that API table and runs timers/promises/events (`d63a90c`). |

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
`storage` permission; `storage.session` is a third, RAM-only area under the same
10 MiB quota (`d63a90c`). Unknown namespaces return `None` and are rejected.

## Permissions model

Dispatch is gated by the permissions declared in the extension's manifest. The
mapping from an `ApiCall` to the namespace it requires lives in
`Host::required_namespace(call)` (**Implemented**):

| `ApiCall` | Required namespace |
| --- | --- |
| `TabsCreate`, `TabsQueryActive`, `TabsList`, `TabsActivate` | `tabs` |
| `StorageSet`, `StorageGet`, `StorageSyncSet`, `StorageSyncGet`, `StorageSessionSet`, `StorageSessionGet` | `storage` (one namespace for all three areas) |
| `AlarmCreate`, `AlarmClear` | `alarms` |
| `Fetch` | `fetch` — resolved through URL patterns in `host_permissions` / `<all_urls>` |
| `SendMessage` | `runtime` (always allowed) |
| `NotificationsCreate` | `notifications` |

A call is dispatched only if the manifest's `permissions` list contains the
required namespace; `runtime` is always allowed. A denied call returns
`ApiResult::Err(String)` with an error string instead of executing. `Host::dispatch`
enforces this at runtime, and the JS fetch hook calls the same `fetch_for`
helper so a script-originated `fetch` cannot bypass the check (`d63a90c`).
`host_permissions` is a separate field for URL patterns and is **never** treated
as API permission for other namespaces: an entry such as `"storage"` there does
not grant the storage API. `fetch` is the exception — its URL must match
`host_permissions` (or `<all_urls>`); URLs containing credentials are rejected,
and only `GET`/`POST` up to 2 MiB are supported. Note that notifications require
`"notifications"` in `permissions` (see `extensions/notify-chrome/`).

## Supported APIs (Rust-side dispatch contract)

These are the API variants defined in `hodo-ext-api`. `ApiRegistry::dispatch`
(**Landed**) owns in-memory `HashMap`s for `local`, `sync` and `session` (RAM-only)
storage under a shared `STORAGE_QUOTA_BYTES = 10 * 1024 * 1024` quota, an
`alarms: BTreeMap` driven by an injectable clock, a tab list and a notification
counter; storage, alarm and notification writes queue events (`d63a90c`).

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
| `storage.session.get(key)` / `.set(key, value)` | `StorageSessionGet` / `StorageSessionSet` | RAM-only area, dropped by `clear_session()`; excluded from the persistent quota. |
| `alarms.create(name, {delayMs, periodMs?})` / `.clear(name)` | `AlarmCreate { name, delay_ms, period_ms, now_ms }` / `AlarmClear { name }` | Schedules on the host clock; `advance_clock` fires at most one `AlarmFired` per alarm per tick; one-shot alarms are removed after firing. |
| `fetch(url, {method, body})` | `Fetch { url, method, body }` | Permission-gated GET/POST through the shared `fetch_for` helper; returns `{status, body, ok}` (2 MiB cap). |
| `runtime.sendMessage(msg)` | `SendMessage { message }` | Queues a `MessageReceived` event; listeners may return a Promise and the first reply wins. |
| `notifications.create(title, message)` | `NotificationsCreate { title, message }` | Returns an incrementing `notificationId`; queues `NotificationShown`. |

Results use `ApiResult::Ok(serde_json::Value)` on success and
`ApiResult::Err(String)` on a denied call or error.

### Events

`ExtEvent` (**Landed**) is the queue side of the bridge. Calls push events in
dispatch order; the host takes them all with `ApiRegistry::drain_events`, which
empties the queue. `hodo-js` drains them on each `tick()` and fires the matching
listeners (`d63a90c`).

| `ExtEvent` | Emitted by | JS listener payload |
| --- | --- | --- |
| `StorageChanged { area, key, old_value, new_value }` | `StorageSet`, `StorageSyncSet` (not `session`) | `storage.onChanged(changes, area)` with `changes[key].oldValue` / `.newValue` |
| `AlarmFired { name }` | `alarms` due on the injected clock | `alarms.onAlarm(alarm)` with `alarm.name` |
| `MessageReceived { message }` | `SendMessage` | `runtime.onMessage(message, sender, reply)`; returning a Promise is supported |
| `TabActivated { tab_id }` | `TabsActivate` | internal event queue |
| `NotificationShown { title }` | `NotificationsCreate` | internal event queue |

## JavaScript execution and the async runtime (`d63a90c`)

`hodo-js` embeds `boa_engine` 0.22 behind the default-on `js` feature
(`--no-default-features` leaves an `eval` stub). `ExtScriptRuntime::new` /
`eval` / `tick` / `registry` create one boa `Context` and install `chrome` and
`browser` as the **same** JS object — the two dialects are interchangeable
inside a script. Since Wave A the runtime also owns a browser-like event loop:
a FIFO task queue of due timers and drained `ExtEvent`s, with
`HodoJobExecutor::drain` running boa promise jobs after **every** task, so a
microtask runs before the next timer. `tick()` runs queued tasks, timers and
their microtasks to quiescence (bounded by `MAX_TASKS_PER_TICK = 4096`);
timers are bounded by `hodo_types::limits::MAX_ACTIVE_TIMERS = 100`.

Exposed surface: `storage.local` / `storage.sync` / `storage.session`
(get/set, plus `storage.onChanged`), `alarms.create/clear` + `onAlarm`,
`tabs.create/query/list`, `runtime.sendMessage`, `runtime.onMessage`,
`runtime.connect` (`Port` with `postMessage`/`onMessage`/`onDisconnect`),
`notifications.create`, `setTimeout`/`setInterval`/`clearTimeout`/
`clearInterval` and a permission-gated global `fetch`. All calls are
async-capable; `Host` creates one runtime per extension, exposes only the
namespaces allowed by its manifest and delivers drained events to listeners.

The runtime is single-threaded (boa's `Context` is `!Send`), so it uses
`Rc<RefCell<ApiRegistry>>` with a thread-local handle table. Script-driven
`eval` and `Function` compilation are blocked by host hooks
("content security policy: eval is disabled"), and `Host::load_dir` rejects
manifest CSPs containing `'unsafe-eval'` / `'wasm-unsafe-eval'` or weaker
`script-src` / `object-src` defaults (`HostError::Csp`).

### Code samples

`browser.*` is always Promise-style; `chrome.*` follows the dialect configured
by the host (MV3 → Promise-style, MV2 → callback-style). Both names exist in
the same script.

Timers (`setTimeout` / `setInterval`, with microtask ordering):

```js
const id = setInterval(() => console.log('tick'), 100);
setTimeout(() => clearInterval(id), 500);
Promise.resolve().then(() => console.log('microtask first'));
```

Alarms on the host-injected `Clock` (tests drive this without sleeping):

```js
chrome.alarms.onAlarm.addListener((alarm) => {
  chrome.storage.local.set({ lastAlarm: alarm.name });
});
chrome.alarms.create('poll', { delayMs: 100, periodMs: 1000 });
// chrome.alarms.clear('poll') cancels it.
```

Storage areas and `storage.onChanged` (the `session` area is RAM-only and is
excluded from the 10 MiB quota):

```js
await chrome.storage.local.set({ mode: 'async' });
await chrome.storage.session.set({ lastMode: 'async' });
chrome.storage.onChanged.addListener((changes, area) => {
  console.log(area, changes.mode.oldValue, '→', changes.mode.newValue);
});
```

`fetch` (permission-gated via `host_permissions` / `<all_urls>`, GET/POST, 2 MiB
cap; rejected as `Promise.reject(TypeError)` for other methods):

```js
const res = await fetch('https://example.com/data', { method: 'GET' });
if (res.ok) console.log(res.status, await res.text());
```

Messaging: async `onMessage` replies and `Port` (`runtime.connect`):

```js
browser.runtime.onMessage.addListener((message) =>
  browser.storage.local.get(['flavor']).then((r) => r.flavor) // first reply wins
);
const port = browser.runtime.connect('popup');
port.onMessage.addListener((msg) => browser.storage.local.set({ portMessage: msg }));
port.postMessage({ hello: 'hodo' });
```

MV2 `chrome.*` callbacks set `runtime.lastError` on failure; a callback-style
API accepts a function as its final argument and reports errors through the
property rather than rejecting a Promise. This is exposed on both the `chrome`
and `browser` objects (`Object.defineProperty(..., 'lastError', ...)`) for MV2
compatibility; Promise-style calls reject instead. See
`extensions/async-chrome/bg.js` and `extensions/promises-gecko/bg.js`.

### i18n (`4aec263`)

`Manifest` now carries `default_locale`, and `hodo-ext-manifest::resolve_messages`
resolves `__MSG_name__` tokens in `name`, `description` and `action_title`
against `_locales/<locale>/messages.json`. Locale selection walks
normalized exact locale → language → `default_locale` → default language;
message text supports `$name$` / `$1`–`$9` placeholders and `@@extension_id`,
`@@ui_locale`, `@@bidi_*` predefines in a single non-recursive pass. Parsing
keeps the raw token in `name` and stores the resolved value in `resolved_name`.
CSS/HTML are not localized, and every locale currently gets LTR bidi values.
Round 2 adds a real `ru` fixture: `extensions/i18n-gecko/_locales/ru/messages.json:1-8`
(`appName` = "Демо i18n", `appDesc` = "Демо-расширение i18n в стиле Gecko для браузера hodo");
`crates/hodo-ext-i18n/src/lib.rs:640-654` loads and resolves both `en` and `ru`
from `extensions/i18n-gecko/`. This is still extension-message i18n, not a
browser-chrome locale bundle.

## MV3 idle lifecycle and state persistence (`d63a90c`)

MV2 background scripts stay persistent. MV3 service workers are unloaded by
`Host::tick()` after `IDLE_UNLOAD_MS = 30_000` ms without events or API calls,
and woken again by `Host::emit_event()` (which reloads the worker from its
script and delivers the event to listeners). `Host::context_is_live()` reports
residency, and `RuntimeConfig::idle_unload_ms` is `Some(30_000)` only for MV3.
Because worker globals vanish on unload, persistent state lives in
`storage.local`/`storage.sync` (the shared 10 MiB quota), not in JS variables;
`storage.session` is RAM-only and is dropped by `ApiRegistry::clear_session()`
when the browser session ends. The injectable `Clock` lets tests advance
timers, alarms and idle time deterministically without sleeping.

## Installing an extension

Drop an unpacked extension directory (one containing `manifest.json`) into the
repo's `extensions/` folder. Ten examples ship with hodo:

- `hello-chrome/` — MV3, Chrome dialect: `background.service_worker`, `action`, `chrome.storage.local` + `chrome.tabs` usage.
- `hello-gecko/` — MV2, Gecko dialect: `background.scripts`, `gecko_id`, `browser_action`, `browser.*` usage.
- `tabs-demo-chrome/` — MV3 tab APIs.
- `storage-demo-gecko/` — MV2 local storage.
- `notify-chrome/` — MV3 notifications with the `notifications` permission.
- `messages-gecko/` — MV2 `runtime.onMessage` style messaging.
- `popup-chrome/` — MV3 `popup.html` with real 16/48px PNG icons.
- `i18n-gecko/` — MV2 `_locales/{en,ru}/messages.json` with `__MSG__` tokens (en resolved by `4aec263`; `ru` fixture added in round 2 — `extensions/i18n-gecko/_locales/ru/messages.json:1-8`).
- `async-chrome/` — MV3 service worker mixing `await chrome.storage.*` with MV2 callbacks and `chrome.alarms.onAlarm` (`d63a90c`).
- `promises-gecko/` — MV2 `browser.*` Promise chains, async `onMessage` reply and a `runtime.connect` `Port` (`d63a90c`).

`Host::load_dir(path)` (**Landed**) reads `manifest.json` from such a directory
(capped at 2 MiB) and validates its CSP. `Host::run_background_scripts(ext_index)`
evaluates the extension's `bg.js` / service worker in a per-extension runtime;
`tick()` / `poll()` drive timers, alarms, events and MV3 idle unload, and
`emit_event()` wakes a parked worker (`d63a90c`).

## Not yet (honest gaps)

- **Shell GUI (M1)** — windowed browser chrome; headless screenshots already work.
- **Popup / toolbar UI** for `action.default_popup` pages.
- **Content scripts** injection or lifecycle.
- **`web_accessible_resources`** handling.
- Out-of-process sandboxing of the extension host is a later milestone (**M3**).
- **Full CSP enforcement for arbitrary extension pages** — only manifest CSP
  validation and script-level `eval` blocking have landed (`d63a90c`).
- **Claimed ext P1×3 bug-cycle fixes** — not audited in round 2: no bug ledger
  and no matching change in `crates/hodo-ext-*` were found, so no "fixed" claim
  is made here. Under-claiming is intentional; re-check when a ledger exists.

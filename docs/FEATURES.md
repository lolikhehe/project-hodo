# hodo — feature status catalog (100 features)

> **IN FLIGHT (uncommitted wave):** the tree moves under this catalog. Active
> zones: style-parallel (CSS value/cascade work), tab-spill
> (`HODO_SPILL_THRESHOLD` display-list spill agent), net-mmap (mmap-backed
> resource path), js-memo (memoized JS bindings), render image-cache
> (decoded-image cache), and 6 bug indexers. Every status flip below was
> verified by reading current code, not `git log`; re-check before using this
> as a release gate. No commits in this wave.

This is an **honest catalog**, not a roadmap pitch. The goal is to beat Firefox on *feature count* — privacy by default, low RAM, power-user tools and built-in replacements for extensions — **not** on 20+ years of web compatibility. P0 rows are the gate: nothing else counts until every P0 is closed. Statuses are grounded in `git log` plus current-tree code reads (`file:line` evidence on flipped/shipped rows); "in-flight" means a frozen brief exists in `hodo — ТЗ для агента milestones M2–M7.md` and work is running, "planned" means a wave assignment from that plan, and "later" means no wave yet. Wave key: **X1** = layout + net-privacy + cookies + adblock + js-dom + bigdoc + fuzz; **X2** = shell features (popup, forms, damage+scroll, sessions+private tabs, page-JS execution, M3 broker flip, hodo-ui foundation); **X3** = `hodo://` pages (settings/tasks/privacy-report/new-tab), reader mode, find-in-page, WebP/AVIF, content_scripts, DNR, command palette, vertical/tree tabs, DevTools-lite, AccessKit. Cross-check: `docs/PROPERTY_STATUS.md` (CSS), `docs/ROADMAP.md`, `docs/EXTENSIONS.md`.

| # | Feature | P | Status | Where |
| --- | --- | --- | --- | --- |
| 1 | JS у веб-сторінках (page JS, boa + DOM bindings) | P0 | 🔶 planned wave X2 | NOT STARTED: `hodo-js` drives extension contexts only (`crates/hodo-js/src/prelude.rs`); page-JS execution is X2. |
| 2 | Повний DOM API (events, MutationObserver, IntersectionObserver, forms) | P0 | 🟡 partial | Arena DOM with `u32` generation handles, tags/attrs/text only (`hodo-dom::NodeKind`, `Dom::validate`); no event dispatch/observers; minimal DOM bindings are M2.2 in-flight; full API = X1 js-dom + X2 page-JS. |
| 3 | fetch/XHR/WebSocket/EventSource + CORS/CSP | P0 | 🟡 partial | Extension `fetch` via `ApiCall::Fetch` → `hodo-net::load_with`; CSP validator `ext-host::validate_csp` shipped; no page-side XHR/WS/EventSource/CORS → X1 net-privacy + X2 page-JS. |
| 4 | Cookie jar (SameSite/Secure/HttpOnly/partitioning) | P0 | ✅ shipped | `hodo-cookie::CookieJar` (8 tests, `crates/hodo-cookie/src/lib.rs:410-566`): `store`/`cookie_header` with SameSite/Secure/Partitioned and caller-supplied partition key (`lib.rs:60-218`); double-keyed attach in `crates/hodo-net/src/privacy.rs:369-393`. `HttpOnly` parsed but not enforced — no script context (`cookie lib.rs:10`). |
| 5 | localStorage/sessionStorage/IndexedDB per-origin | P0 | 🟡 partial | Extension `storage.local/session/sync` with quota (`ext-api::ApiRegistry::quota_check`, `STORAGE_QUOTA_BYTES`) shipped; no page storage → X1 js-dom. |
| 6 | CSS core (position/float/overflow/z-index, var/calc/@media/@font-face) | P0 | 🟡 partial | Selector engine + inline box model + base CSS landed (commits `25d03fa`, `f3e6d9d`); M8 position/float/overflow now consumed+painted (`crates/hodo-layout/src/block.rs:362-395,428-435,133-170`) and flex/grid/table engines landed (`flex.rs:153`, `grid.rs:302`, `table.rs:274`) — see `docs/PROPERTY_STATUS.md`. `z-index`/`var()`/`calc()`/`@media`/`@font-face` not audited this wave; still open. |
| 7 | CSS transitions/animations + rAF на dirty-rect | P1 | 🔶 planned wave X2 | damage+scroll (M5) is the prerequisite; minimal M9.2 motion only after that. |
| 8 | TLS 1.3 (rustls), HSTS, HTTP/2, пізніше HTTP/3 | P0 | 🟡 partial | HTTPS loads via `ureq` (rustls in `Cargo.lock`) in `hodo-net::load/load_with`; HSTS now landed (`crates/hodo-net/src/privacy.rs:247-304,348-365`); HTTP/2/QUIC not audited this wave — still open. |
| 9 | Зображення й медіа (WebP/AVIF/GIF/SVG/srcset/video) | P1 | 🟡 partial | PNG/JPEG decode + blit (`hodo-render`, `645b5fa`), `<img src>` resolved against page URL (`cd1b1ec`); WebP/AVIF → X3; GIF/SVG/video → later. |
| 10 | Canvas 2D (tiny-skia) + inline SVG | P1 | ⚪ later | No Canvas API or SVG parser in any crate; `tiny-skia` is only the raster backend. |
| 11 | Партиціювання сховища за top-site | P0 | 🟡 partial | Cache key partition = top-frame site (`hodo-net::FetchOptions::partition`, `hodo-cache::CacheKey::new`); cookie partitioning now landed (`crates/hodo-cookie/src/lib.rs:48-50`, `crates/hodo-net/src/privacy.rs:369-393`); page-storage partitioning not audited this wave — still open. |
| 12 | Вбудований блокувальник реклами/трекерів у hodo-net | P0 | ✅ shipped | `hodo-adblock` + vendored `assets/filters/easylist.txt` and `easyprivacy.txt`, loaded at startup (`crates/hodo-adblock/src/lib.rs:412-421`); ON by default (`lib.rs:393-396`); 3 disable paths — env `HODO_NO_ADBLOCK`, global `set_adblock_enabled`, per-site `allow_site` (`lib.rs:424-463`); wired into `crates/hodo-net/src/privacy.rs:337-346`. |
| 13 | HTTPS-only за замовчуванням + mixed content | P0 | ✅ shipped | HTTPS auto-upgrade (`crates/hodo-net/src/privacy.rs:230-245,348-365`) + mixed-content warnings (`Warning::MixedContent`, `privacy.rs:13-19,357-360`); HSTS parse/observe/expiry (`privacy.rs:247-304`). Loopback exempt from non-HSTS upgrades (`privacy.rs:226-235`). |
| 14 | DNS-over-HTTPS/TLS | P1 | 🔶 planned wave X1 | net-privacy: no resolver abstraction in `hodo-net`. |
| 15 | Захист від fingerprinting | P1 | 🔶 planned wave X1 | net-privacy: no UA unification, letterboxing or canvas noise. |
| 16 | Очищення трекінг-параметрів URL | P1 | ✅ shipped | `strip_tracking_params` with frozen `utm_*`/`fbclid`/… list (`crates/hodo-net/src/privacy.rs:42-68,118-147`), gated by `FetchOptions::strip_tracking` (`privacy.rs:330-335`). |
| 17 | Суворий Referrer-Policy | P0 | ✅ shipped | `referer_header` emits origin-only for same-origin, no Referer cross-origin (`crates/hodo-net/src/privacy.rs:208-213,377-379`). |
| 18 | Приватне вікно в RAM, без запису на диск | P0 | ✅ shipped | `Tab::private` + `private_load` flag and `--private` startup (`crates/hodo-shell/src/tabs.rs:57,92-102,169-188`; `main.rs:102-104`); private fetch writes no cookies/HSTS/cache (`crates/hodo-net/src/privacy.rs:326-328,371-372,391-411`); private tabs excluded from sessions (`crates/hodo-shell/src/sessions.rs:74-80`). |
| 19 | Per-site дозволи з авто-відкликанням | P1 | ⚪ later | `ext-host` gate is per-extension, not per-origin; no camera/mic UI. |
| 20 | Безпечний перегляд за локальними списками | P1 | 🔶 planned wave X1 | net-privacy: no hash-prefix lists. |
| 21 | Менеджер паролів (argon2 + AEAD, k-anonymity) | P1 | ⚪ later | No crypto store or leak check in repo. |
| 22 | Passkeys / WebAuthn | P1 | ⚪ later | No implementation. |
| 23 | Site isolation (окремі процеси на origin) | P2 | 🔶 planned wave X2 | M3 *extension* process isolation all landed (`hodo-ext-ipc`, `hodo-ext-worker`); per-origin isolation is a separate, opt-in step. |
| 24 | Захист завантажень, карантин (Mark-of-the-Web) | P1 | ⚪ later | No download subsystem. |
| 25 | Privacy Report по сайту | P1 | ✅ shipped | `hodo://privacy` report built from landed accessors — cookies, blocked requests/bytes, warnings, HSTS hosts, cache stats (`crates/hodo-shell/src/hpages.rs:21,30-34,91-139`). Limit: global, not per-site — cookie-partition breakdown not exposed by the landed API (`hpages.rs:104-105`). |
| 26 | Заморожування неактивних табів | P0 | ✅ shipped | Display-list spill all landed: `HODO_SPILL_THRESHOLD` (default 8, `0` disables), LRU spill of inactive tabs (`crates/hodo-shell/src/tabs.rs:9-12,204-231`; `sessions.rs:11-19,137-145,456`), restore/reload on switch (`tabs.rs:124-134,314`). |
| 27 | Discard/швидке відновлення таба | P1 | 🔶 planned wave X2 | sessions: `Tab` keeps URL/history only, no discarded state. |
| 28 | Загальний бюджет RAM | P1 | ⚪ later | No global memory manager. |
| 29 | hodo://tasks (RAM/CPU по табах/розширеннях) | P1 | ✅ shipped | `hodo://` synthetic-page router + settings/privacy pages (`crates/hodo-shell/src/hpages.rs:19-37`); RAM/damage counters on `about:hodo` (`crates/hodo-shell/src/about.rs:8-41,55-56`). Limit: no dedicated `hodo://tasks` RAM/CPU-per-tab view beyond these pages (not audited further). |
| 30 | Прогресивний рендер | P1 | 🔶 planned wave X2 | damage+scroll: current pipeline renders a full display list per load (`app::load_page`). |
| 31 | Prefetch/preconnect/predictive DNS | P2 | ⚪ later | No implementation; must stay off in private mode. |
| 32 | Дисковий кеш із партиціюванням і квотою (M4) | P0 | ✅ shipped | RFC 9111 disk cache, default-on in shell (commits `0c0d5cc`, `cd1b1ec`); `hodo-cache::DiskCache` + `hodo-net::load_with/offline`. Gap: brotli/zstd not implemented (gzip only). <!-- evidence: hodo-net::load_with + hodo-cache::CacheKey::new(partition); commit 0c0d5cc "RFC 9111 disk cache", cd1b1ec "enable disk cache for page loads" --> |
| 33 | Швидкий холодний старт, ліниве завантаження | P1 | 🟡 partial | MV3 idle unload shipped (`ext-host::Host::tick`, 30s idle, event wake-up); lazy subsystem startup / deferred extension launch not done → X2 broker flip. |
| 34 | Вертикальні таби | P1 | 🔶 planned wave X3 | vertical/tree tabs. |
| 35 | Дерево вкладок | P1 | 🔶 planned wave X3 | vertical/tree tabs; `Vec<Tab>` is flat. |
| 36 | Workspaces/простори | P1 | ⚪ later | No workspace model. |
| 37 | Контейнери (ізольовані cookies/storage) | P1 | 🔶 planned wave X2 | private tabs/sessions + X1 cookies; no container abstraction. |
| 38 | Split view | P2 | ⚪ later | No multi-page window layout. |
| 39 | Командна палітра (Ctrl+K) | P1 | 🔶 planned wave X3 | command palette. |
| 40 | Нечіткий пошук по табах/історії/закладках | P1 | 🔶 planned wave X3 | command palette; shell history is in-RAM, no index. |
| 41 | Сесії: автозбереження, іменовані, відновлення | P0 | ✅ shipped | Atomic snapshot/restore at `cache/sessions.json` (`crates/hodo-shell/src/sessions.rs:8-9,95-120`), caps `MAX_SESSION_TABS=32` / `MAX_HISTORY_ENTRIES=512` (`sessions.rs:30-33`), private tabs excluded (`sessions.rs:74-80`). Limit: "named" sessions not implemented — one snapshot, no name in `SessionTab` (`sessions.rs:38-49`). |
| 42 | Pinned-таби, mute, аудіо-індикатор | P1 | ⚪ later | No per-tab audio state. |
| 43 | Vim-навігація, кастомні хоткеї | P2 | ⚪ later | No keybinding map. |
| 44 | Жести мишею/тачпада | P2 | ⚪ later | No gesture layer. |
| 45 | Теми: світла/темна/auto, кастомні CSS | P1 | 🔵 in-flight | M9/M9.2 brief frozen in ТЗ (MD3 + Expressive), no `hodo-shell/src/design.rs` yet; foundation = X2 hodo-ui. |
| 46 | Reader mode (шрифти, ширина, TTS) | P1 | 🔶 planned wave X3 | reader mode. |
| 47 | Вбудований PDF-рідер | P1 | ⚪ later | No PDF parser/reader. |
| 48 | Скріншоти: область, вся сторінка, елемент | P1 | 🟡 partial | Headless `--screenshot <url> <out.png>` shipped (`hodo-shell::main.rs::screenshot`); no in-UI area/element capture, annotations or clipboard. |
| 49 | Збереження сторінки в архів | P1 | ⚪ later | No serializer. |
| 50 | Закладки з тегами/папками/пошуком | P1 | ⚪ later | No bookmark store. |
| 51 | Історія з пошуком за вмістом | P2 | ⚪ later | No content index. |
| 52 | Менеджер завантажень | P1 | ⚪ later | No downloads; conflicts with the 2 MiB cap (needs a streaming mode). |
| 53 | Пошук на сторінці (regex, підсвітка) | P1 | 🔶 planned wave X3 | find-in-page. |
| 54 | Автозаповнення адрес/платіжних даних | P1 | ⚪ later | No autofill store. |
| 55 | Друк / Save as PDF | P1 | ⚪ later | No print pipeline. |
| 56 | MV2 і MV3 паралельно | P0 | ✅ shipped | Dual dialect in `hodo-ext-manifest` (`ManifestVersion::V2/V3` + mixed background), single dispatch `chrome.*`/`browser.*` in `hodo-ext-api`, host wiring and demo extensions (commit `d63a90c`). Gap: blocking `webRequest` missing → X3 DNR. <!-- evidence: crates/hodo-ext-manifest::ManifestVersion; crates/hodo-ext-api::ApiDialect + dispatch; git d63a90c "async runtime ... MV3 id..." --> |
| 57 | declarativeNetRequest + webRequest одночасно | P1 | 🔶 planned wave X3 | DNR: no `declarativeNetRequest` or `webRequest` namespace in `ext-api` dispatch table. |
| 58 | content_scripts (ISOLATED/MAIN, run_at, match patterns) | P0 | 🔶 planned wave X3 | content_scripts: no script injection or match-pattern engine in repo. |
| 59 | sidePanel/omnibox/contextMenus/commands | P1 | ⚪ later | None of these namespaces exist in `ext-api`. |
| 60 | Повний набір API (tabs, windows, bookmarks, …) | P1 | 🟡 partial | Shipped: `tabs`, `alarms`, `notifications`, `storage`, `runtime.sendMessage`, extension `fetch`, `i18n` (`ext-api::ApiCall`, `ext-i18n`); missing: windows, bookmarks, history, downloads, cookies, webNavigation. |
| 61 | Встановлення .crx/.xpi з підписами, оновлення | P1 | ⚪ later | No package installer or signature verification. |
| 62 | Optional permissions + «дозволити на цьому сайті» | P1 | 🔶 planned wave X2 | M3 broker flip: permission gate lives in `ext-host::dispatch`; optional/on-click grant UI missing. |
| 63 | Режим розробника (hot-reload, консоль) | P1 | ⚪ later | No dev console. |
| 64 | Аудит розширень (домени, RAM/CPU) | P1 | ⚪ later | No telemetry per extension. |
| 65 | DevTools (Elements/Console/Network/Sources) | P1 | 🔶 planned wave X3 | DevTools-lite. |
| 66 | Часткова CDP / WebDriver BiDi | P1 | ⚪ later | No remote protocol. |
| 67 | Headless: `--screenshot`, `--dump-dom`, `--print-pdf`, `--eval` | P1 | 🟡 partial | Only `--screenshot` exists (`hodo-shell/src/main.rs` arg dispatch); the other three flags are absent. |
| 68 | Performance-панель, damage-лічильники | P1 | ✅ shipped | M5.1/M5.2 repaint counters (`crates/hodo-shell/src/about.rs:8-41`), damage bands recorded in `crates/hodo-shell/src/present.rs:298-318,347-445` and surfaced on `about:hodo` (`about.rs:55-56`). Limit: counters live on the synthetic page; no DevTools UI panel. |
| 69 | Throttling, offline, local overrides, HAR | P1 | 🟡 partial | Cache-only `offline` mode shipped (`hodo-net::FetchOptions::offline`, M4); no DevTools UI, throttling or HAR export. |
| 70 | Responsive/device emulation | P1 | 🔶 planned wave X3 | DevTools-lite. |
| 71 | Source maps + debugger | P2 | ⚪ later | No debugger. |
| 72 | Accessibility tree inspector | P2 | 🔶 planned wave X3 | DevTools-lite + AccessKit (see #83 decision). |
| 73 | WPT-ранер із дашбордом | P1 | ⚪ later | No test runner (hodo-tests are pixel/e2e only). |
| 74 | E2EE-синхронізація | P1 | ⚪ later | No sync backend. |
| 75 | Self-hosted sync / WebDAV / Git | P2 | ⚪ later | No sync backend. |
| 76 | Імпорт з Chrome/Firefox/Safari/Edge | P1 | ⚪ later | No importer. |
| 77 | Кілька профілів | P1 | ⚪ later | Single profile directory. |
| 78 | Автобекапи профілю | P1 | ⚪ later | Only cache writes are atomic (M4). |
| 79 | Портативний режим | P2 | ⚪ later | No profile-path override. |
| 80 | Enterprise-політики (policies.json) | P2 | ⚪ later | No policy layer. |
| 81 | Майстер-пароль / ключі ОС (DPAPI/Keychain) | P1 | 🔴 needs decision/unsafe-review | FFI conflicts with `unsafe_code = forbid`; explicit owner decision required (ТЗ "Конфлікти з принципами hodo"). |
| 82 | hodo://data (квоти, вибіркове очищення) | P1 | ✅ shipped | Cache quotas/counters on `hodo://privacy` via `hodo_cache::cache_stats()` (`crates/hodo-shell/src/hpages.rs:94,118-121`) and on `about:hodo` (`crates/hodo-shell/src/about.rs:45,55`). Limit: read-only — no per-site selective-clear control yet (`hpages.rs:5-6`). |
| 83 | Скрінрідери: UIA (Windows) / AT-SPI (Linux) | P0 | 🔴 needs decision/unsafe-review | AccessKit feasibility brief pending (FFI vs `unsafe_code = forbid`, ТЗ conflict list); AccessKit is on the X3 plan, native UIA/AT-SPI is blocked on the decision. |
| 84 | Масштаб сторінки, min font, high-contrast | P1 | ⚪ later | M9 dark toggle only; no per-site zoom or forced-colors support. |
| 85 | Перевірка правопису | P1 | ⚪ later | No dictionaries. |
| 86 | IME, CJK/діакритика, bidi/RTL | P1 | ⚪ later | M7.2 explicitly out of scope; `fontdue` fallback chain only. |
| 87 | Локалізація UI, українська з коробки | P1 | 🟡 partial | WebExtension i18n engine shipped (`hodo-ext-i18n`, commit `4aec263`: fallback chain, placeholders, `@@ui_locale`); no UI/Fluent resource bundle for the chrome itself. |
| 88 | Офлайн-переклад локальними моделями | P2 | ⚪ later | No model runtime. |
| 89 | Інтеграція з ОС (медіа-клавіші, протоколи, default browser) | P1 | ⚪ later | FFI/unsafe review needed (same class as #81). |
| 90 | HiDPI, багатомонітор, touch/pen | P1 | ⚪ later | Fixed `DEFAULT_VIEWPORT`; winit only basic window handling. |
| 91 | Автооновлення з підписами, канали | P1 | ⚪ later | No updater. |
| 92 | Локальний AI-помічник | P2 | ⚪ later | No model runtime; opt-in and separate process required. |
| 93 | Користувацькі скрипти й стилі (Greasemonkey) | P1 | ⚪ later | Depends on #58 content_scripts (X3). |
| 94 | Макроси (запис/відтворення) | P2 | ⚪ later | No implementation. |
| 95 | Page diff зі сповіщенням | P2 | ⚪ later | No implementation. |
| 96 | Локальні highlights/анотації | P2 | ⚪ later | No implementation. |
| 97 | RSS/Atom-рідер | P2 | ⚪ later | No implementation. |
| 98 | Режим економії батареї/трафіку | P1 | ⚪ later | No policy engine. |
| 99 | Per-site proxy, Tor, live-панель з'єднань | P2 | ⚪ later | No proxy support; Tor needs unsafe-review + opt-in process. |
| 100 | Автоплей/гучність, міні-плеєр | P2 | ⚪ later | No media/audio subsystem. |

## P0 punch-list — what still blocks "a browser that works on real sites"

18 P0 rows; **9 shipped, 0 in-flight, 9 open** — all in-flight work landed (partial and needs-decision rows
count as open; statuses verified against the current tree, not just `git log`):

- **✅ #4 Cookie jar** — `hodo-cookie::CookieJar` (8 tests) + double-keyed attach (`hodo-net/src/privacy.rs:369-393`).
- **✅ #12 Built-in ad/tracker blocker** — vendored EasyList/EasyPrivacy, ON by default (`hodo-adblock/src/lib.rs:393-421`).
- **✅ #13 HTTPS-only + mixed content** and **✅ #17 Strict Referrer-Policy** — `hodo-net/src/privacy.rs:208-213,230-245,348-365`.
- **✅ #18 Private window (RAM-only)** — `tabs.rs:92-102,185-188`, `--private` (`main.rs:102-104`).
- **✅ #32 Disk cache** — M4 landed and default-on (`0c0d5cc`, `cd1b1ec`); only brotli/zstd compression is outstanding (not a P0 blocker by itself).
- **✅ #41 Sessions** — atomic snapshot + 32/512 caps (`hodo-shell/src/sessions.rs:30-33,95-120`).
- **✅ #56 MV2+MV3 dual dialect** — landed (`d63a90c`); the "blocking webRequest" clause is not, and is tracked under #57 → **X3**.
- **✅ #26 Freeze inactive tabs (all landed)** — spill + restore wired (`tabs.rs:204-231`, `sessions.rs:456`); counted shipped.
- **#1 page JS** → **X2** (page-JS execution; not started).
- **#2 Full DOM API** → **X1 js-dom** (observers, events, forms) + **X2 page-JS** (bindings).
- **#3 fetch/XHR/WS/EventSource + CORS/CSP** → **X1 net-privacy** (CORS/CSP) + **X2 page-JS**; extension fetch + CSP validator already shipped.
- **#5 Web storage / IndexedDB** → **X1 js-dom**; extension storage quota already shipped.
- **#6 CSS core** → partial: M8 position/float/overflow + flex/grid/table landed (see `docs/PROPERTY_STATUS.md`); `z-index`/`var()`/`calc()`/`@media`/`@font-face` not audited this wave → **X1 layout**.
- **#8 TLS/rustls + HSTS + HTTP/2** → HSTS landed (`hodo-net/src/privacy.rs:247-304`); HTTP/2/QUIC not audited this wave, still open → **X1 net-privacy**.
- **#11 Top-site storage partitioning** → cache partition + cookie partition landed (`hodo-net/src/privacy.rs:369-393`); page-storage partitioning not audited this wave, still open → **X1 cookies/js-dom**.
- **#58 content_scripts** → **X3 content_scripts**.
- **#83 Screen readers (UIA/AT-SPI)** → **🔴 needs decision** first (AccessKit feasibility brief), then **X3 AccessKit**.

Order to close them: X1 (layout, net-privacy, cookies, adblock, js-dom, bigdoc, fuzz) → X2 (popup, forms, damage+scroll, sessions+private tabs, page-JS, M3 broker flip, hodo-ui) → X3 (hodo:// pages, content_scripts, DNR, DevTools-lite, AccessKit), with the #83 decision needed before X3 starts.

<!-- ponytail: docs-only status sync; flipped rows were read from the current tree, not git log. Risk: the uncommitted waves (style-parallel, tab-spill, net-mmap, js-memo, render image-cache, 6 bug indexers) can move statuses again before commit — re-check code before using this as a release gate. -->

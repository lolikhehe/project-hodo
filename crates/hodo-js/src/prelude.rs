//! JavaScript prelude: thin dual-style wrappers over one native dispatch table.

use hodo_ext_api::{canonical_namespace, ApiDialect, RuntimeConfig};

/// Per-runtime, per-tick PURE-read memo. Injected into both preludes; the Rust
/// side owns the PURE list, the cap and the epoch that invalidates it.
const MEMO_JS: &str = r#"
var __hodoMemoEpoch = 1;
var __hodoMemo = { entries: {}, order: [], epoch: __hodoMemoEpoch };
var __hodoMemoMiss = {};
var __hodoHashCache = {};
var __hodoHashCount = 0;
var __hodoKeyCache = {};
var __hodoKeyCount = 0;
function __hodoMemoCanon(value) {
  if (value === null || typeof value !== 'object') {
    var text = JSON.stringify(value);
    return text === undefined ? 'undefined' : text;
  }
  if (Array.isArray(value)) {
    var parts = [];
    for (var i = 0; i < value.length; i++) parts.push(__hodoMemoCanon(value[i]));
    return '[' + parts.join(',') + ']';
  }
  var keys = Object.keys(value).sort();
  var fields = [];
  for (var i = 0; i < keys.length; i++) {
    fields.push(JSON.stringify(keys[i]) + ':' + __hodoMemoCanon(value[keys[i]]));
  }
  return '{' + fields.join(',') + '}';
}
function __hodoMemoHash(text) {
  var cached = __hodoHashCache[text];
  if (cached !== undefined) return cached;
  var hash = 0x811c9dc5;
  var prime = 0x01000193;
  for (var i = 0; i < text.length; i++) {
    hash ^= text.charCodeAt(i);
    // FNV-1a multiply without Math.imul (a native call per character in boa):
    // split into 16-bit halves so every intermediate stays below 2^53.
    var lo = hash & 0xffff;
    var hi = hash >>> 16;
    hash = (lo * prime + (((hi * prime) & 0xffff) << 16)) >>> 0;
  }
  if (__hodoHashCount >= 256) {
    __hodoHashCache = {};
    __hodoHashCount = 0;
  }
  __hodoHashCache[text] = hash;
  __hodoHashCount++;
  return hash >>> 0;
}
function __hodoMemoKey(name, args) {
  // JSON.stringify is canonical for arrays of primitives (the common case);
  // object params go through the sorted-key canonicalizer.
  var simple = Array.isArray(args);
  if (simple) {
    for (var i = 0; i < args.length; i++) {
      var value = args[i];
      if (value !== null && typeof value === 'object') { simple = false; break; }
    }
  }
  var text = simple ? JSON.stringify(args) : __hodoMemoCanon(args);
  return name + '|' + __hodoMemoHash(text === undefined ? 'undefined' : text);
}
function __hodoMemoSync() {
  if (__hodoMemo.epoch === __hodoMemoEpoch) return;
  __hodoMemo.entries = {};
  __hodoMemo.order = [];
  __hodoMemo.epoch = __hodoMemoEpoch;
}
function __hodoMemoKeyFast(name, args) {
  if (args && args.length === 1 && typeof args[0] === 'string') {
    var short = name + '|' + args[0];
    var known = __hodoKeyCache[short];
    if (known !== undefined) return known;
    var key = __hodoMemoKey(name, args);
    if (__hodoKeyCount >= 256) {
      __hodoKeyCache = {};
      __hodoKeyCount = 0;
    }
    __hodoKeyCache[short] = key;
    __hodoKeyCount++;
    return key;
  }
  return __hodoMemoKey(name, args);
}
function __hodoMemoGet(key) {
  __hodoMemoSync();
  return __hodoMemo.entries[key] || __hodoMemoMiss;
}
function __hodoMemoPut(key, value) {
  __hodoMemoSync();
  if (__hodoMemo.entries[key]) return;
  if (__hodoMemo.order.length >= __HODO_MEMO_CAP__) {
    delete __hodoMemo.entries[__hodoMemo.order.shift()];
  }
  // One resolved Promise per entry: repeated pure reads can share it because
  // `.then` handlers are independent.
  __hodoMemo.entries[key] = { value: value, promise: Promise.resolve(value) };
  __hodoMemo.order.push(key);
}
var __hodoPure = __HODO_PURE__;
"#;

// ESM graph glue. The shell owns the graph: it emits, per module and in
// topological order, `__hodoModuleBaseUrl=<U>;` followed by
// `__hodoRunModule(<U>, [<depU>...], function(__mods){ ...; return {...}; })`.
// Deps are absolute URLs in first-mention order, so by the time a factory runs
// every dep's exports are already in the registry. hodo-js only owns the
// registry, the factory call, and the dynamic-import promise glue.
const MODULE_JS: &str = r#"
// URL -> export namespace. Null prototype so a URL like `__proto__` can never
// alias Object.prototype; ids are strings, not property-name surprises.
var __hodoModules = Object.create(null);
// Set by the emitted script immediately before each __hodoRunModule call.
var __hodoModuleBaseUrl = '';
function __hodoRunModule(url, deps, factory) {
  var existing = __hodoModules[url];
  if (existing !== undefined) return existing;
  var mods = [];
  for (var i = 0; i < deps.length; i++) {
    var dep = __hodoModules[deps[i]];
    if (dep === undefined) {
      throw new Error('module ' + url + ': missing dependency ' + deps[i]);
    }
    mods.push(dep);
  }
  // The shell emits `__hodoModuleBaseUrl=<U>;` before each call; set it here
  // too so a factory's own synchronous import() resolves against its URL even
  // if the host has not emitted that line yet.
  __hodoModuleBaseUrl = url;
  // The factory runs as a plain call under the strict graph-eval path, so
  // `this === undefined` and undeclared assignments are ReferenceErrors.
  var raw = factory(mods);
  // Snapshot own enumerable exports. Live bindings are intentionally NOT
  // modelled in wave 1: later mutation of `raw` cannot reach a registered
  // consumer, matching the frozen "snapshot the returned object" contract.
  var snapshot = Object.create(null);
  if (raw !== null && raw !== undefined) {
    var keys = Object.keys(raw);
    for (var k = 0; k < keys.length; k++) snapshot[keys[k]] = raw[keys[k]];
  }
  __hodoModules[url] = snapshot;
  return snapshot;
}
// Dynamic import. The shell rewrites `import(spec)` to `__hodoDynImport(spec)`
// (boa has no native import syntax). Pending id -> { resolve, reject }.
var __hodoDynPending = Object.create(null);
var __hodoDynNextId = 1;
// Wave-1 string resolution: absolute URLs pass through; relative specs are
// joined to the current module's directory and `.`/`..` segments collapsed.
// No percent-encoding, punycode or default-port canonicalisation.
function __hodoResolveModuleSpec(spec, base) {
  var text = String(spec);
  if (/^[A-Za-z][A-Za-z0-9+.-]*:/.test(text)) return text;
  var rest = String(base || '');
  var prefix = '';
  var origin = /^([A-Za-z][A-Za-z0-9+.-]*:\/\/[^/]*)(\/.*)?$/.exec(rest);
  if (origin) { prefix = origin[1]; rest = origin[2] || '/'; }
  else { prefix = ''; }
  rest = rest.replace(/[?#].*$/, '');
  var dir = rest.slice(0, rest.lastIndexOf('/') + 1);
  var joined = text.charAt(0) === '/' ? text : dir + text;
  var parts = [];
  var segs = joined.split('/');
  for (var i = 0; i < segs.length; i++) {
    var seg = segs[i];
    if (seg === '' || seg === '.') continue;
    if (seg === '..') { parts.pop(); continue; }
    parts.push(seg);
  }
  return prefix + '/' + parts.join('/');
}
function __hodoDynImport(spec) {
  var id = __hodoDynNextId++;
  // The shell normally assigns `__hodoModuleBaseUrl` right before each module
  // factory. Fall back to the page URL so a bare import() still resolves with
  // an authority if the host has not emitted that line yet.
  var base = __hodoModuleBaseUrl ||
    ((typeof window !== 'undefined' && window.location) ? window.location.href : '');
  var resolved = __hodoResolveModuleSpec(spec, base);
  return new Promise(function (resolve, reject) {
    __hodoDynPending[id] = { resolve: resolve, reject: reject };
    // A throwing native must not strand a permanently pending promise.
    try {
      __hodo_load_dynamic_module(resolved, base, id);
    } catch (err) {
      var entry = __hodoDynPending[id];
      if (entry) { delete __hodoDynPending[id]; reject(err); }
    }
  });
}
// Shell-invoked settle callbacks. A late callback for an unknown or already
// settled id is a documented no-op: first settle wins, ids are never reused.
function __hodoDynResolved(id, exportsObj) {
  var entry = __hodoDynPending[id];
  if (!entry) return;
  delete __hodoDynPending[id];
  try { entry.resolve(exportsObj); } catch (e) { __hodoReportError(e); }
}
function __hodoDynRejected(id, msg) {
  var entry = __hodoDynPending[id];
  if (!entry) return;
  delete __hodoDynPending[id];
  try { entry.reject(new Error(String(msg))); } catch (e) { __hodoReportError(e); }
}
"#;

fn with_memo_config(js: &str) -> String {
    let pure = crate::PURE_METHODS
        .iter()
        .map(|name| format!("'{name}':1"))
        .collect::<Vec<_>>()
        .join(",");
    js.replace("__HODO_MEMO__", MEMO_JS)
        .replace("__HODO_MODULES__", MODULE_JS)
        .replace("__HODO_PURE__", &format!("{{{pure}}}"))
        .replace("__HODO_MEMO_CAP__", &crate::MEMO_CAP.to_string())
}

const HELPERS: &str = r#"
var console = {
  log: function () {}, info: function () {}, warn: function () {},
  error: function () {}, debug: function () {}
};
function __hodoReportError(e) {
  try { __hodo_log(String(e && (e.stack || e.message) || e)); } catch (_) {}
}
function __hodoFreezeDeep(o, seen) {
  if (!o || typeof o !== 'object' || Object.isFrozen(o)) return o;
  seen = seen || [];
  if (seen.indexOf(o) !== -1) return o;
  seen.push(o);
  var keys = Object.keys(o);
  for (var i = 0; i < keys.length; i++) __hodoFreezeDeep(o[keys[i]], seen);
  return Object.freeze(o);
}
function __hodoArgs(a) { return Array.prototype.slice.call(a); }
__HODO_MEMO__
__HODO_MODULES__
var __hodoState = { pending: {}, listeners: {}, lastError: null, nextId: 1 };
function __hodoPost(name, args) {
  return __hodo_post_task(name, args);
}
function __hodoCall(name, args, callback, callbackOnly) {
  var cb = (typeof callback === 'function') ? callback : null;
  var memoKey = __hodoPure[name] ? __hodoMemoKeyFast(name, args) : null;
  var entry = memoKey ? __hodoMemoGet(memoKey) : __hodoMemoMiss;
  if (memoKey && entry === __hodoMemoMiss) {
    var direct = __hodo_read_pure(name, args);
    if (direct !== undefined) {
      __hodoMemoPut(memoKey, direct);
      entry = __hodoMemoGet(memoKey);
    }
  }
  if (entry !== __hodoMemoMiss) {
    __hodoState.lastError = null;
    if (cb) entry.promise.then(function (v) { try { cb(v); } catch (e) { __hodoReportError(e); } });
    return callbackOnly ? undefined : entry.promise;
  }
  var id = __hodoPost(name, args);
  var result;
  if (callbackOnly) {
    __hodoState.pending[id] = { cb: cb, memoKey: memoKey };
  } else {
    result = new Promise(function (resolve, reject) {
      __hodoState.pending[id] = { resolve: resolve, reject: reject, cb: cb, memoKey: memoKey };
    });
  }
  return result;
}
function __hodoSettle(id, ok, value, error) {
  var entry = __hodoState.pending[id];
  if (!entry) return;
  delete __hodoState.pending[id];
  if (!ok) {
    var err = new Error(error);
    if (entry.cb) {
      __hodoState.lastError = err;
      try { entry.cb(undefined); } catch (e) { __hodoReportError(e); }
    } else if (entry.reject) {
      entry.reject(err);
    }
    return;
  }
  __hodoState.lastError = null;
  if (entry.memoKey) __hodoMemoPut(entry.memoKey, value);
  if (entry.cb) {
    try { entry.cb(value); } catch (e) { __hodoReportError(e); }
  }
  if (entry.resolve) entry.resolve(value);
}
function __hodoAddListener(name, fn) {
  if (typeof fn !== 'function') return;
  if (!__hodoState.listeners[name]) __hodoState.listeners[name] = [];
  __hodoState.listeners[name].push(fn);
}
function __hodoRemoveListener(name, fn) {
  var list = __hodoState.listeners[name] || [];
  var i = list.indexOf(fn);
  if (i !== -1) list.splice(i, 1);
}
function __hodoEvent(name) {
  return {
    addListener: function (fn) { __hodoAddListener(name, fn); },
    removeListener: function (fn) { __hodoRemoveListener(name, fn); },
    hasListener: function (fn) { return (__hodoState.listeners[name] || []).indexOf(fn) !== -1; }
  };
}
function __hodoFire(name, a, b, c) {
  var list = (__hodoState.listeners[name] || []).slice();
  for (var i = 0; i < list.length; i++) {
    try { list[i](a, b, c); } catch (e) { __hodoReportError(e); }
  }
}
function __hodoFireMessage(message) {
  var list = (__hodoState.listeners['runtime.onMessage'] || []).slice();
  // Listener-aware delivery: no listener means no work and no reply to route.
  if (!list.length) return false;
  function sendResponse(value) {
    var result = value === undefined ? null : value;
    // Real reply path: hand the resolved response to the host through the
    // native reply queue. The host owns correlation (ExtEvent/ApiResponse).
    try { __hodo_message_reply(message, result); }
    catch (e) { __hodoReportError(e); }
    return true;
  }
  for (var i = 0; i < list.length; i++) {
    try { list[i](message, {}, sendResponse); } catch (e) { __hodoReportError(e); }
  }
  return true;
}
function __hodoApiCall(style, name, args) {
  var cb = (args.length && typeof args[args.length - 1] === 'function') ? args[args.length - 1] : null;
  var rest = cb ? args.slice(0, -1) : args;
  if (style === 'browser') return __hodoCall(name, rest, cb, false);
  if (style === 'chrome-cb') return __hodoCall(name, rest, cb, true);
  return __hodoCall(name, rest, cb, cb !== null);
}
function setTimeout(fn, delay) {
  if (typeof fn !== 'function') throw new TypeError('setTimeout expects a function');
  var ms = Number(delay);
  if (!isFinite(ms) || ms < 1) ms = 1;
  var rest = [];
  for (var i = 2; i < arguments.length; i++) rest.push(arguments[i]);
  return __hodo_set_timeout(fn, ms, rest);
}
function setInterval(fn, delay) {
  if (typeof fn !== 'function') throw new TypeError('setInterval expects a function');
  var ms = Number(delay);
  if (!isFinite(ms) || ms < 1) ms = 1;
  var rest = [];
  for (var i = 2; i < arguments.length; i++) rest.push(arguments[i]);
  return __hodo_set_interval(fn, ms, rest);
}
function clearTimeout(id) { __hodo_clear_timer(id); }
function clearInterval(id) { __hodo_clear_timer(id); }
function __hodoPort(name) {
  var port = {
    name: name || '',
    postMessage: function (message) { __hodoPost('runtime.portMessage', [name, message]); },
    disconnect: function () { __hodoFire('__portDisconnect:' + name); },
    onMessage: __hodoEvent('__portMessage:' + name),
    onDisconnect: __hodoEvent('__portDisconnect:' + name)
  };
  __hodoFire('runtime.onConnect', port);
  return Object.freeze(port);
}
function __hodoSendMessage(message, callback) {
  var resolveFn = null, rejectFn = null, timer = null, responded = false;
  function respond(value) {
    if (responded) return true;
    responded = true;
    if (timer !== null) clearTimeout(timer);
    var result = value === undefined ? null : value;
    if (callback) {
      try { callback(result); } catch (e) { __hodoReportError(e); }
    } else if (resolveFn) {
      resolveFn(result);
    }
    return true;
  }
  return new Promise(function (resolve, reject) {
    resolveFn = resolve;
    rejectFn = reject;
    var waitingAsync = false;
    var listeners = (__hodoState.listeners['runtime.onMessage'] || []).slice();
    for (var i = 0; i < listeners.length; i++) {
      try {
        var reply = listeners[i](message, {}, respond);
        if (reply === true) {
          waitingAsync = true;
        } else if (reply && typeof reply.then === 'function') {
          waitingAsync = true;
          reply.then(respond, reject);
        } else if (reply !== undefined) {
          respond(reply);
        }
      } catch (e) {
        reject(e);
        return;
      }
    }
    if (!responded) {
      timer = setTimeout(function () { respond(null); }, 1000);
      if (!waitingAsync) {
        __hodoCall('runtime.sendMessage', [message], null, false).then(respond, function (e) {
          if (!responded) {
            responded = true;
            if (timer !== null) clearTimeout(timer);
            reject(e);
          }
        });
      }
    }
  });
}
function __hodoFetch(url, options) {
  var opts = options || {};
  var method = String(opts.method || 'GET').toUpperCase();
  if (method !== 'GET' && method !== 'POST') {
    return Promise.reject(new TypeError('fetch: unsupported method ' + method));
  }
  var body = opts.body === undefined ? '' : String(opts.body);
  return __hodoCall('fetch', [String(url), method, body], null, false).then(function (r) {
    return Object.freeze({
      ok: r.ok,
      status: r.status,
      text: function () { return Promise.resolve(r.body); },
      json: function () { return Promise.resolve(JSON.parse(r.body)); },
      arrayBuffer: function () {
        var s = r.body, n = s.length, a = new Uint8Array(n);
        for (var i = 0; i < n; i++) a[i] = s.charCodeAt(i) & 255;
        return Promise.resolve(a.buffer);
      }
    });
  }, function (e) {
    throw new TypeError(e.message || String(e));
  });
}
"#;

fn storage_ns(style: &str) -> String {
    let local = |area: &str| -> String {
        format!(
            "{area}: {{
               set: function () {{ return __hodoApiCall('{style}', 'storage.{area}.set', __hodoArgs(arguments)); }},
               get: function () {{
                 var a = arguments, name = 'storage.{area}.get';
                 if (__hodoPure[name] &&
                     !(a.length && typeof a[a.length - 1] === 'function')) {{
                   if (__hodoMemo.epoch !== __hodoMemoEpoch) __hodoMemoSync();
                   var short = name + '|' + a[0];
                   var key = __hodoKeyCache[short];
                   if (key === undefined) {{
                     key = __hodoMemoKey(name, a);
                     if (__hodoKeyCount >= 256) {{ __hodoKeyCache = {{}}; __hodoKeyCount = 0; }}
                     __hodoKeyCache[short] = key;
                     __hodoKeyCount++;
                   }}
                   var entry = __hodoMemo.entries[key];
                   if (entry) {{
                     __hodoState.lastError = null;
                     return entry.promise;
                   }}
                 }}
                 return __hodoApiCall('{style}', name, __hodoArgs(a));
               }},
               remove: function () {{ return __hodoApiCall('{style}', 'storage.{area}.remove', __hodoArgs(arguments)); }}
             }}"
        )
    };
    format!(
        "storage: {{ {}, {}, {}, onChanged: __hodoEvent('storage.onChanged') }}",
        local("local"),
        local("sync"),
        local("session")
    )
}

const TABS_NS: &str = r#"tabs: {
  create: function () { return __hodoApiCall('STYLE', 'tabs.create', __hodoArgs(arguments)); },
  list: function () { return __hodoApiCall('STYLE', 'tabs.list', __hodoArgs(arguments)); },
  query: function () { return __hodoApiCall('STYLE', 'tabs.query', __hodoArgs(arguments)); }
}"#;

fn runtime_ns(style: &str) -> String {
    let send = match style {
        "browser" => "return __hodoSendMessage(arguments[0], null);".to_string(),
        "chrome-cb" => {
            "var cb = (arguments.length > 1 && typeof arguments[1] === 'function') ? arguments[1] : null; \
             if (cb) { __hodoSendMessage(arguments[0], cb); return undefined; } \
             return __hodoApiCall('chrome-cb', 'runtime.sendMessage', [arguments[0]]);"
                .to_string()
        }
        _ => {
            "var cb = (arguments.length > 1 && typeof arguments[1] === 'function') ? arguments[1] : null; \
             return __hodoSendMessage(arguments[0], cb);"
                .to_string()
        }
    };
    format!(
        "runtime: {{
  sendMessage: function () {{ {send} }},
  connect: function (name) {{ return __hodoPort(name); }},
  getURL: function (path) {{ return __hodoApiCall('{style}', 'runtime.getURL', [path || '']); }},
  getManifest: function () {{ return __hodoApiCall('{style}', 'runtime.getManifest', []); }},
  get id() {{ return __hodoApiCall('{style}', 'runtime.id', []); }},
  onMessage: __hodoEvent('runtime.onMessage'),
  onConnect: __hodoEvent('runtime.onConnect')
}}"
    )
}

const ALARMS_NS: &str = r#"alarms: {
  create: function (name, info) { return __hodoApiCall('STYLE', 'alarms.create', [name, info || {}]); },
  clear: function (name) { return __hodoApiCall('STYLE', 'alarms.clear', [name]); },
  onAlarm: __hodoEvent('alarms.onAlarm')
}"#;

const NOTIFICATIONS_NS: &str = r#"notifications: {
  create: function () { return __hodoApiCall('STYLE', 'notifications.create', __hodoArgs(arguments)); }
}"#;

/// `i18n.getMessage` and the `action` aliases. PURE reads flow through the same
/// dispatch table as storage; `__hodoApiCall` memoizes every PURE_METHODS name.
fn compat_ns(style: &str) -> String {
    format!(
        "i18n: {{
  getMessage: function () {{ return __hodoApiCall('{style}', 'i18n.getMessage', __hodoArgs(arguments)); }},
  getUILanguage: function () {{ return 'en-US'; }}
}},
action: {{ onClicked: __hodoEvent('action.onClicked') }},
browserAction: {{ onClicked: __hodoEvent('action.onClicked') }}"
    )
}

/// Build the complete prelude for one extension context.
#[must_use]
pub fn build_prelude(config: &RuntimeConfig) -> String {
    let style = match config.dialect {
        ApiDialect::ChromeMv2 => "chrome-cb",
        ApiDialect::ChromeMv3 => "chrome",
        ApiDialect::Gecko => "browser",
    };
    let wants = |ns: &str| {
        ns == "runtime"
            || config
                .allowed_namespaces
                .iter()
                .any(|p| canonical_namespace(p) == Some(ns))
    };
    // `browser.*` is always Promise-style; only the chrome object follows the
    // manifest dialect. Both share the same native dispatch table.
    let entries_for = |api_style: &str, runtime_style: &str| -> Vec<String> {
        let mut entries = Vec::new();
        if wants("storage") {
            entries.push(storage_ns(api_style));
        }
        if wants("tabs") {
            entries.push(TABS_NS.replace("STYLE", api_style));
        }
        entries.push(runtime_ns(runtime_style));
        if wants("alarms") {
            entries.push(ALARMS_NS.replace("STYLE", api_style));
        }
        if wants("notifications") {
            entries.push(NOTIFICATIONS_NS.replace("STYLE", api_style));
        }
        entries.push(compat_ns(api_style));
        entries
    };
    let browser_entries = entries_for("browser", "browser");
    let chrome_entries = entries_for(style, style);

    let mut out = with_memo_config(HELPERS);
    out.push_str("var __hodoApi = {\n");
    out.push_str(&browser_entries.join(",\n"));
    out.push_str("\n};\n");
    out.push_str("Object.defineProperty(__hodoApi.runtime, 'lastError', {\n");
    out.push_str("  get: function () { return __hodoState.lastError; }\n});\n");
    out.push_str("var __hodoChrome = {\n");
    out.push_str(&chrome_entries.join(",\n"));
    out.push_str("\n};\n");
    out.push_str("Object.defineProperty(__hodoChrome.runtime, 'lastError', {\n");
    out.push_str("  get: function () { return __hodoState.lastError; }\n});\n");
    out.push_str("var chrome = __hodoChrome;\n");
    out.push_str("var browser = __hodoApi;\n");
    out.push_str(
        "__hodoFreezeDeep(chrome);\n__hodoFreezeDeep(browser);\nvar fetch = __hodoFetch;\n",
    );
    out
}

/// Prelude for a *page* (popup) context. Deliberately shares nothing with the
/// extension prelude: no `chrome`, `browser`, `fetch`, `i18n` or `action` is
/// ever installed here — only console, timers, promise plumbing, the DOM
/// wrappers, the minimal `window` alias and the internal `__hodoFetch` helper.
const PAGE_STATIC: &str = r#"
var console = {
  log: function () { __hodo_log(__hodoJoin(arguments)); },
  info: function () { __hodo_log(__hodoJoin(arguments)); },
  warn: function () { __hodo_log(__hodoJoin(arguments)); },
  error: function () { __hodo_log(__hodoJoin(arguments)); },
  debug: function () { __hodo_log(__hodoJoin(arguments)); }
};
function __hodoJoin(args) {
  var out = [];
  for (var i = 0; i < args.length; i++) {
    var v = args[i];
    if (typeof v === 'string') out.push(v);
    else { try { out.push(JSON.stringify(v)); } catch (_) { out.push(String(v)); } }
  }
  return out.join(' ');
}
function __hodoReportError(e) {
  try { __hodo_log(String(e && (e.stack || e.message) || e)); } catch (_) {}
}
__HODO_MEMO__
__HODO_MODULES__
var __hodoState = { pending: {}, listeners: {}, lastError: null, nextId: 1 };
function __hodoPost(name, args) { return __hodo_post_task(name, args); }
function __hodoCall(name, args, callback, callbackOnly) {
  var cb = (typeof callback === 'function') ? callback : null;
  var memoKey = __hodoPure[name] ? __hodoMemoKeyFast(name, args) : null;
  var entry = memoKey ? __hodoMemoGet(memoKey) : __hodoMemoMiss;
  if (memoKey && entry === __hodoMemoMiss) {
    var direct = __hodo_read_pure(name, args);
    if (direct !== undefined) {
      __hodoMemoPut(memoKey, direct);
      entry = __hodoMemoGet(memoKey);
    }
  }
  if (entry !== __hodoMemoMiss) {
    __hodoState.lastError = null;
    if (cb) entry.promise.then(function (v) { try { cb(v); } catch (e) { __hodoReportError(e); } });
    return callbackOnly ? undefined : entry.promise;
  }
  var id = __hodoPost(name, args);
  var result;
  if (callbackOnly) {
    __hodoState.pending[id] = { cb: cb, memoKey: memoKey };
  } else {
    result = new Promise(function (resolve, reject) {
      __hodoState.pending[id] = { resolve: resolve, reject: reject, cb: cb, memoKey: memoKey };
    });
  }
  return result;
}
function __hodoSettle(id, ok, value, error) {
  var entry = __hodoState.pending[id];
  if (!entry) return;
  delete __hodoState.pending[id];
  if (!ok) {
    var err = new Error(error);
    if (entry.cb) {
      __hodoState.lastError = err;
      try { entry.cb(undefined); } catch (e) { __hodoReportError(e); }
    } else if (entry.reject) {
      entry.reject(err);
    }
    return;
  }
  __hodoState.lastError = null;
  if (entry.memoKey) __hodoMemoPut(entry.memoKey, value);
  if (entry.cb) {
    try { entry.cb(value); } catch (e) { __hodoReportError(e); }
  }
  if (entry.resolve) entry.resolve(value);
}
function __hodoFetch(url, options) {
  var opts = options || {};
  var method = String(opts.method || 'GET').toUpperCase();
  if (method !== 'GET' && method !== 'POST') {
    return Promise.reject(new TypeError('fetch: unsupported method ' + method));
  }
  var body = opts.body === undefined ? '' : String(opts.body);
  return __hodoCall('fetch', [String(url), method, body], null, false).then(function (r) {
    return Object.freeze({
      ok: r.ok,
      status: r.status,
      text: function () { return Promise.resolve(r.body); },
      json: function () { return Promise.resolve(JSON.parse(r.body)); },
      arrayBuffer: function () {
        var s = r.body, n = s.length, a = new Uint8Array(n);
        for (var i = 0; i < n; i++) a[i] = s.charCodeAt(i) & 255;
        return Promise.resolve(a.buffer);
      }
    });
  }, function (e) {
    throw new TypeError(e.message || String(e));
  });
}
// Page fetch rides the shell's GET-only cached loader hook; unsupported methods
// are rejected by __hodoFetch rather than silently faked.
function fetch(url, options) { return __hodoFetch(url, options); }
function setTimeout(fn, delay) {
  if (typeof fn !== 'function') throw new TypeError('setTimeout expects a function');
  var ms = Number(delay);
  if (!isFinite(ms) || ms < 1) ms = 1;
  var rest = [];
  for (var i = 2; i < arguments.length; i++) rest.push(arguments[i]);
  return __hodo_set_timeout(fn, ms, rest);
}
function setInterval(fn, delay) {
  if (typeof fn !== 'function') throw new TypeError('setInterval expects a function');
  var ms = Number(delay);
  if (!isFinite(ms) || ms < 1) ms = 1;
  var rest = [];
  for (var i = 2; i < arguments.length; i++) rest.push(arguments[i]);
  return __hodo_set_interval(fn, ms, rest);
}
function clearTimeout(id) { __hodo_clear_timer(id); }
function clearInterval(id) { __hodo_clear_timer(id); }
// Attribute view rebuilt from the real get_attribute native. Dom has no public
// attrs() iterator, so names are collected as setAttribute/innerHTML are used;
// that includes Lit's `$lit$...` marker attributes, which template stamping
// needs to see. Filtering by the native keeps the enumeration honest.
var __hodoDomAttributeNames = [
  'id', 'class', 'name', 'src', 'href', 'alt', 'title', 'type', 'value',
  'placeholder', 'role', 'style', 'width', 'height', 'rel', 'target',
  'action', 'method', 'for', 'content', 'charset', 'lang', 'dir',
  'checked', 'selected', 'disabled', 'readonly', 'required', 'hidden'
];
var __hodoSeenAttributeNames = [];
function __hodoNoteAttributeNames(text) {
  var re = /([^\s"'>=/]+)\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/g;
  var match;
  while ((match = re.exec(String(text))) !== null) {
    if (__hodoSeenAttributeNames.indexOf(match[1]) === -1) {
      __hodoSeenAttributeNames.push(match[1]);
    }
  }
}
function __hodoAttributeNamesOf(node) {
  var out = [];
  var seen = {};
  var all = __hodoDomAttributeNames.concat(__hodoSeenAttributeNames);
  for (var i = 0; i < all.length; i++) {
    var name = all[i];
    if (seen[name]) continue;
    seen[name] = true;
    if (__hodo_dom_get_attribute(node, name) !== null) out.push(name);
  }
  return out;
}
function __hodoDomAttributes(node) {
  var names = __hodoAttributeNamesOf(node);
  var out = [];
  for (var i = 0; i < names.length; i++) {
    out.push({ name: names[i], value: __hodo_dom_get_attribute(node, names[i]) });
  }
  out.getNamedItem = function (name) {
    for (var j = 0; j < out.length; j++) {
      if (out[j].name === String(name)) return out[j];
    }
    return null;
  };
  return out;
}
// Custom-element wave-1 state. The prelude is eval'd fresh per page context,
// so all of this is naturally per-runtime. Keys are the stable DOM handles
// handed out by alloc_handle (one per live NodeId).
var __hodoUpgraded = {};
var __hodoWrappers = {};
var __hodoCustomElements = {};
var __hodoWhenDefined = {};
var __hodoPendingNode = null;
function __hodoConnect(inst) {
  if (!inst || inst.__hodoConnected) return;
  inst.__hodoConnected = true;
  if (typeof inst.connectedCallback === 'function') {
    try { inst.connectedCallback(); } catch (e) { __hodoReportError(e); }
  }
}
// Install the handle-backed DOM surface onto an object. Shared by plain
// wrappers and by custom-element instances constructed through HTMLElement().
function __hodoDomProps(api, node) {
  api.__hodoNode = node;
  var isFragment = __hodo_dom_tag(node) === '#document-fragment';
  api.append = function () {
    for (var i = 0; i < arguments.length; i++) {
      var value = arguments[i];
      api.appendChild(value instanceof Node ? value
        : document.createTextNode(String(value)));
    }
  };
  api.prepend = function () {
    var kids = __hodo_dom_child_nodes(node);
    var ref = kids.length ? __hodoWrapAny(kids[0]) : null;
    for (var i = 0; i < arguments.length; i++) {
      var value = arguments[i];
      api.insertBefore(value instanceof Node ? value
        : document.createTextNode(String(value)), ref);
    }
  };
  api.replaceChildren = function () {
    var kids = __hodo_dom_child_nodes(node);
    for (var i = 0; i < kids.length; i++) api.removeChild(__hodoWrapAny(kids[i]));
    api.append.apply(api, arguments);
  };
  api.contains = function (other) {
    if (!other || other.__hodoNode === undefined) return false;
    var target = other.__hodoNode;
    var found = false;
    var handles = [node];
    var cursor;
    while ((cursor = handles.shift()) !== undefined) {
      if (cursor === target) { found = true; break; }
      var kids = __hodo_dom_child_nodes(cursor);
      for (var j = 0; j < kids.length; j++) handles.push(kids[j]);
    }
    return found;
  };
  api.appendChild = function (child) {
    // The native flattens a DocumentFragment. Children only become connected
    // when they land in the live tree, not while parked in a fragment.
    var moved = child instanceof DocumentFragment ? child.childNodes : null;
    __hodo_dom_append_child(node, child.__hodoNode);
    if (moved) {
      for (var i = 0; i < moved.length; i++) __hodoConnect(moved[i]);
    } else if (!isFragment) {
      __hodoConnect(child);
    }
    return child;
  };
  api.insertBefore = function (newNode, refNode) {
    // A null/undefined ref appends; fragments flatten in the native.
    var refHandle = (refNode === null || refNode === undefined) ? null : refNode.__hodoNode;
    var moved = newNode instanceof DocumentFragment ? newNode.childNodes : null;
    __hodo_dom_insert_before(node, newNode.__hodoNode, refHandle);
    if (moved) {
      for (var i = 0; i < moved.length; i++) __hodoConnect(moved[i]);
    } else if (!isFragment) {
      __hodoConnect(newNode);
    }
    return newNode;
  };
  api.removeChild = function (child) { __hodo_dom_remove_child(node, child.__hodoNode); return child; };
  api.remove = function () {
    var parent = __hodo_dom_parent(node);
    if (parent !== null) __hodo_dom_remove_child(parent, node);
  };
  api.getAttribute = function (name) { return __hodo_dom_get_attribute(node, String(name)); };
  api.setAttribute = function (name, value) {
    __hodoNoteAttributeNames([String(name) + '="' + String(value) + '"'].join(''));
    __hodo_dom_set_attribute(node, String(name), String(value));
  };
  api.hasAttribute = function (name) {
    return __hodo_dom_get_attribute(node, String(name)) !== null;
  };
  api.hasAttributes = function () { return __hodoAttributeNamesOf(node).length > 0; };
  api.getAttributeNames = function () { return __hodoAttributeNamesOf(node); };
  // shortcut: Dom has no attribute-removal native, so removal clears the value.
  api.removeAttribute = function (name) {
    __hodo_dom_set_attribute(node, String(name), '');
  };
  api.toggleAttribute = function (name, force) {
    var on = force === undefined
      ? !api.hasAttribute(name) : !!force;
    if (on) api.setAttribute(name, '');
    else api.removeAttribute(name);
    return on;
  };
  // Scoped to this node's subtree (the native starts at the handle, and hides
  // #shadow-root subtrees unless the handle itself is a shadow root).
  api.querySelector = function (selector) {
    return __hodoWrapAny(__hodo_dom_query(node, String(selector)));
  };
  api.querySelectorAll = function (selector) {
    var handles = __hodo_dom_query_all(node, String(selector));
    var out = [];
    for (var i = 0; i < handles.length; i++) {
      var wrapped = __hodoWrapAny(handles[i]);
      if (wrapped !== null) out.push(wrapped);
    }
    return out;
  };
  api.addEventListener = function (kind, fn) {
    __hodo_dom_add_listener(node, String(kind), __hodoEventListener(fn));
  };
  api.removeEventListener = function (kind, fn) {
    __hodo_dom_remove_listener(node, String(kind), __hodoEventListener(fn));
  };
  api.dispatchEvent = function (evt) {
    evt.target = this;
    return __hodo_dom_dispatch_event(node, evt, !!evt.bubbles);
  };
  api.classList = {
    add: function (c) { __hodo_dom_class_list(node, 'add', String(c)); },
    remove: function (c) { __hodo_dom_class_list(node, 'remove', String(c)); },
    toggle: function (c) { return __hodo_dom_class_list(node, 'toggle', String(c)); },
    contains: function (c) { return __hodo_dom_class_list(node, 'contains', String(c)); }
  };
  Object.defineProperty(api, 'textContent', {
    get: function () { return __hodo_dom_text(node); },
    set: function (v) { __hodo_dom_set_text(node, String(v)); }
  });
  Object.defineProperty(api, 'innerText', {
    get: function () { return __hodo_dom_text(node); },
    set: function (v) { __hodo_dom_set_text(node, String(v)); }
  });
  // URL reflection: resolved against the page URL by the native; writes route
  // through setAttribute so DOM + wrapper stay in sync (real sites assign .src/.href).
  Object.defineProperty(api, 'src', {
    get: function () { return __hodo_dom_src(node); },
    set: function (v) { __hodo_dom_set_attribute(node, 'src', String(v)); },
    enumerable: true
  });
  Object.defineProperty(api, 'href', {
    get: function () { return __hodo_dom_href(node); },
    set: function (v) { __hodo_dom_set_attribute(node, 'href', String(v)); },
    enumerable: true
  });
  var type = __hodo_dom_node_type(node);
  var tag = __hodo_dom_tag(node);
  api.nodeType = type;
  api.nodeName = type === 9 ? '#document'
    : type === 3 ? '#text'
    : type === 8 ? '#comment'
    : type === 11 ? '#document-fragment'
    : String(tag).toUpperCase();
  api.tagName = type === 1 ? api.nodeName : null;
  // Local name / namespace are honest for our HTML-only arena. ownerDocument
  // is resolved lazily (the live `document` is built after this helper).
  api.localName = type === 1 ? String(tag).toLowerCase() : null;
  api.namespaceURI = 'http://www.w3.org/1999/xhtml';
  Object.defineProperty(api, 'ownerDocument', {
    get: function () { return __hodoOwnerDocument(node); }
  });
  // Minimal serialization: text verbatim (escaped), elements as
  // <tag attr="v">…</tag>, void tags without a close. Fidelity limits: only
  // the curated attribute names from __hodoDomAttributes survive, namespaces
  // and entity fidelity are not modelled, and shadow content is hidden (the
  // children native filters #shadow-root). Enough for polyfill reads.
  Object.defineProperty(api, 'innerHTML', {
    get: function () { return __hodoSerialize(__hodoInnerTarget(node)); },
    set: function (value) {
      __hodoNoteAttributeNames(value);
      __hodo_dom_set_inner_html(__hodoInnerTarget(node), String(value));
    },
    enumerable: true
  });
  // Per-access builds from the real natives: no caching, no invalidation risk.
  Object.defineProperty(api, 'children', {
    get: function () {
      var handles = __hodo_dom_children(node);
      var out = [];
      for (var i = 0; i < handles.length; i++) {
        out.push(__hodoWrapAny(handles[i]));
      }
      return out;
    }
  });
  Object.defineProperty(api, 'childNodes', {
    get: function () {
      var handles = __hodo_dom_child_nodes(node);
      var out = [];
      for (var i = 0; i < handles.length; i++) {
        out.push(__hodoWrapAny(handles[i]));
      }
      return out;
    }
  });
  Object.defineProperty(api, 'firstChild', {
    get: function () {
      var handles = __hodo_dom_child_nodes(node);
      return handles.length ? __hodoWrapAny(handles[0]) : null;
    }
  });
  Object.defineProperty(api, 'lastChild', {
    get: function () {
      var handles = __hodo_dom_child_nodes(node);
      return handles.length ? __hodoWrapAny(handles[handles.length - 1]) : null;
    }
  });
  Object.defineProperty(api, 'firstElementChild', {
    get: function () {
      var handles = __hodo_dom_children(node);
      return handles.length ? __hodoWrapAny(handles[0]) : null;
    }
  });
  Object.defineProperty(api, 'lastElementChild', {
    get: function () {
      var handles = __hodo_dom_children(node);
      return handles.length ? __hodoWrapAny(handles[handles.length - 1]) : null;
    }
  });
  Object.defineProperty(api, 'previousSibling', {
    get: function () { return __hodoSibling(node, -1); }
  });
  Object.defineProperty(api, 'nextSibling', {
    get: function () { return __hodoSibling(node, 1); }
  });
  Object.defineProperty(api, 'attributes', {
    get: function () { return __hodoDomAttributes(node); }
  });
  Object.defineProperty(api, 'parentNode', {
    get: function () { return __hodoWrapAny(__hodo_dom_parent(node)); }
  });
  Object.defineProperty(api, 'parentElement', {
    get: function () {
      var handle = __hodo_dom_parent(node);
      return (handle !== null && __hodo_dom_node_type(handle) === 1)
        ? __hodoWrapAny(handle) : null;
    }
  });
}
// DOM EventListener accepts either a function or an object with handleEvent;
// Lit's EventPart is the latter. The native listener registry requires a
// callable stored in a global slot, so adapt objects once and reuse the wrapper
// for removeEventListener identity.
function __hodoEventListener(listener) {
  if (typeof listener === 'function') return listener;
  if (listener !== null && typeof listener === 'object'
      && typeof listener.handleEvent === 'function') {
    if (listener.__hodoListenerWrapper === undefined) {
      var target = listener;
      var wrapper = function (event) { return target.handleEvent(event); };
      Object.defineProperty(listener, '__hodoListenerWrapper', {
        value: wrapper, configurable: true
      });
    }
    return listener.__hodoListenerWrapper;
  }
  return listener; // the native reports the type error
}
function __hodoDomWrap(node, extra, accessors, proto) {
  if (node === null || node === undefined) return null;
  var upgraded = __hodoUpgraded[node];
  if (upgraded !== undefined) return upgraded;
  // Plain wrappers get a separate identity cache: __hodoUpgraded is reserved
  // for custom-element instances, so caching there would block later upgrades.
  var cached = __hodoWrappers[node];
  if (cached !== undefined) return cached;
  // Ordinary {} so libraries can stash private state (__styleInfo, etc.) on
  // wrappers; only the DOM surface below is frozen with Object.freeze.
  var api = {};
  __hodoDomProps(api, node);
  if (extra) {
    var keys = Object.keys(extra);
    for (var i = 0; i < keys.length; i++) api[keys[i]] = extra[keys[i]];
  }
  if (accessors) {
    var names = Object.keys(accessors);
    for (var j = 0; j < names.length; j++) {
      Object.defineProperty(api, names[j], accessors[names[j]]);
    }
  }
  if (proto) Object.setPrototypeOf(api, proto);
  __hodoWrappers[node] = api;
  return api;
}
// Sibling lookups reuse the real child-list native; no duplicate sibling table.
function __hodoSibling(node, delta) {
  var parent = __hodo_dom_parent(node);
  if (parent === null) return null;
  var handles = __hodo_dom_child_nodes(parent);
  for (var i = 0; i < handles.length; i++) {
    if (handles[i] === node) {
      var j = i + delta;
      return (j >= 0 && j < handles.length) ? __hodoWrapAny(handles[j]) : null;
    }
  }
  return null;
}
// CharacterData's data/nodeValue/length are views over the same text natives
// that back textContent (installed by __hodoDomProps).
function __hodoCharacterDataAccessors(node) {
  var get = function () { return __hodo_dom_text(node); };
  var set = function (v) { __hodo_dom_set_text(node, String(v)); };
  return {
    data: { get: get, set: set, enumerable: true },
    nodeValue: { get: get, set: set, enumerable: true },
    length: {
      get: function () { return String(__hodo_dom_text(node)).length; },
      enumerable: true
    }
  };
}
// Wrap by actual node kind so childNodes/children hand out typed wrappers.
function __hodoWrapAny(node) {
  if (node === null || node === undefined) return null;
  var type = __hodo_dom_node_type(node);
  var proto = HTMLElement.prototype;
  var accessors = null;
  if (type === 9) proto = Document.prototype;
  else if (type === 11) {
    proto = __hodo_dom_tag(node) === '#shadow-root'
      ? ShadowRoot.prototype : DocumentFragment.prototype;
  }
  else if (type === 1) {
    var tagName = String(__hodo_dom_tag(node)).toLowerCase();
    if (tagName === 'template') proto = HTMLTemplateElement.prototype;
    else if (tagName === 'svg') proto = SVGSVGElement.prototype;
    else if (__hodoHtmlElementClasses[tagName]) {
      proto = __hodoHtmlElementClasses[tagName].prototype;
    }
  } else if (type === 3) {
    proto = Text.prototype;
    accessors = __hodoCharacterDataAccessors(node);
  } else if (type === 8) {
    proto = Comment.prototype;
    accessors = __hodoCharacterDataAccessors(node);
  }
  return __hodoDomWrap(node, null, accessors, proto);
}
// Construct the registered class for a node, attaching its handle during the
// base HTMLElement constructor so the constructor body can already use the
// DOM surface. Falls back to a plain wrapper if the ctor cannot be built.
function __hodoMakeInstance(node, ctor) {
  var existing = __hodoUpgraded[node];
  if (existing !== undefined) return existing;
  __hodoPendingNode = node;
  var inst;
  try {
    inst = new ctor();
  } catch (e) {
    __hodoPendingNode = null;
    __hodoReportError(e);
    return __hodoDomWrap(node, null, null, HTMLElement.prototype);
  }
  __hodoPendingNode = null;
  if (__hodoUpgraded[node] === undefined) {
    // ctor did not route through HTMLElement() (plain function / no super()):
    // install the handle here so its methods still reach the DOM.
    __hodoDomProps(inst, node);
    __hodoUpgraded[node] = inst;
  }
  return inst;
}

// Event construction. Unlike the DOM element constructors, `new Event()` is
// legal and subclassing works: these are ordinary constructible functions, so
// the derived constructor reaches this body through the normal prototype chain.
function Event(type, init) {
  var opts = init || {};
  this.type = String(type === undefined ? '' : type);
  this.bubbles = !!opts.bubbles;
  this.cancelable = !!opts.cancelable;
  this.target = null;
  this.defaultPrevented = false;
}
Event.prototype.preventDefault = function () {
  if (this.cancelable) this.defaultPrevented = true;
};
Event.prototype.stopPropagation = function () {
  // Honored by dispatchEvent's native loop below; the host-driven
  // dispatch_page_event path in hodo-js still ignores it (no event there).
  this.__hodoStopPropagation = true;
};
// Deprecated but still the API the webcomponents polyfill reaches for.
Event.prototype.initEvent = function (type, bubbles, cancelable) {
  this.type = String(type === undefined ? '' : type);
  this.bubbles = !!bubbles;
  this.cancelable = !!cancelable;
  this.defaultPrevented = false;
};
function CustomEvent(type, init) {
  Event.call(this, type, init);
  var opts = init || {};
  this.detail = opts.detail === undefined ? null : opts.detail;
}
CustomEvent.prototype = Object.create(Event.prototype);
CustomEvent.prototype.constructor = CustomEvent;

// UIEvent family. Plain constructible functions like Event/CustomEvent, so
// `new`, `instanceof` and `class X extends ...` all work through the normal
// chain: MouseEvent -> UIEvent -> Event. All device fields are honest defaults:
// hodo has no synthetic input device, so no coordinates or keys are invented.
// shortcut: the host click payload carries x/y as raw JSON straight to list
// listeners (fire_page_event never constructs a MouseEvent), so clientX/clientY
// stay zero; upgrade when a real event-construction path exists.
function UIEvent(type, init) {
  Event.call(this, type, init);
  var opts = init || {};
  this.view = null; // honest: no Window view object semantics here
  this.detail = opts.detail === undefined ? 0 : opts.detail;
}
UIEvent.prototype = Object.create(Event.prototype);
UIEvent.prototype.constructor = UIEvent;

function MouseEvent(type, init) {
  UIEvent.call(this, type, init);
  var opts = init || {};
  this.clientX = opts.clientX === undefined ? 0 : opts.clientX;
  this.clientY = opts.clientY === undefined ? 0 : opts.clientY;
  this.screenX = opts.screenX === undefined ? 0 : opts.screenX;
  this.screenY = opts.screenY === undefined ? 0 : opts.screenY;
  this.pageX = opts.pageX === undefined ? 0 : opts.pageX;
  this.pageY = opts.pageY === undefined ? 0 : opts.pageY;
  this.button = opts.button === undefined ? 0 : opts.button;
  this.buttons = opts.buttons === undefined ? 0 : opts.buttons;
  this.relatedTarget = opts.relatedTarget === undefined ? null : opts.relatedTarget;
  this.altKey = !!opts.altKey;
  this.ctrlKey = !!opts.ctrlKey;
  this.shiftKey = !!opts.shiftKey;
  this.metaKey = !!opts.metaKey;
}
MouseEvent.prototype = Object.create(UIEvent.prototype);
MouseEvent.prototype.constructor = MouseEvent;
// The webcomponents polyfill capability-sniffs this method; initializing the
// base Event fields is honest, coordinates stay at their defaults.
MouseEvent.prototype.initMouseEvent = function (type, bubbles, cancelable) {
  this.initEvent(type, bubbles, cancelable);
};

function KeyboardEvent(type, init) {
  UIEvent.call(this, type, init);
  var opts = init || {};
  this.key = opts.key === undefined ? '' : opts.key;
  this.code = opts.code === undefined ? '' : opts.code;
  this.keyCode = opts.keyCode === undefined ? 0 : opts.keyCode;
  this.which = opts.which === undefined ? 0 : opts.which;
  this.ctrlKey = !!opts.ctrlKey;
  this.altKey = !!opts.altKey;
  this.shiftKey = !!opts.shiftKey;
  this.metaKey = !!opts.metaKey;
}
KeyboardEvent.prototype = Object.create(UIEvent.prototype);
KeyboardEvent.prototype.constructor = KeyboardEvent;

function FocusEvent(type, init) {
  UIEvent.call(this, type, init);
  var opts = init || {};
  this.relatedTarget = opts.relatedTarget === undefined ? null : opts.relatedTarget;
}
FocusEvent.prototype = Object.create(UIEvent.prototype);
FocusEvent.prototype.constructor = FocusEvent;

function InputEvent(type, init) {
  UIEvent.call(this, type, init);
  var opts = init || {};
  this.inputType = opts.inputType === undefined ? '' : opts.inputType;
  this.data = opts.data === undefined ? null : opts.data;
}
InputEvent.prototype = Object.create(UIEvent.prototype);
InputEvent.prototype.constructor = InputEvent;

// Minimal DOM class surface. Constructors are illegal to invoke directly
// (a bare `new Element()` has no backing node); only the wrappers above are
// real instances. Plain functions give boa a normal prototype chain so
// `instanceof` and `class Foo extends HTMLElement` both work.
function Node() {
  if (this.constructor === Node) throw new TypeError('Illegal constructor: Node');
}
function Element() {
  if (this.constructor === Element) throw new TypeError('Illegal constructor: Element');
}
function HTMLElement() {
  if (this.constructor === HTMLElement) throw new TypeError('Illegal constructor: HTMLElement');
  // Custom-element upgrade hook: a derived class calling super() receives the
  // handle staged by __hodoMakeInstance before its constructor body runs.
  if (__hodoPendingNode !== null && __hodoUpgraded[__hodoPendingNode] === undefined) {
    __hodoDomProps(this, __hodoPendingNode);
    __hodoUpgraded[__hodoPendingNode] = this;
    __hodoPendingNode = null;
  }
}
function Document() {
  if (this.constructor === Document) throw new TypeError('Illegal constructor: Document');
}
// CharacterData is an abstract-ish base: direct `new` is spec-illegal, derived
// classes work because `this.constructor` is the derived class.
function CharacterData() {
  if (this.constructor === CharacterData) throw new TypeError('Illegal constructor: CharacterData');
}
function Text() {
  if (this.constructor === Text) throw new TypeError('Illegal constructor: Text');
}
function Comment() {
  if (this.constructor === Comment) throw new TypeError('Illegal constructor: Comment');
}
// Legacy DOM node classes the polyfills iterate over to patch prototypes.
// Their wrappers never surface in hodo's HTML arena, but capability checks
// (`window[a].prototype`) must not throw while the bundle loads.
function CDATASection() {}
function ProcessingInstruction() {}
CDATASection.prototype = Object.create(Text.prototype);
CDATASection.prototype.constructor = CDATASection;
ProcessingInstruction.prototype = Object.create(CharacterData.prototype);
ProcessingInstruction.prototype.constructor = ProcessingInstruction;
// SVG class surfaces exist for capability sniffing / prototype patching only.
// hodo has no SVG raster, so no geometry methods are faked: the common
// polyfill battery merely reads SVGElement.prototype to sniff `classList`.
function SVGElement() {
  if (this.constructor === SVGElement) throw new TypeError('Illegal constructor: SVGElement');
}
function SVGSVGElement() {
  if (this.constructor === SVGSVGElement) throw new TypeError('Illegal constructor: SVGSVGElement');
}
Element.prototype = Object.create(Node.prototype);
Element.prototype.constructor = Element;
HTMLElement.prototype = Object.create(Element.prototype);
HTMLElement.prototype.constructor = HTMLElement;
Document.prototype = Object.create(Node.prototype);
Document.prototype.constructor = Document;
CharacterData.prototype = Object.create(Node.prototype);
CharacterData.prototype.constructor = CharacterData;
Text.prototype = Object.create(CharacterData.prototype);
Text.prototype.constructor = Text;
Comment.prototype = Object.create(CharacterData.prototype);
Comment.prototype.constructor = Comment;
SVGElement.prototype = Object.create(Element.prototype);
SVGElement.prototype.constructor = SVGElement;
SVGSVGElement.prototype = Object.create(SVGElement.prototype);
SVGSVGElement.prototype.constructor = SVGSVGElement;
// No SVG tree semantics here: the nearest-svg accessors are honest nulls, not
// invented ancestors.
Object.defineProperties(SVGElement.prototype, {
  ownerSVGElement: { get: function () { return null; }, configurable: true },
  viewportElement: { get: function () { return null; }, configurable: true }
});
// HTML*Element constructor surfaces the archive bundles patch and `instanceof`
// against. Direct `new` is illegal (no backing node); real element wrappers get
// the matching prototype through the tag map below. Only names the bundles read.
function HTMLAnchorElement() { if (this.constructor === HTMLAnchorElement) throw new TypeError('Illegal constructor: HTMLAnchorElement'); }
function HTMLAreaElement() { if (this.constructor === HTMLAreaElement) throw new TypeError('Illegal constructor: HTMLAreaElement'); }
function HTMLButtonElement() { if (this.constructor === HTMLButtonElement) throw new TypeError('Illegal constructor: HTMLButtonElement'); }
function HTMLCanvasElement() { if (this.constructor === HTMLCanvasElement) throw new TypeError('Illegal constructor: HTMLCanvasElement'); }
function HTMLDialogElement() { if (this.constructor === HTMLDialogElement) throw new TypeError('Illegal constructor: HTMLDialogElement'); }
function HTMLFieldSetElement() { if (this.constructor === HTMLFieldSetElement) throw new TypeError('Illegal constructor: HTMLFieldSetElement'); }
function HTMLFormElement() { if (this.constructor === HTMLFormElement) throw new TypeError('Illegal constructor: HTMLFormElement'); }
function HTMLIFrameElement() { if (this.constructor === HTMLIFrameElement) throw new TypeError('Illegal constructor: HTMLIFrameElement'); }
function HTMLImageElement() { if (this.constructor === HTMLImageElement) throw new TypeError('Illegal constructor: HTMLImageElement'); }
function HTMLInputElement() { if (this.constructor === HTMLInputElement) throw new TypeError('Illegal constructor: HTMLInputElement'); }
function HTMLMediaElement() { if (this.constructor === HTMLMediaElement) throw new TypeError('Illegal constructor: HTMLMediaElement'); }
function HTMLOptionElement() { if (this.constructor === HTMLOptionElement) throw new TypeError('Illegal constructor: HTMLOptionElement'); }
function HTMLOptGroupElement() { if (this.constructor === HTMLOptGroupElement) throw new TypeError('Illegal constructor: HTMLOptGroupElement'); }
function HTMLSelectElement() { if (this.constructor === HTMLSelectElement) throw new TypeError('Illegal constructor: HTMLSelectElement'); }
function HTMLSlotElement() { if (this.constructor === HTMLSlotElement) throw new TypeError('Illegal constructor: HTMLSlotElement'); }
function HTMLTextAreaElement() { if (this.constructor === HTMLTextAreaElement) throw new TypeError('Illegal constructor: HTMLTextAreaElement'); }
function HTMLUnknownElement() { if (this.constructor === HTMLUnknownElement) throw new TypeError('Illegal constructor: HTMLUnknownElement'); }
var __hodoHtmlElementClasses = {
  a: HTMLAnchorElement, area: HTMLAreaElement, button: HTMLButtonElement,
  canvas: HTMLCanvasElement, dialog: HTMLDialogElement, fieldset: HTMLFieldSetElement,
  form: HTMLFormElement, iframe: HTMLIFrameElement, img: HTMLImageElement,
  input: HTMLInputElement, option: HTMLOptionElement, optgroup: HTMLOptGroupElement,
  select: HTMLSelectElement, slot: HTMLSlotElement, textarea: HTMLTextAreaElement,
  audio: HTMLMediaElement, video: HTMLMediaElement
};
for (var __hodoHtmlCtorName in __hodoHtmlElementClasses) {
  var __hodoHtmlCtor = __hodoHtmlElementClasses[__hodoHtmlCtorName];
  __hodoHtmlCtor.prototype = Object.create(HTMLElement.prototype);
  __hodoHtmlCtor.prototype.constructor = __hodoHtmlCtor;
}
// width/height reflect the real attributes; values like '100%' stay the exact
// strings the DOM holds. No SVGAnimatedLength or numeric coercion is invented.
function __hodoSvgAttr(name) {
  return function () {
    var node = this.__hodoNode;
    return node === undefined ? null : __hodo_dom_get_attribute(node, name);
  };
}
Object.defineProperties(SVGSVGElement.prototype, {
  width: { get: __hodoSvgAttr('width'), configurable: true, enumerable: true },
  height: { get: __hodoSvgAttr('height'), configurable: true, enumerable: true }
});
Node.ELEMENT_NODE = 1;
Node.ATTRIBUTE_NODE = 2;
Node.TEXT_NODE = 3;
Node.CDATA_SECTION_NODE = 4;
Node.ENTITY_REFERENCE_NODE = 5;
Node.ENTITY_NODE = 6;
Node.PROCESSING_INSTRUCTION_NODE = 7;
Node.COMMENT_NODE = 8;
Node.DOCUMENT_NODE = 9;
Node.DOCUMENT_TYPE_NODE = 10;
Node.DOCUMENT_FRAGMENT_NODE = 11;
Node.NOTATION_NODE = 12;

// Unlike the element/document constructors, `new DocumentFragment()` is legal:
// the native allocates an unparented #document-fragment node (hodo-html already
// uses that shape for template contents). It stays a valid handle, is skipped by
// Dom::walk so it never paints, and ordinary functions keep subclassing working.
function __hodoFragmentProps(api, node) {
  __hodoDomProps(api, node);
  api.nodeType = Node.DOCUMENT_FRAGMENT_NODE;
  api.nodeName = '#document-fragment';
  api.tagName = null;
  return api;
}
function DocumentFragment() {
  __hodoFragmentProps(this, __hodo_dom_create_fragment());
}
DocumentFragment.prototype = Object.create(Node.prototype);
DocumentFragment.prototype.constructor = DocumentFragment;

// ---- Shadow DOM / templates / detached documents -------------------------
// Side tables are keyed by the stable opaque node handle, so repeated wraps
// share identity even though wrapper objects are frozen.
var __hodoShadowRoots = {};
var __hodoTemplateContents = {};
var __hodoDocs = {};
var __hodoDocTitles = {};

function __hodoRootHandle(node) {
  var cur = node;
  var parent;
  while ((parent = __hodo_dom_parent(cur)) !== null) {
    if (__hodo_dom_tag(parent) === '#shadow-root') return parent;
    cur = parent;
  }
  return cur;
}
function __hodoOwnerDocument(node) {
  var root = __hodoRootHandle(node);
  return __hodoDocs[root] !== undefined ? __hodoDocs[root] : document;
}
function __hodoTemplateContentHandle(node) {
  var handle = __hodoTemplateContents[node];
  if (handle === undefined) {
    handle = __hodo_dom_create_fragment();
    __hodoTemplateContents[node] = handle;
  }
  return handle;
}
function __hodoInnerTarget(node) {
  return __hodo_dom_tag(node) === 'template'
    ? __hodoTemplateContentHandle(node) : node;
}
var __hodoVoidTags = {
  area: 1, base: 1, br: 1, col: 1, embed: 1, hr: 1, img: 1,
  input: 1, link: 1, meta: 1, param: 1, source: 1, track: 1, wbr: 1
};
function __hodoEscape(value) {
  return String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}
function __hodoSerialize(node) {
  var out = '';
  var kids = __hodo_dom_child_nodes(node);
  for (var i = 0; i < kids.length; i++) {
    var kid = kids[i];
    var type = __hodo_dom_node_type(kid);
    if (type === 3) out += __hodoEscape(__hodo_dom_text(kid));
    else if (type === 8) out += '<!--' + __hodo_dom_text(kid) + '-->';
    else {
      var tag = String(__hodo_dom_tag(kid)).toLowerCase();
      var attrs = __hodoDomAttributes(kid);
      var open = '<' + tag;
      for (var j = 0; j < attrs.length; j++) {
        open += ' ' + attrs[j].name + '="' + __hodoEscape(attrs[j].value) + '"';
      }
      open += '>';
      out += __hodoVoidTags[tag] ? open
        : open + __hodoSerialize(kid) + '</' + tag + '>';
    }
  }
  return out;
}

// Capability surface: defining HTMLTemplateElement (with a working content
// fragment and deep clone) makes the webcomponents bundle's template polyfill
// detect native support and skip its DOMParser/createHTMLDocument('template')
// patch entirely. ShadowRoot is the class its `instanceof ShadowRoot` checks
// use; the ShadyDOM branch is skipped because attachShadow/getRootNode exist.
function ShadowRoot() {
  if (this.constructor === ShadowRoot) throw new TypeError('Illegal constructor: ShadowRoot');
}
ShadowRoot.prototype = Object.create(DocumentFragment.prototype);
ShadowRoot.prototype.constructor = ShadowRoot;
// Constructable stylesheet surface. Lit capability-sniffs `replace` here plus
// `adoptedStyleSheets` on Document.prototype to select the constructable path
// instead of its <style>-injection fallback. shortcut: wave-1 stores CSS text
// only — no parse, no rules and no engine cascade wiring; <style> elements
// stamped into shadow roots are still picked up by the shell's normal walk.
function CSSStyleSheet(options) {
  this.__hodoCssText = '';
  this.__hodoOptions = options === undefined ? null : options;
  this.cssText = '';
}
CSSStyleSheet.prototype.replaceSync = function (text) {
  this.__hodoCssText = String(text === undefined ? '' : text);
  this.cssText = this.__hodoCssText;
};
CSSStyleSheet.prototype.replace = function (text) {
  this.replaceSync(text);
  return Promise.resolve(this);
};
Object.defineProperty(CSSStyleSheet.prototype, 'cssRules', {
  get: function () { return []; }, configurable: true
});
CSSStyleSheet.prototype.insertRule = function () {
  throw new TypeError('CSSStyleSheet.insertRule is not supported in hodo page scripts');
};
CSSStyleSheet.prototype.deleteRule = function () {
  throw new TypeError('CSSStyleSheet.deleteRule is not supported in hodo page scripts');
};
globalThis.CSSStyleSheet = CSSStyleSheet;
// Storage-only accessor on both prototypes: libraries may set/read the array,
// but wave-1 does not feed it into styling. Defining it on Document.prototype
// is what Lit's feature probe checks.
function __hodoAdoptedStyleSheets() {
  return {
    get: function () { return this.__hodoAdoptedStyleSheets || []; },
    set: function (value) { this.__hodoAdoptedStyleSheets = value; },
    configurable: true, enumerable: true
  };
}
Object.defineProperty(ShadowRoot.prototype, 'adoptedStyleSheets', __hodoAdoptedStyleSheets());
Object.defineProperty(Document.prototype, 'adoptedStyleSheets', __hodoAdoptedStyleSheets());
function HTMLTemplateElement() {
  if (this.constructor === HTMLTemplateElement) {
    throw new TypeError('Illegal constructor: HTMLTemplateElement');
  }
}
HTMLTemplateElement.prototype = Object.create(HTMLElement.prototype);
HTMLTemplateElement.prototype.constructor = HTMLTemplateElement;
Object.defineProperty(HTMLTemplateElement.prototype, 'content', {
  get: function () {
    var node = this.__hodoNode;
    if (node === undefined) return null;
    var handle = __hodoTemplateContentHandle(node);
    return __hodoDomWrap(handle, null, null, DocumentFragment.prototype);
  },
  configurable: true
});
// source-parsed <template> children live in hodo-html's side-map fragment,
// which is unparented and unreachable from JS; `.content` here is an empty
// view in that case. Runtime-created templates (Lit's path) parse into this
// fragment via the innerHTML setter, so cloning/importing works.

function DOMParser() {}
DOMParser.prototype.parseFromString = function (html, mime) {
  var type = String(mime === undefined ? 'text/html' : mime).toLowerCase();
  if (type !== 'text/html') {
    throw new TypeError('DOMParser: only text/html is supported in hodo page scripts');
  }
  return __hodoNewDocument(String(html));
};

// The bundle constructs one MutationObserver at load and one TreeWalker pair;
// with native shadow DOM its ShadyDOM branch stays inactive, so these only
// need to be honest capability surfaces (no mutation records are generated).
function MutationObserver(callback) { this.__hodoCallback = callback; }
MutationObserver.prototype.observe = function () {};
MutationObserver.prototype.disconnect = function () {};
MutationObserver.prototype.takeRecords = function () { return []; };
var NodeFilter = {
  SHOW_ALL: -1, SHOW_ELEMENT: 1, SHOW_ATTRIBUTE: 2, SHOW_TEXT: 4,
  SHOW_CDATA_SECTION: 8, SHOW_ENTITY_REFERENCE: 16, SHOW_ENTITY: 32,
  SHOW_PROCESSING_INSTRUCTION: 64, SHOW_COMMENT: 128, SHOW_DOCUMENT: 256,
  SHOW_DOCUMENT_TYPE: 512, SHOW_DOCUMENT_FRAGMENT: 1024, SHOW_NOTATION: 2048,
  FILTER_ACCEPT: 1, FILTER_REJECT: 2, FILTER_SKIP: 3
};
function __hodoNodeTypeBit(handle) {
  switch (__hodo_dom_node_type(handle)) {
    case 1: return 1; case 2: return 2; case 3: return 4; case 4: return 8;
    case 5: return 16; case 6: return 32; case 7: return 64; case 8: return 128;
    case 9: return 256; case 10: return 512; case 11: return 1024; case 12: return 2048;
    default: return 0;
  }
}
function __hodoTreeWalker(root, whatToShow, filter) {
  var current = root;
  var api = {
    whatToShow: whatToShow === undefined ? -1 : whatToShow,
    filter: filter === undefined ? null : filter,
    expandEntityReferences: true
  };
  Object.defineProperty(api, 'currentNode', {
    get: function () { return current; },
    set: function (handle) {
      current = (handle && handle.__hodoNode !== undefined)
        ? handle.__hodoNode : handle;
    },
    enumerable: true
  });
  // Three-way result, exactly as DOM specifies. A SHOW_ mask miss is SKIP
  // (subtree still traversed), never REJECT. FILTER_SKIP drops only the node;
  // FILTER_REJECT prunes the whole subtree. Lit stamps parts by counting the
  // nodes nextNode() returns, so conflating SKIP and REJECT shifts every part
  // index and makes ChildPart materialisation grab the wrong marker.
  function classify(handle) {
    var bit = __hodoNodeTypeBit(handle);
    if (bit === 0 || (api.whatToShow !== -1 && (api.whatToShow & bit) === 0)) {
      return NodeFilter.FILTER_SKIP;
    }
    if (api.filter === null || api.filter === undefined) return NodeFilter.FILTER_ACCEPT;
    var asFunction = typeof api.filter === 'function' ? api.filter : api.filter.acceptNode;
    if (typeof asFunction !== 'function') return NodeFilter.FILTER_ACCEPT;
    var result;
    try {
      result = asFunction.call(
        typeof api.filter === 'function' ? undefined : api.filter,
        __hodoWrapAny(handle));
    } catch (e) {
      __hodoReportError(e);
      return NodeFilter.FILTER_ACCEPT;
    }
    return result === NodeFilter.FILTER_REJECT || result === NodeFilter.FILTER_SKIP
      ? result : NodeFilter.FILTER_ACCEPT;
  }
  // First ACCEPTED node in handle's subtree, in document (pre-)order.
  function firstAccepted(handle) {
    var verdict = classify(handle);
    if (verdict === NodeFilter.FILTER_REJECT) return null;
    if (verdict === NodeFilter.FILTER_ACCEPT) return handle;
    var kids = __hodo_dom_child_nodes(handle);
    for (var i = 0; i < kids.length; i++) {
      var hit = firstAccepted(kids[i]);
      if (hit !== null) return hit;
    }
    return null;
  }
  // Last node in reverse document order within handle's subtree.
  function lastAccepted(handle) {
    var verdict = classify(handle);
    if (verdict === NodeFilter.FILTER_REJECT) return null;
    var kids = __hodo_dom_child_nodes(handle);
    for (var i = kids.length - 1; i >= 0; i--) {
      var hit = lastAccepted(kids[i]);
      if (hit !== null) return hit;
    }
    return verdict === NodeFilter.FILTER_ACCEPT ? handle : null;
  }
  api.nextNode = function () {
    if (current === null || current === undefined) return null;
    var kids = __hodo_dom_child_nodes(current);
    for (var k = 0; k < kids.length; k++) {
      var child = firstAccepted(kids[k]);
      if (child !== null) return move(child);
    }
    var node = current;
    while (node !== null && node !== undefined) {
      var parent = __hodo_dom_parent(node);
      if (parent === null) return null;
      var siblings = __hodo_dom_child_nodes(parent);
      var index = siblings.indexOf(node);
      for (var s = index + 1; s < siblings.length; s++) {
        var hit = firstAccepted(siblings[s]);
        if (hit !== null) return move(hit);
      }
      node = parent;
    }
    return null;
  };
  api.previousNode = function () {
    if (current === null || current === undefined) return null;
    var kids = __hodo_dom_child_nodes(current);
    for (var k = kids.length - 1; k >= 0; k--) {
      var child = lastAccepted(kids[k]);
      if (child !== null) return move(child);
    }
    var node = current;
    while (node !== null && node !== undefined) {
      var parent = __hodo_dom_parent(node);
      if (parent === null) return null;
      var siblings = __hodo_dom_child_nodes(parent);
      var index = siblings.indexOf(node);
      for (var s = index - 1; s >= 0; s--) {
        var hit = lastAccepted(siblings[s]);
        if (hit !== null) return move(hit);
      }
      node = parent;
    }
    return null;
  };
  function move(handle) {
    if (handle !== null && handle !== undefined) {
      current = handle;
      api.currentNode = handle;
      return __hodoWrapAny(handle);
    }
    return null;
  }
  api.parentNode = function () { return move(__hodo_dom_parent(current)); };
  api.firstChild = function () {
    var handles = __hodo_dom_child_nodes(current);
    for (var i = 0; i < handles.length; i++) {
      var hit = firstAccepted(handles[i]);
      if (hit !== null) return move(hit);
    }
    return null;
  };
  api.lastChild = function () {
    var handles = __hodo_dom_child_nodes(current);
    for (var i = handles.length - 1; i >= 0; i--) {
      var hit = lastAccepted(handles[i]);
      if (hit !== null) return move(hit);
    }
    return null;
  };
  api.nextSibling = function () {
    var parent = __hodo_dom_parent(current);
    if (parent === null) return null;
    var handles = __hodo_dom_child_nodes(parent);
    var index = handles.indexOf(current);
    for (var i = index + 1; i < handles.length; i++) {
      var hit = firstAccepted(handles[i]);
      if (hit !== null) return move(hit);
    }
    return null;
  };
  api.previousSibling = function () {
    var parent = __hodo_dom_parent(current);
    if (parent === null) return null;
    var handles = __hodo_dom_child_nodes(parent);
    var index = handles.indexOf(current);
    for (var i = index - 1; i >= 0; i--) {
      var hit = lastAccepted(handles[i]);
      if (hit !== null) return move(hit);
    }
    return null;
  };
  return api;
}

// Composition-path walk: parents are real arena links, so shadow children
// terminate at their #shadow-root wrapper and everything else at the document.
Node.prototype.getRootNode = function () {
  var node = this.__hodoNode;
  if (node === undefined) return this;
  if (__hodo_dom_tag(node) === '#shadow-root') return __hodoWrapAny(node);
  var cur = node;
  var parent;
  while ((parent = __hodo_dom_parent(cur)) !== null) {
    if (__hodo_dom_tag(parent) === '#shadow-root') return __hodoWrapAny(parent);
    cur = parent;
  }
  if (__hodoDocs[cur] !== undefined) return __hodoDocs[cur];
  return cur === __hodoRoot ? document : __hodoWrapAny(cur);
};

function __hodoQueryMethods(root) {
  return {
    querySelector: function (selector) {
      return __hodoWrapAny(__hodo_dom_query(root, String(selector)));
    },
    querySelectorAll: function (selector) {
      var handles = __hodo_dom_query_all(root, String(selector));
      var out = [];
      for (var i = 0; i < handles.length; i++) {
        var wrapped = __hodoWrapAny(handles[i]);
        if (wrapped !== null) out.push(wrapped);
      }
      return out;
    },
    getElementById: function (id) {
      return __hodoWrapAny(__hodo_dom_get_element_by_id(root, String(id)));
    }
  };
}

Element.prototype.attachShadow = function (options) {
  var mode = (options || {}).mode;
  if (mode !== 'open' && mode !== 'closed') {
    throw new TypeError('attachShadow: mode must be "open" or "closed"');
  }
  var handle = __hodo_dom_attach_shadow(this.__hodoNode);
  if (handle === null) throw new Error('attachShadow: element already has a shadow root');
  var methods = __hodoQueryMethods(handle);
  var root = __hodoDomWrap(handle, methods, {
    host: { get: function () { return __hodoWrapAny(__hodo_dom_parent(handle)); } },
    mode: { get: function () { return mode; } }
  }, ShadowRoot.prototype);
  __hodoShadowRoots[this.__hodoNode] = { root: root, mode: mode };
  return root;
};
// Closed roots stay reachable from JS code holding the root object (the
// element's own code), but the public `shadowRoot` getter stays null.
Object.defineProperty(Element.prototype, 'shadowRoot', {
  get: function () {
    var entry = __hodoShadowRoots[this.__hodoNode];
    return entry !== undefined && entry.mode === 'open' ? entry.root : null;
  },
  configurable: true
});

function __hodoNewDocument(html) {
  var root = __hodo_dom_parse_detached(String(html));
  var methods = __hodoQueryMethods(root);
  var api = __hodoDomWrap(root, {
    querySelector: methods.querySelector,
    querySelectorAll: methods.querySelectorAll,
    getElementById: methods.getElementById,
    createElement: function (tag) { return __hodoCreateElement(String(tag)); },
    createDocumentFragment: function () { return new DocumentFragment(); },
    createTextNode: function (data) {
      return __hodoWrapAny(__hodo_dom_create_text_node(String(data)));
    },
    createComment: function (data) {
      return __hodoWrapAny(__hodo_dom_create_comment(String(data)));
    },
    importNode: function (node, deep) {
      return __hodoCloneNode(node.__hodoNode, !!deep);
    }
  }, {
    documentElement: {
      get: function () {
        var htmlEl = __hodoWrapAny(__hodo_dom_find_tag(root, 'html'));
        return htmlEl !== null ? htmlEl : __hodoWrapAny(root);
      }
    },
    body: {
      get: function () {
        var body = __hodoWrapAny(__hodo_dom_find_tag(root, 'body'));
        return body !== null ? body : __hodoWrapAny(root);
      }
    },
    head: {
      get: function () { return __hodoWrapAny(__hodo_dom_find_tag(root, 'head')); }
    },
    title: {
      get: function () { return __hodoDocTitles[root] || ''; },
      set: function (value) { __hodoDocTitles[root] = String(value); }
    }
  }, Document.prototype);
  __hodoDocs[root] = api;
  return api;
}
function __hodoUnsupportedMethod(name) {
  return function () { throw new TypeError(name + ' is not supported in hodo page scripts'); };
}
// Wave-1 clone: a detached copy with the curated attribute names copied and,
// when `deep`, recursively cloned children. shortcut: custom-element instances
// clone as plain base elements (no re-upgrade), and only the curated attribute
// names survive until Dom exposes a real attrs() iterator.
function __hodoCloneNode(handle, deep) {
  var type = __hodo_dom_node_type(handle);
  if (type === 3) return __hodoWrapAny(__hodo_dom_create_text_node(__hodo_dom_text(handle)));
  if (type === 8) return __hodoWrapAny(__hodo_dom_create_comment(__hodo_dom_text(handle)));
  if (type === 9) throw new TypeError('cloneNode: document nodes are not supported in hodo page scripts');
  var copy;
  if (type === 11) {
    copy = __hodoWrapAny(__hodo_dom_create_fragment());
  } else {
    copy = __hodoWrapAny(__hodo_dom_create_element(__hodo_dom_tag(handle)));
    var attrs = __hodoDomAttributes(handle);
    for (var i = 0; i < attrs.length; i++) {
      __hodo_dom_set_attribute(copy.__hodoNode, attrs[i].name, attrs[i].value);
    }
  }
  if (deep) {
    var kids = __hodo_dom_child_nodes(handle);
    for (var j = 0; j < kids.length; j++) {
      copy.appendChild(__hodoCloneNode(kids[j], true));
    }
  }
  // Templates clone their content even for a shallow clone (spec behavior);
  // Lit relies on that when stamping a cached template result.
  if (type === 1 && String(__hodo_dom_tag(handle)).toLowerCase() === 'template') {
    var sourceContent = __hodoTemplateContentHandle(handle);
    var targetContent = __hodoTemplateContentHandle(copy.__hodoNode);
    var contentKids = __hodo_dom_child_nodes(sourceContent);
    for (var k = 0; k < contentKids.length; k++) {
      copy.content.appendChild(__hodoCloneNode(contentKids[k], true));
    }
  }
  return copy;
}
Node.prototype.cloneNode = function (deep) {
  return __hodoCloneNode(this.__hodoNode, !!deep);
};

function __hodoCreateElement(tag) {
  var raw = String(tag);
  var node = __hodo_dom_create_element(raw);
  var ctor = __hodoCustomElements[raw.toLowerCase()];
  if (ctor !== undefined) return __hodoMakeInstance(node, ctor);
  return __hodoWrapAny(node);
}

"#;

/// Per-page half of the page prelude: captures the DOM root, `location`,
/// `history`, the custom-element registry and rAF state. Evaluated once per
/// real page load (after the live DOM/URL are known); the STATIC half is primed
/// ahead of time by [`crate::prime_page_prelude`]. The two halves concatenate
/// back into the exact original prelude — see the byte-identity test in lib.rs.
const PAGE_DYNAMIC: &str = r#"var __hodoRoot = __hodo_dom_root();
var document = __hodoDomWrap(__hodoRoot, {
  implementation: {
    createHTMLDocument: function (title) {
      var doc = __hodoNewDocument('');
      if (title !== undefined) doc.title = String(title);
      return doc;
    }
  },
  createTreeWalker: function (root, whatToShow, filter) {
    return __hodoTreeWalker(root === undefined || root === null
      ? __hodoRoot : root.__hodoNode, whatToShow, filter);
  },
  getElementById: function (id) {
    return __hodoWrapAny(__hodo_dom_get_element_by_id(__hodoRoot, String(id)));
  },
  querySelector: function (selector) {
    return __hodoWrapAny(__hodo_dom_query(__hodoRoot, String(selector)));
  },
  querySelectorAll: function (selector) {
    var handles = __hodo_dom_query_all(__hodoRoot, String(selector));
    var out = [];
    for (var i = 0; i < handles.length; i++) {
      var wrapped = __hodoWrapAny(handles[i]);
      if (wrapped !== null) out.push(wrapped);
    }
    return out;
  },
  // Snapshot array instead of a live HTMLCollection; good enough for the
  // bundle's `getElementsByTagName('script'|'link')[0]` reads.
  getElementsByTagName: function (name) {
    var handles = __hodo_dom_find_all_tag(__hodoRoot, String(name));
    var out = [];
    for (var i = 0; i < handles.length; i++) {
      var wrapped = __hodoWrapAny(handles[i]);
      if (wrapped !== null) out.push(wrapped);
    }
    return out;
  },
  createElement: __hodoCreateElement,
  createDocumentFragment: function () { return new DocumentFragment(); },
  createTextNode: function (data) {
    return __hodoWrapAny(__hodo_dom_create_text_node(String(data)));
  },
  createComment: function (data) {
    return __hodoWrapAny(__hodo_dom_create_comment(String(data)));
  },
  createRange: __hodoUnsupportedMethod('document.createRange'),
  // Spec import is a clone into the calling document; our clones already live
  // in the one page arena, so cloneNode(deep) is the honest equivalent here.
  importNode: function (node, deep) {
    return __hodoCloneNode(node.__hodoNode, !!deep);
  },
  // The pre-`new Event()` API the webcomponents polyfill uses at load time.
  // Only the two event-interface domTypes that map cleanly to our Event shape
  // are accepted; anything else throws instead of returning a broken object.
  createEvent: function (domType) {
    var name = String(domType);
    if (name === 'MouseEvents') return new MouseEvent('');
    if (name === 'UIEvents') return new UIEvent('');
    if (name === 'KeyboardEvent') return new KeyboardEvent('');
    if (name === 'Event' || name === 'HTMLEvents') return new Event('');
    throw new TypeError('createEvent: unsupported event interface: ' + name);
  },
  // Buffered until the current script completes; the shell parses the text and
  // appends the resulting children to <body>. After load this is a warned no-op.
  write: function () {
    var text = '';
    for (var i = 0; i < arguments.length; i++) text += String(arguments[i]);
    __hodo_doc_write(text);
  },
  close: function () { __hodo_doc_close(); }
}, {
  // Kept in RuntimeState so the shell integrator can mirror it into the tab
  // title. hodo-js has no shell dependency and cannot do that itself.
  title: {
    get: function () { return __hodo_get_title(); },
    set: function (value) { __hodo_set_title(String(value)); },
    enumerable: true
  },
  documentElement: {
    get: function () {
      return __hodoWrapAny(__hodo_dom_find_tag(__hodoRoot, 'html'));
    },
    enumerable: true
  },
  head: {
    get: function () {
      return __hodoWrapAny(__hodo_dom_find_tag(__hodoRoot, 'head'));
    },
    enumerable: true
  },
  body: {
    get: function () {
      return __hodoWrapAny(__hodo_dom_find_tag(__hodoRoot, 'body'));
    },
    enumerable: true
  },
  // TODO(hodo-js): no script-execution position is tracked yet; honest null.
  currentScript: {
    get: function () { return null; },
    enumerable: true
  }
}, Document.prototype);

// `window` is the global object itself: globals a script declares are visible
// through it, and the properties below are the only web-isms this wave adds.
var window = globalThis;
var self = globalThis;
// Capability surface only: `window` is the global object and cannot be
// constructed. Linking its prototype to Window.prototype gives both
// `window instanceof Window` and the inherited prototype properties the
// webcomponents polyfill patches onto Window.prototype.
function Window() {
  if (this.constructor === Window) throw new TypeError('Illegal constructor: Window');
}
Object.setPrototypeOf(window, Window.prototype);
// Wave-1 URL reflection: a tiny split-based parser over the threaded href. No
// percent-decoding, punycode, IPv6 canonicalization or default-port elision:
// those need a real URL implementation. Query/fragment keep their leading ?/#
// so `location.search.slice(1)` works; a missing query is the empty string.
function __hodoParseLocation(href) {
  var text = String(href || '');
  var rest = text;
  var protocol = '', host = '', hostname = '', port = '';
  var pathname = '', search = '', hash = '';
  var cut = rest.indexOf('#');
  if (cut >= 0) { hash = rest.slice(cut); rest = rest.slice(0, cut); }
  cut = rest.indexOf('?');
  if (cut >= 0) { search = rest.slice(cut); rest = rest.slice(0, cut); }
  cut = rest.indexOf('://');
  if (cut >= 0) {
    protocol = rest.slice(0, cut + 1);
    var after = rest.slice(cut + 3);
    var slash = after.indexOf('/');
    host = slash < 0 ? after : after.slice(0, slash);
    pathname = slash < 0 ? '/' : after.slice(slash);
    var colon = host.lastIndexOf(':');
    if (colon > 0 && host.indexOf(']') < colon) {
      hostname = host.slice(0, colon);
      port = host.slice(colon + 1);
    } else {
      hostname = host;
    }
  } else {
    pathname = rest;
  }
  return {
    href: text, protocol: protocol, host: host, hostname: hostname,
    port: port, pathname: pathname, search: search, hash: hash,
    origin: (protocol && host) ? protocol + '//' + host : ''
  };
}
// Wave-1 navigation surface. The href lives in JS so history.replaceState can
// rewrite it; the URL is re-parsed on every write. Real navigation (network,
// popstate, router reaction) is deliberately out of scope.
var __hodoLocationHref = __hodo_page_url();
var __hodoLocation = null;
function __hodoRefreshLocation() {
  var parts = __hodoParseLocation(__hodoLocationHref);
  parts.assign = function (url) { __hodoSetLocationUrl(url); };
  parts.replace = parts.assign;
  var fields = ['href', 'pathname', 'search', 'hash'];
  for (var i = 0; i < fields.length; i++) {
    (function (field) {
      Object.defineProperty(parts, field, {
        get: function () { return __hodoParseLocation(__hodoLocationHref)[field]; },
        set: function (value) {
          var p = __hodoParseLocation(__hodoLocationHref);
          p[field] = String(value);
          var tail = p.pathname + p.search + p.hash;
          var next = field === 'href' ? String(value)
            : (p.protocol ? p.protocol + '//' + p.host + tail : tail);
          __hodoLocationHref = __hodoResolveModuleSpec(next, __hodoLocationHref);
          __hodoRefreshLocation();
        },
        enumerable: true
      });
    })(fields[i]);
  }
  return parts;
}
function __hodoSetLocationUrl(url) {
  if (url === undefined || url === null || url === '') return;
  var text = String(url);
  if (text.charAt(0) === '#') {
    var p = __hodoParseLocation(__hodoLocationHref);
    text = p.pathname + p.search + text;
  }
  __hodoLocationHref = __hodoResolveModuleSpec(text, __hodoLocationHref);
  __hodoLocation = __hodoRefreshLocation();
}
__hodoLocation = __hodoRefreshLocation();
Object.defineProperty(window, 'location', {
  get: function () { return __hodoLocation; },
  configurable: false,
  enumerable: true
});

// Router-facing history. Router init wraps pushState/replaceState/forward/go/
// back on the object itself, so all five must be own, writable properties.
// shortcut: one session-local entry, no stack, popstate or router reaction.
var __hodoHistoryState = null;
var __hodoHistoryNoopWarned = false;
function __hodoHistoryNoop(name) {
  return function () {
    if (!__hodoHistoryNoopWarned) {
      __hodoHistoryNoopWarned = true;
      __hodo_log('hodo: history.' + name + ' is a no-op in page scripts');
    }
  };
}
var history = {
  length: 1,
  scrollRestoration: 'auto',
  get state() { return __hodoHistoryState; },
  pushState: function (state, title, url) {
    __hodoHistoryState = state === undefined ? null : state;
    __hodoSetLocationUrl(url);
  },
  replaceState: function (state, title, url) {
    __hodoHistoryState = state === undefined ? null : state;
    __hodoSetLocationUrl(url);
  },
  back: __hodoHistoryNoop('back'),
  forward: __hodoHistoryNoop('forward'),
  go: __hodoHistoryNoop('go')
};
window.history = history;

// Schedulers. rAF is a house 16ms timer: the shell's wait/tick loop drains it,
// which is enough for a Lit-style update to stamp its first frame.
var __hodoRafTimers = Object.create(null);
var __hodoRafNextId = 1;
function requestAnimationFrame(callback) {
  if (typeof callback !== 'function') throw new TypeError('requestAnimationFrame expects a function');
  var id = __hodoRafNextId++;
  __hodoRafTimers[id] = setTimeout(function () {
    delete __hodoRafTimers[id];
    try { callback(__hodoPerformance.now()); } catch (e) { __hodoReportError(e); }
  }, 16);
  return id;
}
function cancelAnimationFrame(id) {
  var timer = __hodoRafTimers[id];
  if (timer !== undefined) {
    delete __hodoRafTimers[id];
    clearTimeout(timer);
  }
}
function queueMicrotask(callback) {
  if (typeof callback !== 'function') throw new TypeError('queueMicrotask expects a function');
  return Promise.resolve().then(callback);
}

// shortcut: observers are capability-only — constructor/observe/disconnect are
// present so feature detection and polyfill setup succeed, but callbacks never
// fire. No real observation is performed.
function __hodoCapabilityObserver(callback) { this.callback = callback; }
__hodoCapabilityObserver.prototype.observe = function () {};
__hodoCapabilityObserver.prototype.unobserve = function () {};
__hodoCapabilityObserver.prototype.disconnect = function () {};
__hodoCapabilityObserver.prototype.takeRecords = function () { return []; };
function MutationObserver(callback) { __hodoCapabilityObserver.call(this, callback); }
function IntersectionObserver(callback) { __hodoCapabilityObserver.call(this, callback); }
function ResizeObserver(callback) { __hodoCapabilityObserver.call(this, callback); }
MutationObserver.prototype = IntersectionObserver.prototype = ResizeObserver.prototype =
  __hodoCapabilityObserver.prototype;

// Minimal computed style: only the inline font size the bundles read. An empty
// fontSize is deliberate — NaN guards in the bundles skip style-dependent work.
function getComputedStyle(element) {
  var inline = (element && element.style) ? element.style : null;
  return {
    fontSize: (inline && typeof inline.fontSize === 'string') ? inline.fontSize : '',
    getPropertyValue: function (name) {
      return (inline && typeof inline.getPropertyValue === 'function')
        ? inline.getPropertyValue(String(name)) : '';
    }
  };
}
function matchMedia(query) {
  return {
    media: String(query === undefined ? '' : query),
    matches: false,
    onchange: null,
    addEventListener: function () {},
    removeEventListener: function () {},
    addListener: function () {},
    removeListener: function () {},
    dispatchEvent: function () { return true; }
  };
}
// Wall-clock origin (Date.now), not a monotonic clock. getEntriesByType is
// empty: hodo has no navigation timing data.
var __hodoPerformanceOrigin = Date.now();
var __hodoPerformance = {
  now: function () { return Date.now() - __hodoPerformanceOrigin; },
  getEntriesByType: function () { return []; }
};
window.requestAnimationFrame = requestAnimationFrame;
window.cancelAnimationFrame = cancelAnimationFrame;
window.queueMicrotask = queueMicrotask;
window.MutationObserver = MutationObserver;
window.IntersectionObserver = IntersectionObserver;
window.ResizeObserver = ResizeObserver;
window.getComputedStyle = getComputedStyle;
window.matchMedia = matchMedia;
window.performance = __hodoPerformance;
// UTF-8 byte encoder the Sentry/abtest chunks use for payload sizing. Surrogate
// pairs are encoded; lone surrogates become U+FFFD. No TextDecoder is exposed
// because the bundle only feature-detects it.
function TextEncoder() {}
TextEncoder.prototype.encode = function (input) {
  var text = String(input === undefined ? '' : input);
  var out = [];
  for (var i = 0; i < text.length; i++) {
    var c = text.charCodeAt(i);
    if (c < 0x80) {
      out.push(c);
    } else if (c < 0x800) {
      out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
    } else if (c >= 0xd800 && c <= 0xdbff && i + 1 < text.length &&
               text.charCodeAt(i + 1) >= 0xdc00 &&
               text.charCodeAt(i + 1) <= 0xdfff) {
      var cp = 0x10000 + ((c - 0xd800) << 10) + (text.charCodeAt(++i) - 0xdc00);
      out.push(0xf0 | (cp >> 18), 0x80 | ((cp >> 12) & 0x3f),
               0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
    } else {
      out.push(0xef, 0xbf, 0xbd);
    }
  }
  return new Uint8Array(out);
};
window.TextEncoder = TextEncoder;
// Capability surface for the webcomponents loader: it only checks that
// `window.URL` exists, then installs its own URL implementation over it. The
// constructor parses the same fields as `location`; blob URL creation is
// honestly unsupported (hodo has no blob store).
function URL(href) {
  var parts = __hodoParseLocation(href);
  this.href = parts.href;
  this.protocol = parts.protocol;
  this.host = parts.host;
  this.hostname = parts.hostname;
  this.port = parts.port;
  this.pathname = parts.pathname;
  this.search = parts.search;
  this.hash = parts.hash;
}
URL.prototype.toString = function () { return String(this.href); };
URL.createObjectURL = __hodoUnsupportedMethod('URL.createObjectURL');
URL.revokeObjectURL = __hodoUnsupportedMethod('URL.revokeObjectURL');
window.URL = URL;
window.addEventListener = function (type, fn) {
  __hodo_dom_add_listener(__hodoRoot, String(type), __hodoEventListener(fn));
};
window.removeEventListener = function (type, fn) {
  __hodo_dom_remove_listener(__hodoRoot, String(type), __hodoEventListener(fn));
};
window.dispatchEvent = function (evt) {
  evt.target = window;
  return __hodo_dom_dispatch_event(__hodoRoot, evt, !!evt.bubbles);
};
window.setTimeout = setTimeout;
window.setInterval = setInterval;
window.clearTimeout = clearTimeout;
window.clearInterval = clearInterval;
window.fetch = fetch;
// Minimal CustomElementRegistry. Honest wave-1 surface: define/get/whenDefined,
// upgrade of existing tags, construction on createElement, and connectedCallback
// at upgrade/append. attributeChangedCallback, disconnectedCallback, observedAttributes
// and shadow DOM are intentionally absent (no fake versions of them are exposed).
function __hodoIsCustomName(name) {
  if (name.length < 2) return false;
  if (name[0] < 'a' || name[0] > 'z') return false;
  var dash = name.indexOf('-');
  if (dash <= 0 || dash >= name.length - 1) return false;
  for (var i = 0; i < name.length; i++) {
    var c = name[i];
    if (!((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') ||
          c === '-' || c === '_')) return false;
  }
  return true;
}
// Class surface for capability sniffing / prototype patching. Direct `new` is
// spec-illegal (a registry with no document behind it); the single window
// instance below is aligned to this prototype so
// `customElements instanceof CustomElementRegistry` holds and polyfills may
// patch CustomElementRegistry.prototype safely.
function CustomElementRegistry() {
  if (this.constructor === CustomElementRegistry) {
    throw new TypeError('Illegal constructor: CustomElementRegistry');
  }
}
var __hodoCustomElementRegistry = {
  define: function (name, ctor) {
    var n = String(name).toLowerCase();
    if (typeof ctor !== 'function') {
      throw new TypeError('customElements.define: constructor must be a function');
    }
    if (!__hodoIsCustomName(n)) {
      throw new SyntaxError('customElements.define: invalid custom element name');
    }
    if (__hodoCustomElements[n] !== undefined) {
      throw new Error('customElements.define: name is already defined');
    }
    __hodoCustomElements[n] = ctor;
    // Upgrade elements already in the document: one instance per matching tag.
    var handles = __hodo_dom_find_all_tag(__hodoRoot, n);
    for (var i = 0; i < handles.length; i++) {
      __hodoConnect(__hodoMakeInstance(handles[i], ctor));
    }
    // Resolve any whenDefined waiters with the constructor, spec-style.
    var waiters = __hodoWhenDefined[n];
    if (waiters) {
      for (var j = 0; j < waiters.length; j++) waiters[j](ctor);
      delete __hodoWhenDefined[n];
    }
  },
  get: function (name) {
    var ctor = __hodoCustomElements[String(name).toLowerCase()];
    return ctor === undefined ? null : ctor;
  },
  whenDefined: function (name) {
    var n = String(name).toLowerCase();
    if (__hodoCustomElements[n] !== undefined) {
      return Promise.resolve(__hodoCustomElements[n]);
    }
    return new Promise(function (resolve) {
      if (!__hodoWhenDefined[n]) __hodoWhenDefined[n] = [];
      __hodoWhenDefined[n].push(resolve);
    });
  }
};
Object.setPrototypeOf(__hodoCustomElementRegistry, CustomElementRegistry.prototype);
window.customElements = __hodoCustomElementRegistry;
// Explicit throwing getters instead of a Proxy: a script can catch these and
// sees exactly which global is missing. No fake objects are handed out.
function __hodoUnsupportedGlobal(name, ErrorType) {
  var kind = ErrorType || ReferenceError;
  Object.defineProperty(window, name, {
    configurable: false,
    get: function () {
      throw new kind(name + ' is not supported in hodo page scripts');
    }
  });
}
// Capability-detection surface, not a lie farm: only properties a page can
// honestly observe here are present; every other read yields undefined so
// feature detection (`if (navigator.xyz)`) stays safe instead of throwing.
// Language is a fixed en-US default: no sys-locale hook reaches the prelude.
var navigator = Object.freeze({
  userAgent: 'Mozilla/5.0 (hodo/0.1; +https://github.com/lolikhehe/project-hodo)',
  language: 'en-US',
  languages: Object.freeze(['en-US']),
  platform: 'Win32',
  vendor: '',
  // Honest no-op: analytics uses it for every ping; hodo sends no beacons.
  sendBeacon: function () { return true; }
});
__hodoUnsupportedGlobal('screen');
__hodoUnsupportedGlobal('Range', TypeError);
"#;

/// The STATIC (cacheable) half of the page prelude, with placeholders filled.
/// Contains only class/helper definitions — no DOM-native calls at eval time.
#[must_use]
pub fn build_page_static() -> String {
    with_memo_config(PAGE_STATIC)
}

/// The per-page half of the page prelude. Captures the live DOM root and URL,
/// so it must be evaluated once the real page state is installed.
#[must_use]
pub fn build_page_dynamic() -> &'static str {
    PAGE_DYNAMIC
}

/// Build the page (popup) prelude for one runtime. Injection is intentionally
/// minimal: a page must never be handed an extension API object.
///
/// Byte-identical to the original single-string prelude: all placeholders live
/// in the STATIC half, so substituting before concatenation is equivalent.
#[must_use]
pub fn build_page_prelude() -> String {
    let mut out = build_page_static();
    out.push_str(PAGE_DYNAMIC);
    out
}

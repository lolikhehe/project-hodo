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

fn with_memo_config(js: &str) -> String {
    let pure = crate::PURE_METHODS
        .iter()
        .map(|name| format!("'{name}':1"))
        .collect::<Vec<_>>()
        .join(",");
    js.replace("__HODO_MEMO__", MEMO_JS)
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
const PAGE_HELPERS: &str = r#"
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
// Minimal honest attribute view, rebuilt on every access from the real attr
// native. shortcut: only common HTML attribute names are enumerated; arbitrary
// custom names stay invisible until Dom exposes a real attrs() iterator.
var __hodoDomAttributeNames = [
  'id', 'class', 'name', 'src', 'href', 'alt', 'title', 'type', 'value',
  'placeholder', 'role', 'style', 'width', 'height', 'rel', 'target',
  'action', 'method', 'for', 'content', 'charset', 'lang', 'dir',
  'checked', 'selected', 'disabled', 'readonly', 'required', 'hidden'
];
function __hodoDomAttributes(node) {
  var out = [];
  for (var i = 0; i < __hodoDomAttributeNames.length; i++) {
    var name = __hodoDomAttributeNames[i];
    var value = __hodo_dom_get_attribute(node, name);
    if (value !== null) out.push({ name: name, value: value });
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
  api.removeChild = function (child) { __hodo_dom_remove_child(node, child.__hodoNode); return child; };
  api.getAttribute = function (name) { return __hodo_dom_get_attribute(node, String(name)); };
  api.setAttribute = function (name, value) { __hodo_dom_set_attribute(node, String(name), String(value)); };
  api.addEventListener = function (kind, fn) { __hodo_dom_add_listener(node, String(kind), fn); };
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
  var tag = __hodo_dom_tag(node);
  var isDoc = tag === null || tag === undefined;
  api.nodeType = isDoc ? 9 : 1;
  api.nodeName = isDoc ? '#document' : String(tag).toUpperCase();
  api.tagName = isDoc ? null : api.nodeName;
  // Per-access builds from the real natives: no caching, no invalidation risk.
  Object.defineProperty(api, 'children', {
    get: function () {
      var handles = __hodo_dom_children(node);
      var out = [];
      for (var i = 0; i < handles.length; i++) {
        out.push(__hodoDomWrap(handles[i], null, null, HTMLElement.prototype));
      }
      return out;
    }
  });
  Object.defineProperty(api, 'childNodes', {
    get: function () {
      var handles = __hodo_dom_child_nodes(node);
      var out = [];
      for (var i = 0; i < handles.length; i++) {
        // shortcut: text nodes get an element-shaped view until a node-kind
        // wrapper exists; the child list and identity are still real.
        out.push(__hodoDomWrap(handles[i], null, null, HTMLElement.prototype));
      }
      return out;
    }
  });
  Object.defineProperty(api, 'attributes', {
    get: function () { return __hodoDomAttributes(node); }
  });
  Object.defineProperty(api, 'parentNode', {
    get: function () {
      var handle = __hodo_dom_parent(node);
      if (handle === null) return null;
      var parentProto = __hodo_dom_tag(handle) === null
        ? Document.prototype : HTMLElement.prototype;
      return __hodoDomWrap(handle, null, null, parentProto);
    }
  });
  Object.defineProperty(api, 'parentElement', {
    get: function () {
      var handle = __hodo_dom_parent(node);
      if (handle === null || __hodo_dom_tag(handle) === null) return null;
      return __hodoDomWrap(handle, null, null, HTMLElement.prototype);
    }
  });
}
function __hodoDomWrap(node, extra, accessors, proto) {
  if (node === null || node === undefined) return null;
  var upgraded = __hodoUpgraded[node];
  if (upgraded !== undefined) return upgraded;
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
  return Object.freeze(api);
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
Element.prototype = Object.create(Node.prototype);
Element.prototype.constructor = Element;
HTMLElement.prototype = Object.create(Element.prototype);
HTMLElement.prototype.constructor = HTMLElement;
Document.prototype = Object.create(Node.prototype);
Document.prototype.constructor = Document;
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
// Absent this rung: throw instead of handing out a fake Range/clone/import.
function __hodoUnsupportedMethod(name) {
  return function () { throw new TypeError(name + ' is not supported in hodo page scripts'); };
}
Node.prototype.cloneNode = __hodoUnsupportedMethod('cloneNode');

var __hodoRoot = __hodo_dom_root();
var document = __hodoDomWrap(__hodoRoot, {
  getElementById: function (id) {
    return __hodoDomWrap(__hodo_dom_get_element_by_id(__hodoRoot, String(id)), null, null, HTMLElement.prototype);
  },
  querySelector: function (selector) {
    return __hodoDomWrap(__hodo_dom_query(__hodoRoot, String(selector)), null, null, HTMLElement.prototype);
  },
  querySelectorAll: function (selector) {
    var handles = __hodo_dom_query_all(__hodoRoot, String(selector));
    var out = [];
    for (var i = 0; i < handles.length; i++) {
      var wrapped = __hodoDomWrap(handles[i], null, null, HTMLElement.prototype);
      if (wrapped !== null) out.push(wrapped);
    }
    return out;
  },
  createElement: function (tag) {
    var raw = String(tag);
    var node = __hodo_dom_create_element(raw);
    var ctor = __hodoCustomElements[raw.toLowerCase()];
    if (ctor !== undefined) return __hodoMakeInstance(node, ctor);
    return __hodoDomWrap(node, null, null, HTMLElement.prototype);
  },
  createDocumentFragment: function () { return new DocumentFragment(); },
  createRange: __hodoUnsupportedMethod('document.createRange'),
  importNode: __hodoUnsupportedMethod('document.importNode'),
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
  head: {
    get: function () {
      return __hodoDomWrap(__hodo_dom_find_tag(__hodoRoot, 'head'), null, null, HTMLElement.prototype);
    },
    enumerable: true
  },
  body: {
    get: function () {
      return __hodoDomWrap(__hodo_dom_find_tag(__hodoRoot, 'body'), null, null, HTMLElement.prototype);
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
Object.defineProperty(window, 'location', {
  value: Object.freeze({ href: __hodo_page_url() }),
  writable: false,
  configurable: false,
  enumerable: true
});
window.addEventListener = function (type, fn) {
  if (typeof fn !== 'function') throw new TypeError('addEventListener expects a function');
  __hodo_dom_add_listener(__hodoRoot, String(type), fn);
};
window.dispatchEvent = function (evt) {
  evt.target = window;
  return __hodo_dom_dispatch_event(__hodoRoot, evt, !!evt.bubbles);
};
window.setTimeout = setTimeout;
window.setInterval = setInterval;
window.clearTimeout = clearTimeout;
window.clearInterval = clearInterval;
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
window.customElements = {
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
  vendor: ''
});
__hodoUnsupportedGlobal('screen');
__hodoUnsupportedGlobal('history');
__hodoUnsupportedGlobal('Range', TypeError);
"#;

/// Build the page (popup) prelude for one runtime. Injection is intentionally
/// minimal: a page must never be handed an extension API object.
#[must_use]
pub fn build_page_prelude() -> String {
    with_memo_config(PAGE_HELPERS)
}

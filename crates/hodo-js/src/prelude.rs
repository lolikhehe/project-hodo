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
/// wrappers and the internal `__hodoFetch` helper.
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
function __hodoDomWrap(node, extra) {
  if (node === null || node === undefined) return null;
  var api = {
    __hodoNode: node,
    appendChild: function (child) { __hodo_dom_append_child(node, child.__hodoNode); return child; },
    removeChild: function (child) { __hodo_dom_remove_child(node, child.__hodoNode); return child; },
    getAttribute: function (name) { return __hodo_dom_get_attribute(node, String(name)); },
    setAttribute: function (name, value) { __hodo_dom_set_attribute(node, String(name), String(value)); },
    addEventListener: function (kind, fn) { __hodo_dom_add_listener(node, String(kind), fn); },
    classList: {
      add: function (c) { __hodo_dom_class_list(node, 'add', String(c)); },
      remove: function (c) { __hodo_dom_class_list(node, 'remove', String(c)); },
      toggle: function (c) { return __hodo_dom_class_list(node, 'toggle', String(c)); },
      contains: function (c) { return __hodo_dom_class_list(node, 'contains', String(c)); }
    }
  };
  Object.defineProperty(api, 'textContent', {
    get: function () { return __hodo_dom_text(node); },
    set: function (v) { __hodo_dom_set_text(node, String(v)); }
  });
  Object.defineProperty(api, 'innerText', {
    get: function () { return __hodo_dom_text(node); },
    set: function (v) { __hodo_dom_set_text(node, String(v)); }
  });
  if (extra) {
    var keys = Object.keys(extra);
    for (var i = 0; i < keys.length; i++) api[keys[i]] = extra[keys[i]];
  }
  return Object.freeze(api);
}
var __hodoRoot = __hodo_dom_root();
var document = __hodoDomWrap(__hodoRoot, {
  getElementById: function (id) {
    return __hodoDomWrap(__hodo_dom_get_element_by_id(__hodoRoot, String(id)));
  },
  querySelector: function (selector) {
    return __hodoDomWrap(__hodo_dom_query(__hodoRoot, String(selector)));
  },
  querySelectorAll: function (selector) {
    var handles = __hodo_dom_query_all(__hodoRoot, String(selector));
    var out = [];
    for (var i = 0; i < handles.length; i++) {
      var wrapped = __hodoDomWrap(handles[i]);
      if (wrapped !== null) out.push(wrapped);
    }
    return out;
  },
  createElement: function (tag) {
    return __hodoDomWrap(__hodo_dom_create_element(String(tag)));
  }
});
"#;

/// Build the page (popup) prelude for one runtime. Injection is intentionally
/// minimal: a page must never be handed an extension API object.
#[must_use]
pub fn build_page_prelude() -> String {
    with_memo_config(PAGE_HELPERS)
}

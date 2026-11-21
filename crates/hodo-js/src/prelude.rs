//! JavaScript prelude: thin dual-style wrappers over one native dispatch table.

use hodo_ext_api::{canonical_namespace, ApiDialect, RuntimeConfig};

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
var __hodoState = { pending: {}, listeners: {}, lastError: null, nextId: 1 };
function __hodoPost(name, args) {
  return __hodo_post_task(name, args);
}
function __hodoCall(name, args, callback, callbackOnly) {
  var id = __hodoPost(name, args);
  var cb = (typeof callback === 'function') ? callback : null;
  if (callbackOnly) {
    __hodoState.pending[id] = { cb: cb };
    return undefined;
  }
  return new Promise(function (resolve, reject) {
    __hodoState.pending[id] = { resolve: resolve, reject: reject, cb: cb };
  });
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
  for (var i = 0; i < list.length; i++) {
    try { list[i](message, {}, function () {}); } catch (e) { __hodoReportError(e); }
  }
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
               get: function () {{ return __hodoApiCall('{style}', 'storage.{area}.get', __hodoArgs(arguments)); }},
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

// Stubs keep older demos (i18n, popup action) executable; M2.3/M2.2 replace them.
const COMPAT_NS: &str = r#"i18n: {
  getMessage: function (key) { return String(key); },
  getUILanguage: function () { return 'en-US'; }
},
action: { onClicked: __hodoEvent('action.onClicked') },
browserAction: { onClicked: __hodoEvent('action.onClicked') }"#;

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
        entries.push(COMPAT_NS.to_string());
        entries
    };
    let browser_entries = entries_for("browser", "browser");
    let chrome_entries = entries_for(style, style);

    let mut out = String::from(HELPERS);
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

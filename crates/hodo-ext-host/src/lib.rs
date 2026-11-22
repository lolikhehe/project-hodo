//! Extension host with permission gating and process isolation on by default.
//!
//! Owns one shared API registry, the injectable clock used by timers/alarms and
//! the live background contexts. MV2 contexts stay persistent; idle MV3 service
//! workers are unloaded by [`Host::tick`] and woke again by [`Host::emit_event`]
//! (their state lives in `storage`, not in JS globals).
//!
//! Background scripts run in `hodo-ext-worker` processes supervised by
//! [`broker::Broker`] by default. `HODO_NO_ISOLATION=1` opts back into the
//! legacy in-process path; [`HostBuilder::isolated`] overrides programmatically.

use std::cell::RefCell;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use hodo_ext_api::{
    canonical_namespace, ApiCall, ApiDialect, ApiRegistry, ApiResult, Clock, ExtEvent,
    RuntimeConfig, RuntimeInfo, SystemClock,
};
use hodo_ext_manifest::{Manifest, ManifestVersion};
use serde_json::{json, Value};

pub mod broker;
pub mod worker;

/// Same cap as `hodo_net::MAX_BYTES`; a manifest or script is never allowed to grow unbounded.
const MAX_MANIFEST_BYTES: usize = hodo_types::limits::MAX_MANIFEST_BYTES;
/// MV3 service workers unload after 30 seconds without events or API calls.
const IDLE_UNLOAD_MS: u64 = 30_000;

#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: Manifest,
    pub root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("manifest error: {0}")]
    Manifest(#[from] hodo_ext_manifest::ManifestError),
    #[error("csp: {0}")]
    Csp(String),
    #[error("io: {0}")]
    Io(String),
}

pub struct Host {
    extensions: Vec<Extension>,
    /// Static PURE-read context, parallel to `extensions`.
    runtime_infos: Vec<Rc<RuntimeInfo>>,
    /// Shared with every background-script runtime so JS effects land here too.
    registry: Rc<RefCell<ApiRegistry>>,
    clock: Rc<dyn Clock>,
    /// `None` for MV2 or for an unloaded MV3 service worker.
    contexts: Vec<Option<hodo_js::ExtScriptRuntime>>,
    last_activity: Vec<u64>,
    diagnostics: RefCell<Vec<String>>,
    /// Isolation is on by default; `HODO_NO_ISOLATION=1` opts into in-process.
    isolated: bool,
    /// `None` in in-process mode; taken out of `Host` while polling.
    broker: Option<broker::Broker>,
    disabled: Vec<bool>,
    digests: Vec<String>,
    /// Per-context queues: a message waits for its listener's context instead
    /// of being drained by whichever runtime happens to tick first.
    event_queues: Vec<Vec<QueuedEvent>>,
    has_message_listener: Vec<bool>,
    /// Events for isolated workers that could not be delivered immediately
    /// (spawn in backoff, worker exiting). Replayed on the next (re)spawn.
    worker_pending_events: Vec<Vec<(String, Value)>>,
}

/// Either a shared registry event or an event injected by the browser/worker.
enum QueuedEvent {
    Shared(ExtEvent),
    Raw(String, Value),
}

/// Builder for a [`Host`]. Background workers are isolated by default;
/// `HODO_NO_ISOLATION=1` opts back into the legacy in-process path.
pub struct HostBuilder {
    isolated: bool,
    clock: Rc<dyn Clock>,
    worker_path: Option<PathBuf>,
    worker_env: Vec<(String, String)>,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self {
            isolated: !env_flag("HODO_NO_ISOLATION"),
            clock: Rc::new(SystemClock),
            worker_path: None,
            worker_env: Vec::new(),
        }
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value == "1")
}

impl HostBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn isolated(mut self, isolated: bool) -> Self {
        self.isolated = isolated;
        self
    }

    #[must_use]
    pub fn clock(mut self, clock: Rc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Override the worker binary path (used by spawn end-to-end tests).
    #[must_use]
    pub fn worker_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.worker_path = Some(path.into());
        self
    }

    /// Test-only worker environment injection (e.g. crash simulation).
    #[must_use]
    pub fn worker_env(mut self, key: &str, value: &str) -> Self {
        self.worker_env.push((key.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn build(self) -> Host {
        let isolated = self.isolated && !env_flag("HODO_NO_ISOLATION");
        let mut host = Host::with_clock(self.clock);
        host.isolated = isolated;
        if isolated {
            let mut broker = broker::Broker::default();
            if let Some(path) = self.worker_path {
                broker.set_worker_path(path);
            }
            for (key, value) in self.worker_env {
                broker.set_worker_env(&key, &value);
            }
            host.broker = Some(broker);
        }
        host
    }
}

impl Default for Host {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            runtime_infos: Vec::new(),
            registry: Rc::new(RefCell::new(ApiRegistry::new())),
            clock: Rc::new(SystemClock),
            contexts: Vec::new(),
            last_activity: Vec::new(),
            diagnostics: RefCell::new(Vec::new()),
            isolated: false,
            broker: None,
            disabled: Vec::new(),
            digests: Vec::new(),
            event_queues: Vec::new(),
            has_message_listener: Vec::new(),
            worker_pending_events: Vec::new(),
        }
    }
}

impl Host {
    pub fn new() -> Self {
        HostBuilder::new().build()
    }

    /// Host with an injected clock: tests drive timers/alarms/idle without sleeping.
    #[must_use]
    pub fn with_clock(clock: Rc<dyn Clock>) -> Self {
        Self {
            clock,
            ..Self::default()
        }
    }

    /// Load an unpacked extension directory containing `manifest.json`.
    pub fn load_dir(&mut self, path: &Path) -> Result<&Extension, HostError> {
        let file = fs::File::open(path.join("manifest.json"))
            .map_err(|err| HostError::Io(err.to_string()))?;
        let mut bytes = Vec::new();
        let read = file
            .take(MAX_MANIFEST_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|err| HostError::Io(err.to_string()))?;
        if read > MAX_MANIFEST_BYTES {
            return Err(HostError::Io("manifest too large".into()));
        }
        let digest = manifest_digest(&bytes);
        let json = String::from_utf8(bytes).map_err(|err| HostError::Io(err.to_string()))?;
        let mut manifest = hodo_ext_manifest::parse(&json)?;
        validate_csp(&json, manifest.version)?;
        // Resolve __MSG_ tokens once, at load, so every consumer (toolbar
        // tooltips, badges, popups) sees the same localized action title.
        if let Err(err) = hodo_ext_manifest::resolve_messages(path, &mut manifest) {
            self.diagnostics
                .borrow_mut()
                .push(format!("{}: i18n resolution failed: {err}", manifest.name));
        }
        let info = Self::runtime_info(&manifest, &digest, path);

        // A missing background script is cosmetic: warn, but still load.
        for script in manifest
            .background_scripts
            .iter()
            .chain(manifest.service_worker.iter())
        {
            if !path.join(script).exists() {
                eprintln!(
                    "hodo-ext-host: warning: background script '{script}' not found in {}",
                    path.display()
                );
            }
        }

        self.extensions.push(Extension {
            manifest,
            root: path.to_path_buf(),
        });
        self.runtime_infos.push(info);
        self.contexts.push(None);
        self.last_activity.push(self.clock.now_ms());
        self.disabled.push(false);
        self.digests.push(digest);
        self.event_queues.push(Vec::new());
        self.has_message_listener.push(false);
        self.worker_pending_events.push(Vec::new());
        if let Some(broker) = self.broker.as_mut() {
            broker.ensure(self.extensions.len() - 1);
        }
        Ok(self.extensions.last().expect("just pushed an extension"))
    }

    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Immutable context for the PURE `runtime.*` / `i18n.*` reads.
    fn runtime_info(manifest: &Manifest, digest: &str, root: &Path) -> Rc<RuntimeInfo> {
        // shortcut: without a gecko id the Chrome id is a stable local digest,
        // not the Web Store key-derived a-p alphabet; a real signed id arrives
        // with packaged extensions.
        let id = manifest
            .gecko_id
            .clone()
            .unwrap_or_else(|| digest.to_string());
        let url_base = if manifest.gecko_id.is_some() {
            format!("moz-extension://{id}")
        } else {
            format!("chrome-extension://{id}")
        };
        let messages = hodo_ext_i18n::load_locales(
            root,
            manifest.default_locale.as_deref(),
            &hodo_ext_i18n::ui_locale(),
        )
        .ok()
        .map(|m| m.with_extension_id(&id));
        let manifest_json = json!({
            "manifest_version": match manifest.version {
                ManifestVersion::V2 => 2,
                ManifestVersion::V3 => 3,
            },
            "name": manifest.resolved_name.as_deref().unwrap_or(&manifest.name),
            "version": &manifest.ext_version,
            "description": &manifest.description,
            "permissions": &manifest.permissions,
            "host_permissions": &manifest.host_permissions,
            "default_locale": &manifest.default_locale,
        });
        Rc::new(RuntimeInfo {
            id,
            url_base,
            manifest: manifest_json,
            messages,
        })
    }

    /// Localized toolbar title: `action_title` with the resolved name fallback.
    #[must_use]
    pub fn extension_title(&self, ext_index: usize) -> Option<&str> {
        self.extensions.get(ext_index).map(|ext| {
            ext.manifest.action_title.as_deref().unwrap_or(
                ext.manifest
                    .resolved_name
                    .as_deref()
                    .unwrap_or(&ext.manifest.name),
            )
        })
    }

    /// Whether the MV3 service worker (isolated or in-process) is resident.
    #[must_use]
    pub fn context_is_live(&self, ext_index: usize) -> bool {
        if self.isolated {
            return self.broker.as_ref().is_some_and(|b| b.is_live(ext_index));
        }
        self.contexts.get(ext_index).is_some_and(Option::is_some)
    }

    /// Dispatch only if the extension's manifest declares the API namespace
    /// (`runtime` is always allowed). `fetch` uses host permissions instead.
    pub fn dispatch(&mut self, ext_index: usize, call: &ApiCall) -> ApiResult {
        if let ApiCall::Fetch { url, method, body } = call {
            return self.fetch(ext_index, url, method, body);
        }
        let Some(ext) = self.extensions.get(ext_index) else {
            return ApiResult::Err("no such extension".into());
        };
        let ns = Self::required_namespace(call);
        // `runtime` and `i18n` are always available; manifest permissions gate
        // the rest. The PURE reads below carry no side effect.
        let allowed =
            ns == "runtime" || ns == "i18n" || ext.manifest.permissions.iter().any(|p| p == ns);
        if !allowed {
            return ApiResult::Err(format!(
                "permission denied: {ns} requires manifest permission"
            ));
        }
        let call = match call {
            ApiCall::AlarmCreate {
                name,
                delay_ms,
                period_ms,
                ..
            } => ApiCall::AlarmCreate {
                name: name.clone(),
                delay_ms: *delay_ms,
                period_ms: *period_ms,
                now_ms: self.clock.now_ms(),
            },
            other => other.clone(),
        };
        self.touch(ext_index);
        if let Some(info) = self.runtime_infos.get(ext_index) {
            self.registry.borrow_mut().set_runtime_info(Rc::clone(info));
        }
        self.registry.borrow_mut().dispatch(&call)
    }

    /// Evaluates the extension's background scripts (MV2) / service worker (MV3)
    /// against the host's registry. Also drains the new event loop, so callback
    /// and Promise chains settle before this returns.
    ///
    /// # Errors
    /// Returns a short error when a script is unreadable, too large, or throws.
    pub fn run_background_scripts(&mut self, ext_index: usize) -> Result<String, String> {
        let sources = Self::background_sources(&self.extensions, ext_index)?;
        if sources.is_empty() {
            return Ok("no background scripts".into());
        }
        if self.isolated && !self.disabled[ext_index] {
            return match self.start_worker(ext_index)? {
                broker::StartOutcome::Started => Ok("worker started".to_string()),
                broker::StartOutcome::Deferred { retry_at } => {
                    let line = format!(
                        "worker start deferred until {retry_at}ms; retained events replay on restart"
                    );
                    self.diagnostics.borrow_mut().push(line);
                    Ok("worker start deferred".to_string())
                }
            };
        }

        let config = self.runtime_config(ext_index);
        let mut runtime = hodo_js::ExtScriptRuntime::with_config_and_registry(
            config,
            Rc::clone(&self.registry),
            Rc::clone(&self.clock),
        );
        let fetch_ext = self.extensions[ext_index].clone();
        runtime.set_fetch_hook(Rc::new(move |call| match call {
            ApiCall::Fetch { url, method, body } => fetch_for(&fetch_ext, url, method, body),
            other => ApiResult::Err(format!("unexpected hook call: {other:?}")),
        }));
        let result = runtime
            .eval(&sources.join("\n"))
            .map_err(|err| err.lines().next().unwrap_or("js error").to_string());
        self.drain_logs(&mut runtime);
        if result.is_ok() {
            self.has_message_listener[ext_index] = Self::on_message_listener(&mut runtime);
            self.contexts[ext_index] = Some(runtime);
            self.touch(ext_index);
        }
        result
    }

    /// The exact source-loading order used by the in-process path. The worker
    /// receives the same list in `__hodo_ready` and joins it itself.
    fn background_sources(
        extensions: &[Extension],
        ext_index: usize,
    ) -> Result<Vec<String>, String> {
        let ext = extensions
            .get(ext_index)
            .ok_or_else(|| "no such extension".to_string())?;
        let scripts: Vec<&str> = ext
            .manifest
            .background_scripts
            .iter()
            .map(String::as_str)
            .chain(ext.manifest.service_worker.iter().map(String::as_str))
            .collect();

        let mut sources = Vec::new();
        for script in scripts {
            let path = ext.root.join(script);
            // Missing files are cosmetic; `load_dir` already warns about them.
            if !path.is_file() {
                continue;
            }
            let mut bytes = Vec::new();
            let read = fs::File::open(&path)
                .and_then(|file| {
                    file.take(MAX_MANIFEST_BYTES as u64 + 1)
                        .read_to_end(&mut bytes)
                })
                .map_err(|err| format!("background script {script}: {err}"))?;
            if read > MAX_MANIFEST_BYTES {
                return Err("background script too large".into());
            }
            sources.push(
                String::from_utf8(bytes)
                    .map_err(|err| format!("background script {script}: {err}"))?,
            );
        }
        Ok(sources)
    }

    /// Drive every live context: alarms, queued API tasks, timers, microtasks
    /// and MV3 idle unload. Returns the amount of work performed.
    ///
    /// Shared `ExtEvent`s are drained here, once per tick, and distributed by
    /// listener: the first runtime to tick can no longer steal every message.
    pub fn tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        self.registry.borrow_mut().advance_clock(now);
        let pending = self.registry.borrow_mut().drain_events();
        let mut work = 0;
        let mut logs = Vec::new();
        let mut delivered_messages = std::collections::HashSet::new();

        for index in 0..self.contexts.len() {
            if self.isolated && self.broker.as_ref().is_some_and(|b| b.is_live(index)) {
                continue;
            }
            let mut mine = Vec::new();
            for (position, event) in pending.iter().enumerate() {
                match event {
                    ExtEvent::MessageReceived { .. } => {
                        if !delivered_messages.contains(&position)
                            && self
                                .has_message_listener
                                .get(index)
                                .copied()
                                .unwrap_or(false)
                        {
                            mine.push(event.clone());
                            delivered_messages.insert(position);
                        }
                    }
                    _ => mine.push(event.clone()),
                }
            }
            let queued = std::mem::take(&mut self.event_queues[index]);
            if let Some(runtime) = self.contexts[index].as_mut() {
                for item in queued {
                    Self::fire_queued(runtime, item);
                }
                self.has_message_listener[index] = Self::on_message_listener(runtime);
                for event in mine {
                    Self::fire_shared(runtime, event);
                }
                work += runtime.tick();
                logs.extend(runtime.take_logs());
                if now.saturating_sub(self.last_activity[index]) > IDLE_UNLOAD_MS
                    && self.is_mv3(index)
                {
                    self.contexts[index] = None;
                }
            } else {
                self.event_queues[index].extend(queued);
                self.event_queues[index].extend(mine.into_iter().map(QueuedEvent::Shared));
            }
        }
        self.diagnostics.borrow_mut().extend(logs);

        if self.isolated {
            let mut broker = self
                .broker
                .take()
                .expect("isolated hosts always own a broker");
            work += broker.poll(now, self);
            self.broker = Some(broker);
            // `poll` may have restarted a worker whose backoff ended; deliver
            // everything retained while it was down.
            self.replay_pending_worker_events();
        }
        work
    }

    /// Alias for [`Host::tick`] used by the shell's event loop.
    pub fn poll(&mut self) -> usize {
        self.tick()
    }

    /// Wake a (possibly unloaded) context and deliver one event to its
    /// listeners. MV3 contexts reload from the script; persistent state lives
    /// in `storage`, so a wake keeps prior results.
    pub fn emit_event(&mut self, ext_index: usize, name: &str, payload: Value) {
        if self.disabled.get(ext_index).copied().unwrap_or(false) {
            return;
        }
        if self.isolated {
            if !self.broker.as_ref().is_some_and(|b| b.is_live(ext_index)) && self.is_mv3(ext_index)
            {
                // A deferred spawn is not a wake: retain the event below.
                let _ = self.start_worker(ext_index);
            }
            let now = self.clock.now_ms();
            let delivered = {
                let Some(broker) = self.broker.as_mut() else {
                    return;
                };
                broker.touch(ext_index, now);
                broker.send_event(ext_index, name, payload.clone())
            };
            if !delivered {
                if let Some(pending) = self.worker_pending_events.get_mut(ext_index) {
                    pending.push((name.to_string(), payload));
                }
            }
            return;
        }
        // Keep a wake from letting this one runtime drain everybody's events.
        self.drain_registry_to_queues();
        if !self.context_is_live(ext_index) {
            let _ = self.run_background_scripts(ext_index);
        }
        self.touch(ext_index);
        self.emit_local_event(ext_index, name, payload);
    }

    /// Move shared events into per-context queues before any runtime ticks.
    fn drain_registry_to_queues(&mut self) {
        for event in self.registry.borrow_mut().drain_events() {
            match event {
                ExtEvent::MessageReceived { .. } => {
                    let target = (0..self.event_queues.len()).find(|&index| {
                        self.has_message_listener
                            .get(index)
                            .copied()
                            .unwrap_or(false)
                    });
                    if let Some(target) = target {
                        self.event_queues[target].push(QueuedEvent::Shared(event));
                    }
                }
                other => {
                    for queue in &mut self.event_queues {
                        queue.push(QueuedEvent::Shared(other.clone()));
                    }
                }
            }
        }
    }

    /// Deliver to in-process contexts only (no worker echo).
    fn emit_local_event(&mut self, ext_index: usize, name: &str, payload: Value) {
        if let Some(runtime) = self.contexts.get_mut(ext_index).and_then(Option::as_mut) {
            runtime.fire_event(name, payload);
            runtime.tick();
            let logs = runtime.take_logs();
            for line in logs {
                eprintln!("hodo-ext-host: {line}");
                self.diagnostics.borrow_mut().push(line);
            }
        } else {
            self.event_queues[ext_index].push(QueuedEvent::Raw(name.to_string(), payload));
        }
    }

    /// Queue an event for a context that has not ticked yet (popup / MV3 wake).
    pub fn queue_event(&mut self, ext_index: usize, name: &str, payload: Value) {
        if let Some(queue) = self.event_queues.get_mut(ext_index) {
            queue.push(QueuedEvent::Raw(name.to_string(), payload));
        }
    }

    /// Queue a `runtime.onMessage` delivery for a context with a listener.
    pub fn queue_message(&mut self, ext_index: usize, message: Value) {
        if let Some(queue) = self.event_queues.get_mut(ext_index) {
            queue.push(QueuedEvent::Shared(ExtEvent::MessageReceived { message }));
        }
    }

    /// Attach an in-process context (e.g. a freshly opened popup) and hand it
    /// the events queued while it did not exist.
    pub fn attach_context(&mut self, ext_index: usize, mut runtime: hodo_js::ExtScriptRuntime) {
        let queued = std::mem::take(&mut self.event_queues[ext_index]);
        for item in queued {
            Self::fire_queued(&mut runtime, item);
        }
        let _ = runtime.tick();
        self.has_message_listener[ext_index] = Self::on_message_listener(&mut runtime);
        if let Some(slot) = self.contexts.get_mut(ext_index) {
            *slot = Some(runtime);
        }
    }

    fn fire_queued(runtime: &mut hodo_js::ExtScriptRuntime, item: QueuedEvent) {
        match item {
            QueuedEvent::Shared(event) => Self::fire_shared(runtime, event),
            QueuedEvent::Raw(name, payload) => runtime.fire_event(&name, payload),
        }
    }

    fn fire_shared(runtime: &mut hodo_js::ExtScriptRuntime, event: ExtEvent) {
        if let Some((name, payload)) = Self::event_wire(event) {
            runtime.fire_event(&name, payload);
        }
    }

    fn on_message_listener(runtime: &mut hodo_js::ExtScriptRuntime) -> bool {
        runtime
            .eval("(__hodoState.listeners['runtime.onMessage']||[]).length")
            .ok()
            .and_then(|count| count.trim().parse::<usize>().ok())
            .is_some_and(|count| count > 0)
    }

    /// Translate a shared event into the same JS event hodo-js would fire, so
    /// the host can route it per context instead of letting one runtime drain all.
    fn event_wire(event: ExtEvent) -> Option<(String, Value)> {
        match event {
            ExtEvent::StorageChanged {
                area,
                key,
                old_value,
                new_value,
            } => Some((
                "storage.onChanged".to_string(),
                json!({
                    "area": area,
                    "changes": { key: { "oldValue": old_value, "newValue": new_value } }
                }),
            )),
            ExtEvent::AlarmFired { name } => {
                Some(("alarms.onAlarm".to_string(), json!({ "name": name })))
            }
            ExtEvent::MessageReceived { message } => {
                Some(("runtime.onMessage".to_string(), message))
            }
            ExtEvent::TabActivated { .. } | ExtEvent::NotificationShown { .. } => None,
        }
    }

    fn start_worker(&mut self, ext_index: usize) -> Result<broker::StartOutcome, String> {
        let digest = self
            .digests
            .get(ext_index)
            .cloned()
            .ok_or_else(|| "no such extension".to_string())?;
        let ready = self.ready_payload(ext_index);
        let now = self.clock.now_ms();
        let mut broker = self
            .broker
            .take()
            .ok_or_else(|| "isolation is not enabled".to_string())?;
        let result = broker.start(ext_index, &digest, ready, now, self);
        self.broker = Some(broker);
        if result.is_ok() {
            self.replay_pending_worker_events();
        }
        result
    }

    /// Drain retained events into newly live workers, keeping anything that
    /// still cannot be delivered for the next restart.
    fn replay_pending_worker_events(&mut self) {
        let Some(mut broker) = self.broker.take() else {
            return;
        };
        for ext_index in 0..self.worker_pending_events.len() {
            if !broker.is_live(ext_index) || self.worker_pending_events[ext_index].is_empty() {
                continue;
            }
            let mut undelivered = Vec::new();
            for (name, payload) in std::mem::take(&mut self.worker_pending_events[ext_index]) {
                if !broker.send_event(ext_index, &name, payload.clone()) {
                    undelivered.push((name, payload));
                }
            }
            self.worker_pending_events[ext_index] = undelivered;
        }
        self.broker = Some(broker);
    }

    fn ready_payload(&self, ext_index: usize) -> Value {
        let config = self.runtime_config(ext_index);
        let dialect = match config.dialect {
            ApiDialect::ChromeMv2 => "chrome_mv2",
            ApiDialect::ChromeMv3 => "chrome_mv3",
            ApiDialect::Gecko => "gecko",
        };
        // Without these sources the worker evaluates nothing (serde default).
        let background_scripts = match Self::background_sources(&self.extensions, ext_index) {
            Ok(sources) => sources,
            Err(err) => {
                self.diagnostics.borrow_mut().push(err);
                Vec::new()
            }
        };
        json!({
            "config": {
                "extension_name": config.extension_name,
                "idle_unload_ms": config.idle_unload_ms
            },
            "allowed_namespaces": config.allowed_namespaces,
            "host_permissions": config.host_permissions,
            "dialect": dialect,
            "background_scripts": background_scripts
        })
    }

    /// Route a worker-originated message to listener contexts. Reply
    /// correlation stays on the frozen `ExtEvent` / `ApiResponse` surfaces.
    /// Isolated targets need a broker frame — the in-process `event_queues`
    /// are never drained for a live worker.
    fn route_worker_message(&mut self, from: usize, message: Value) {
        let in_process_target = (0..self.contexts.len()).find(|&index| {
            index != from
                && self
                    .has_message_listener
                    .get(index)
                    .copied()
                    .unwrap_or(false)
        });
        if let Some(target) = in_process_target {
            self.queue_message(target, message.clone());
            if !self.context_is_live(target) && self.is_mv3(target) {
                let _ = self.run_background_scripts(target);
            }
        }
        if self.isolated {
            // A worker cannot report its JS listeners to the host, so fan out
            // to the other live workers; each runtime's onMessage handler is a
            // no-op when it has no listener for this message.
            let targets: Vec<usize> = (0..self.contexts.len())
                .filter(|&index| {
                    index != from
                        && Some(index) != in_process_target
                        && self
                            .broker
                            .as_ref()
                            .is_some_and(|broker| broker.is_live(index))
                })
                .collect();
            for target in targets {
                self.send_to_worker(target, "runtime.onMessage", message.clone());
            }
        }
    }

    fn send_to_worker(&mut self, target: usize, name: &str, payload: Value) {
        if !self.broker.as_ref().is_some_and(|b| b.is_live(target)) && self.is_mv3(target) {
            let _ = self.start_worker(target);
        }
        let now = self.clock.now_ms();
        let delivered = {
            let Some(broker) = self.broker.as_mut() else {
                return;
            };
            broker.touch(target, now);
            broker.send_event(target, name, payload.clone())
        };
        if !delivered {
            if let Some(pending) = self.worker_pending_events.get_mut(target) {
                pending.push((name.to_string(), payload));
            }
        }
    }

    /// Public test/observability surface for the supervised workers.
    #[must_use]
    pub fn worker_is_live(&self, ext_index: usize) -> bool {
        self.isolated && self.broker.as_ref().is_some_and(|b| b.is_live(ext_index))
    }

    #[must_use]
    pub fn worker_pid(&self, ext_index: usize) -> Option<u32> {
        self.broker.as_ref()?.pid(ext_index)
    }

    #[must_use]
    pub fn worker_rss_bytes(&mut self, ext_index: usize) -> Option<u64> {
        self.broker.as_mut()?.worker_rss_bytes(ext_index)
    }

    #[must_use]
    pub fn extension_disabled(&self, ext_index: usize) -> bool {
        self.disabled.get(ext_index).copied().unwrap_or(false)
    }

    pub fn shutdown_worker(&mut self, ext_index: usize) {
        let now = self.clock.now_ms();
        if let Some(broker) = self.broker.as_mut() {
            broker.shutdown(ext_index, now);
        }
    }

    pub fn terminate_worker(&mut self, ext_index: usize) {
        if let Some(broker) = self.broker.as_mut() {
            broker.terminate(ext_index);
        }
    }

    fn touch(&mut self, ext_index: usize) {
        if let Some(last) = self.last_activity.get_mut(ext_index) {
            *last = self.clock.now_ms();
        }
    }

    fn is_mv3(&self, ext_index: usize) -> bool {
        self.extensions.get(ext_index).is_some_and(|ext| {
            ext.manifest.version == ManifestVersion::V3 && ext.manifest.service_worker.is_some()
        })
    }

    fn runtime_config(&self, ext_index: usize) -> RuntimeConfig {
        let ext = &self.extensions[ext_index];
        if let Some(info) = self.runtime_infos.get(ext_index) {
            self.registry.borrow_mut().set_runtime_info(Rc::clone(info));
        }
        let allowed: Vec<String> = ext
            .manifest
            .permissions
            .iter()
            .filter_map(|p| canonical_namespace(p).map(str::to_string))
            .collect();
        RuntimeConfig {
            dialect: match ext.manifest.version {
                ManifestVersion::V2 => ApiDialect::ChromeMv2,
                ManifestVersion::V3 => ApiDialect::ChromeMv3,
            },
            extension_name: ext.manifest.name.clone(),
            allowed_namespaces: allowed,
            host_permissions: ext.manifest.host_permissions.clone(),
            idle_unload_ms: self.is_mv3(ext_index).then_some(IDLE_UNLOAD_MS),
        }
    }

    fn drain_logs(&self, runtime: &mut hodo_js::ExtScriptRuntime) {
        for line in runtime.take_logs() {
            eprintln!("hodo-ext-host: {line}");
            self.diagnostics.borrow_mut().push(line);
        }
    }

    /// Drain diagnostic lines (unhandled rejections, callback errors, JS
    /// `console.error` calls) collected from all contexts.
    pub fn take_diagnostics(&self) -> Vec<String> {
        std::mem::take(&mut self.diagnostics.borrow_mut())
    }

    fn fetch(&mut self, ext_index: usize, url: &str, method: &str, body: &str) -> ApiResult {
        let Some(ext) = self.extensions.get(ext_index).cloned() else {
            return ApiResult::Err("no such extension".into());
        };
        self.touch(ext_index);
        fetch_for(&ext, url, method, body)
    }

    pub fn required_namespace(call: &ApiCall) -> &'static str {
        match call {
            ApiCall::TabsCreate { .. }
            | ApiCall::TabsQueryActive
            | ApiCall::TabsList
            | ApiCall::TabsActivate { .. } => "tabs",
            ApiCall::StorageSet { .. }
            | ApiCall::StorageGet { .. }
            | ApiCall::StorageSyncSet { .. }
            | ApiCall::StorageSyncGet { .. }
            | ApiCall::StorageSessionSet { .. }
            | ApiCall::StorageSessionGet { .. } => "storage",
            ApiCall::AlarmCreate { .. } | ApiCall::AlarmClear { .. } => "alarms",
            ApiCall::Fetch { .. } => "fetch",
            ApiCall::NotificationsCreate { .. } => "notifications",
            ApiCall::SendMessage { .. } => "runtime",
            ApiCall::I18nGetMessage { .. } => "i18n",
            ApiCall::RuntimeGetUrl { .. } | ApiCall::RuntimeGetManifest | ApiCall::RuntimeId => {
                "runtime"
            }
        }
    }
}

/// Stable in-process digest of the exact manifest bytes. The worker echoes it
/// in `Hello`; no body-supplied digest is ever trusted.
fn manifest_digest(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

impl broker::BrokerHost for Host {
    fn dispatch_wire_call(&mut self, ext_index: usize, method: &str, params: Value) -> ApiResult {
        let calls = match hodo_js::decode_api_call(method, params) {
            Ok(calls) => calls,
            Err(err) => return ApiResult::Err(err),
        };
        let mut last = Value::Null;
        for call in calls {
            match self.dispatch(ext_index, &call) {
                ApiResult::Ok(value) => last = value,
                ApiResult::Err(err) => return ApiResult::Err(err),
            }
        }
        ApiResult::Ok(last)
    }

    fn route_worker_event(&mut self, ext_index: usize, name: &str, payload: Value) {
        match name {
            "runtime.onMessage" => self.route_worker_message(ext_index, payload),
            _ => self.emit_local_event(ext_index, name, payload),
        }
    }

    fn diagnostic(&mut self, ext_index: usize, line: String) {
        let name = self.extensions.get(ext_index).map_or_else(
            || format!("ext {ext_index}"),
            |ext| ext.manifest.name.clone(),
        );
        self.diagnostics
            .borrow_mut()
            .push(format!("{name}: {line}"));
    }

    fn disable(&mut self, ext_index: usize, reason: String) {
        if let Some(disabled) = self.disabled.get_mut(ext_index) {
            *disabled = true;
        }
        self.diagnostic(ext_index, format!("extension disabled: {reason}"));
        // Tell surviving contexts; a disabled extension can never be woken.
        if let Some(broker) = self.broker.as_mut() {
            for index in 0..self.contexts.len() {
                if index != ext_index && !self.disabled.get(index).copied().unwrap_or(false) {
                    broker.send_event(
                        index,
                        "__hodoExtensionDisabled",
                        json!({ "extension": ext_index, "reason": reason }),
                    );
                }
            }
        }
    }

    fn ready_payload(&self, ext_index: usize) -> Option<(String, Value)> {
        if self.disabled.get(ext_index).copied().unwrap_or(true) {
            return None;
        }
        Some((
            self.digests.get(ext_index).cloned()?,
            Host::ready_payload(self, ext_index),
        ))
    }
}

/// Shared by direct host dispatch and the per-runtime JS fetch hook, so the
/// permission check cannot be bypassed by script-originated calls.
fn fetch_for(ext: &Extension, url: &str, method: &str, body: &str) -> ApiResult {
    if !url_allowed(ext, url) {
        return ApiResult::Err("fetch denied: host permission required".into());
    }
    let method = method.to_ascii_uppercase();
    if method != "GET" && method != "POST" {
        return ApiResult::Err(format!("fetch: unsupported method {method}"));
    }
    if body.len() > hodo_net::MAX_BYTES || url.len() > hodo_net::MAX_BYTES {
        return ApiResult::Err("fetch: request exceeds 2 MiB cap".into());
    }
    match fetch_bytes(url, &method, body) {
        Ok((status, bytes)) => {
            let text = bytes
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>();
            ApiResult::Ok(
                json!({ "ok": (200..300).contains(&status), "status": status, "body": text }),
            )
        }
        Err(err) => ApiResult::Err(format!("fetch: {err}")),
    }
}

/// Never weaken the manifest's CSP below the MV3/MV2 defaults.
fn validate_csp(json: &str, version: ManifestVersion) -> Result<(), HostError> {
    let root: Value = serde_json::from_str(json).map_err(|err| HostError::Csp(err.to_string()))?;
    let Some(csp) = root.get("content_security_policy").and_then(Value::as_str) else {
        return Ok(());
    };
    let lower = csp.to_ascii_lowercase();
    for token in [
        "'unsafe-eval'",
        "'unsafe-inline'",
        "'wasm-unsafe-eval'",
        "blob:",
        "data:",
        "http:",
        "https:",
    ] {
        if lower.contains(token) {
            return Err(HostError::Csp(format!("forbidden CSP token {token}")));
        }
    }
    if !lower.contains("'self'") && !lower.contains("'none'") {
        return Err(HostError::Csp(
            "script sources must be 'self' or 'none'".into(),
        ));
    }
    if version == ManifestVersion::V2 && !lower.contains("object-src") {
        return Err(HostError::Csp("MV2 requires object-src 'self'".into()));
    }
    Ok(())
}

/// Match a URL against `host_permissions` (or a `<all_urls>` permission).
/// Credentials in the URL are always rejected.
fn url_allowed(ext: &Extension, url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if !matches!(scheme, "http" | "https" | "file") || url.contains('@') {
        return false;
    }
    let host = rest.split('/').next().unwrap_or("");
    let path = rest.find('/').map_or("", |index| &rest[index..]);
    ext.manifest
        .host_permissions
        .iter()
        .chain(ext.manifest.permissions.iter())
        .any(|pattern| {
            if pattern == "<all_urls>" {
                return matches!(scheme, "http" | "https");
            }
            let Some((pattern_scheme, pattern_rest)) = pattern.split_once("://") else {
                return false;
            };
            if pattern_scheme != "*" && pattern_scheme != scheme {
                return false;
            }
            let pattern_host = pattern_rest.split('/').next().unwrap_or("");
            let host_ok = if let Some(suffix) = pattern_host.strip_prefix("*.") {
                host == suffix || host.ends_with(&format!(".{suffix}"))
            } else {
                pattern_host == "*" || pattern_host.eq_ignore_ascii_case(host)
            };
            if !host_ok {
                return false;
            }
            let pattern_path = pattern_rest
                .find('/')
                .map_or("", |index| &pattern_rest[index..]);
            path.starts_with(pattern_path.trim_end_matches('*'))
        })
}

/// GET goes through the frozen `hodo_net::load` (2 MiB cap, shared agent).
/// POST uses the same pinned ureq version until hodo-net exposes a body API.
fn fetch_bytes(url: &str, method: &str, body: &str) -> Result<(u16, Vec<u8>), String> {
    if method == "GET" {
        // `file:///` + `/C:/...` is a Windows path shape hodo-net cannot open;
        // hand it `file://C:/...` instead (still no host-side path escaping).
        let load_url = if let Some(rest) = url.strip_prefix("file:///") {
            let bytes = rest.as_bytes();
            if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
                format!("file://{rest}")
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        };
        let resource = hodo_net::load(&load_url).map_err(|err| err.to_string())?;
        let bytes = match resource {
            hodo_net::Resource::Html(text) => text.into_bytes(),
            hodo_net::Resource::Other { data, .. } => data,
        };
        return Ok((200, bytes));
    }
    let mut response = ureq::post(url)
        .send(body.to_string())
        .map_err(|err| err.to_string())?;
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = response
            .body_mut()
            .as_reader()
            .read(&mut chunk)
            .map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > hodo_net::MAX_BYTES {
            return Err("resource exceeds cap".into());
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    Ok((status, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_ext_api::FakeClock;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    /// Unit tests assert in-process script effects; the browser default is now
    /// isolated, so opt out explicitly here.
    fn in_process_host() -> Host {
        HostBuilder::new().isolated(false).build()
    }

    #[test]
    fn isolation_is_on_by_default() {
        // HODO_NO_ISOLATION is the documented opt-out; the builder default must
        // be isolated so background scripts run in worker processes.
        if std::env::var("HODO_NO_ISOLATION").is_ok_and(|v| v == "1") {
            return;
        }
        assert!(HostBuilder::new().build().broker.is_some());
    }

    fn repo_ext(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../extensions")
            .join(name)
    }

    fn storage_get(host: &mut Host, key: &str) -> Option<String> {
        match host.dispatch(0, &ApiCall::StorageGet { key: key.into() }) {
            ApiResult::Ok(value) => value.as_str().map(str::to_owned),
            ApiResult::Err(err) => panic!("StorageGet({key}) failed: {err}"),
        }
    }

    struct TempExt {
        root: PathBuf,
    }

    impl TempExt {
        fn new(manifest: &str) -> Self {
            let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("hodo-ext-host-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("manifest.json"), manifest).unwrap();
            Self { root }
        }
    }

    impl Drop for TempExt {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    const CHROME_STORAGE: &str = r#"{
        "manifest_version": 3,
        "name": "Chrome Storage",
        "version": "1.0.0",
        "permissions": ["storage"],
        "background": { "service_worker": "missing-sw.js" }
    }"#;

    const GECKO_TABS: &str = r#"{
        "manifest_version": 2,
        "name": "Gecko Tabs",
        "version": "1.0.0",
        "permissions": ["tabs", "storage"],
        "browser_specific_settings": { "gecko": { "id": "gecko@example.test" } },
        "background": { "scripts": ["missing-bg.js"] }
    }"#;

    const NO_PERMISSIONS: &str = r#"{
        "manifest_version": 3,
        "name": "No Permissions",
        "version": "1.0.0",
        "permissions": []
    }"#;

    const NO_PERMISSIONS_BG: &str = r#"{
        "manifest_version": 3,
        "name": "No Permissions Background",
        "version": "1.0.0",
        "permissions": [],
        "background": { "scripts": ["bg.js"] }
    }"#;

    const HOST_PERMISSION_ONLY: &str = r#"{
        "manifest_version": 3,
        "name": "Host Permission Only",
        "version": "1.0.0",
        "host_permissions": ["storage"],
        "permissions": []
    }"#;

    const NOTIFICATIONS_ONLY: &str = r#"{
        "manifest_version": 3,
        "name": "Notifications Only",
        "version": "1.0.0",
        "permissions": ["notifications"]
    }"#;

    fn assert_ok(result: ApiResult, label: &str) {
        if let ApiResult::Err(msg) = result {
            panic!("{label}: expected Ok, got Err({msg})");
        }
    }

    fn assert_err_contains(result: ApiResult, needle: &str, label: &str) {
        match result {
            ApiResult::Err(msg) => assert!(
                msg.contains(needle),
                "{label}: expected '{msg}' to contain '{needle}'"
            ),
            ApiResult::Ok(_) => panic!("{label}: expected Err, got Ok"),
        }
    }

    #[test]
    fn chrome_storage_allows_storage_but_denies_tabs() {
        let tmp = TempExt::new(CHROME_STORAGE);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).expect("chrome ext should load");
        assert_eq!(host.extensions().len(), 1);

        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_ok(host.dispatch(0, &set), "StorageSet");

        let tabs = ApiCall::TabsCreate {
            url: "https://example.test".into(),
        };
        assert_err_contains(host.dispatch(0, &tabs), "permission denied", "TabsCreate");
    }

    #[test]
    fn gecko_tabs_and_storage_both_allowed() {
        let tmp = TempExt::new(GECKO_TABS);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).expect("gecko ext should load");

        let tabs = ApiCall::TabsCreate {
            url: "https://example.test".into(),
        };
        assert_ok(host.dispatch(0, &tabs), "TabsCreate");
        assert_ok(
            host.dispatch(0, &ApiCall::TabsQueryActive),
            "TabsQueryActive",
        );

        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_ok(host.dispatch(0, &set), "StorageSet");
        assert_ok(
            host.dispatch(0, &ApiCall::StorageGet { key: "k".into() }),
            "StorageGet",
        );
    }

    #[test]
    fn runtime_always_allowed_without_permissions() {
        let tmp = TempExt::new(NO_PERMISSIONS);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).expect("no-perm ext should load");

        let msg = ApiCall::SendMessage {
            message: Default::default(),
        };
        assert_ok(host.dispatch(0, &msg), "SendMessage");
    }

    #[test]
    fn host_permissions_do_not_grant_api_access() {
        let tmp = TempExt::new(HOST_PERMISSION_ONLY);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).expect("ext should load");
        let set = ApiCall::StorageSet {
            key: "k".into(),
            value: Default::default(),
        };
        assert_err_contains(
            host.dispatch(0, &set),
            "permission denied",
            "StorageSet via host_permissions",
        );
    }

    #[test]
    fn notifications_permission_allows_notifications_but_denies_storage() {
        let tmp = TempExt::new(NOTIFICATIONS_ONLY);
        let mut host = in_process_host();
        host.load_dir(&tmp.root)
            .expect("notifications ext should load");

        assert_ok(
            host.dispatch(
                0,
                &ApiCall::NotificationsCreate {
                    title: "t".into(),
                    message: "m".into(),
                },
            ),
            "NotificationsCreate",
        );
        assert_err_contains(
            host.dispatch(
                0,
                &ApiCall::StorageSet {
                    key: "k".into(),
                    value: Default::default(),
                },
            ),
            "permission denied",
            "StorageSet with notifications-only permission",
        );
    }

    #[test]
    fn oversized_manifest_is_rejected() {
        let tmp = TempExt::new("");
        fs::write(
            tmp.root.join("manifest.json"),
            vec![b' '; MAX_MANIFEST_BYTES + 1],
        )
        .unwrap();
        let mut host = in_process_host();
        assert!(matches!(
            host.load_dir(&tmp.root),
            Err(HostError::Io(msg)) if msg == "manifest too large"
        ));
    }

    #[test]
    fn bad_index_is_an_error() {
        let mut host = in_process_host();
        let msg = ApiCall::SendMessage {
            message: Default::default(),
        };
        assert_err_contains(host.dispatch(0, &msg), "no such extension", "bad index");
    }

    #[test]
    fn malformed_manifest_surfaces_manifest_error() {
        let tmp = TempExt::new("{ this is not valid json");
        let mut host = in_process_host();
        assert!(matches!(
            host.load_dir(&tmp.root),
            Err(HostError::Manifest(_)) | Err(HostError::Csp(_))
        ));
    }

    #[test]
    fn weaker_csp_is_rejected() {
        let manifest = r#"{
            "manifest_version": 3, "name": "Weak CSP", "version": "1.0.0",
            "content_security_policy": "script-src 'self' 'unsafe-eval' https://cdn.example"
        }"#;
        let tmp = TempExt::new(manifest);
        let mut host = in_process_host();
        assert!(matches!(host.load_dir(&tmp.root), Err(HostError::Csp(_))));

        let ok = r#"{
            "manifest_version": 3, "name": "Strong CSP", "version": "1.0.0",
            "content_security_policy": "script-src 'self'; object-src 'none'"
        }"#;
        let tmp_ok = TempExt::new(ok);
        assert!(in_process_host().load_dir(&tmp_ok.root).is_ok());
    }

    #[test]
    fn host_resolves_action_title_at_load_time() {
        let mut host = in_process_host();
        host.load_dir(&repo_ext("i18n-gecko")).unwrap();
        assert_eq!(
            host.extensions()[0].manifest.action_title.as_deref(),
            Some("i18n Demo")
        );
        assert_eq!(host.extension_title(0), Some("i18n Demo"));
    }

    #[test]
    fn per_context_event_queue_waits_for_the_listener_context() {
        let tmp = TempExt::new(CHROME_STORAGE);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).unwrap();
        host.queue_message(0, json!("popup-message"));
        host.queue_event(0, "probe", json!({"from": "popup"}));
        assert_eq!(host.event_queues[0].len(), 2);

        let runtime = hodo_js::ExtScriptRuntime::with_allowed(&["storage", "runtime"]);
        host.attach_context(0, runtime);
        assert!(
            host.event_queues[0].is_empty(),
            "the attached context must drain its own queue"
        );
    }

    #[test]
    fn required_namespace_mapping() {
        assert_eq!(
            Host::required_namespace(&ApiCall::StorageSet {
                key: "k".into(),
                value: Default::default(),
            }),
            "storage"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::StorageGet { key: "k".into() }),
            "storage"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::TabsCreate { url: "u".into() }),
            "tabs"
        );
        assert_eq!(Host::required_namespace(&ApiCall::TabsQueryActive), "tabs");
        assert_eq!(
            Host::required_namespace(&ApiCall::SendMessage {
                message: Default::default()
            }),
            "runtime"
        );
        assert_eq!(
            Host::required_namespace(&ApiCall::Fetch {
                url: "u".into(),
                method: "GET".into(),
                body: String::new()
            }),
            "fetch"
        );
    }

    #[test]
    fn real_hello_gecko_background_writes_visible_storage() {
        let mut host = in_process_host();
        host.load_dir(&repo_ext("hello-gecko"))
            .expect("hello-gecko should load");
        host.run_background_scripts(0)
            .expect("hello-gecko bg.js should evaluate");

        // StorageChanged events are delivered to JS listeners during `tick`,
        // so the registry queue is empty by the time the host looks at it.
        assert!(host.registry.borrow_mut().drain_events().is_empty());
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
    }

    #[test]
    fn real_hello_chrome_service_worker_writes_visible_storage() {
        let mut host = in_process_host();
        host.load_dir(&repo_ext("hello-chrome"))
            .expect("hello-chrome should load");
        host.run_background_scripts(0)
            .expect("hello-chrome bg.js should evaluate");
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello from a Chrome-style MV3 extension")
        );
    }

    #[test]
    fn permission_filter_blocks_unpermitted_js_namespace() {
        let tmp = TempExt::new(NO_PERMISSIONS_BG);
        fs::write(
            tmp.root.join("bg.js"),
            "chrome.storage.local.set({ blocked: 1 });",
        )
        .unwrap();
        let mut host = in_process_host();
        host.load_dir(&tmp.root).expect("ext should load");

        let err = host
            .run_background_scripts(0)
            .expect_err("storage must be filtered out of the JS globals");
        assert!(
            err.contains("TypeError") || err.contains("undefined"),
            "unexpected error: {err}"
        );
        assert!(host.registry.borrow_mut().drain_events().is_empty());
        let probe = host.registry.borrow_mut().dispatch(&ApiCall::StorageGet {
            key: "blocked".into(),
        });
        match probe {
            ApiResult::Ok(value) => assert!(value.is_null()),
            ApiResult::Err(err) => panic!("unexpected registry error: {err}"),
        }
    }

    #[test]
    fn running_background_twice_replays_events_without_accumulating() {
        let mut host = in_process_host();
        host.load_dir(&repo_ext("hello-gecko")).unwrap();
        host.run_background_scripts(0).unwrap();
        let first = host.registry.borrow_mut().drain_events();
        host.run_background_scripts(0).unwrap();
        let second = host.registry.borrow_mut().drain_events();

        assert!(first.is_empty());
        assert!(second.is_empty());
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
        assert_eq!(
            storage_get(&mut host, "greeting").as_deref(),
            Some("Hello Gecko")
        );
    }

    #[test]
    fn no_background_scripts_is_not_an_error() {
        let tmp = TempExt::new(NO_PERMISSIONS);
        let mut host = in_process_host();
        host.load_dir(&tmp.root).unwrap();
        assert_eq!(
            host.run_background_scripts(0).unwrap(),
            "no background scripts"
        );
    }

    #[test]
    fn all_bundled_demo_extensions_execute() {
        let demos = [
            "hello-chrome",
            "hello-gecko",
            "i18n-gecko",
            "messages-gecko",
            "notify-chrome",
            "popup-chrome",
            "storage-demo-gecko",
            "tabs-demo-chrome",
            "async-chrome",
            "promises-gecko",
        ];
        let mut host = in_process_host();
        for name in demos {
            host.load_dir(&repo_ext(name))
                .unwrap_or_else(|err| panic!("{name}: {err}"));
            let index = host.extensions().len() - 1;
            host.run_background_scripts(index)
                .unwrap_or_else(|err| panic!("{name}: {err}"));
            host.tick();
        }
        assert_eq!(host.extensions().len(), 10);
    }

    #[test]
    fn mv3_idle_unloads_and_event_wakes_context() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        assert!(host.context_is_live(0));
        assert_eq!(storage_get(&mut host, "mode").as_deref(), Some("async"));

        clock.set_ms(IDLE_UNLOAD_MS + 1);
        host.tick();
        assert!(!host.context_is_live(0), "idle MV3 worker must unload");

        host.emit_event(0, "probe", json!({"wake": true}));
        assert!(
            host.context_is_live(0),
            "subscribed event must wake the worker"
        );
        assert_eq!(storage_get(&mut host, "mode").as_deref(), Some("async"));
    }

    #[test]
    fn mv2_context_stays_persistent_when_idle() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("hello-gecko")).unwrap();
        host.run_background_scripts(0).unwrap();
        clock.set_ms(IDLE_UNLOAD_MS * 10);
        host.tick();
        assert!(
            host.context_is_live(0),
            "MV2 background pages are persistent"
        );
    }

    #[test]
    fn alarms_drain_on_tick_without_sleeping() {
        let clock = Rc::new(FakeClock::new(0));
        let mut host = Host::with_clock(clock.clone());
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        host.dispatch(
            0,
            &ApiCall::AlarmCreate {
                name: "tick".into(),
                delay_ms: 5,
                period_ms: None,
                now_ms: 0,
            },
        );
        clock.set_ms(5);
        host.tick();
        match host.dispatch(
            0,
            &ApiCall::StorageGet {
                key: "lastAlarm".into(),
            },
        ) {
            ApiResult::Ok(value) => assert_eq!(value, json!("tick")),
            ApiResult::Err(err) => panic!("alarm listener did not run: {err}"),
        }
    }

    #[test]
    fn fetch_denied_without_host_permission() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "No Fetch", "version": "1.0.0",
                "permissions": [],
                "background": { "service_worker": "bg.js" }
            }"#,
        );
        fs::write(
            tmp.root.join("bg.js"),
            "fetch('https://example.test/data.json').then(function (r) { __hodo_log('unexpected'); }, function (e) { __hodo_log('denied:' + e.message); });",
        )
        .unwrap();
        let mut host = in_process_host();
        host.load_dir(&tmp.root).unwrap();
        host.run_background_scripts(0).unwrap();
        host.tick();
        let logs = host.take_diagnostics();
        assert!(
            logs.iter().any(|line| line.contains("fetch denied")),
            "unexpected logs: {logs:?}"
        );
        assert_err_contains(
            host.dispatch(
                0,
                &ApiCall::Fetch {
                    url: "https://example.test/".into(),
                    method: "GET".into(),
                    body: String::new(),
                },
            ),
            "fetch denied",
            "direct fetch dispatch",
        );
    }

    #[test]
    fn fetch_allowed_with_host_permission_reads_local_file() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "Fetch File", "version": "1.0.0",
                "host_permissions": ["file:///*"],
                "background": { "service_worker": "bg.js" }
            }"#,
        );
        let data_path = tmp.root.join("data.txt");
        fs::write(&data_path, "hello from fetch").unwrap();
        let url = format!("file:///{}", data_path.to_string_lossy().replace('\\', "/"));
        let script = format!(
            "fetch('{url}').then(function (r) {{ return r.text(); }}).then(function (t) {{ __hodo_log('text:' + t); }}, function (e) {{ __hodo_log('err:' + e.message); }});"
        );
        fs::write(tmp.root.join("bg.js"), script).unwrap();
        let mut host = in_process_host();
        host.load_dir(&tmp.root).unwrap();
        host.run_background_scripts(0).unwrap();
        host.tick();
        let logs = host.take_diagnostics();
        assert!(
            logs.iter()
                .any(|line| line.contains("text:hello from fetch")),
            "unexpected logs: {logs:?}"
        );
    }

    /// Manual timing probe (not part of the fast suite): `cargo test -p
    /// hodo-ext-host api_roundtrip_timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn api_roundtrip_timing() {
        use std::time::Instant;

        const N: u32 = 1000;
        let mut host = in_process_host();
        host.load_dir(&repo_ext("async-chrome")).unwrap();
        host.run_background_scripts(0).unwrap();
        let mut runtime = hodo_js::ExtScriptRuntime::with_allowed_and_registry(
            &["storage"],
            Rc::clone(&host.registry),
        );

        let started = Instant::now();
        for i in 0..N {
            runtime
                .eval(&format!("browser.storage.local.set({{roundtrip: {i}}});"))
                .unwrap();
        }
        let js_each = started.elapsed().as_secs_f64() * 1e6 / f64::from(N);

        let started = Instant::now();
        for i in 0..N {
            assert_ok(
                host.dispatch(
                    0,
                    &ApiCall::StorageSet {
                        key: "roundtrip".into(),
                        value: json!(i),
                    },
                ),
                "timed dispatch",
            );
        }
        let host_each = started.elapsed().as_secs_f64() * 1e6 / f64::from(N);
        println!(
            "API roundtrip: JS eval+tick {js_each:.1} us/op, host dispatch {host_each:.1} us/op"
        );
    }

    #[test]
    fn url_permission_matching_rejects_credentials_and_wrong_hosts() {
        let tmp = TempExt::new(
            r#"{
                "manifest_version": 3, "name": "Match", "version": "1.0.0",
                "host_permissions": ["https://example.com/*"]
            }"#,
        );
        let mut host = in_process_host();
        host.load_dir(&tmp.root).unwrap();
        let ext = host.extensions()[0].clone();
        assert!(url_allowed(&ext, "https://example.com/a"));
        assert!(!url_allowed(&ext, "https://example.com.evil.test/a"));
        assert!(!url_allowed(&ext, "https://user:pass@example.com/a"));
        assert!(!url_allowed(&ext, "file:///C:/secret.txt"));
    }
}

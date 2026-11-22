//! In-process half of extension isolation: spawn, handshake, frame routing,
//! lifecycle and resource watchdogs. The worker is untrusted throughout; every
//! permission decision is delegated back to `Host::dispatch`.

use std::path::PathBuf;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use hodo_ext_api::ApiResult;
use hodo_ext_ipc::{
    HandshakeState, Message, PendingCalls, MAX_EXT_CONTEXTS, MAX_OUTSTANDING_CALLS,
};
use serde_json::{json, Value};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

use crate::worker::{self, SpawnSpec, WorkerProc};

/// Restart delays after a crash, in milliseconds.
pub const BACKOFF_MS: [u64; 3] = [100, 200, 400];
/// An API call that has not been answered (and has produced no frames) hangs.
pub const HANG_TIMEOUT_MS: u64 = 5_000;
/// Broker-side ping cadence; a worker with a JS bridge replies with a pong.
pub const PING_INTERVAL_MS: u64 = 1_000;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const RSS_POLL_MS: u64 = 500;
const RSS_TREND_SAMPLES: usize = 3;
const MIN_RSS_BUDGET: u64 = 64 * 1024 * 1024;
const SHUTDOWN_GRACE_MS: u64 = 250;

/// A message whose serialized frame body would exceed the IPC cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TooBig {
    pub len: usize,
}

/// Serialize `msg` only when its frame body fits the IPC cap. `encode()` adds a
/// one-byte version prefix, so the bound is `json.len() + 1`. Every host→worker
/// write path calls this before `WorkerProc::send` so an oversized payload
/// degrades to a diagnostic instead of panicking the host in `encode`'s assert.
pub(crate) fn try_encode_bounded(msg: &Message) -> Result<Vec<u8>, TooBig> {
    let body = serde_json::to_vec(msg).expect("serializing a Message cannot fail");
    let len = body.len() + 1; // VERSION byte prepended by `encode`
    if len > hodo_ext_ipc::MAX_MSG_BYTES {
        Err(TooBig { len })
    } else {
        Ok(body)
    }
}

/// The broker asks the host for these operations instead of knowing about
/// `Host` itself, so frame routing is testable without spawning anything.
pub trait BrokerHost {
    /// Decode and dispatch through the real permission gate.
    fn dispatch_wire_call(&mut self, ext_index: usize, method: &str, params: Value) -> ApiResult;
    /// Route a non-protocol browser event (messages, notifications, ...).
    fn route_worker_event(&mut self, ext_index: usize, name: &str, payload: Value);
    fn diagnostic(&mut self, ext_index: usize, line: String);
    fn disable(&mut self, ext_index: usize, reason: String);
    /// Manifest digest and the real `__hodo_ready` payload for a (re)spawn.
    fn ready_payload(&self, ext_index: usize) -> Option<(String, Value)>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeVerdict {
    Accept,
    Fatal(&'static str),
}

/// Compare two secrets without leaking their contents through an early exit.
/// Lengths are folded in too, so a short token can never match a long one by
/// reading only the short prefix.
#[must_use]
fn constant_time_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    let mut diff = left.len() ^ right.len();
    for index in 0..left.len().max(right.len()) {
        diff |= usize::from(
            left.get(index).copied().unwrap_or(0) ^ right.get(index).copied().unwrap_or(0),
        );
    }
    diff == 0
}

/// Validate the *first* frame from a freshly spawned channel. The expected
/// token and digest come from the spawn, never from the message body.
#[must_use]
pub fn handshake_verdict(
    state: &HandshakeState,
    msg: &Message,
    expected_token: &str,
    expected_digest: &str,
) -> HandshakeVerdict {
    if state.saw_hello() {
        return HandshakeVerdict::Fatal("duplicate hello");
    }
    match msg {
        Message::Hello {
            token,
            manifest_digest,
        } if constant_time_eq(token, expected_token)
            && constant_time_eq(manifest_digest, expected_digest) =>
        {
            HandshakeVerdict::Accept
        }
        Message::Hello { .. } => HandshakeVerdict::Fatal("bad token or manifest digest"),
        _ => HandshakeVerdict::Fatal("first frame was not hello"),
    }
}

/// Pure routing table for worker-to-broker frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameAction {
    Fatal(&'static str),
    ApiCall {
        id: u64,
        method: String,
        params: Value,
    },
    Log(String),
    Ready,
    Pong,
    Event {
        name: String,
        payload: Value,
    },
    ApiResponse {
        id: u64,
    },
    Shutdown,
}

#[must_use]
pub fn classify_worker_frame(msg: Message) -> FrameAction {
    match msg {
        Message::Hello { .. } => FrameAction::Fatal("unexpected hello after handshake"),
        Message::ApiCall { id: 0, .. } => FrameAction::Fatal("api call id must be >= 1"),
        Message::ApiCall { id, method, params } => FrameAction::ApiCall { id, method, params },
        Message::ApiResponse { id, .. } => FrameAction::ApiResponse { id },
        Message::Event { name, payload } => match name.as_str() {
            "__hodo_log" => FrameAction::Log(
                payload
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
            "__hodo_ready" => FrameAction::Ready,
            "__hodo_pong" => FrameAction::Pong,
            _ => FrameAction::Event { name, payload },
        },
        Message::Shutdown => FrameAction::Shutdown,
    }
}

/// Outcome of a (re)spawn request. A crash budget may defer the spawn without
/// it being an error; callers must not report a live worker in that case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartOutcome {
    Started,
    Deferred { retry_at: u64 },
}

/// 100 / 200 / 400 ms, then give up on the extension (never on the browser).
#[derive(Debug, Default)]
pub struct Backoff {
    attempt: usize,
    next_at_ms: Option<u64>,
}

impl Backoff {
    /// Returns `false` when the retry budget is exhausted.
    pub fn schedule(&mut self, now: u64) -> bool {
        if self.attempt >= BACKOFF_MS.len() {
            return false;
        }
        self.next_at_ms = Some(now.saturating_add(BACKOFF_MS[self.attempt]));
        self.attempt += 1;
        true
    }

    #[must_use]
    pub fn due(&self, now: u64) -> bool {
        self.next_at_ms.is_some_and(|at| now >= at)
    }

    /// Earliest time a deferred restart may be attempted (for diagnostics).
    #[must_use]
    pub fn next_at(&self) -> u64 {
        self.next_at_ms.unwrap_or(0)
    }

    pub fn take(&mut self) {
        self.next_at_ms = None;
    }

    #[must_use]
    pub fn exhausted(&self) -> bool {
        self.attempt >= BACKOFF_MS.len()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// MV3 idle bookkeeping: `idle_ms` is `None` for persistent contexts.
#[must_use]
pub fn should_unload(now: u64, last_activity: u64, idle_ms: Option<u64>) -> bool {
    idle_ms.is_some_and(|ms| now.saturating_sub(last_activity) > ms)
}

/// RSS watchdog using measured baseline × the context cap, with a trend so a
/// transient allocation spike does not kill a healthy worker.
#[derive(Debug, Default)]
pub struct RssWatchdog {
    baseline: Option<u64>,
    over: usize,
}

impl RssWatchdog {
    /// Returns `true` when the worker must be killed.
    pub fn observe(&mut self, bytes: u64) -> bool {
        let baseline = (*self.baseline.get_or_insert(bytes)).max(1);
        let budget = (baseline.saturating_mul(MAX_EXT_CONTEXTS as u64)).max(MIN_RSS_BUDGET);
        if bytes > budget {
            self.over += 1;
        } else {
            self.over = 0;
        }
        self.over >= RSS_TREND_SAMPLES
    }

    #[must_use]
    pub fn baseline(&self) -> Option<u64> {
        self.baseline
    }
}

#[derive(Default)]
struct Slot {
    proc: Option<worker::WorkerProc>,
    handshake: HandshakeState,
    pending: PendingCalls,
    backoff: Backoff,
    rss: RssWatchdog,
    last_activity: u64,
    started_at: u64,
    last_call: Option<u64>,
    last_ping: u64,
    next_rss_poll: u64,
    /// Deadline for a pending graceful shutdown, or for the hard kill after
    /// the `Shutdown` frame was actually written. `None` means no shutdown.
    shutdown_at: Option<u64>,
    /// `Shutdown` already queued: cancel is no longer possible, new events must
    /// be replayed on the next spawn instead of written to a doomed pipe.
    shutdown_sent: bool,
    idle_ms: Option<u64>,
}

pub struct Broker {
    slots: Vec<Slot>,
    worker_path: PathBuf,
    extra_env: Vec<(String, String)>,
    system: Option<System>,
}

impl Default for Broker {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            worker_path: worker::default_worker_path(),
            extra_env: Vec::new(),
            system: None,
        }
    }
}

impl Broker {
    pub fn set_worker_path(&mut self, path: PathBuf) {
        self.worker_path = path;
    }

    /// Test-only environment injection (crash injection in the e2e harness).
    pub fn set_worker_env(&mut self, key: &str, value: &str) {
        self.extra_env.push((key.to_string(), value.to_string()));
    }

    pub fn ensure(&mut self, ext_index: usize) {
        if self.slots.len() <= ext_index {
            self.slots.resize_with(ext_index + 1, Slot::default);
        }
    }

    #[must_use]
    pub fn is_live(&self, ext_index: usize) -> bool {
        self.slots.get(ext_index).is_some_and(|s| s.proc.is_some())
    }

    #[must_use]
    pub fn pid(&self, ext_index: usize) -> Option<u32> {
        self.slots
            .get(ext_index)
            .and_then(|s| s.proc.as_ref())
            .map(worker::WorkerProc::pid)
    }

    /// Start (or restart) a worker. A live channel is a no-op reported as
    /// [`StartOutcome::Started`]; a crash budget that has not elapsed yet
    /// returns [`StartOutcome::Deferred`] so the host can retain and replay
    /// events instead of pretending the worker woke up.
    pub fn start<H: BrokerHost>(
        &mut self,
        ext_index: usize,
        digest: &str,
        ready: Value,
        now: u64,
        host: &mut H,
    ) -> Result<StartOutcome, String> {
        self.ensure(ext_index);
        if self.is_live(ext_index) {
            return Ok(StartOutcome::Started);
        }
        let slot = &mut self.slots[ext_index];
        if slot.backoff.attempt > 0 && !slot.backoff.due(now) {
            let retry_at = slot.backoff.next_at();
            host.diagnostic(
                ext_index,
                format!("worker start deferred until {retry_at}ms; event retained"),
            );
            return Ok(StartOutcome::Deferred { retry_at });
        }
        let token = worker::session_token();
        let spec = SpawnSpec {
            worker_path: self.worker_path.clone(),
            token: token.clone(),
            digest: digest.to_string(),
            extra_env: self.extra_env.clone(),
        };
        let proc = match worker::spawn(&spec) {
            Ok(proc) => proc,
            Err(err) => {
                self.crash(ext_index, &format!("spawn failed: {err}"), now, host);
                return Err(format!("spawn failed: {err}"));
            }
        };
        slot.proc = Some(proc);
        slot.handshake = HandshakeState::new();
        slot.pending = PendingCalls::new();
        slot.rss = RssWatchdog::default();
        slot.last_call = None;
        slot.shutdown_at = None;
        slot.shutdown_sent = false;
        slot.next_rss_poll = 0;
        slot.last_activity = now;
        slot.started_at = now;
        slot.last_ping = now;
        slot.idle_ms = ready
            .get("config")
            .and_then(|c| c.get("idle_unload_ms"))
            .and_then(Value::as_u64);

        // The worker logs sandbox setup before emitting Hello; those lines are
        // diagnostics, not handshake frames. Anything else before Hello is fatal.
        let mut handshake_logs = Vec::new();
        let first = loop {
            let outcome = self.slots[ext_index]
                .proc
                .as_mut()
                .expect("just assigned")
                .frames
                .recv_timeout(HANDSHAKE_TIMEOUT);
            match outcome {
                Ok(Ok(Message::Event { name, payload })) if name == "__hodo_log" => {
                    handshake_logs.push(
                        payload
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                Ok(Ok(msg)) => break Ok(msg),
                Ok(Err(err)) => break Err(err),
                Err(RecvTimeoutError::Timeout) => break Err("handshake timeout".to_string()),
                Err(RecvTimeoutError::Disconnected) => {
                    break Err("handshake channel closed".to_string())
                }
            }
        };
        for line in handshake_logs {
            host.diagnostic(ext_index, line);
        }
        let first = match first {
            Ok(msg) => msg,
            Err(reason) => {
                self.crash(ext_index, &reason, now, host);
                return Err(reason);
            }
        };
        let verdict = {
            let state = &self.slots[ext_index].handshake;
            handshake_verdict(state, &first, &token, digest)
        };
        if let HandshakeVerdict::Fatal(reason) = verdict {
            self.crash(ext_index, reason, now, host);
            return Err(reason.to_string());
        }
        {
            let state = &mut self.slots[ext_index].handshake;
            let _ = state.observe(&first);
            state.mark_ready();
        }
        let ready_msg = Message::Event {
            name: "__hodo_ready".to_string(),
            payload: ready,
        };
        // `encode` asserts on an oversized body; the shared bounded guard keeps
        // every host→worker write (including ready) below the IPC cap.
        if let Err(too_big) = try_encode_bounded(&ready_msg) {
            let reason = format!(
                "ready payload is {} bytes, exceeding the {} byte IPC cap",
                too_big.len,
                hodo_ext_ipc::MAX_MSG_BYTES
            );
            self.kill_and_clear(ext_index);
            host.diagnostic(ext_index, reason.clone());
            return Err(reason);
        }
        let sent = self.slots[ext_index]
            .proc
            .as_mut()
            .is_some_and(|p| p.send(&ready_msg).is_ok());
        if !sent {
            self.crash(ext_index, "ready write failed", now, host);
            return Err("ready write failed".to_string());
        }
        // Do not reset the crash budget here: three crashes in quick succession
        // must disable the extension even though each restart handshakes fine.
        self.sample_rss(ext_index, now);
        Ok(StartOutcome::Started)
    }

    /// Poll every worker once. Returns the number of frames handled.
    pub fn poll<H: BrokerHost>(&mut self, now: u64, host: &mut H) -> usize {
        let mut work = 0;
        let indices: Vec<usize> = (0..self.slots.len()).collect();
        for index in indices {
            if !self.is_live(index) {
                let (digest, ready) = match host.ready_payload(index) {
                    Some(payload) => payload,
                    None => continue,
                };
                let slot = &self.slots[index];
                if slot.backoff.due(now) {
                    let _ = self.start(index, &digest, ready, now, host);
                }
                continue;
            }

            let mut died: Option<String> = None;
            let mut stderr_lines = Vec::new();
            {
                let slot = &mut self.slots[index];
                while let Ok(line) = slot.proc.as_ref().expect("live").stderr.try_recv() {
                    stderr_lines.push(line);
                }
            }
            for line in stderr_lines {
                host.diagnostic(index, format!("worker stderr: {line}"));
            }

            {
                let slot = &mut self.slots[index];
                if let Some(status) = slot.proc.as_mut().and_then(WorkerProc::try_wait) {
                    died = Some(format!("worker exited ({status})"));
                }
            }

            if died.is_none() {
                let mut frames = Vec::new();
                {
                    let slot = &mut self.slots[index];
                    let proc = slot.proc.as_mut().expect("live");
                    loop {
                        match proc.frames.try_recv() {
                            Ok(frame) => frames.push(frame),
                            Err(std::sync::mpsc::TryRecvError::Empty) => break,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                died = Some("stdout channel closed".to_string());
                                break;
                            }
                        }
                    }
                }
                for frame in frames {
                    match frame {
                        Ok(msg) => {
                            work += 1;
                            if self.route_frame(index, msg, now, host) {
                                died = Some("fatal protocol frame".to_string());
                                break;
                            }
                        }
                        Err(err) => {
                            died = Some(err);
                            break;
                        }
                    }
                }
            }

            if let Some(reason) = died {
                self.reap(index);
                let graceful = self.slots[index].shutdown_at.take().is_some();
                self.slots[index].shutdown_sent = false;
                if graceful {
                    self.slots[index].backoff.reset();
                    host.diagnostic(index, "worker shut down cleanly".to_string());
                } else {
                    self.crash(index, &reason, now, host);
                }
                continue;
            }

            // A worker that stayed healthy for a while earns a fresh budget.
            let stable = now.saturating_sub(self.slots[index].started_at) > 10_000;
            if stable && self.slots[index].backoff.attempt > 0 {
                self.slots[index].backoff.reset();
            }

            let slot = &mut self.slots[index];
            if should_unload(now, slot.last_activity, slot.idle_ms) {
                let idle_ms = slot.idle_ms.unwrap_or_default();
                self.shutdown(index, now);
                self.slots[index].last_activity = now;
                host.diagnostic(index, format!("idle worker unloaded after {idle_ms}ms"));
            }
            if now.saturating_sub(self.slots[index].last_ping) >= PING_INTERVAL_MS {
                self.slots[index].last_ping = now;
                let ping = Message::Event {
                    name: "__hodo_ping".to_string(),
                    payload: json!({ "at": now }),
                };
                if let Err(too_big) = try_encode_bounded(&ping) {
                    host.diagnostic(
                        index,
                        format!(
                            "ping frame is {} bytes, exceeding the {} byte IPC cap; skipped",
                            too_big.len,
                            hodo_ext_ipc::MAX_MSG_BYTES
                        ),
                    );
                } else if let Some(proc) = self.slots[index].proc.as_mut() {
                    let _ = proc.send(&ping);
                }
            }
            if self.slots[index]
                .last_call
                .is_some_and(|at| now.saturating_sub(at) > HANG_TIMEOUT_MS)
            {
                let reason = format!("worker hung: api call unanswered for {HANG_TIMEOUT_MS}ms");
                self.kill_and_clear(index);
                self.crash(index, &reason, now, host);
                continue;
            }
            if now >= self.slots[index].next_rss_poll {
                self.slots[index].next_rss_poll = now.saturating_add(RSS_POLL_MS);
                if self.sample_rss(index, now) {
                    let reason = "rss exceeded baseline x 16 for 3 samples".to_string();
                    self.kill_and_clear(index);
                    self.crash(index, &reason, now, host);
                }
            }
            if let Some(at) = self.slots[index].shutdown_at {
                if now >= at {
                    if self.slots[index].shutdown_sent {
                        // Hard deadline: the worker ignored the queued Shutdown.
                        host.diagnostic(index, "worker shutdown grace expired; killed".to_string());
                        self.kill_and_clear(index);
                        self.slots[index].shutdown_at = None;
                        self.slots[index].shutdown_sent = false;
                    } else {
                        // Grace expired with no intervening event: only now is
                        // Shutdown queued, so events inside the grace window
                        // always travel *before* it and reach the worker.
                        let sent = self.slots[index]
                            .proc
                            .as_mut()
                            .is_some_and(|p| p.send(&Message::Shutdown).is_ok());
                        if sent {
                            self.slots[index].shutdown_sent = true;
                            self.slots[index].shutdown_at =
                                Some(now.saturating_add(SHUTDOWN_GRACE_MS));
                        } else {
                            self.slots[index].shutdown_at = None;
                        }
                    }
                }
            }
        }
        work
    }

    /// Send a browser event to a live worker. During the shutdown grace
    /// window an event cancels the pending shutdown (the worker is still
    /// needed); after the `Shutdown` frame is queued the worker is doomed, so
    /// the send is refused and the host retains the event for replay.
    pub fn send_event(&mut self, ext_index: usize, name: &str, payload: Value) -> bool {
        let Some(slot) = self.slots.get_mut(ext_index) else {
            return false;
        };
        if slot.shutdown_at.is_some() {
            if slot.shutdown_sent {
                return false;
            }
            // An event arrived while the worker was still usable: keep it.
            slot.shutdown_at = None;
        }
        let Some(proc) = slot.proc.as_mut() else {
            return false;
        };
        let msg = Message::Event {
            name: name.to_string(),
            payload,
        };
        // Never reach `encode`'s assert; a `false` return makes the host retain
        // the event (an oversized one is normally filtered at the emit boundary).
        if try_encode_bounded(&msg).is_err() {
            return false;
        }
        proc.send(&msg).is_ok()
    }

    /// Record activity (the host supplies its injectable clock).
    pub fn touch(&mut self, ext_index: usize, now: u64) {
        if let Some(slot) = self.slots.get_mut(ext_index) {
            slot.last_activity = now;
        }
    }

    /// Start a clean shutdown. The `Shutdown` frame is deliberately *not*
    /// queued yet: it is written only when the grace deadline expires, so an
    /// event arriving in the grace window reaches the worker first (and
    /// cancels the shutdown in [`Broker::send_event`]).
    pub fn shutdown(&mut self, ext_index: usize, now: u64) {
        if let Some(slot) = self.slots.get_mut(ext_index) {
            if slot.proc.is_some() && !slot.shutdown_sent && slot.shutdown_at.is_none() {
                slot.shutdown_at = Some(now.saturating_add(SHUTDOWN_GRACE_MS));
            }
        }
    }

    /// Immediate termination (used by idle unload, watchdogs and tests).
    pub fn terminate(&mut self, ext_index: usize) {
        self.kill_and_clear(ext_index);
        if let Some(slot) = self.slots.get_mut(ext_index) {
            slot.shutdown_at = None;
            slot.shutdown_sent = false;
            slot.backoff.reset();
        }
    }

    /// Current worker RSS for diagnostics. Reads the sysinfo cache (refreshing
    /// just this pid) but never calls `RssWatchdog::observe`: a user checking
    /// memory usage must not seed/overwrite the baseline or trip the trend.
    /// The `&mut self` is only for the sysinfo refresh.
    #[must_use]
    pub fn worker_rss_bytes(&mut self, ext_index: usize) -> Option<u64> {
        self.read_rss_bytes(ext_index)
    }

    /// Returns `true` if the frame killed the channel.
    fn route_frame<H: BrokerHost>(
        &mut self,
        ext_index: usize,
        msg: Message,
        now: u64,
        host: &mut H,
    ) -> bool {
        let slot = &mut self.slots[ext_index];
        slot.last_activity = now;
        match classify_worker_frame(msg) {
            FrameAction::Fatal(reason) => {
                host.diagnostic(ext_index, format!("protocol violation: {reason}"));
                return true;
            }
            FrameAction::ApiCall { id, method, params } => {
                if id == 0 || slot.pending.len() >= MAX_OUTSTANDING_CALLS {
                    host.diagnostic(ext_index, "api call id out of range".to_string());
                    return true;
                }
                if slot.pending.insert(id).is_err() {
                    host.diagnostic(ext_index, format!("duplicate api call id {id}"));
                    return true;
                }
                slot.last_call = Some(now);
                let result = host.dispatch_wire_call(ext_index, &method, params);
                slot.pending.remove(id);
                slot.last_call = None;
                let reply = match result {
                    ApiResult::Ok(value) => Message::ApiResponse {
                        id,
                        result: Some(value),
                        error: None,
                    },
                    ApiResult::Err(err) => Message::ApiResponse {
                        id,
                        result: None,
                        error: Some(err),
                    },
                };
                if let Err(too_big) = try_encode_bounded(&reply) {
                    host.diagnostic(
                        ext_index,
                        format!(
                            "api reply for call {id} is {} bytes, exceeding the {} byte IPC cap; dropped",
                            too_big.len,
                            hodo_ext_ipc::MAX_MSG_BYTES
                        ),
                    );
                } else if let Some(proc) = slot.proc.as_mut() {
                    if proc.send(&reply).is_err() {
                        return true;
                    }
                }
            }
            FrameAction::ApiResponse { id } => {
                host.diagnostic(ext_index, format!("unsolicited api response {id}"));
                slot.last_call = None;
            }
            FrameAction::Log(line) => host.diagnostic(ext_index, line),
            FrameAction::Ready => host.diagnostic(ext_index, "duplicate ready event".to_string()),
            FrameAction::Pong => slot.last_call = None,
            FrameAction::Event { name, payload } => {
                host.route_worker_event(ext_index, &name, payload);
            }
            FrameAction::Shutdown => {
                host.diagnostic(ext_index, "worker requested shutdown".to_string());
                return true;
            }
        }
        false
    }

    /// Read the worker's current RSS without touching watchdog state.
    fn read_rss_bytes(&mut self, ext_index: usize) -> Option<u64> {
        let pid = self
            .slots
            .get(ext_index)?
            .proc
            .as_ref()
            .map(WorkerProc::pid)?;
        let system = self.system.get_or_insert_with(|| {
            System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            )
            .unwrap_or_else(|_| System::new().expect("initialising sysinfo"))
        });
        system.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), true);
        system.process(Pid::from_u32(pid)).map(|p| p.memory())
    }

    fn sample_rss(&mut self, ext_index: usize, _now: u64) -> bool {
        let Some(bytes) = self.read_rss_bytes(ext_index) else {
            return false;
        };
        let Some(slot) = self.slots.get_mut(ext_index) else {
            return false;
        };
        slot.rss.observe(bytes)
    }

    fn reap(&mut self, ext_index: usize) {
        if let Some(slot) = self.slots.get_mut(ext_index) {
            // `try_wait` already reaped the child; the Job and cwd drop here.
            slot.proc = None;
        }
    }

    fn kill_and_clear(&mut self, ext_index: usize) {
        if let Some(slot) = self.slots.get_mut(ext_index) {
            if let Some(proc) = slot.proc.as_mut() {
                proc.kill();
            }
            slot.proc = None;
        }
    }

    fn crash<H: BrokerHost>(&mut self, ext_index: usize, reason: &str, now: u64, host: &mut H) {
        self.kill_and_clear(ext_index);
        host.diagnostic(ext_index, format!("worker crashed: {reason}"));
        self.ensure(ext_index);
        let exhausted = !self.slots[ext_index].backoff.schedule(now);
        if exhausted {
            host.disable(ext_index, reason.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodo_ext_ipc::{encode, IpcError};
    use serde_json::json;

    #[derive(Default)]
    struct MockHost {
        lines: Vec<String>,
    }

    impl BrokerHost for MockHost {
        fn dispatch_wire_call(&mut self, _ext: usize, _method: &str, _params: Value) -> ApiResult {
            ApiResult::Ok(Value::Null)
        }
        fn route_worker_event(&mut self, _ext: usize, _name: &str, _payload: Value) {}
        fn diagnostic(&mut self, _ext: usize, line: String) {
            self.lines.push(line);
        }
        fn disable(&mut self, _ext: usize, _reason: String) {}
        fn ready_payload(&self, _ext: usize) -> Option<(String, Value)> {
            None
        }
    }

    fn hello() -> Message {
        Message::Hello {
            token: "tok".into(),
            manifest_digest: "digest".into(),
        }
    }

    #[test]
    fn handshake_accepts_only_the_bound_token_and_digest() {
        let mut state = HandshakeState::new();
        assert_eq!(
            handshake_verdict(&state, &hello(), "tok", "digest"),
            HandshakeVerdict::Accept
        );
        assert_eq!(
            handshake_verdict(&state, &hello(), "other", "digest"),
            HandshakeVerdict::Fatal("bad token or manifest digest")
        );
        assert_eq!(
            handshake_verdict(&state, &hello(), "tok", "other"),
            HandshakeVerdict::Fatal("bad token or manifest digest")
        );
        let _ = state.observe(&hello());
        assert_eq!(
            handshake_verdict(&state, &hello(), "tok", "digest"),
            HandshakeVerdict::Fatal("duplicate hello")
        );
    }

    #[test]
    fn non_hello_first_frame_is_fatal() {
        let state = HandshakeState::new();
        let call = Message::ApiCall {
            id: 1,
            method: "tabs.list".into(),
            params: json!([]),
        };
        assert!(state.validate_worker_frame(&call).is_err());
        assert_eq!(
            handshake_verdict(&state, &call, "tok", "digest"),
            HandshakeVerdict::Fatal("first frame was not hello")
        );
        // A malformed first frame fails in the codec before the state machine.
        assert!(!encode(&Message::Shutdown).is_empty());
        assert!(matches!(decode_bad_frame(), Err(IpcError::BadVersion(2))));
    }

    fn decode_bad_frame() -> Result<Message, IpcError> {
        let mut frame = encode(&Message::Shutdown);
        frame[4] = 2;
        hodo_ext_ipc::decode(&frame)
    }

    #[test]
    fn frame_routing_table_covers_every_variant() {
        assert_eq!(
            classify_worker_frame(Message::Hello {
                token: "t".into(),
                manifest_digest: "d".into()
            }),
            FrameAction::Fatal("unexpected hello after handshake")
        );
        assert_eq!(
            classify_worker_frame(Message::ApiCall {
                id: 0,
                method: "x".into(),
                params: json!([])
            }),
            FrameAction::Fatal("api call id must be >= 1")
        );
        assert_eq!(
            classify_worker_frame(Message::ApiCall {
                id: 7,
                method: "storage.local.set".into(),
                params: json!(["k", "v"])
            }),
            FrameAction::ApiCall {
                id: 7,
                method: "storage.local.set".into(),
                params: json!(["k", "v"])
            }
        );
        assert_eq!(
            classify_worker_frame(Message::ApiResponse {
                id: 3,
                result: Some(json!(1)),
                error: None
            }),
            FrameAction::ApiResponse { id: 3 }
        );
        assert_eq!(
            classify_worker_frame(Message::Event {
                name: "__hodo_log".into(),
                payload: json!({ "message": "hi" })
            }),
            FrameAction::Log("hi".into())
        );
        assert_eq!(
            classify_worker_frame(Message::Event {
                name: "__hodo_ready".into(),
                payload: json!({})
            }),
            FrameAction::Ready
        );
        assert_eq!(
            classify_worker_frame(Message::Event {
                name: "__hodo_pong".into(),
                payload: json!({})
            }),
            FrameAction::Pong
        );
        assert_eq!(
            classify_worker_frame(Message::Event {
                name: "runtime.onMessage".into(),
                payload: json!({"a": 1})
            }),
            FrameAction::Event {
                name: "runtime.onMessage".into(),
                payload: json!({"a": 1})
            }
        );
        assert_eq!(
            classify_worker_frame(Message::Shutdown),
            FrameAction::Shutdown
        );
    }

    #[test]
    fn backoff_is_100_200_400_then_exhausted() {
        let mut backoff = Backoff::default();
        for (attempt, expected) in BACKOFF_MS.iter().enumerate() {
            assert!(!backoff.due(
                99 + if attempt == 0 {
                    0
                } else {
                    BACKOFF_MS[attempt - 1]
                }
            ));
            assert!(backoff.schedule(100));
            assert!(backoff.due(100 + expected));
            backoff.take();
            assert!(!backoff.due(u64::MAX));
        }
        assert!(backoff.exhausted());
        assert!(!backoff.schedule(1000));
        backoff.reset();
        assert!(!backoff.exhausted());
        assert!(backoff.schedule(1000));
    }

    #[test]
    fn idle_unload_respects_mv3_flag() {
        assert!(!should_unload(31_000, 0, None));
        assert!(!should_unload(30_000, 0, Some(30_000)));
        assert!(should_unload(30_001, 0, Some(30_000)));
        assert!(!should_unload(40_000, 30_000, Some(30_000)));
        assert!(should_unload(70_001, 30_000, Some(30_000)));
    }

    #[test]
    fn rss_watchdog_needs_baseline_times_sixteen_and_a_trend() {
        let mut watchdog = RssWatchdog::default();
        assert!(!watchdog.observe(10));
        assert_eq!(watchdog.baseline(), Some(10));
        for _ in 0..2 {
            assert!(!watchdog.observe(MIN_RSS_BUDGET + 1));
        }
        assert!(watchdog.observe(MIN_RSS_BUDGET + 1));
        let mut small = RssWatchdog::default();
        for _ in 0..10 {
            assert!(!small.observe(10 * (MAX_EXT_CONTEXTS as u64)));
        }
    }

    #[test]
    fn secret_compare_is_constant_time_and_length_aware() {
        assert!(constant_time_eq("tok", "tok"));
        assert!(constant_time_eq("", ""));
        assert!(!constant_time_eq("tok", "tox"));
        assert!(!constant_time_eq("tok", "to"));
        assert!(!constant_time_eq("tok", "toks"));
        assert!(!constant_time_eq("", "x"));
        assert_eq!(
            handshake_verdict(&HandshakeState::new(), &hello(), "tox", "digest"),
            HandshakeVerdict::Fatal("bad token or manifest digest")
        );
    }

    #[test]
    fn start_during_backoff_reports_deferred_instead_of_lying() {
        let mut broker = Broker::default();
        broker.set_worker_path(PathBuf::from("hodo-worker-that-does-not-exist"));
        let mut host = MockHost::default();
        let err = broker
            .start(0, "digest", json!({}), 1_000, &mut host)
            .expect_err("spawn must fail");
        assert!(err.contains("spawn failed"));
        let retry_at = broker.slots[0].backoff.next_at();
        assert!(retry_at > 1_000);
        assert!(host.lines.iter().any(|line| line.contains("crashed")));

        let outcome = broker
            .start(0, "digest", json!({}), retry_at - 1, &mut host)
            .expect("deferral is not an error");
        assert_eq!(outcome, StartOutcome::Deferred { retry_at });
        assert!(!broker.is_live(0), "no worker may be claimed live");
        assert!(host
            .lines
            .iter()
            .any(|line| line.contains("worker start deferred")));
    }

    #[test]
    fn diagnostic_rss_reads_do_not_mutate_the_watchdog() {
        let mut broker = Broker::default();
        broker.ensure(0);
        assert!(!broker.slots[0].rss.observe(10));
        for _ in 0..2 {
            assert!(!broker.slots[0].rss.observe(MIN_RSS_BUDGET + 1));
        }
        let baseline = broker.slots[0].rss.baseline();
        let over = broker.slots[0].rss.over;
        assert_eq!(over, 2);
        // No live process in this unit test, so the current-RSS probe has no
        // sample to report; it must still leave the watchdog state untouched.
        for _ in 0..10 {
            assert_eq!(broker.worker_rss_bytes(0), None);
        }
        assert_eq!(broker.slots[0].rss.baseline(), baseline);
        assert_eq!(
            broker.slots[0].rss.over, over,
            "a probe must not trip the trend"
        );
    }

    #[test]
    fn bounded_encode_rejects_oversized_frames_from_every_write_path() {
        let big = "x".repeat(hodo_ext_ipc::MAX_MSG_BYTES);
        // Event frames cover emit_event, disable fan-out and ready.
        for name in ["__hodo_ping", "__hodoExtensionDisabled", "__hodo_ready"] {
            let err = try_encode_bounded(&Message::Event {
                name: name.into(),
                payload: json!({ "body": big }),
            })
            .expect_err("{name} frame must be bounded");
            assert!(err.len > hodo_ext_ipc::MAX_MSG_BYTES, "{name}: {err:?}");
        }
        // The api-reply path uses ApiResponse.
        let err = try_encode_bounded(&Message::ApiResponse {
            id: 1,
            result: Some(json!({ "body": big })),
            error: None,
        })
        .expect_err("api reply must be bounded");
        assert!(err.len > hodo_ext_ipc::MAX_MSG_BYTES, "{err:?}");

        // Ordinary frames on each path still encode.
        assert!(try_encode_bounded(&Message::Event {
            name: "__hodo_ping".into(),
            payload: json!({ "at": 0u64 })
        })
        .is_ok());
        assert!(try_encode_bounded(&Message::ApiResponse {
            id: 1,
            result: Some(json!(null)),
            error: None
        })
        .is_ok());
    }

    #[test]
    fn bounded_encode_boundary_matches_cap_exactly() {
        // The cap covers the VERSION-prefixed body, so a frame fits exactly when
        // `json.len() + 1 == MAX_MSG_BYTES`. `try_encode_bounded` rejects only on
        // `len > cap` and `encode` asserts only on `body.len() > cap`; this test
        // locks both sides of that `>`/`<=` pairing against an off-by-one on
        // either path. Build the boundary payload deterministically: measure the
        // empty envelope and pad its string field to hit the cap minus one byte.
        const CAP: usize = hodo_ext_ipc::MAX_MSG_BYTES;
        let empty = Message::Event {
            name: String::new(),
            payload: json!(null),
        };
        let envelope = serde_json::to_vec(&empty)
            .expect("serializing a Message cannot fail")
            .len();
        let pad = CAP - 1 - envelope; // -1 for the VERSION byte the guard adds

        // Exactly at the cap: the bounded guard accepts and `encode` does not
        // panic on its assert.
        let exact = Message::Event {
            name: "x".repeat(pad),
            payload: json!(null),
        };
        assert_eq!(
            serde_json::to_vec(&exact)
                .expect("serializing a Message cannot fail")
                .len()
                + 1,
            CAP
        );
        assert!(
            try_encode_bounded(&exact).is_ok(),
            "a cap-sized frame must pass the bounded guard"
        );
        let _ = encode(&exact); // must not panic: encode's assert allows == cap

        // One byte over: the bounded guard rejects and `encode`'s assert fires.
        let over = Message::Event {
            name: "x".repeat(pad + 1),
            payload: json!(null),
        };
        assert_eq!(
            serde_json::to_vec(&over)
                .expect("serializing a Message cannot fail")
                .len()
                + 1,
            CAP + 1
        );
        let err = try_encode_bounded(&over).expect_err("a cap+1 frame must be rejected");
        assert_eq!(err.len, CAP + 1);
        assert!(
            std::panic::catch_unwind(|| encode(&over)).is_err(),
            "encode must assert on a cap+1 frame"
        );
    }
}

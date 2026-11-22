//! One OS process per isolated extension worker: spawn, containment, and the
//! two pipes used by the broker. No protocol decisions live here.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::{SystemTime, UNIX_EPOCH};

use hodo_ext_ipc::{encode, FrameReader, Message};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use win32job::{ExtendedLimitInfo, Job};

/// Unique per broker instance; never read from a worker message.
static TOKEN_SEQ: AtomicU64 = AtomicU64::new(1);
static DIR_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct SpawnSpec {
    pub worker_path: PathBuf,
    pub token: String,
    pub digest: String,
    /// Test-only injection (e.g. crash injection); never read from a worker.
    pub extra_env: Vec<(String, String)>,
}

/// A session secret. The worker echoes it in `Hello`; the broker binds the
/// channel to the extension it spawned, never to an id in the message body.
#[must_use]
pub fn session_token() -> String {
    let mut hasher = DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    TOKEN_SEQ.fetch_add(1, Ordering::Relaxed).hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Default worker location: next to the running browser binary.
#[must_use]
pub fn default_worker_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("hodo"));
    exe.parent()
        .unwrap_or_else(|| Path::new(""))
        .join("hodo-ext-worker")
}

/// Kills and reaps a child on early return unless disarmed. Covers the reader
/// thread spawns in [`spawn`], which can fail after the OS process exists.
struct CleanupGuard<F: FnMut()>(Option<F>);

impl<F: FnMut()> CleanupGuard<F> {
    fn new(cleanup: F) -> Self {
        Self(Some(cleanup))
    }

    fn disarm(mut self) {
        self.0 = None;
    }
}

impl<F: FnMut()> Drop for CleanupGuard<F> {
    fn drop(&mut self) {
        if let Some(mut cleanup) = self.0.take() {
            cleanup();
        }
    }
}

/// Owns the worker's process and the broker's end of its pipes.
pub struct WorkerProc {
    child: Child,
    stdin: ChildStdin,
    /// The stdout reader thread pushes complete frames here.
    pub frames: Receiver<Result<Message, String>>,
    pub stderr: Receiver<String>,
    /// Kept alive for the worker's lifetime; dropped after the child is reaped.
    _cwd: TempDir,
    /// Dropping the job (kill-on-close) terminates every worker descendant.
    #[cfg(windows)]
    _job: Job,
}

impl WorkerProc {
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// # Errors
    /// Propagates pipe write errors; a closed pipe means the worker is gone.
    pub fn send(&mut self, msg: &Message) -> io::Result<()> {
        self.stdin.write_all(&encode(msg))?;
        self.stdin.flush()
    }

    /// Kill and reap. Also called on every normal shutdown: the worker is an
    /// untrusted child and must never be left as a zombie.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Non-blocking reap on Linux (and everywhere else); `Some` means exited.
    pub fn try_wait(&mut self) -> Option<std::process::ExitStatus> {
        self.child.try_wait().ok().flatten()
    }
}

/// Spawn with a cleaned environment, an empty cwd and piped stdio.
///
/// # Errors
/// Fails closed if the OS cannot spawn or if Windows Job assignment fails.
pub fn spawn(spec: &SpawnSpec) -> io::Result<WorkerProc> {
    let cwd = TempDir::new()?;
    let mut command = Command::new(&spec.worker_path);
    command
        .arg("--manifest-digest")
        .arg(&spec.digest)
        .env_clear()
        .env("HODO_EXT_TOKEN", &spec.token)
        .envs(spec.extra_env.iter().map(|(key, value)| (key, value)))
        .current_dir(cwd.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;
    #[cfg(windows)]
    let job = match contain(&child) {
        Ok(job) => job,
        Err(err) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(err);
        }
    };

    let stdin = child.stdin.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        io::Error::other("worker stdin is missing")
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        io::Error::other("worker stdout is missing")
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        io::Error::other("worker stderr is missing")
    })?;

    // From here on, every early return (notably a reader-thread spawn failure)
    // must reap the child. `cleanup` is disarmed only when WorkerProc takes it.
    let cleanup = CleanupGuard::new(|| {
        let _ = child.kill();
        let _ = child.wait();
    });

    let (frames_tx, frames) = mpsc::channel();
    std::thread::Builder::new()
        .name("hodo-ext-worker-frames".to_string())
        .spawn(move || {
            let mut reader = FrameReader::new(stdout);
            loop {
                let outcome = match reader.read_frame() {
                    Ok(Some(msg)) => Ok(msg),
                    Ok(None) => Err("worker closed stdout".to_string()),
                    Err(err) => Err(format!("worker frame: {err}")),
                };
                let stop = outcome.is_err();
                if frames_tx.send(outcome).is_err() || stop {
                    break;
                }
            }
        })?;

    let (stderr_tx, stderr_rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("hodo-ext-worker-stderr".to_string())
        .spawn(move || {
            let mut text = Vec::new();
            let mut pipe = stderr;
            let _ = pipe.read_to_end(&mut text);
            let line = String::from_utf8_lossy(&text).trim().to_string();
            if !line.is_empty() {
                let _ = stderr_tx.send(line);
            }
        })?;

    cleanup.disarm();
    Ok(WorkerProc {
        child,
        stdin,
        frames,
        stderr: stderr_rx,
        _cwd: cwd,
        #[cfg(windows)]
        _job: job,
    })
}

/// Windows: kill-on-close, no breakaway. Assigned *after* spawn, so a worker
/// that fails to start cannot leave detached descendants behind.
#[cfg(windows)]
fn contain(child: &Child) -> io::Result<Job> {
    let mut limits = ExtendedLimitInfo::new();
    limits.limit_kill_on_job_close();
    let job = Job::create_with_limit_info(&limits).map_err(io::Error::other)?;
    job.assign_process(child.as_raw_handle() as isize)
        .map_err(io::Error::other)?;
    Ok(job)
}

/// Empty, unique working directory; removed when the worker goes away.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> io::Result<Self> {
        let n = DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "hodo-ext-worker-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            n
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    /// Thread creation cannot be failed on demand, so exercise the guard's
    /// structure with a mock kill+wait callback instead.
    #[test]
    fn cleanup_guard_reaps_on_drop_and_can_be_disarmed() {
        let killed = Rc::new(Cell::new(false));
        let waited = Rc::new(Cell::new(false));
        {
            let (k, w) = (Rc::clone(&killed), Rc::clone(&waited));
            let _guard = CleanupGuard::new(move || {
                k.set(true);
                w.set(true);
            });
        }
        assert!(killed.get() && waited.get(), "early return must reap");

        let kept = Rc::new(Cell::new(false));
        let flag = Rc::clone(&kept);
        CleanupGuard::new(move || flag.set(true)).disarm();
        assert!(!kept.get(), "a successfully spawned worker must stay alive");
    }
}

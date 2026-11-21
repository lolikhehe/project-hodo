//! Boa 0.22 job executor and host hooks for extension contexts.
//!
//! Boa's `SimpleJobExecutor` runs timers on the real clock and drains every job
//! at once. Extensions need a browser-like loop instead: microtasks are drained
//! after each task, and timers are driven by our own injectable clock. The
//! executor below keeps all jobs in queues we control; `drain` is called by
//! `ExtScriptRuntime` after every task.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use boa_engine::builtins::promise::{OperationType, Promise};
use boa_engine::context::HostHooks;
use boa_engine::error::JsNativeError;
use boa_engine::job::{Job, JobExecutor};
use boa_engine::object::JsObject;
use boa_engine::{Context, JsError, JsResult, JsString};

/// FIFO job queue shared by the context and the runtime.
#[derive(Default)]
pub(crate) struct HodoJobExecutor {
    jobs: RefCell<VecDeque<Job>>,
    errors: RefCell<Vec<String>>,
}

impl HodoJobExecutor {
    /// Run the jobs currently queued, including jobs queued by other jobs.
    /// Returns the number of jobs executed.
    pub(crate) fn drain(&self, context: &mut Context) -> usize {
        let mut ran = 0;
        loop {
            let job = self.jobs.borrow_mut().pop_front();
            let Some(job) = job else { break };
            let result = match job {
                Job::PromiseJob(job) => job.call(context),
                Job::GenericJob(job) => job.call(context),
                // Timers are owned by the runtime's task queue, not by boa.
                Job::TimeoutJob(_) | Job::IntervalJob(_) => continue,
                // Async/finalization jobs cannot be driven without blocking the
                // host thread; extension APIs are synchronous on this side.
                Job::AsyncJob(_) | Job::FinalizationRegistryCleanupJob(_) => continue,
                _ => continue,
            };
            ran += 1;
            if let Err(err) = result {
                self.errors.borrow_mut().push(kind_of_error(&err));
            }
        }
        ran
    }

    pub(crate) fn take_errors(&self) -> Vec<String> {
        std::mem::take(&mut self.errors.borrow_mut())
    }

    #[allow(dead_code)]
    pub(crate) fn pending(&self) -> usize {
        self.jobs.borrow().len()
    }
}

fn kind_of_error(err: &JsError) -> String {
    err.to_string()
        .lines()
        .next()
        .unwrap_or("job error")
        .to_string()
}

impl JobExecutor for HodoJobExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, _context: &mut Context) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        self.drain(context);
        Ok(())
    }
}

/// Tracks rejected promises and enforces the "no eval / no remote code" policy.
pub(crate) struct ExtensionHooks {
    extension_name: String,
    unhandled: RefCell<HashMap<JsObject<Promise>, String>>,
    block_compile: Cell<bool>,
}

impl ExtensionHooks {
    pub(crate) fn new(extension_name: String) -> Self {
        Self {
            extension_name,
            unhandled: RefCell::new(HashMap::new()),
            block_compile: Cell::new(false),
        }
    }

    pub(crate) fn set_block_compile(&self, block: bool) {
        self.block_compile.set(block);
    }

    /// Report rejections that never got a handler, then forget them.
    pub(crate) fn take_unhandled(&self) -> Vec<String> {
        let entries = std::mem::take(&mut *self.unhandled.borrow_mut());
        entries
            .into_values()
            .map(|reason| {
                format!(
                    "{}: unhandled promise rejection: {reason}",
                    self.extension_name
                )
            })
            .collect()
    }
}

impl HostHooks for ExtensionHooks {
    fn promise_rejection_tracker(
        &self,
        promise: &JsObject<Promise>,
        operation: OperationType,
        _context: &mut Context,
    ) {
        let mut unhandled = self.unhandled.borrow_mut();
        match operation {
            OperationType::Reject => {
                unhandled.insert(promise.clone(), "no handler".to_string());
            }
            OperationType::Handle => {
                unhandled.remove(promise);
            }
        }
    }

    fn ensure_can_compile_strings(
        &self,
        _realm: boa_engine::realm::Realm,
        _parameters: &[JsString],
        _body: &JsString,
        _direct: bool,
        _context: &mut Context,
    ) -> JsResult<()> {
        if self.block_compile.get() {
            return Err(JsNativeError::error()
                .with_message(Cow::Borrowed("content security policy: eval is disabled"))
                .into());
        }
        Ok(())
    }
}

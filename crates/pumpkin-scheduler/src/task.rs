use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::channel::oneshot;

use crate::{CancellationHandle, ExecutionDomain, SchedulerError};

/// Identifies a task within the scheduler that admitted it.
///
/// IDs are not reused by that scheduler, but may coincide across schedulers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchedulerTaskId(pub(crate) u64);

impl SchedulerTaskId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Carries a task's identity, ancestry, and cancellation handle.
///
/// IDs remain stable across suspension and executor thread changes. A cloned
/// context remains useful for tracing after completion, but does not keep the
/// task admitted or allow further child submissions once it has finished.
#[derive(Clone, Debug)]
pub struct TaskContext {
    pub(crate) id: SchedulerTaskId,
    pub(crate) chain: SchedulerTaskId,
    pub(crate) parent: Option<SchedulerTaskId>,
    pub(crate) owner: Arc<()>,
    pub(crate) cancellation: CancellationHandle,
}

impl TaskContext {
    /// Returns a handle that can cancel this task and its active descendants.
    #[must_use]
    pub fn cancellation_handle(&self) -> CancellationHandle {
        self.cancellation.clone()
    }

    #[must_use]
    pub const fn id(&self) -> SchedulerTaskId {
        self.id
    }
    /// Identifies the root of this call chain, even after the root has completed.
    #[must_use]
    pub const fn chain(&self) -> SchedulerTaskId {
        self.chain
    }
    /// Identifies the original parent, or `None` for a root task.
    ///
    /// This remains the original parent even when completed ancestors are
    /// removed from the scheduler's cancellation bookkeeping.
    #[must_use]
    pub const fn parent(&self) -> Option<SchedulerTaskId> {
        self.parent
    }
    #[must_use]
    pub const fn domain(&self) -> ExecutionDomain {
        ExecutionDomain::Global
    }
}

/// The owned future driven after a task is admitted.
pub type TaskFuture<T> = Pin<Box<dyn Future<Output = Result<T, SchedulerError>> + Send + 'static>>;
/// A factory that receives the admitted task's context and creates its future.
pub type TaskWork<T = ()> = Box<dyn FnOnce(TaskContext) -> TaskFuture<T> + Send + 'static>;

/// Describes owned work to admit to an execution domain.
///
/// Creating or submitting a request does not invoke its factory. The driver
/// calls it on the task's first poll, unless the task is cancelled beforehand.
/// Both the factory and its future run on the driver, so neither may block or
/// perform a long CPU job.
pub struct TaskRequest<T = ()> {
    pub(crate) domain: ExecutionDomain,
    pub(crate) parent: Option<TaskContext>,
    pub(crate) work: TaskWork<T>,
}

impl<T> TaskRequest<T> {
    /// Creates a request for an independent root task.
    ///
    /// Submission assigns its task ID and starts a new call chain. Domain
    /// support and available capacity are checked at submission, not here.
    pub fn new<F, Fut>(domain: ExecutionDomain, work: F) -> Self
    where
        F: FnOnce(TaskContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, SchedulerError>> + Send + 'static,
    {
        Self {
            domain,
            parent: None,
            work: Box::new(move |context| Box::pin(work(context))),
        }
    }

    /// Creates a request in the parent's domain and call chain.
    ///
    /// Submission requires an active, uncancelled parent from the same scheduler.
    /// Children use the same capacity as roots, without reserved slots; a
    /// suspended parent still occupies its own slot.
    ///
    /// Cancellation of an active ancestor also cancels this child. Completing a
    /// parent does not wait for or cancel its children, so the parent must await
    /// any children whose results it needs before it completes.
    pub fn child<F, Fut>(parent: &TaskContext, work: F) -> Self
    where
        F: FnOnce(TaskContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, SchedulerError>> + Send + 'static,
    {
        let mut request = Self::new(parent.domain(), work);
        request.parent = Some(parent.clone());
        request
    }

    #[must_use]
    pub const fn domain(&self) -> ExecutionDomain {
        self.domain
    }
}

/// An awaitable result for one admitted task.
///
/// The result becomes available after the task's future has been dropped and its
/// admission slot released. It includes execution, cancellation, and cleanup
/// errors. Awaiting this handle does not wait for detached children or work
/// launched on another executor.
///
/// Dropping the handle leaves the task running. Obtain a
/// [`Self::cancellation_handle`] before dropping it to retain the ability to
/// request cancellation later.
#[must_use = "await the handle to observe task completion or failure"]
pub struct TaskHandle<T = ()> {
    pub(crate) context: TaskContext,
    pub(crate) result: oneshot::Receiver<Result<T, SchedulerError>>,
}

impl<T> TaskHandle<T> {
    /// Returns a cloneable handle for requesting cancellation independently of this result.
    #[must_use]
    pub fn cancellation_handle(&self) -> CancellationHandle {
        self.context.cancellation_handle()
    }

    #[must_use]
    pub const fn context(&self) -> &TaskContext {
        &self.context
    }
    #[must_use]
    pub const fn id(&self) -> SchedulerTaskId {
        self.context.id()
    }
    #[must_use]
    pub const fn domain(&self) -> ExecutionDomain {
        self.context.domain()
    }
}

impl<T> Future for TaskHandle<T> {
    type Output = Result<T, SchedulerError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.result)
            .poll(cx)
            .map(|result| result.unwrap_or(Err(SchedulerError::Stopped)))
    }
}

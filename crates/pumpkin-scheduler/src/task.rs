use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::channel::oneshot;

use crate::{CancellationHandle, ExecutionDomain, SchedulerError};

/// Identifies one admitted task within a scheduler
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchedulerTaskId(pub(crate) u64);

impl SchedulerTaskId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Identifies a task and the call chain it belongs to
///
/// Task, parent, and chain IDs remain unchanged across suspension and worker
/// moves
/// Cloned contexts remain valid as metadata after their task leaves admission
/// Child requests require an active, uncancelled parent
#[derive(Clone, Debug)]
pub struct TaskContext {
    pub(crate) id: SchedulerTaskId,
    pub(crate) chain: SchedulerTaskId,
    pub(crate) parent: Option<SchedulerTaskId>,
    pub(crate) owner: Arc<()>,
    pub(crate) cancellation: CancellationHandle,
}

impl TaskContext {
    /// Returns the same cancellation capability held by this task's result handle
    #[must_use]
    pub fn cancellation_handle(&self) -> CancellationHandle {
        self.cancellation.clone()
    }

    #[must_use]
    pub const fn id(&self) -> SchedulerTaskId {
        self.id
    }
    /// Returns the original root task's ID, including after root completion
    #[must_use]
    pub const fn chain(&self) -> SchedulerTaskId {
        self.chain
    }
    /// Returns the parent ID assigned at submission
    ///
    /// The ID remains unchanged during cancellation reparenting, preserving the
    /// original call chain for tracing
    #[must_use]
    pub const fn parent(&self) -> Option<SchedulerTaskId> {
        self.parent
    }
    #[must_use]
    pub const fn domain(&self) -> ExecutionDomain {
        ExecutionDomain::Global
    }
}

pub type TaskFuture<T> = Pin<Box<dyn Future<Output = Result<T, SchedulerError>> + Send + 'static>>;
pub type TaskWork<T = ()> = Box<dyn FnOnce(TaskContext) -> TaskFuture<T> + Send + 'static>;

/// Owned work to submit to a domain
///
/// The factory is queued during submission and invoked by the driver
/// Both the factory and its future must avoid blocking or long-running CPU-heavy work
pub struct TaskRequest<T = ()> {
    pub(crate) domain: ExecutionDomain,
    pub(crate) parent: Option<TaskContext>,
    pub(crate) work: TaskWork<T>,
}

impl<T> TaskRequest<T> {
    /// Starts an independent root chain when this request is submitted
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

    /// Inherits the parent's domain and chain, including downward cancellation
    ///
    /// Submission checks that the parent is still active in the same scheduler
    /// Children share the root admission limit without reserved capacity
    /// Children remain active after parent completion
    /// Callers must await children whose results are required before the parent completes
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

/// The owned result of one accepted task
///
/// Accepted work continues when the handle is dropped
/// Explicit cancellation is available through [`Self::cancellation_handle`]
/// Results are delivered after the task's future is dropped and admission is released
/// Detached children and external work have their own completion lifetimes
#[must_use = "await the handle to observe task completion or failure"]
pub struct TaskHandle<T = ()> {
    pub(crate) context: TaskContext,
    pub(crate) result: oneshot::Receiver<Result<T, SchedulerError>>,
}

impl<T> TaskHandle<T> {
    /// Returns a cloneable cancellation handle that can be passed to another task
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

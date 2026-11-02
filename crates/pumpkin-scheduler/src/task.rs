use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::channel::oneshot;

use crate::{ExecutionDomain, SchedulerError};

/// Identity assigned to one admitted task within a scheduler
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchedulerTaskId(pub(crate) u64);

impl SchedulerTaskId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Owned causal metadata preserved across suspension and worker moves
///
/// A child inherits this chain only while it's parent is still valid
#[derive(Clone, Debug)]
pub struct TaskContext {
    pub(crate) id: SchedulerTaskId,
    pub(crate) chain: SchedulerTaskId,
    pub(crate) parent: Option<SchedulerTaskId>,
    pub(crate) owner: Arc<()>,
}

impl TaskContext {
    #[must_use]
    pub const fn id(&self) -> SchedulerTaskId {
        self.id
    }
    #[must_use]
    pub const fn chain(&self) -> SchedulerTaskId {
        self.chain
    }
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

/// Owned asynchronous work. The factory itself runs on the Global domain scheduler driver
pub struct TaskRequest<T = ()> {
    pub(crate) domain: ExecutionDomain,
    pub(crate) parent: Option<TaskContext>,
    pub(crate) work: TaskWork<T>,
}

impl<T> TaskRequest<T> {
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

/// Awaitable owned result
///
/// Note: Dropping the handle does not cancel it's active work
#[must_use = "await the handle to observe task completion or failure"]
pub struct TaskHandle<T = ()> {
    pub(crate) context: TaskContext,
    pub(crate) result: oneshot::Receiver<Result<T, SchedulerError>>,
}

impl<T> TaskHandle<T> {
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

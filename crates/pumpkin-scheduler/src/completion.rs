use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    task::{Context, Poll},
};

use futures::channel::oneshot;

use crate::{SchedulerError, SchedulerTaskId, TaskContext, TaskFuture, TaskWork};

/// Type-erased task with result delivery independent of its future
///
/// Retaining the result sender separately allows cancellation to resolve the
/// result without polling the future again
pub trait ScheduledTask: Send {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()>;
    fn finish(self: Box<Self>, error: Option<SchedulerError>) -> Completion;
}

pub enum Outcome {
    Completed,
    Cancelled,
    Failed,
}

pub struct Completion {
    pub(crate) outcome: Outcome,
    deliver: Box<dyn FnOnce() + Send>,
}

impl Completion {
    pub(crate) fn deliver(self) {
        (self.deliver)();
    }
}

struct Task<T> {
    id: SchedulerTaskId,
    future: TaskFuture<T>,
    output: Option<Result<T, SchedulerError>>,
    sender: oneshot::Sender<Result<T, SchedulerError>>,
}

pub fn task<T: Send + 'static>(
    context: TaskContext,
    work: TaskWork<T>,
    sender: oneshot::Sender<Result<T, SchedulerError>>,
) -> Box<dyn ScheduledTask> {
    Box::new(Task {
        id: context.id(),
        // Defer the factory call so factory panics use the same unwind boundary as polling
        future: Box::pin(async move { work(context).await }),
        output: None,
        sender,
    })
}

impl<T: Send + 'static> ScheduledTask for Task<T> {
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let output = match catch_unwind(AssertUnwindSafe(|| self.future.as_mut().poll(cx))) {
            Ok(Poll::Pending) => return Poll::Pending,
            Ok(Poll::Ready(output)) => output,
            Err(payload) => Err(panic_error(self.id, payload.as_ref())),
        };
        self.output = Some(output);
        Poll::Ready(())
    }

    fn finish(self: Box<Self>, error: Option<SchedulerError>) -> Completion {
        let Self {
            id,
            future,
            output,
            sender,
        } = *self;
        // Run destructors outside the queue lock to allow scheduler callbacks
        // Catch cleanup panics before publishing the result
        let result = catch_unwind(AssertUnwindSafe(|| {
            drop(future);
            error.map_or_else(|| output.unwrap_or(Err(SchedulerError::Stopped)), Err)
        }))
        .unwrap_or_else(|payload| Err(panic_error(id, payload.as_ref())));
        let outcome = match &result {
            Ok(_) => Outcome::Completed,
            Err(SchedulerError::Cancelled { .. }) => Outcome::Cancelled,
            Err(_) => Outcome::Failed,
        };
        Completion {
            outcome,
            deliver: Box::new(move || {
                // Sending to a dropped receiver returns the owned result
                // Contain destructor panics so they cannot terminate the driver
                if catch_unwind(AssertUnwindSafe(|| drop(sender.send(result)))).is_err() {
                    tracing::error!(
                        task_id = id.get(),
                        "Scheduler task result delivery panicked"
                    );
                }
            }),
        }
    }
}

fn panic_error(task: SchedulerTaskId, payload: &(dyn Any + Send)) -> SchedulerError {
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_else(|| {
            payload.downcast_ref::<&str>().map_or_else(
                || "non-string panic payload".to_owned(),
                |message| (*message).to_owned(),
            )
        });
    SchedulerError::TaskPanicked { task, message }
}

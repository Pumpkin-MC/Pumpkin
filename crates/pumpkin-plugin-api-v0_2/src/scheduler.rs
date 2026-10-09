//! Task scheduling for the async v0.2 world
//!
//! Registering and cancelling are synchronous host calls, so you can do them anywhere; the task
//! body is an async callback that the host starts when the task fires, and it can await host
//! imports like any other handler
//!
//! # Where to look
//!
//! * [`schedule_delayed_task`] runs a task once
//! * [`schedule_repeating_task`] runs it on a period
//! * [`cancel_task`] stops future runs

use std::{future::Future, sync::Arc};

use crate::{
    Server,
    host::scheduler,
    registry::{LocalBoxFuture, Registry},
};

// Tasks are stored as trait objects, and `AsyncFn` is not dyn-compatible while a closure that
// returns a boxed future is, so callers write `|server| async move { ... }`
type TaskHandler = dyn Fn(Server) -> LocalBoxFuture<'static, ()> + Send + Sync;

static TASK_HANDLERS: Registry<TaskHandler> = Registry::new();

pub(crate) async fn dispatch(handler_id: u32, server: Server) {
    // The registry lock is released before the task runs
    if let Some(handler) = TASK_HANDLERS.get(handler_id) {
        handler(server).await;
    }
}

fn register_handler<F, Fut>(handler: F) -> u32
where
    F: Fn(Server) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + 'static,
{
    TASK_HANDLERS.register(Arc::new(move |server| Box::pin(handler(server))))
}

/// Schedules a task to run once after `delay_ticks` and returns the host's task id
///
/// There are two unrelated ids in play: the handler id is the key into this crate's table and only
/// the host's callback uses it, while the returned task id is the host's name for this scheduled
/// run and is what [`cancel_task`] takes
///
/// # Examples
///
/// ```rust,ignore
/// scheduler::schedule_delayed_task(20, |server| async move {
///     server.broadcast("one second later".to_string()).await;
/// });
/// ```
pub fn schedule_delayed_task<F, Fut>(delay_ticks: u64, handler: F) -> u32
where
    F: Fn(Server) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + 'static,
{
    scheduler::schedule_delayed_task(register_handler(handler), delay_ticks)
}

/// Schedules a task to run after `delay_ticks` and then every `period_ticks`
///
/// Returns the host's task id, which [`cancel_task`] takes
///
/// A run can start while an earlier one is still suspended, so runs may overlap; the closure must
/// not assume exclusive access to state it shares between runs
///
/// # Examples
///
/// ```rust,ignore
/// let task = scheduler::schedule_repeating_task(20, 20 * 60, |server| async move {
///     server.broadcast("another minute".to_string()).await;
/// });
/// ```
pub fn schedule_repeating_task<F, Fut>(delay_ticks: u64, period_ticks: u64, handler: F) -> u32
where
    F: Fn(Server) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + 'static,
{
    scheduler::schedule_repeating_task(register_handler(handler), delay_ticks, period_ticks)
}

/// Cancels a scheduled task by the task id that scheduling returned
///
/// This stops future runs only; a run that is already suspended keeps going until it finishes or
/// the host cancels it
pub fn cancel_task(task_id: u32) {
    scheduler::cancel_task(task_id);
}

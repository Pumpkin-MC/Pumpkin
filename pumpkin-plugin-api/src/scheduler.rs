//! Task scheduling API for plugins.
//!
//! This module provides the ability for plugins to schedule closures to be
//! executed after a delay or repeatedly in the server's main tick loop.
//!
//! # Example
//!
//! ```rust,ignore
//! use pumpkin_plugin_api::scheduler::SchedulerExt;
//!
//! context.schedule_delayed_task(20, |server| {
//!     server.log("One second has passed!");
//! });
//! ```

use crate::wit::pumpkin::plugin::context::Server;
use crate::wit::pumpkin::plugin::scheduler;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// A type alias for a closure that can be scheduled as a task.
pub type TaskHandler = Box<dyn FnMut(Server) + Send>;

pub(crate) struct Task {
    handler: TaskHandler,
}

pub(crate) static TASK_HANDLERS: Mutex<LazyTaskHandlers> = Mutex::new(LazyTaskHandlers {
    handlers: BTreeMap::new(),
    next_id: 0,
});

pub(crate) struct LazyTaskHandlers {
    handlers: BTreeMap<u32, Task>,
    next_id: u32,
}

impl LazyTaskHandlers {
    /// Registers a new task handler and returns its unique ID.
    pub fn register(&mut self, handler: TaskHandler) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.handlers.insert(id, Task { handler });
        id
    }

    /// Executes the task handler for the given ID.
    pub fn handle(&mut self, id: u32, server: Server) {
        if let Some(task) = self.handlers.get_mut(&id) {
            (task.handler)(server);
        }
    }
}

/// Extension trait to provide ergonomic task scheduling on `Context` and `Server`.
pub trait SchedulerExt {
    /// Schedules a task to be executed once after the specified number of ticks.
    ///
    /// * `delay_ticks`: Number of game ticks to wait before execution.
    /// * `handler`: Closure to execute.
    ///
    /// Returns a unique task ID.
    fn schedule_delayed_task<F>(&self, delay_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static;

    /// Schedules a task to be executed repeatedly.
    ///
    /// * `delay_ticks`: Number of game ticks to wait before the first execution.
    /// * `period_ticks`: Number of ticks between subsequent executions.
    /// * `handler`: Closure to execute.
    ///
    /// Returns a unique task ID.
    fn schedule_repeating_task<F>(&self, delay_ticks: u64, period_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static;
}

impl SchedulerExt for crate::Context {
    fn schedule_delayed_task<F>(&self, delay_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static,
    {
        schedule_delayed_task(delay_ticks, handler)
    }

    fn schedule_repeating_task<F>(&self, delay_ticks: u64, period_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static,
    {
        schedule_repeating_task(delay_ticks, period_ticks, handler)
    }
}

impl SchedulerExt for crate::Server {
    fn schedule_delayed_task<F>(&self, delay_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static,
    {
        schedule_delayed_task(delay_ticks, handler)
    }

    fn schedule_repeating_task<F>(&self, delay_ticks: u64, period_ticks: u64, handler: F) -> u32
    where
        F: FnMut(Server) + Send + 'static,
    {
        schedule_repeating_task(delay_ticks, period_ticks, handler)
    }
}

/// Lower-level function to schedule a delayed task.
/// Prefer using [`SchedulerExt`] for a more ergonomic API.
pub fn schedule_delayed_task<F>(delay_ticks: u64, handler: F) -> u32
where
    F: FnMut(Server) + Send + 'static,
{
    let handler_id = TASK_HANDLERS.lock().unwrap().register(Box::new(handler));
    scheduler::schedule_delayed_task(handler_id, delay_ticks)
}

/// Lower-level function to schedule a repeating task.
/// Prefer using [`SchedulerExt`] for a more ergonomic API.
pub fn schedule_repeating_task<F>(delay_ticks: u64, period_ticks: u64, handler: F) -> u32
where
    F: FnMut(Server) + Send + 'static,
{
    let handler_id = TASK_HANDLERS.lock().unwrap().register(Box::new(handler));
    scheduler::schedule_repeating_task(handler_id, delay_ticks, period_ticks)
}

/// Cancels a scheduled task.
pub fn cancel_task(task_id: u32) {
    scheduler::cancel_task(task_id);
}

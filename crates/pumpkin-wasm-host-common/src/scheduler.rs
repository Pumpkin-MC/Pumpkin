use crate::plugin::WasmPlugin;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::{Arc, Mutex, Weak};

pub type TaskId = u32;

pub struct ScheduledTask {
    pub id: TaskId,
    pub plugin: Arc<WasmPlugin>,
    pub handler_id: u32,
    pub next_tick: u64,
    pub period: Option<u64>,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.next_tick == other.next_tick
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order so BinaryHeap is a min-heap
        other.next_tick.cmp(&self.next_tick)
    }
}

pub struct TaskScheduler {
    tasks: Mutex<BinaryHeap<ScheduledTask>>,
    cancelled_tasks: Mutex<HashSet<TaskId>>,
    disabled_plugins: Mutex<Vec<Weak<WasmPlugin>>>,
    next_task_id: std::sync::atomic::AtomicU32,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BinaryHeap::new()),
            cancelled_tasks: Mutex::new(HashSet::new()),
            disabled_plugins: Mutex::new(Vec::new()),
            next_task_id: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub fn schedule_delayed_task(
        &self,
        plugin: Arc<WasmPlugin>,
        handler_id: u32,
        delay: u64,
        current_tick: u64,
    ) -> TaskId {
        let id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let mut disabled_plugins = self
            .disabled_plugins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if Self::is_plugin_disabled(&mut disabled_plugins, &plugin) {
            return id;
        }
        let task = ScheduledTask {
            id,
            plugin,
            handler_id,
            next_tick: current_tick + delay,
            period: None,
        };
        self.tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(task);
        id
    }

    pub fn schedule_repeating_task(
        &self,
        plugin: Arc<WasmPlugin>,
        handler_id: u32,
        delay: u64,
        period: u64,
        current_tick: u64,
    ) -> TaskId {
        let id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let mut disabled_plugins = self
            .disabled_plugins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if Self::is_plugin_disabled(&mut disabled_plugins, &plugin) {
            return id;
        }
        let task = ScheduledTask {
            id,
            plugin,
            handler_id,
            next_tick: current_tick + delay,
            period: Some(period),
        };
        self.tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(task);
        id
    }

    pub fn cancel_task(&self, id: TaskId) {
        self.cancelled_tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id);
    }

    pub fn disable_plugin(&self, plugin: &Arc<WasmPlugin>) {
        let mut disabled_plugins = self
            .disabled_plugins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !Self::is_plugin_disabled(&mut disabled_plugins, plugin) {
            disabled_plugins.push(Arc::downgrade(plugin));
        }

        self.tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|task| !Arc::ptr_eq(&task.plugin, plugin));
    }

    fn is_plugin_disabled(
        disabled_plugins: &mut Vec<Weak<WasmPlugin>>,
        plugin: &Arc<WasmPlugin>,
    ) -> bool {
        disabled_plugins.retain(|entry| entry.strong_count() > 0);
        let plugin = Arc::downgrade(plugin);
        disabled_plugins
            .iter()
            .any(|entry| Weak::ptr_eq(entry, &plugin))
    }

    /// Pops the tasks due at `current_tick`, leaving out cancelled ones and those of disabled plugins.
    pub fn take_due_tasks(&self, current_tick: u64) -> Vec<ScheduledTask> {
        let mut tasks_to_run = Vec::new();

        {
            let mut tasks = self
                .tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut cancelled = self
                .cancelled_tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            while let Some(task) = tasks.peek() {
                if task.next_tick > current_tick {
                    break;
                }

                let Some(task) = tasks.pop() else {
                    break;
                };
                if cancelled.remove(&task.id) {
                    continue;
                }

                tasks_to_run.push(task);
            }
        }

        let mut disabled_plugins = self
            .disabled_plugins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        tasks_to_run.retain(|task| !Self::is_plugin_disabled(&mut disabled_plugins, &task.plugin));
        tasks_to_run
    }

    /// Queues a repeating task for its next run, unless its plugin has been disabled in the meantime.
    pub fn reschedule(&self, mut task: ScheduledTask, current_tick: u64) {
        let Some(period) = task.period else {
            return;
        };
        task.next_tick = current_tick + period;
        let mut disabled_plugins = self
            .disabled_plugins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !Self::is_plugin_disabled(&mut disabled_plugins, &task.plugin) {
            self.tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(task);
        }
    }
}

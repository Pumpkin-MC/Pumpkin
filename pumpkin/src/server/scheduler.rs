use crate::plugin::loader::wasm::wasm_host::WasmPlugin;
use crate::server::Server;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering as AtomicOrdering;
use tokio::sync::Mutex;

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
            next_task_id: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub async fn schedule_delayed_task(
        &self,
        plugin: Arc<WasmPlugin>,
        handler_id: u32,
        delay: u64,
        current_tick: u64,
    ) -> TaskId {
        let id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let task = ScheduledTask {
            id,
            plugin,
            handler_id,
            next_tick: current_tick + delay,
            period: None,
        };
        self.tasks.lock().await.push(task);
        id
    }

    pub async fn schedule_repeating_task(
        &self,
        plugin: Arc<WasmPlugin>,
        handler_id: u32,
        delay: u64,
        period: u64,
        current_tick: u64,
    ) -> TaskId {
        let id = self.next_task_id.fetch_add(1, AtomicOrdering::SeqCst);
        let task = ScheduledTask {
            id,
            plugin,
            handler_id,
            next_tick: current_tick + delay,
            period: Some(period),
        };
        self.tasks.lock().await.push(task);
        id
    }

    pub async fn cancel_task(&self, id: TaskId) {
        self.cancelled_tasks.lock().await.insert(id);
    }

    pub async fn cancel_all_tasks(&self, plugin: &Arc<WasmPlugin>) {
        let tasks = self.tasks.lock().await;
        let mut cancelled = self.cancelled_tasks.lock().await;
        for task in tasks.iter() {
            if Arc::ptr_eq(&task.plugin, plugin) {
                cancelled.insert(task.id);
            }
        }
    }

    pub async fn tick(&self, server: &Arc<Server>) {
        let current_tick = server.tick_count.load(AtomicOrdering::Relaxed) as u64;
        let mut tasks_to_run = Vec::new();

        {
            let mut tasks = self.tasks.lock().await;
            let mut cancelled = self.cancelled_tasks.lock().await;

            while let Some(task) = tasks.peek() {
                if task.next_tick > current_tick {
                    break;
                }

                let task = tasks.pop().unwrap();
                if cancelled.remove(&task.id) {
                    continue;
                }

                tasks_to_run.push(task);
            }
        }

        for mut task in tasks_to_run {
            // Run the task
            let plugin = task.plugin.clone();
            let handler_id = task.handler_id;
            let server_clone = server.clone();

            tokio::spawn(async move {
                let mut store = plugin.store.lock().await;
                match plugin.plugin_instance {
                    crate::plugin::loader::wasm::wasm_host::PluginInstance::V0_1(ref instance) => {
                        if let Ok(server_res) = store.data_mut().add_server(server_clone) {
                            let _ = instance
                                .call_handle_task(&mut *store, handler_id, server_res)
                                .await;
                        }
                    }
                }
            });

            // If repeating, schedule next run
            if let Some(period) = task.period {
                task.next_tick = current_tick + period;
                self.tasks.lock().await.push(task);
            }
        }
    }
}

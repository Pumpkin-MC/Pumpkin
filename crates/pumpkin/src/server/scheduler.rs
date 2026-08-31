use crate::plugin::loader::wasm::wasm_host::WasmPlugin;
use crate::server::Server;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::{Arc, Mutex};

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

    pub fn schedule_delayed_task(
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

    pub fn cancel_all_tasks(&self, plugin: &Arc<WasmPlugin>) {
        let tasks = self
            .tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut cancelled = self
            .cancelled_tasks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for task in tasks.iter() {
            if Arc::ptr_eq(&task.plugin, plugin) {
                cancelled.insert(task.id);
            }
        }
    }

    pub fn tick(&self, server: &Arc<Server>) {
        let current_tick = server.tick_count.load(AtomicOrdering::Relaxed) as u64;
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

        for mut task in tasks_to_run {
            // Run the task
            let plugin = task.plugin.clone();
            let handler_id = task.handler_id;
            let server_clone = server.clone();

            server.spawn_task(async move {
                let mut store = plugin.store.lock().await;
                match plugin.plugin_instance {
                    crate::plugin::loader::wasm::wasm_host::PluginInstance::V0_1(ref instance) => {
                        if let Ok(server_res) = store.data_mut().add_server(server_clone) {
                            let server_rep = server_res.rep();
                            let _ = instance
                                .call_handle_task(&mut *store, handler_id, server_res)
                                .await;
                            let _ = store
                                .data_mut()
                                .resource_table
                                .delete::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                                    wasmtime::component::Resource::new_own(server_rep),
                                );
                        }
                    }
                }
            });

            // If repeating, schedule next run
            if let Some(period) = task.period {
                task.next_tick = current_tick + period;
                self.tasks
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(task);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledFunctionEvent {
    pub id: String,
    pub trigger_tick: u64,
    pub function_name: String,
    pub is_tag: bool,
}

impl PartialOrd for ScheduledFunctionEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledFunctionEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other.trigger_tick.cmp(&self.trigger_tick)
    }
}

#[derive(Default)]
pub struct ScheduledFunctionQueue {
    queue: Mutex<BinaryHeap<ScheduledFunctionEvent>>,
}

impl ScheduledFunctionQueue {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            queue: Mutex::new(BinaryHeap::new()),
        }
    }

    pub fn schedule(
        &self,
        id: String,
        trigger_tick: u64,
        function_name: String,
        is_tag: bool,
        replace: bool,
    ) {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if replace {
            let mut retained = Vec::new();
            while let Some(event) = queue.pop() {
                if event.id != id {
                    retained.push(event);
                }
            }
            for event in retained {
                queue.push(event);
            }
        }
        queue.push(ScheduledFunctionEvent {
            id,
            trigger_tick,
            function_name,
            is_tag,
        });
    }

    pub fn remove(&self, id: &str) -> usize {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut count = 0;
        let mut retained = Vec::new();
        while let Some(event) = queue.pop() {
            if event.id == id {
                count += 1;
            } else {
                retained.push(event);
            }
        }
        for event in retained {
            queue.push(event);
        }
        count
    }

    #[must_use]
    pub fn get_event_ids(&self) -> Vec<String> {
        let queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut ids: Vec<String> = queue.iter().map(|e| e.id.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn tick(&self, server: &Arc<Server>, current_tick: u64) {
        let mut to_run = Vec::new();
        {
            let mut queue = self
                .queue
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            while let Some(event) = queue.peek() {
                if event.trigger_tick > current_tick {
                    break;
                }
                if let Some(event) = queue.pop() {
                    to_run.push(event);
                }
            }
        }
        for event in to_run {
            let _ = crate::data::datapack::DatapackManager::execute_function_from_console(
                server,
                &event.function_name,
            );
        }
    }
}

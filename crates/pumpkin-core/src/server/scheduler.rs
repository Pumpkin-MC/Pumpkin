use crate::server::Server;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::Mutex;

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

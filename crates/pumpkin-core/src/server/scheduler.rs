use crate::server::Server;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
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
    /// Events popped by [`Self::tick`] whose function is still running. A save taken
    /// meanwhile keeps them, so a self-rescheduling loop is not lost when the snapshot
    /// lands between popping it and the function scheduling its next run.
    /// Always locked after `queue`.
    running: Mutex<Vec<ScheduledFunctionEvent>>,
}

impl ScheduledFunctionQueue {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            queue: Mutex::new(BinaryHeap::new()),
            running: Mutex::new(Vec::new()),
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

    /// Serializes the pending events like vanilla's `TimerQueue`, with each trigger
    /// time converted from server ticks to absolute game time.
    #[must_use]
    pub fn to_saved_nbt(&self, current_tick: u64, game_time: i64) -> NbtCompound {
        let queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let running = self
            .running
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // A running event that has not rescheduled itself yet is saved as due now.
        let in_flight = running
            .iter()
            .filter(|event| !queue.iter().any(|queued| queued.id == event.id));
        let events = queue
            .iter()
            .chain(in_flight)
            .map(|event| {
                let remaining = i64::try_from(event.trigger_tick.saturating_sub(current_tick))
                    .unwrap_or(i64::MAX);
                let mut callback = NbtCompound::new();
                callback.put_string(
                    "Type",
                    if event.is_tag {
                        "minecraft:function_tag".to_string()
                    } else {
                        "minecraft:function".to_string()
                    },
                );
                callback.put_string(
                    "Name",
                    event
                        .function_name
                        .strip_prefix('#')
                        .unwrap_or(&event.function_name)
                        .to_string(),
                );
                let mut nbt = NbtCompound::new();
                nbt.put_string("Name", event.id.clone());
                nbt.put_long("TriggerTime", game_time.saturating_add(remaining));
                nbt.put_compound("Callback", callback);
                NbtTag::Compound(nbt)
            })
            .collect();
        let mut data = NbtCompound::new();
        data.put("events", NbtTag::List(events));
        data
    }

    /// Restores events written by [`Self::to_saved_nbt`]. Events that came due while
    /// the server was down run on the next tick, like vanilla.
    pub fn load_saved_nbt(&self, data: &NbtCompound, current_tick: u64, game_time: i64) {
        let Some(events) = data.get_list("events") else {
            return;
        };
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for event in events {
            let NbtTag::Compound(event) = event else {
                continue;
            };
            let (Some(id), Some(trigger_time), Some(callback)) = (
                event.get_string("Name"),
                event.get_long("TriggerTime"),
                event.get_compound("Callback"),
            ) else {
                continue;
            };
            let Some(name) = callback.get_string("Name") else {
                continue;
            };
            let is_tag = callback.get_string("Type") == Some("minecraft:function_tag");
            let function_name = if is_tag {
                format!("#{name}")
            } else {
                name.to_string()
            };
            let remaining = u64::try_from(trigger_time.saturating_sub(game_time)).unwrap_or(0);
            queue.push(ScheduledFunctionEvent {
                id: id.to_string(),
                trigger_tick: current_tick.saturating_add(remaining),
                function_name,
                is_tag,
            });
        }
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
            self.running
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone_from(&to_run);
        };
        for event in to_run {
            let _ = crate::data::datapack::DatapackManager::execute_function_from_console(
                server,
                &event.function_name,
            );
            let _queue = self
                .queue
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut running = self
                .running
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(index) = running.iter().position(|e| e.id == event.id) {
                running.swap_remove(index);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NbtTag, ScheduledFunctionEvent, ScheduledFunctionQueue};

    #[test]
    fn saved_events_keep_their_remaining_delay_across_a_restart() {
        let queue = ScheduledFunctionQueue::new();
        queue.schedule("ns:loop".into(), 105, "ns:loop".into(), false, true);
        queue.schedule("#ns:tag".into(), 140, "#ns:tag".into(), true, true);
        let saved = queue.to_saved_nbt(100, 5000);

        // The new server starts counting ticks from zero, at a later game time.
        let restored = ScheduledFunctionQueue::new();
        restored.load_saved_nbt(&saved, 0, 5000);
        assert_eq!(restored.get_event_ids(), vec!["#ns:tag", "ns:loop"]);
        let events = restored
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .into_sorted_vec();
        let by_id = |id: &str| {
            events
                .iter()
                .find(|e| e.id == id)
                .map(|e| (e.trigger_tick, e.function_name.clone(), e.is_tag))
        };
        assert_eq!(by_id("ns:loop"), Some((5, "ns:loop".to_string(), false)));
        assert_eq!(by_id("#ns:tag"), Some((40, "#ns:tag".to_string(), true)));
    }

    #[test]
    fn overdue_saved_events_run_on_the_next_tick() {
        let queue = ScheduledFunctionQueue::new();
        queue.schedule("ns:late".into(), 10, "ns:late".into(), false, true);
        let saved = queue.to_saved_nbt(0, 100);

        let restored = ScheduledFunctionQueue::new();
        restored.load_saved_nbt(&saved, 7, 500);
        let trigger = restored
            .queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .peek()
            .map(|e| e.trigger_tick);
        assert_eq!(trigger, Some(7));
    }

    #[test]
    fn running_events_are_saved_until_they_reschedule() {
        let queue = ScheduledFunctionQueue::new();
        queue.running.lock().unwrap().push(ScheduledFunctionEvent {
            id: "ns:loop".into(),
            trigger_tick: 7,
            function_name: "ns:loop".into(),
            is_tag: false,
        });
        let saved = queue.to_saved_nbt(7, 7);
        assert_eq!(saved.get_list("events").map(<[NbtTag]>::len), Some(1));

        queue.schedule("ns:loop".into(), 8, "ns:loop".into(), false, true);
        let saved = queue.to_saved_nbt(7, 7);
        assert_eq!(saved.get_list("events").map(<[NbtTag]>::len), Some(1));
    }
}

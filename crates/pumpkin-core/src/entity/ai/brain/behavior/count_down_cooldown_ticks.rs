use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryModuleType};
use super::{BehaviorControl, Status};

pub struct CountDownCooldownTicks {
    memory: MemoryModuleType<i32>,
    required: [MemoryModuleId; 1],
    status: Status,
}

impl CountDownCooldownTicks {
    #[must_use]
    pub const fn new(memory: MemoryModuleType<i32>) -> Self {
        Self {
            memory,
            required: [memory.id()],
            status: Status::Stopped,
        }
    }
}

impl BehaviorControl for CountDownCooldownTicks {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &self.required
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !tick.state.has_memory_value(self.memory.id()) {
            return false;
        }
        self.status = Status::Running;
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(value) = tick
            .state
            .get(self.memory)
            .copied()
            .filter(|value| *value > 0)
        {
            tick.state.set(self.memory, value - 1);
        } else {
            self.do_stop(tick);
        }
    }

    fn do_stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(self.memory.id());
        self.status = Status::Stopped;
    }

    fn debug_string(&self) -> String {
        "CountDownCooldownTicks".to_owned()
    }
}

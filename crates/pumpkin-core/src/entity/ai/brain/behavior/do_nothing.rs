use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::MemoryModuleId;
use super::{BehaviorControl, Status};

pub struct DoNothing {
    min_duration: i32,
    max_duration: i32,
    status: Status,
    end_timestamp: i64,
}

impl DoNothing {
    #[must_use]
    pub const fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            min_duration,
            max_duration,
            status: Status::Stopped,
            end_timestamp: 0,
        }
    }
}

impl BehaviorControl for DoNothing {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &[]
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.status = Status::Running;
        let duration = self.min_duration
            + tick
                .mob
                .get_random()
                .random_range(0..(self.max_duration + 1 - self.min_duration).max(1));
        self.end_timestamp = tick.time + i64::from(duration);
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        if tick.time > self.end_timestamp {
            self.do_stop(tick);
        }
    }

    fn do_stop(&mut self, _tick: &mut BrainTick<'_>) {
        self.status = Status::Stopped;
    }

    fn debug_string(&self) -> String {
        "DoNothing".to_owned()
    }
}

//! Port of `behavior/DoNothing.java`, which implements `BehaviorControl` directly rather than
//! extending `Behavior` -- it has no entry condition and always starts.

use rand::RngExt;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, BehaviorStatus};
use crate::entity::ai::brain::memory::{MemoryKeyId, MemoryStatus};
use crate::entity::mob::Mob;

pub struct DoNothing {
    min_duration: i32,
    max_duration: i32,
    status: BehaviorStatus,
    end_timestamp: i64,
}

impl DoNothing {
    /// `new DoNothing(minDuration, maxDuration)` (`DoNothing.java:14-17`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(min_duration: i32, max_duration: i32) -> Box<dyn Behavior> {
        Box::new(Self {
            min_duration,
            max_duration,
            status: BehaviorStatus::Stopped,
            end_timestamp: 0,
        })
    }
}

impl Behavior for DoNothing {
    /// `getRequiredMemories()` returns `Set.of()` (`DoNothing.java:24-27`).
    fn required_memories(&self) -> &[(MemoryKeyId, MemoryStatus)] {
        &[]
    }

    fn status(&self) -> BehaviorStatus {
        self.status
    }

    /// `tryStart` (`DoNothing.java:29-35`): unconditionally succeeds.
    fn try_start(&mut self, mob: &dyn Mob, _brain: &Brain, game_time: i64) -> bool {
        self.status = BehaviorStatus::Running;
        let span = self.max_duration + 1 - self.min_duration;
        let duration = if span > 1 {
            self.min_duration + mob.get_random().random_range(0..span)
        } else {
            self.min_duration
        };
        self.end_timestamp = game_time + i64::from(duration);
        true
    }

    /// `tickOrStop` (`DoNothing.java:37-42`): stops only once the duration elapses.
    fn tick_or_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        if game_time > self.end_timestamp {
            self.do_stop(mob, brain, game_time);
        }
    }

    fn do_stop(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) {
        self.status = BehaviorStatus::Stopped;
    }

    fn debug_name(&self) -> &'static str {
        "DoNothing"
    }
}

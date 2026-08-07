//! Port of `behavior/LookAtTargetSink.java`.
//!
//! The `LOOK_TARGET` counterpart of `MoveToTargetSink`: the only reader of that memory, draining
//! it into the mob's existing `LookControl`.

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::ai::brain::memory::{LookTargetMemory, MemoryKeyId, MemoryStatus};
use crate::entity::mob::Mob;

pub struct LookAtTargetSink;

impl LookAtTargetSink {
    /// `new LookAtTargetSink(minDuration, maxDuration)` (`LookAtTargetSink.java:10-12`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(min_duration: i32, max_duration: i32) -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::with_duration(
            Self,
            vec![(MemoryKeyId::LookTarget, MemoryStatus::ValuePresent)],
            min_duration,
            max_duration,
        ))
    }
}

impl TimedBehavior for LookAtTargetSink {
    fn debug_name(&self) -> &'static str {
        "LookAtTargetSink"
    }

    /// `canStillUse` (`LookAtTargetSink.java:14-16`). See `PositionTracker::is_visible_by` for
    /// the degraded visibility test.
    fn can_still_use(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        brain
            .get::<LookTargetMemory>()
            .is_some_and(|target| target.is_visible_by())
    }

    /// `tick` (`LookAtTargetSink.java:22-24`).
    ///
    /// The memory read completes and its lock is released before the `look_control` lock is
    /// taken; the two must never be nested in the other order.
    fn tick(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        let Some(position) = brain
            .get::<LookTargetMemory>()
            .and_then(|target| target.current_position())
        else {
            return;
        };
        let mut look_control = mob.get_mob_entity().look_control.lock().unwrap();
        look_control.look_at_position(mob, position);
    }

    /// `stop` (`LookAtTargetSink.java:18-20`).
    fn stop(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.erase::<LookTargetMemory>();
    }
}

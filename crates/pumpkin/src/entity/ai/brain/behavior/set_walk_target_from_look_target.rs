//! Port of `behavior/SetWalkTargetFromLookTarget.java`.
//!
//! One of the several behaviors that *write* `WALK_TARGET`; `MoveToTargetSink` is the only one
//! that reads it. This is the memory-ownership arbitration that replaces the Goal system's
//! per-control exclusivity.

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, OneShot, OneShotTrigger};
use crate::entity::ai::brain::memory::{
    LookTargetMemory, MemoryKeyId, MemoryStatus, WalkTarget, WalkTargetMemory,
};
use crate::entity::mob::Mob;

pub struct SetWalkTargetFromLookTarget {
    speed_modifier: f32,
    close_enough_distance: i32,
}

impl SetWalkTargetFromLookTarget {
    /// `SetWalkTargetFromLookTarget.create(speedModifier, closeEnoughDistance)`
    /// (`:11-13`). Entry condition: `WALK_TARGET` absent, `LOOK_TARGET` present (`:19`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(speed_modifier: f32, close_enough_distance: i32) -> Box<dyn Behavior> {
        Box::new(OneShot::new(
            Self {
                speed_modifier,
                close_enough_distance,
            },
            vec![
                (MemoryKeyId::WalkTarget, MemoryStatus::ValueAbsent),
                (MemoryKeyId::LookTarget, MemoryStatus::ValuePresent),
            ],
        ))
    }
}

impl OneShotTrigger for SetWalkTargetFromLookTarget {
    fn debug_name(&self) -> &'static str {
        "SetWalkTargetFromLookTarget"
    }

    /// `:20-27`.
    fn trigger(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        let Some(look_target) = brain.get::<LookTargetMemory>() else {
            return false;
        };
        brain.set::<WalkTargetMemory>(WalkTarget::new(
            look_target,
            self.speed_modifier,
            self.close_enough_distance,
        ));
        true
    }
}

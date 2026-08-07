//! Port of `behavior/RandomStroll.java`, `fly` variant only (the one Allay uses,
//! `AllayAi.java:86`).
//!
//! DEVIATION: vanilla's `RandomStroll.fly` picks its destination with
//! `AirAndWaterRandomPos.getPos(body, 10, 7, -2, viewX, viewZ, PI/2)`
//! (`RandomStroll.java:79-82`), which biases the roll toward the mob's view direction and
//! validates the candidate against the pathfinder's malus table. Pumpkin has no
//! `AirAndWaterRandomPos`/`LandRandomPos` port. This uses the same +/-10 horizontal, +/-7
//! vertical box (`RandomStroll.MAX_XZ_DIST`/`MAX_Y_DIST`, `:19-20`) with a uniform roll and no
//! view bias, matching what the existing `WanderAroundGoal` already does
//! (`ai/goal/wander_around.rs:63-79`). The result is a less directed wander than vanilla's.

use rand::RngExt;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, OneShot, OneShotTrigger};
use crate::entity::ai::brain::memory::{
    MemoryKeyId, MemoryStatus, PositionTracker, WalkTarget, WalkTargetMemory,
};
use crate::entity::mob::Mob;

/// `RandomStroll.MAX_XZ_DIST` (`:19`).
const MAX_XZ_DIST: f64 = 10.0;
/// `RandomStroll.MAX_Y_DIST` (`:20`).
const MAX_Y_DIST: f64 = 7.0;

pub struct RandomStrollFly {
    speed_modifier: f32,
}

impl RandomStrollFly {
    /// `RandomStroll.fly(speedModifier)` (`RandomStroll.java:35-37`). Entry condition is
    /// `WALK_TARGET` absent (`:46`), so a stroll never overwrites an in-flight walk target.
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(speed_modifier: f32) -> Box<dyn Behavior> {
        Box::new(OneShot::new(
            Self { speed_modifier },
            vec![(MemoryKeyId::WalkTarget, MemoryStatus::ValueAbsent)],
        ))
    }
}

impl OneShotTrigger for RandomStrollFly {
    fn debug_name(&self) -> &'static str {
        "RandomStroll.fly"
    }

    /// `strollFlyOrSwim`'s trigger (`RandomStroll.java:46-54`): the trigger returns `true`
    /// whether or not a position was found; `setOrErase` writes the walk target when there is
    /// one. Here a position is always produced, so the erase branch is unreachable.
    fn trigger(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        let pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let mut rng = mob.get_random();
        let target = Vector3::new(
            pos.x + rng.random_range(-MAX_XZ_DIST..=MAX_XZ_DIST),
            pos.y + rng.random_range(-MAX_Y_DIST..=MAX_Y_DIST),
            pos.z + rng.random_range(-MAX_XZ_DIST..=MAX_XZ_DIST),
        );
        brain.set::<WalkTargetMemory>(WalkTarget::new(
            PositionTracker::of_position(target),
            self.speed_modifier,
            0,
        ));
        true
    }
}

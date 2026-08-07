//! Port of `behavior/AnimalPanic.java`.
//!
//! This is the behavior that proves the split-lock design: its `HURT_BY` gate is populated from
//! `LivingEntity::damage_with_context`, i.e. from projectile/block/fluid call sites that run
//! outside the mob's own AI tick. If the memory store were taken out of its mutex for the tick
//! the way `GoalSelector` is, those writes would land on a throwaway `Default` and be lost.
//!
//! DEVIATION: `getPanicPos` (`AnimalPanic.java:89-98`) first looks for nearby water when the mob
//! is on fire and otherwise defers to `LandRandomPos.getPos(mob, 5, 4)`. Pumpkin has no
//! `LandRandomPos`/`AirAndWaterRandomPos` port, so this picks a uniform random offset in the
//! same +/-5 horizontal, +/-4 vertical box and lets the navigator reject unreachable
//! destinations. The on-fire water search is not ported at all.

use rand::RngExt;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::ai::brain::memory::{
    HurtByMemory, IsPanickingMemory, MemoryKeyId, MemoryStatus, PositionTracker, WalkTarget,
    WalkTargetMemory,
};
use crate::entity::mob::Mob;

/// `AnimalPanic.PANIC_DISTANCE_HORIZONTAL` / `PANIC_DISTANCE_VERTICAL` (`:29-30`).
const PANIC_DISTANCE_HORIZONTAL: f64 = 5.0;
const PANIC_DISTANCE_VERTICAL: f64 = 4.0;

pub struct AnimalPanic {
    speed_multiplier: f32,
}

impl AnimalPanic {
    /// `new AnimalPanic(speedMultiplier)` (`AnimalPanic.java:35-37`), whose entry condition is
    /// `IS_PANICKING` REGISTERED + `HURT_BY` REGISTERED and whose duration is 100..120 (`:54`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(speed_multiplier: f32) -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::with_duration(
            Self { speed_multiplier },
            vec![
                (MemoryKeyId::IsPanicking, MemoryStatus::Registered),
                (MemoryKeyId::HurtBy, MemoryStatus::Registered),
            ],
            100,
            120,
        ))
    }
}

impl TimedBehavior for AnimalPanic {
    fn debug_name(&self) -> &'static str {
        "AnimalPanic"
    }

    /// `checkExtraStartConditions` (`AnimalPanic.java:60-63`): the recorded damage type is in
    /// `DamageTypeTags.PANIC_CAUSES`, or the mob is already flagged as panicking.
    fn check_extra_start_conditions(&mut self, _mob: &dyn Mob, brain: &Brain) -> bool {
        let hurt_by_panics = brain.get::<HurtByMemory>().is_some_and(|damage_type| {
            damage_type.has_tag(&tag::DamageType::MINECRAFT_PANIC_CAUSES)
        });
        hurt_by_panics || brain.has_value::<IsPanickingMemory>()
    }

    /// `canStillUse` (`AnimalPanic.java:65-67`) is unconditionally true; the 100..120 duration
    /// is what ends the panic.
    fn can_still_use(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) -> bool {
        true
    }

    /// `start` (`AnimalPanic.java:69-73`).
    fn start(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.set::<IsPanickingMemory>(true);
        brain.erase::<WalkTargetMemory>();
        mob.get_mob_entity().navigator.lock().unwrap().stop();
    }

    /// `stop` (`AnimalPanic.java:75-78`).
    fn stop(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.erase::<IsPanickingMemory>();
    }

    /// `tick` (`AnimalPanic.java:80-87`): only pick a new flee destination once the navigator
    /// has run out of path, so the mob does not re-roll every tick.
    fn tick(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        let navigation_done = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
        if !navigation_done {
            return;
        }

        let pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let mut rng = mob.get_random();
        let panic_to = Vector3::new(
            pos.x + rng.random_range(-PANIC_DISTANCE_HORIZONTAL..=PANIC_DISTANCE_HORIZONTAL),
            pos.y + rng.random_range(-PANIC_DISTANCE_VERTICAL..=PANIC_DISTANCE_VERTICAL),
            pos.z + rng.random_range(-PANIC_DISTANCE_HORIZONTAL..=PANIC_DISTANCE_HORIZONTAL),
        );

        brain.set::<WalkTargetMemory>(WalkTarget::new(
            PositionTracker::of_position(panic_to),
            self.speed_multiplier,
            0,
        ));
    }
}

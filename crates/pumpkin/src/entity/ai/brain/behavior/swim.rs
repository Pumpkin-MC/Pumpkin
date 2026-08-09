//! Port of `behavior/Swim.java`.

use std::sync::atomic::Ordering;

use rand::RngExt;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::mob::Mob;

pub struct Swim {
    chance: f32,
}

impl Swim {
    /// `new Swim<>(chance)` (`Swim.java:11-14`). Empty entry condition (`ImmutableMap.of()`),
    /// so the gate is entirely `checkExtraStartConditions`.
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(chance: f32) -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::new(Self { chance }, Vec::new()))
    }

    /// `Swim.shouldSwim` (`Swim.java:16-18`): in water above the fluid-jump threshold, or in
    /// lava. Same predicate the existing `SwimGoal` uses (`ai/goal/swim.rs:19-26`), which is
    /// where `get_swim_height` stands in for `getFluidJumpThreshold`.
    fn should_swim(mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let in_water = entity.touching_water.load(Ordering::SeqCst)
            && entity.water_height.load() > living.get_swim_height();
        in_water || entity.touching_lava.load(Ordering::SeqCst)
    }
}

impl TimedBehavior for Swim {
    fn debug_name(&self) -> &'static str {
        "Swim"
    }

    fn check_extra_start_conditions(&mut self, mob: &dyn Mob, _brain: &Brain) -> bool {
        Self::should_swim(mob)
    }

    /// `canStillUse` (`Swim.java:24-26`) forwards to the start condition.
    fn can_still_use(&mut self, mob: &dyn Mob, _brain: &Brain, _game_time: i64) -> bool {
        Self::should_swim(mob)
    }

    /// `tick` (`Swim.java:28-32`): vanilla calls `body.getJumpControl().jump()`. Pumpkin has no
    /// separate jump control; setting `LivingEntity::jumping` is what `SwimGoal` does and what
    /// the movement code consumes.
    fn tick(&mut self, mob: &dyn Mob, _brain: &Brain, _game_time: i64) {
        if mob.get_random().random::<f32>() < self.chance {
            mob.get_mob_entity()
                .jump_requested
                .store(true, Ordering::SeqCst);
        }
    }
}

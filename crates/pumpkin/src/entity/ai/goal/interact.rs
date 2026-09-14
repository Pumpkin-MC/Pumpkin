use super::look_at_entity::LookAtEntityGoal;
use super::{Controls, Goal};
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;
use std::sync::Weak;

pub struct InteractGoal {
    look_at_goal: LookAtEntityGoal,
}

impl InteractGoal {
    #[must_use]
    pub fn new(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f32,
        chance: f32,
    ) -> Self {
        Self {
            look_at_goal: LookAtEntityGoal::new(mob_weak, target_type, range, chance, false),
        }
    }
}

impl Goal for InteractGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.look_at_goal.can_start(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.look_at_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.look_at_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.look_at_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.look_at_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.look_at_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        Controls::LOOK | Controls::MOVE
    }
}

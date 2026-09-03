use super::{Controls, Goal};
use crate::entity::mob::Mob;
use std::sync::atomic::Ordering;

#[derive(Default)]
pub struct SitWhenOrderedToGoal {
    goal_control: Controls,
}

impl SitWhenOrderedToGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE.union(Controls::JUMP),
        }
    }
}

impl Goal for SitWhenOrderedToGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !mob.is_sitting() && !mob.is_tamed() {
            return false;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        if entity.touching_water.load(Ordering::Relaxed) {
            return false;
        }

        if !entity.on_ground.load(Ordering::Relaxed) {
            return false;
        }

        mob.is_sitting()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        mob.is_sitting()
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

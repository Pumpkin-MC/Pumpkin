use std::sync::atomic::Ordering;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct SwimGoal {
    goal_control: Controls,
}

impl Default for SwimGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::JUMP,
        }
    }
}

impl Goal for SwimGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            entity.touching_water.load(Ordering::SeqCst)
                || entity.touching_lava.load(Ordering::SeqCst)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            entity.touching_water.load(Ordering::SeqCst)
                || entity.touching_lava.load(Ordering::SeqCst)
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if mob.get_random().random::<f32>() < 0.8 {
                mob.get_mob_entity()
                    .living_entity
                    .jumping
                    .store(true, Ordering::SeqCst);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

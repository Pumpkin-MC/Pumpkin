use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::mob::Mob;
use rand::RngExt;
use std::sync::atomic::Ordering;

/// A goal that makes the mob float in water by jumping.
///
/// When the mob is submerged in water, this goal activates
/// and makes the mob jump to stay afloat.
pub struct SwimGoal {
    goal_control: Controls,
}

impl SwimGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::JUMP,
        })
    }
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
        Box::pin(async {
            let entity = &mob.get_mob_entity().living_entity.entity;
            entity.touching_water.load(Ordering::Relaxed)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let entity = &mob.get_mob_entity().living_entity.entity;
            entity.touching_water.load(Ordering::Relaxed)
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let living = &mob.get_mob_entity().living_entity;
            // Signal a jump to stay afloat
            if mob.get_random().random::<f32>() < 0.8 {
                living.jumping.store(true, Ordering::Relaxed);
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

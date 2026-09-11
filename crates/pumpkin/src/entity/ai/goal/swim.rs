use std::sync::atomic::Ordering;

use super::{Controls, Goal};
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

impl SwimGoal {
    fn is_in_fluid(mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let in_water = entity.touching_water.load(Ordering::SeqCst)
            && entity.water_height.load() > living.get_swim_height();
        in_water || entity.touching_lava.load(Ordering::SeqCst)
    }
}

impl Goal for SwimGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::is_in_fluid(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        Self::is_in_fluid(mob)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if mob.get_random().random::<f32>() < 0.8 {
            mob.get_mob_entity()
                .living_entity
                .jumping
                .store(true, Ordering::SeqCst);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

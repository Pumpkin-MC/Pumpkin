use std::f64::consts::TAU;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct RandomLookAroundGoal {
    goal_control: Controls,
    delta_x: f64,
    delta_z: f64,
    look_time: i32,
}

impl Default for RandomLookAroundGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            delta_x: 0.0,
            delta_z: 0.0,
            look_time: 0,
        }
    }
}

impl Goal for RandomLookAroundGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.get_random().random::<f32>() < 0.02
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        self.look_time >= 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        let d = TAU * mob.get_random().random::<f64>();
        self.delta_x = d.cos();
        self.delta_z = d.sin();
        let look_time = 20 + mob.get_random().random_range(0..20);
        self.look_time = look_time;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        self.look_time -= 1;
        let mut look_control = mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let pos = mob_entity.living_entity.entity.pos.load();
        look_control.look_at(
            mob,
            pos.x + self.delta_x,
            mob_entity.living_entity.entity.get_eye_y(),
            pos.z + self.delta_z,
        );
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

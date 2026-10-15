use std::sync::atomic::Ordering;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub const DEFAULT_SWIMMING_INTERVAL: i32 = 40;

pub struct RandomSwimmingGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    interval: i32,
}

impl RandomSwimmingGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self::with_interval(speed, DEFAULT_SWIMMING_INTERVAL)
    }

    #[must_use]
    pub const fn with_interval(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            interval,
        }
    }

    fn get_swimming_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load_full();
        let current_pos = mob.get_entity().pos.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(-10..=10) as f64;
            let dy = rng.random_range(-7..=7) as f64;
            let dz = rng.random_range(-10..=10) as f64;
            let target = current_pos + Vector3::new(dx, dy, dz);
            let block_pos = BlockPos::new(
                target.x.floor() as i32,
                target.y.floor() as i32,
                target.z.floor() as i32,
            );
            let fluid = world.get_fluid(&block_pos);
            if fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id {
                return Some(target);
            }
        }
        None
    }
}

impl Goal for RandomSwimmingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().has_passengers() {
            return false;
        }

        if !mob.get_entity().touching_water.load(Ordering::Relaxed) {
            return false;
        }

        if mob
            .get_random()
            .random_range(0..to_goal_ticks(self.interval))
            != 0
        {
            return false;
        }

        self.target = Self::get_swimming_position(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let idle = mob.is_navigator_idle();
        !idle
            && !mob.get_entity().has_passengers()
            && mob.get_entity().touching_water.load(Ordering::Relaxed)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(pos, target, self.speed));

            let mut move_control = mob
                .get_mob_entity()
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            move_control.set_wanted_position(target.x, target.y, target.z, self.speed);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

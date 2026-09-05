use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct WaterAvoidingRandomFlyingGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    chance: i32,
}

impl WaterAvoidingRandomFlyingGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            chance: to_goal_ticks(120),
        }
    }

    fn find_air_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(-8.0..=8.0);
            let dy = rng.random_range(-4.0..=7.0);
            let dz = rng.random_range(-8.0..=8.0);

            let check_pos = BlockPos::new(
                (mob_pos.x + dx) as i32,
                (mob_pos.y + dy) as i32,
                (mob_pos.z + dz) as i32,
            );

            let block = world.get_block_state(&check_pos);
            if !block.is_solid() && !block.is_liquid() {
                return Some(Vector3::new(mob_pos.x + dx, mob_pos.y + dy, mob_pos.z + dz));
            }
        }

        None
    }
}

impl Goal for WaterAvoidingRandomFlyingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_random().random_range(0..self.chance) != 0 {
            return false;
        }

        self.target = Self::find_air_target(mob);
        self.target.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let is_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|nav| nav.is_idle());
        !is_idle
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_progress(NavigatorGoal::new(mob_pos, target, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

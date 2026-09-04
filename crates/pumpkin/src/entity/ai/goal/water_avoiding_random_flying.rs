use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::{air_and_water_random_pos, hover_random_pos};
use crate::entity::mob::Mob;
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

    /// Prefers a spot to hover over the ground, and falls back to open air or water.
    fn get_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let wander_direction = mob.get_looking_vector();

        hover_random_pos::get_pos(
            mob,
            8,
            7,
            wander_direction.x,
            wander_direction.z,
            std::f64::consts::FRAC_PI_2,
            3,
            1,
        )
        .or_else(|| {
            air_and_water_random_pos::get_pos(
                mob,
                8,
                4,
                -2,
                wander_direction.x,
                wander_direction.z,
                std::f64::consts::FRAC_PI_2,
            )
        })
    }
}

impl Goal for WaterAvoidingRandomFlyingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().has_passengers() {
            return false;
        }

        if mob.get_random().random_range(0..self.chance) != 0 {
            return false;
        }

        self.target = Self::get_position(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let is_idle = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_idle();
        !is_idle && !mob.get_entity().has_passengers()
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

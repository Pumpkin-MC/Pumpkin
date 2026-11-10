use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::{
    ai::{brain::behavior::utils::get_random_swimmable_pos, pathfinder::NavigatorGoal},
    mob::Mob,
};

const HORIZONTAL_RANGE: i32 = 10;
const VERTICAL_RANGE: i32 = 7;
const NO_ACTION_TIME_LIMIT: i32 = 100;

pub struct RandomSwimmingGoal {
    speed: f64,
    interval: i32,
    target: Option<Vector3<f64>>,
}

impl RandomSwimmingGoal {
    #[must_use]
    pub const fn new(speed: f64, interval: i32) -> Self {
        Self {
            speed,
            interval,
            target: None,
        }
    }
}

impl Goal for RandomSwimmingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.mob_controlling_passenger().is_some()
            || mob
                .get_mob_entity()
                .living_entity
                .no_action_time
                .load(std::sync::atomic::Ordering::Relaxed)
                >= NO_ACTION_TIME_LIMIT
            || mob
                .get_random()
                .random_range(0..to_goal_ticks(self.interval))
                != 0
        {
            return false;
        }
        self.target = get_random_swimmable_pos(mob, HORIZONTAL_RANGE, VERTICAL_RANGE);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle() && mob.mob_controlling_passenger().is_none()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(
                    mob.get_entity().pos.load(),
                    target,
                    self.speed,
                ));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

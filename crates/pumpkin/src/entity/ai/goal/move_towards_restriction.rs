use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;

pub struct MoveTowardsRestrictionGoal {
    goal_control: Controls,
    speed_modifier: f64,
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
}

impl MoveTowardsRestrictionGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed_modifier,
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
        }
    }
}

impl Goal for MoveTowardsRestrictionGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let block_pos = mob.get_entity().block_pos.load();

        if mob_entity.is_in_position_target_range_pos(&block_pos) {
            return false;
        }

        let home = mob_entity.position_target.load();
        let home_center = Vector3::new(
            f64::from(home.0.x) + 0.5,
            f64::from(home.0.y),
            f64::from(home.0.z) + 0.5,
        );
        let Some(pos) = default_random_pos::get_pos_towards(
            mob,
            16,
            7,
            home_center,
            std::f64::consts::FRAC_PI_2,
        ) else {
            return false;
        };

        self.wanted_x = pos.x;
        self.wanted_y = pos.y;
        self.wanted_z = pos.z;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mob_pos = mob.get_entity().pos.load();
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_progress(NavigatorGoal::new(
            mob_pos,
            Vector3::new(self.wanted_x, self.wanted_y, self.wanted_z),
            self.speed_modifier,
        ));
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

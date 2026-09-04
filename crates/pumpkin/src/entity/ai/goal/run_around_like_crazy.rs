use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;

pub struct RunAroundLikeCrazyGoal {
    goal_control: Controls,
    speed_modifier: f64,
    pos_x: f64,
    pos_y: f64,
    pos_z: f64,
}

impl RunAroundLikeCrazyGoal {
    #[must_use]
    pub const fn new(speed_modifier: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed_modifier,
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
        }
    }
}

impl Goal for RunAroundLikeCrazyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.is_tamed() || !mob.get_entity().has_passengers() {
            return false;
        }

        let Some(pos) = default_random_pos::get_pos(mob, 5, 4) else {
            return false;
        };

        self.pos_x = pos.x;
        self.pos_y = pos.y;
        self.pos_z = pos.z;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let is_idle = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_idle();

        !mob.is_tamed() && !is_idle && mob.get_entity().has_passengers()
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
            Vector3::new(self.pos_x, self.pos_y, self.pos_z),
            self.speed_modifier,
        ));
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

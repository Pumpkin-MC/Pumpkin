use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

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

    fn find_pos_towards(mob: &dyn Mob, target_pos: BlockPos) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        let dx = f64::from(target_pos.0.x) + 0.5 - mob_pos.x;
        let dz = f64::from(target_pos.0.z) + 0.5 - mob_pos.z;
        let base_angle = dz.atan2(dx) - std::f64::consts::FRAC_PI_2;

        for _ in 0..10 {
            let angle =
                base_angle + (2.0 * rng.random_range(0.0..1.0) - 1.0) * std::f64::consts::FRAC_PI_2;
            let t = rng.random_range(0.0..1.0f64).sqrt();
            let dist = t * 16.0 * std::f64::consts::SQRT_2;
            let step_x = -dist * angle.sin();
            let step_z = dist * angle.cos();
            let step_y = rng.random_range(-7..=7);

            let candidate = BlockPos::new(
                (mob_pos.x + step_x) as i32,
                (mob_pos.y + step_y as f64) as i32,
                (mob_pos.z + step_z) as i32,
            );

            let block_at = world.get_block_state(&candidate);
            let block_below = world.get_block_state(&candidate.down());

            if !block_at.is_solid() && block_below.is_solid() {
                return Some(Vector3::new(
                    f64::from(candidate.0.x) + 0.5,
                    f64::from(candidate.0.y),
                    f64::from(candidate.0.z) + 0.5,
                ));
            }
        }

        None
    }
}

impl Goal for MoveTowardsRestrictionGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let block_pos = mob.get_entity().block_pos.load();

        if mob_entity.is_in_position_target_range_pos(&block_pos) {
            return false;
        }

        let home_pos = mob_entity.position_target.load();
        let Some(pos) = Self::find_pos_towards(mob, home_pos) else {
            return false;
        };

        self.wanted_x = pos.x;
        self.wanted_y = pos.y;
        self.wanted_z = pos.z;
        true
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

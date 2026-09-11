use super::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

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

    fn find_random_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(-5.0..=5.0);
            let dy = rng.random_range(-4.0..=4.0);
            let dz = rng.random_range(-5.0..=5.0);

            let check_pos = BlockPos::new(
                (mob_pos.x + dx) as i32,
                (mob_pos.y + dy) as i32,
                (mob_pos.z + dz) as i32,
            );

            let block = world.get_block_state(&check_pos);
            let block_below = world.get_block_state(&check_pos.down());
            if !block.is_solid() && block_below.is_solid() {
                return Some(Vector3::new(
                    f64::from(check_pos.0.x) + 0.5,
                    f64::from(check_pos.0.y),
                    f64::from(check_pos.0.z) + 0.5,
                ));
            }
        }

        None
    }
}

impl Goal for RunAroundLikeCrazyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.is_tamed() || !mob.get_entity().has_passengers() {
            return false;
        }

        let Some(pos) = Self::find_random_pos(mob) else {
            return false;
        };

        self.pos_x = pos.x;
        self.pos_y = pos.y;
        self.pos_z = pos.z;
        true
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let is_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|nav| nav.is_idle());

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

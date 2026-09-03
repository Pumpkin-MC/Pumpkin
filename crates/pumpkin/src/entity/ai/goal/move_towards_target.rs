use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;

const HORIZONTAL_RANGE: f64 = 16.0;
const VERTICAL_RANGE: i32 = 7;
const TARGET_ATTEMPTS: usize = 10;

/// Mirrors vanilla Minecraft's `MoveTowardsTargetGoal`.
///
/// Moves the mob towards its current attack target within a specified maximum distance.
pub struct MoveTowardsTargetGoal {
    goal_control: Controls,
    speed: f64,
    within: f32,
    target: Option<Arc<dyn EntityBase>>,
    wanted_pos: Option<Vector3<f64>>,
}

impl MoveTowardsTargetGoal {
    #[must_use]
    pub const fn new(speed: f64, within: f32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            within,
            target: None,
            wanted_pos: None,
        }
    }

    /// Mirrors vanilla's `DefaultRandomPos.getPosTowards(mob, 16, 7, target.position(), Math.PI / 2)`.
    fn find_pos_towards(mob: &dyn Mob, target_pos: &Vector3<f64>) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let mob_pos = entity.pos.load();
        let world = entity.world.load();

        let dir_x = target_pos.x - mob_pos.x;
        let dir_z = target_pos.z - mob_pos.z;

        let mut rng = mob.get_random();
        let (dir_x, dir_z) = if dir_x == 0.0 && dir_z == 0.0 {
            (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
        } else {
            (dir_x, dir_z)
        };
        let base_angle = dir_z.atan2(dir_x) - std::f64::consts::FRAC_PI_2;

        for _ in 0..TARGET_ATTEMPTS {
            let angle =
                base_angle + (2.0 * rng.random_range(0.0..1.0) - 1.0) * std::f64::consts::FRAC_PI_2;
            let t = rng.random_range(0.0..1.0f64).sqrt();
            let dist = t * HORIZONTAL_RANGE * std::f64::consts::SQRT_2;
            let dx = -dist * angle.sin();
            let dz = dist * angle.cos();

            if dx.abs() > HORIZONTAL_RANGE || dz.abs() > HORIZONTAL_RANGE {
                continue;
            }

            let dy = rng.random_range(-VERTICAL_RANGE..=VERTICAL_RANGE);

            let candidate = BlockPos::new(
                (mob_pos.x + dx) as i32,
                (mob_pos.y + dy as f64) as i32,
                (mob_pos.z + dz) as i32,
            );

            let block_at = world.get_block_state(&candidate);
            let block_below = world.get_block_state(&BlockPos::new(
                candidate.0.x,
                candidate.0.y - 1,
                candidate.0.z,
            ));

            if block_at.is_solid() || !block_below.is_solid() {
                continue;
            }

            return Some(Vector3::new(
                candidate.0.x as f64 + 0.5,
                candidate.0.y as f64,
                candidate.0.z as f64 + 0.5,
            ));
        }

        None
    }
}

impl Goal for MoveTowardsTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target();
        let Some(target) = target else {
            self.target = None;
            return false;
        };

        if !target.get_entity().is_alive() {
            self.target = None;
            return false;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);
        let within_sq = f64::from(self.within) * f64::from(self.within);

        if dist_sq > within_sq {
            self.target = None;
            return false;
        }

        let pos = Self::find_pos_towards(mob, &target_pos);
        let Some(pos) = pos else {
            self.target = None;
            return false;
        };

        self.wanted_pos = Some(pos);
        self.target = Some(target);
        true
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let Some(target) = &self.target else {
            return false;
        };

        if !target.get_entity().is_alive() {
            return false;
        }

        let navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if navigator.is_idle() {
            return false;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);
        let within_sq = f64::from(self.within) * f64::from(self.within);

        dist_sq < within_sq
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(wanted_pos) = self.wanted_pos {
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_progress(NavigatorGoal::new(mob_pos, wanted_pos, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.wanted_pos = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

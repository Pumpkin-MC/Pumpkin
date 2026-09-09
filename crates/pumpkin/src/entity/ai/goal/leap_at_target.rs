use super::{Controls, Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub struct LeapAtTargetGoal {
    goal_control: Controls,
    yd: f32,
    target: Option<Arc<dyn EntityBase>>,
}

impl LeapAtTargetGoal {
    #[must_use]
    pub const fn new(yd: f32) -> Self {
        Self {
            goal_control: Controls::MOVE.union(Controls::JUMP),
            yd,
            target: None,
        }
    }
}

impl Goal for LeapAtTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;

        if !entity.on_ground.load(Ordering::Relaxed) {
            return false;
        }

        let target = mob_entity.get_target();
        let Some(target) = target else {
            return false;
        };

        if !target.get_entity().is_alive() {
            return false;
        }

        let mob_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

        if !(4.0..=16.0).contains(&dist_sq) {
            return false;
        }

        if mob.get_random().random_range(0..to_goal_ticks(5).max(1)) != 0 {
            return false;
        }

        self.target = Some(target);
        true
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let entity = &mob.get_mob_entity().living_entity.entity;
        !entity.on_ground.load(Ordering::Relaxed)
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(target) = &self.target else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let mob_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let movement = entity.velocity.load();

        let dx = target_pos.x - mob_pos.x;
        let dz = target_pos.z - mob_pos.z;
        let dist_xz_sq = dx * dx + dz * dz;

        let delta = if dist_xz_sq > 1.0E-7 {
            let dist_xz = dist_xz_sq.sqrt();
            Vector3::new(
                (dx / dist_xz) * 0.4 + movement.x * 0.2,
                f64::from(self.yd),
                (dz / dist_xz) * 0.4 + movement.z * 0.2,
            )
        } else {
            Vector3::new(movement.x, f64::from(self.yd), movement.z)
        };

        entity.set_velocity(delta);
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

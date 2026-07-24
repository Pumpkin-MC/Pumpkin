//! Vanilla `LeapAtTargetGoal` — used by spiders (yd = 0.4).

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct LeapAtTargetGoal {
    /// Vertical impulse (vanilla spider: 0.4).
    yd: f64,
}

impl LeapAtTargetGoal {
    #[must_use]
    pub fn new(yd: f64) -> Self {
        Self { yd }
    }
}

impl Goal for LeapAtTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            let entity = mob.get_entity();
            // Must be on ground (vanilla checks onGround).
            if !entity.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                return false;
            }
            let mob_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist = mob_pos.squared_distance_to_vec(&target_pos).sqrt();
            // Vanilla: 4.0 < dist < 16.0
            (4.0..16.0).contains(&dist) && mob.get_random().random_range(0..5) == 0
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            !mob.get_entity()
                .on_ground
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return;
            };
            let entity = mob.get_entity();
            let mob_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let dx = target_pos.x - mob_pos.x;
            let dz = target_pos.z - mob_pos.z;
            let horiz = (dx * dx + dz * dz).sqrt().max(1.0e-4);
            let mut vel = entity.velocity.load();
            // Vanilla: add (dx/h * 0.5, yd, dz/h * 0.5) scaled by current speed.
            vel.x += (dx / horiz) * 0.5;
            vel.y = self.yd;
            vel.z += (dz / horiz) * 0.5;
            entity.velocity.store(vel);
        })
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}

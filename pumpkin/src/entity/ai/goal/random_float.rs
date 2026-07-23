//! Ghast-style random floating (vanilla `RandomFloatAroundGoal` simplified).

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct RandomFloatGoal {
    cooldown: i32,
}

impl RandomFloatGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { cooldown: 0 })
    }
}

impl Goal for RandomFloatGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.cooldown -= 1;
            if self.cooldown > 0 {
                return;
            }
            self.cooldown = 20 + mob.get_random().random_range(0..40);

            // Pick a random direction and gentle speed (vanilla ~0.1–0.2).
            let mut rng = mob.get_random();
            let vx = rng.random_range(-0.5..0.5);
            let vy = rng.random_range(-0.25..0.25);
            let vz = rng.random_range(-0.5..0.5);
            drop(rng);

            let entity = mob.get_entity();
            let mut vel = entity.velocity.load();
            // Blend toward new float direction.
            vel = Vector3::new(
                vel.x * 0.2 + vx * 0.4,
                vel.y * 0.2 + vy * 0.3,
                vel.z * 0.2 + vz * 0.4,
            );
            entity.set_velocity(vel);
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

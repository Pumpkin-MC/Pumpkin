use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// A goal that makes the mob run away in a random direction when hurt.
///
/// When the mob's health drops (taking damage), it panics and runs
/// at an increased speed to a random nearby position.
pub struct PanicGoal {
    goal_control: Controls,
    speed: f64,
    is_panicking: bool,
    last_health: f32,
}

impl PanicGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            speed,
            is_panicking: false,
            last_health: -1.0,
        })
    }

    fn find_flee_target(mob: &dyn Mob) -> Vector3<f64> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        // Run to a random position 5-10 blocks away
        let dx = rng.random_range(-10.0f64..10.0);
        let dz = rng.random_range(-10.0f64..10.0);

        Vector3::new(pos.x + dx, pos.y, pos.z + dz)
    }
}

impl Goal for PanicGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let health = mob_entity.living_entity.health.load();

            if self.last_health < 0.0 {
                // First tick, just record health
                self.last_health = health;
                return false;
            }

            // Trigger panic if we took damage
            if health < self.last_health {
                self.last_health = health;
                self.is_panicking = true;
                return true;
            }
            self.last_health = health;
            false
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if !self.is_panicking {
                return false;
            }
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = Self::find_flee_target(mob);
            let mob_entity = mob.get_mob_entity();
            let current_pos = mob_entity.living_entity.entity.pos.load();
            let mut navigator = mob_entity.navigator.lock().await;
            navigator.set_progress(NavigatorGoal {
                current_progress: current_pos,
                destination: target,
                speed: self.speed,
            });
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.is_panicking = false;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

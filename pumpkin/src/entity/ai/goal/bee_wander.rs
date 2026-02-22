use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// Flying wander goal for bees. Unlike ground-based `WanderAroundGoal`,
/// bees pick random air positions and hover towards them with velocity-based movement.
/// Adapted from vanilla `BeeWanderGoal`.
pub struct BeeWanderGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    chance: i32,
}

impl BeeWanderGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            chance: to_goal_ticks(10), // vanilla: nextInt(10) == 0
        }
    }

    /// Pick a random air position nearby, biased towards the bee's view direction.
    /// Vanilla uses HoverRandomPos / AirAndWaterRandomPos; we approximate with random offsets.
    fn find_hover_target(mob: &dyn Mob) -> Vector3<f64> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let yaw_rad = entity.yaw.load().to_radians();
        let mut rng = mob.get_random();

        // Bias direction by view vector (vanilla: getViewVector)
        let view_x = -(yaw_rad.sin() as f64);
        let view_z = yaw_rad.cos() as f64;

        let range = 8.0;
        let y_range = 4.0;

        let dx = view_x * range * 0.5 + rng.random_range(-range..=range) * 0.5;
        let dy = rng.random_range(-y_range..=y_range) * 0.5;
        let dz = view_z * range * 0.5 + rng.random_range(-range..=range) * 0.5;

        Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz)
    }
}

impl Goal for BeeWanderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            // Only start wandering occasionally, like vanilla 1/10 chance
            if mob.get_random().random_range(0..self.chance) != 0 {
                return false;
            }

            self.target = Some(Self::find_hover_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

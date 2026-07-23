use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct WanderAroundGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    chance: i32,
}

impl WanderAroundGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            // Vanilla RandomStroll uses ~120 goal-tick chance. With pathing fixed,
            // keep near-vanilla frequency so mobs stroll without looking frantic.
            chance: to_goal_ticks(120),
        }
    }

    fn find_wander_target(mob: &dyn Mob) -> Vector3<f64> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let mut rng = mob.get_random();

        // Prefer walkable ground near the mob so pathfinding can succeed.
        let horizontal_range = 10.0;
        for _ in 0..16 {
            let dx = rng.random_range(-horizontal_range..=horizontal_range);
            let dz = rng.random_range(-horizontal_range..=horizontal_range);
            let sample_x = pos.x + dx;
            let sample_z = pos.z + dz;
            // Scan a small vertical window around the mob for solid ground.
            for dy in (-2..=2).rev() {
                let feet_y = pos.y + f64::from(dy);
                let feet =
                    pumpkin_util::math::position::BlockPos::floored(sample_x, feet_y, sample_z);
                let below = feet.down();
                let below_state = world.get_block_state(&below);
                let feet_state = world.get_block_state(&feet);
                let head_state = world.get_block_state(&feet.up());
                if below_state.is_solid() && !feet_state.is_solid() && !head_state.is_solid() {
                    return Vector3::new(
                        f64::from(feet.0.x) + 0.5,
                        f64::from(feet.0.y),
                        f64::from(feet.0.z) + 0.5,
                    );
                }
            }
        }

        // Fallback: short same-level step.
        let dx = rng.random_range(-6.0..=6.0);
        let dz = rng.random_range(-6.0..=6.0);
        Vector3::new(pos.x + dx, pos.y, pos.z + dz)
    }
}

impl Goal for WanderAroundGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            // Vanilla-ish: don't wander while sitting / already navigating.
            if mob.is_sitting() {
                return false;
            }
            {
                let navigator = mob.get_mob_entity().navigator.lock().unwrap();
                if !navigator.is_idle() {
                    return false;
                }
            }
            let chance = self.chance.max(1);
            if mob.get_random().random_range(0..chance) != 0 {
                return false;
            }

            self.target = Some(Self::find_wander_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().unwrap();
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
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

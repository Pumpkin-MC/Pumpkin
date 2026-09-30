use super::{Controls, Goal};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::RngExt;

/// Vanilla `RandomSwimmingGoal` (fish strolling).
///
/// Unlike the generic [`WanderAroundGoal`](super::wander_around::WanderAroundGoal), this goal only
/// picks targets whose block is water (`BehaviorUtils#getRandomSwimmablePos` resamples until the
/// candidate block is pathfindable as water), so fish never navigate onto land.
pub struct RandomSwimmingGoal {
    goal_control: Controls,
    speed: f64,
    interval: i32,
    target: Option<Vector3<f64>>,
}

impl RandomSwimmingGoal {
    #[must_use]
    pub const fn new(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            interval,
            target: None,
        }
    }

    /// Vanilla picks a random 3D offset (±10 horizontal, ±7 vertical) around the mob and resamples
    /// up to 10 times until the candidate block is water-pathfindable. Since pumpkin's navigator
    /// has no water-bound pathing constraint, we reject invalid candidates instead of returning
    /// the last one, so fish are never driven out of the water.
    fn find_swimmable_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load_full();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(-10.0..=10.0);
            let dy = rng.random_range(-7.0..=7.0);
            let dz = rng.random_range(-10.0..=10.0);

            let target = Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz);
            let block_pos = BlockPos::containing_vec(target);
            if world
                .get_fluid(&block_pos)
                .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            {
                return Some(target);
            }
        }

        None
    }
}

impl Goal for RandomSwimmingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_random().random_range(0..self.interval) != 0 {
            return false;
        }

        self.target = Self::find_swimmable_target(mob);
        self.target.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !navigator.is_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

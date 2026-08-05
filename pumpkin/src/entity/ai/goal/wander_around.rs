use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct WanderAroundGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    chance: i32,
    /// Vanilla: `WaterAvoidingRandomStrollGoal` overrides `getPosition()` to reject candidate
    /// positions inside a liquid.
    avoid_water: bool,
}

impl WanderAroundGoal {
    /// Vanilla: `RandomStrollGoal.DEFAULT_INTERVAL` (`RandomStrollGoal.java`).
    const DEFAULT_INTERVAL: i32 = 120;

    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self::new_with_interval(speed, Self::DEFAULT_INTERVAL)
    }

    /// Vanilla: `RandomStrollGoal(mob, speedModifier, interval)` / `RandomSwimmingGoal`, whose
    /// callers pass a mob-specific interval instead of the 120-tick default.
    #[must_use]
    pub const fn new_with_interval(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            chance: to_goal_ticks(interval),
            avoid_water: false,
        }
    }

    /// Vanilla: `WaterAvoidingRandomStrollGoal`.
    #[must_use]
    pub const fn new_water_avoiding(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            chance: to_goal_ticks(Self::DEFAULT_INTERVAL),
            avoid_water: true,
        }
    }

    /// Vanilla: `RandomStrollGoal#setInterval`, e.g. `ElderGuardian`'s constructor
    /// overriding its inherited `randomStrollGoal` interval from 80 to 400.
    #[must_use]
    pub const fn with_interval(mut self, interval: i32) -> Self {
        self.chance = to_goal_ticks(interval);
        self
    }

    fn find_wander_target(mob: &dyn Mob) -> Vector3<f64> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let mut rng = mob.get_random();

        let horizontal_range = 10.0;
        let vertical_range = 7.0;

        let dx = rng.random_range(-horizontal_range..=horizontal_range);
        let dy = rng.random_range(-vertical_range..=vertical_range);
        let dz = rng.random_range(-horizontal_range..=horizontal_range);

        Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz)
    }
}

impl Goal for WanderAroundGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_random().random_range(0..self.chance) != 0 {
                return false;
            }

            if self.avoid_water {
                let world = mob.get_entity().world.load();
                // Reroll a bounded number of times looking for a non-liquid landing spot,
                // falling back to the last roll (matching vanilla's "give up" behavior) rather
                // than not moving at all.
                let mut candidate = Self::find_wander_target(mob);
                for _ in 0..10 {
                    let block_pos = pumpkin_util::math::position::BlockPos::new(
                        candidate.x.floor() as i32,
                        candidate.y.floor() as i32,
                        candidate.z.floor() as i32,
                    );
                    if !world.get_block_state(&block_pos).is_liquid() {
                        break;
                    }
                    candidate = Self::find_wander_target(mob);
                }
                self.target = Some(candidate);
                return true;
            }

            self.target = Some(Self::find_wander_target(mob));
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_interval_overrides_default_chance() {
        let default_goal = WanderAroundGoal::new(1.0);
        let overridden = WanderAroundGoal::new(1.0).with_interval(400);
        assert_ne!(default_goal.chance, overridden.chance);
        // Vanilla `ElderGuardian`'s override: `randomStrollGoal.setInterval(400)`.
        assert_eq!(overridden.chance, 200);
    }

    #[test]
    fn default_interval_matches_vanilla() {
        let goal = WanderAroundGoal::new(1.0);
        assert_eq!(goal.chance, to_goal_ticks(120));
    }

    #[test]
    fn custom_interval_is_applied() {
        // Fish family: AbstractFish.FishSwimGoal uses interval 40.
        let goal = WanderAroundGoal::new_with_interval(1.0, 40);
        assert_eq!(goal.chance, to_goal_ticks(40));
        assert_ne!(goal.chance, to_goal_ticks(120));
    }

    #[test]
    fn water_avoiding_keeps_default_interval() {
        let goal = WanderAroundGoal::new_water_avoiding(0.4);
        assert_eq!(goal.chance, to_goal_ticks(120));
        assert!(goal.avoid_water);
    }
}

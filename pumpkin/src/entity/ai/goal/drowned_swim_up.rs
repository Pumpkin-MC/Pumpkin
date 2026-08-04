use std::sync::atomic::Ordering::Relaxed;

use pumpkin_util::math::vector3::Vector3;

use super::drowned_util::is_bright_outside;
use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

/// `Drowned.DrownedSwimUpGoal` (`Drowned.java:482-529`): a submerged drowned swims up toward
/// the surface once it's deep enough.
///
/// Gated on it not being bright outside (same "safe from sunlight" condition as
/// `DrownedGoToBeachGoal`).
///
/// Vanilla drives this with `DefaultRandomPos.getPosTowards` (a random pathfinder-node pick
/// biased toward the surface) plus a `searchingForLand`/`stuck` state pair consumed by
/// `Drowned#wantsToSwim` and the custom `DrownedMoveControl`. This codebase has no swimming
/// pathfinder mode (`NodeEvaluator::can_swim` is unused, see `walk_node_evaluator.rs`) and no
/// per-mob `MoveControl` override point, so this goal approximates the behavior by steering
/// the navigator straight toward the surface above the drowned's current position instead.
pub struct DrownedSwimUpGoal {
    speed: f64,
}

impl DrownedSwimUpGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self { speed })
    }

    fn wants_to_surface(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        !is_bright_outside(&world)
            && entity.touching_water.load(Relaxed)
            && entity.pos.load().y < f64::from(world.sea_level - 2)
    }
}

impl Goal for DrownedSwimUpGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::wants_to_surface(mob) })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { Self::wants_to_surface(mob) })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            if pos.y >= f64::from(world.sea_level - 1) {
                return;
            }
            let navigator = mob.get_mob_entity().navigator.lock().unwrap();
            if !navigator.is_idle() {
                return;
            }
            drop(navigator);
            let destination = Vector3::new(pos.x, f64::from(world.sea_level - 1), pos.z);
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal::new(pos, destination, self.speed));
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

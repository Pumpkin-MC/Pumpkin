use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::drowned_util::is_bright_outside;
use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

/// `Drowned.DrownedGoToWaterGoal` (`Drowned.java:381-440`): a landed drowned retreats to
/// nearby water during the day.
///
/// `getWaterPos` samples 10 random offsets in x/z in `[-10, 10)` and y in `[-5, 2]`
/// (`2 - nextInt(8)`), taking the first that lands on a `Blocks.WATER` block.
pub struct DrownedGoToWaterGoal {
    speed: f64,
    target: Option<Vector3<f64>>,
}

impl DrownedGoToWaterGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            target: None,
        })
    }

    fn find_water_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = mob.get_entity();
        let origin = entity.block_pos.load();
        let world = entity.world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(0..20) - 10;
            let dy = 2 - rng.random_range(0..8);
            let dz = rng.random_range(0..20) - 10;
            let pos = BlockPos::new(origin.0.x + dx, origin.0.y + dy, origin.0.z + dz);
            if world.get_block(&pos) == &Block::WATER {
                return Some(Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y),
                    f64::from(pos.0.z) + 0.5,
                ));
            }
        }
        None
    }
}

impl Goal for DrownedGoToWaterGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let world = mob.get_entity().world.load();
            if !is_bright_outside(&world) {
                return false;
            }
            if mob.get_entity().touching_water.load(Relaxed) {
                return false;
            }
            let Some(target) = Self::find_water_pos(mob) else {
                return false;
            };
            self.target = Some(target);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !mob.get_mob_entity().navigator.lock().unwrap().is_idle() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_entity().pos.load();
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap()
                    .set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

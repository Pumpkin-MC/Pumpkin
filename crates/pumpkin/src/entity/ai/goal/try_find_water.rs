use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::{BlockPos, BlockPosIterator};

/// `TryFindWaterGoal.java`: a stranded, grounded mob paths back to the nearest water block.
///
/// Searches a small box around itself (`Mth.floor(x-2)..=Mth.floor(x+2)`,
/// `Mth.floor(y-2)..=blockY`, `Mth.floor(z-2)..=Mth.floor(z+2)`), first match wins, no distance
/// comparison.
pub struct TryFindWaterGoal;

impl TryFindWaterGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Goal for TryFindWaterGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            if !entity.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                return false;
            }
            let pos = entity.block_pos.load();
            let world = entity.world.load();
            let (fluid, _) = world.get_fluid_and_fluid_state(&pos);
            !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let origin = entity.block_pos.load();
            let world = entity.world.load();

            let mut water_pos: Option<BlockPos> = None;
            for pos in BlockPosIterator::new(
                origin.0.x - 2,
                origin.0.y - 2,
                origin.0.z - 2,
                origin.0.x + 2,
                origin.0.y,
                origin.0.z + 2,
            ) {
                let (fluid, _) = world.get_fluid_and_fluid_state(&pos);
                if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                    water_pos = Some(pos);
                    break;
                }
            }

            if let Some(target) = water_pos {
                let mob_pos = entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(mob_pos, target.to_f64(), 1.0));
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

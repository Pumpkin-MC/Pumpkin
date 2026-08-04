use pumpkin_data::structures::StructureSet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::generation::generator::structure_finder::find_nearest_structure;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, passive::dolphin::DolphinEntity};

/// `Dolphin.DolphinSwimToTreasureGoal`: while `gotFish` and air supply is sufficient, finds the
/// nearest buried treasure and swims to it, clearing `gotFish` on arrival or when stuck.
///
/// Vanilla searches `StructureTags.DOLPHIN_LOCATED`, which currently resolves to buried
/// treasure only; this is ported against `StructureSet::BURIED_TREASURES` directly (matching
/// `find_stronghold` in `pumpkin/src/item/items/ender_eye.rs`, the only other consumer of
/// `find_nearest_structure` in this codebase) rather than a generic structure-tag lookup, since
/// no such generic tag-to-placement-list mapping exists yet.
const MAX_SEARCH_RADIUS_CHUNKS: i32 = 50;
const ARRIVAL_DISTANCE_SQ: f64 = 4.0;

pub struct DolphinSwimToTreasureGoal {
    speed: f64,
    treasure_pos: Option<BlockPos>,
}

impl DolphinSwimToTreasureGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            treasure_pos: None,
        })
    }

    fn find_treasure(mob: &dyn Mob) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let generator = &world.level.world_gen;
        let seed = world.level.seed.0;
        let global_cache = generator.global_structure_cache()?;

        find_nearest_structure(
            entity.block_pos.load(),
            &[&StructureSet::BURIED_TREASURES.placement],
            MAX_SEARCH_RADIUS_CHUNKS,
            seed as i64,
            global_cache,
        )
    }
}

impl Goal for DolphinSwimToTreasureGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(dolphin) = mob.cast_any().downcast_ref::<DolphinEntity>() else {
                return false;
            };
            if !dolphin.got_fish() {
                return false;
            }
            // Vanilla also requires air supply >= 100 here; this codebase's `air_supply`
            // tracking (`pumpkin/src/entity/breath.rs`) is currently player-only, with no
            // generic per-mob air-supply field to read, so that gate is dropped rather than
            // guessed at.

            let Some(pos) = Self::find_treasure(mob) else {
                return false;
            };
            self.treasure_pos = Some(pos);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(pos) = self.treasure_pos else {
                return false;
            };
            let my_pos = mob.get_entity().pos.load();
            my_pos.squared_distance_to_vec(&pos.to_f64()) > ARRIVAL_DISTANCE_SQ
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.treasure_pos = None;
            if let Some(dolphin) = mob.cast_any().downcast_ref::<DolphinEntity>() {
                dolphin.set_got_fish(false);
            }
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(pos) = self.treasure_pos else {
                return;
            };
            let my_pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(my_pos, pos.to_f64(), self.speed));
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

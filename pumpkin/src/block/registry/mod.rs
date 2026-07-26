use super::fluid::FluidBehaviour;
use crate::block::{BlockBehaviour, BlockMetadata, FluidMetadata};
use pumpkin_data::BlockId;
use rustc_hash::FxHashMap;
use std::sync::Arc;

mod default;
mod events;
mod neighbors;
mod place;

pub use default::default_registry;

// ActionResult.java
#[derive(PartialEq, Eq)]
pub enum BlockActionResult {
    /// Action was successful | Same as SUCCESS in vanilla
    Success,
    /// Action was successful and we should swing the hand for the server | Same as `SUCCESS_SERVER` in vanilla
    SuccessServer,
    /// Block other actions from being executed | Same as CONSUME in vanilla
    Consume,
    /// Allow other actions to be executed, but indicate it failed | Same as FAIL in vanilla
    Fail,
    /// Allow other actions to be executed | Same as PASS in vanilla
    Pass,
    /// Use default action for the block: `normal_use` | Same as `PASS_TO_DEFAULT_BLOCK_ACTION` in vanilla
    PassToDefaultBlockAction,
}

impl BlockActionResult {
    #[must_use]
    pub const fn consumes_action(&self) -> bool {
        matches!(self, Self::Consume | Self::Success | Self::SuccessServer)
    }
}

#[derive(Default)]
pub struct BlockRegistry {
    blocks: FxHashMap<BlockId, Arc<dyn BlockBehaviour>>,
    fluids: FxHashMap<u16, Arc<dyn FluidBehaviour>>,
}

#[derive(Debug)]
pub enum BlockPlacingError {
    InvalidGamemode,
    BlockOutOfWorld,
}

impl BlockRegistry {
    pub fn register<T: BlockBehaviour + BlockMetadata + 'static>(&mut self, block: T) {
        let ids = T::ids();
        let val = Arc::new(block);
        self.blocks.reserve(ids.len());
        for i in ids {
            self.blocks.insert(i, val.clone());
        }
    }

    pub fn register_fluid<T: FluidBehaviour + FluidMetadata + 'static>(&mut self, fluid: T) {
        let ids = T::ids();
        let val = Arc::new(fluid);
        self.fluids.reserve(ids.len());
        for i in ids {
            self.fluids.insert(i, val.clone());
        }
    }

    #[must_use]
    pub fn get_pumpkin_block(&self, block: BlockId) -> Option<&Arc<dyn BlockBehaviour>> {
        self.blocks.get(&block)
    }

    #[must_use]
    pub fn get_pumpkin_fluid(&self, fluid_id: u16) -> Option<&Arc<dyn FluidBehaviour>> {
        self.fluids.get(&fluid_id).or_else(|| {
            // Still fluids share behavior with their flowing counterpart
            match fluid_id {
                2 => self.fluids.get(&1),
                4 => self.fluids.get(&3),
                _ => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockActionResult, BlockRegistry, default_registry};
    use pumpkin_data::block_rotation::{Mirror, Rotation};
    use pumpkin_data::fluid::Fluid;
    use pumpkin_data::{Block, BlockState, BlockStateId};

    #[test]
    fn action_result_consumption() {
        assert!(BlockActionResult::Success.consumes_action());
        assert!(BlockActionResult::SuccessServer.consumes_action());
        assert!(BlockActionResult::Consume.consumes_action());
        assert!(!BlockActionResult::Fail.consumes_action());
        assert!(!BlockActionResult::Pass.consumes_action());
        assert!(!BlockActionResult::PassToDefaultBlockAction.consumes_action());
    }

    #[test]
    fn default_registry_resolves_registered_blocks() {
        let registry = default_registry();
        assert!(registry.get_pumpkin_block(Block::CHEST.id).is_some());
        assert!(registry.get_pumpkin_block(Block::FURNACE.id).is_some());
        assert!(registry.get_pumpkin_block(Block::AIR.id).is_none());
    }

    #[test]
    fn still_fluids_fall_back_to_flowing_behaviour() {
        let registry = default_registry();
        assert!(
            registry
                .get_pumpkin_fluid(Fluid::FLOWING_WATER.id)
                .is_some()
        );
        assert!(registry.get_pumpkin_fluid(Fluid::WATER.id).is_some());
        assert!(registry.get_pumpkin_fluid(Fluid::FLOWING_LAVA.id).is_some());
        assert!(registry.get_pumpkin_fluid(Fluid::LAVA.id).is_some());
        assert!(registry.get_pumpkin_fluid(u16::MAX).is_none());
    }

    #[test]
    fn moved_registry_entry_points_remain_reachable() {
        // Typed fn-pointer coercions fail to compile if the moved methods
        // change their paths or signatures.
        std::hint::black_box::<
            fn(&BlockRegistry, &Block, BlockStateId, Mirror) -> &'static BlockState,
        >(BlockRegistry::mirror);
        std::hint::black_box::<
            fn(&BlockRegistry, &Block, BlockStateId, Rotation) -> &'static BlockState,
        >(BlockRegistry::rotate);
        // Existence checks for the moved async entry points.
        std::hint::black_box(BlockRegistry::place_block);
        std::hint::black_box(BlockRegistry::can_place_at);
        std::hint::black_box(BlockRegistry::can_update_at);
        std::hint::black_box(BlockRegistry::on_place);
        std::hint::black_box(BlockRegistry::player_placed);
        std::hint::black_box(BlockRegistry::on_placed);
        std::hint::black_box(BlockRegistry::on_placed_fluid);
        std::hint::black_box(BlockRegistry::on_use);
        std::hint::black_box(BlockRegistry::use_with_item);
        std::hint::black_box(BlockRegistry::use_with_item_fluid);
        std::hint::black_box(BlockRegistry::broken);
        std::hint::black_box(BlockRegistry::on_state_replaced);
        std::hint::black_box(BlockRegistry::on_synced_block_event);
        std::hint::black_box(BlockRegistry::on_entity_collision);
        std::hint::black_box(BlockRegistry::on_entity_collision_fluid);
        std::hint::black_box(BlockRegistry::on_entity_step);
        std::hint::black_box(BlockRegistry::on_landed_upon);
        std::hint::black_box(BlockRegistry::update_entity_movement_after_fall_on);
        std::hint::black_box(BlockRegistry::explode);
        std::hint::black_box(BlockRegistry::post_process_state);
        std::hint::black_box(BlockRegistry::prepare);
        std::hint::black_box(BlockRegistry::get_state_for_neighbor_update);
        std::hint::black_box(BlockRegistry::update_neighbors);
        std::hint::black_box(BlockRegistry::on_neighbor_update);
        std::hint::black_box(BlockRegistry::emits_redstone_power);
        std::hint::black_box(BlockRegistry::get_weak_redstone_power);
        std::hint::black_box(BlockRegistry::get_strong_redstone_power);
        std::hint::black_box(BlockRegistry::get_inside_collision_shape);
    }
}

use std::sync::Arc;

use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{BlockProperties, OakSaplingLikeProperties},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
};
use crate::plugin::api::events::world::structure_grow::{StructureGrowEvent, TreeType};
use crate::world::World;

#[pumpkin_block_from_tag("minecraft:saplings")]
pub struct SaplingBlock;

impl SaplingBlock {
    #[must_use]
    pub fn get_tree_type(block: &Block) -> TreeType {
        match block.name {
            "oak_sapling" => TreeType::Oak,
            "spruce_sapling" => TreeType::Spruce,
            "birch_sapling" => TreeType::Birch,
            "jungle_sapling" => TreeType::Jungle,
            "acacia_sapling" => TreeType::Acacia,
            "dark_oak_sapling" | "pale_oak_sapling" => TreeType::DarkOak,
            "cherry_sapling" => TreeType::Cherry,
            "azalea" | "flowering_azalea" => TreeType::Azalea,
            "mangrove_propagule" => TreeType::Mangrove,
            _ => TreeType::Custom,
        }
    }

    pub fn advance_tree(
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        bone_meal: bool,
    ) {
        if OakSaplingLikeProperties::handles_block_id(block.id) {
            let mut props = OakSaplingLikeProperties::from_state_id(state_id, block);
            if props.stage == 0 {
                props.stage = 1;
                world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);
                return;
            }
        }

        let tree_type = Self::get_tree_type(block);
        let mut event = StructureGrowEvent::new(*pos, tree_type, bone_meal);
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
    }
}

impl BlockBehaviour for SaplingBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::random::<u8>().is_multiple_of(7) {
            let state_id = args.world.get_block_state_id(args.position);
            Self::advance_tree(args.world, args.position, args.block, state_id, false);
        }
    }

    fn is_valid_bonemeal_target(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        rand::random::<f32>() < 0.45
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        {
            Self::advance_tree(args.world, args.position, args.block, args.state_id, true);
        }
    }
}

impl PlantBlockBase for SaplingBlock {}

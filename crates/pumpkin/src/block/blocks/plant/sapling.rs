use std::sync::Arc;

use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{BlockProperties, OakSaplingLikeProperties},
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_util::random::{RandomGenerator, xoroshiro128::Xoroshiro};
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::tree_grower::TreeGrower;
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
            let mut props = OakSaplingLikeProperties::from_state_id(state_id);
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
            if event.cancelled {
                return;
            }
        }
        let Some(grower) = TreeGrower::for_block(block) else {
            return;
        };
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::random::<u64>()));
        grower.grow_tree(world, pos, block, state_id, &mut random);
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
        if args.world.get_max_local_raw_brightness(&args.position.up()) >= 9
            && rand::random_range(0..7) == 0
        {
            let state_id = args.world.get_block_state_id(args.position);
            Self::advance_tree(args.world, args.position, args.block, state_id, false);
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let Some(grower) = TreeGrower::for_block(args.block) else {
            return false;
        };
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::random::<u64>()));
        grower.can_grow(args.world, args.position, args.block, &mut random)
            && args
                .world
                .is_in_build_limit(
                    args.position
                        .offset(Vector3::new(0, grower.min_height(), 0)),
                )
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

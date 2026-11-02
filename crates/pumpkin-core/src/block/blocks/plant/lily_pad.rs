use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockId, BlockStateId, fluid::Fluid};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};
use crate::world::World;

use crate::block::{BlockBehaviour, CanPlaceAtArgs, OnEntityCollisionArgs};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

impl BlockBehaviour for LilyPadBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if args
            .entity
            .get_entity()
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_BOAT)
        {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

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
}

impl PlantBlockBase for LilyPadBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let (block, state) = block_accessor.get_block_and_state(pos);
        let above_state = block_accessor.get_block_state(&pos.up());
        let above_fluid = World::get_fluid_from_state_id(above_state.id);
        let is_source_water = (block == &Block::WATER && state.id == Block::WATER.default_state.id)
            || state.is_waterlogged()
            || matches!(
                block.id,
                BlockId::KELP
                    | BlockId::KELP_PLANT
                    | BlockId::SEAGRASS
                    | BlockId::TALL_SEAGRASS
                    | BlockId::BUBBLE_COLUMN
            );
        (is_source_water || block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_LILY_PAD))
            && *above_fluid == Fluid::EMPTY
    }
}

use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::BlockFuture;
use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::WARPED_ROOTS, BlockId::CRIMSON_ROOTS].into()
    }
}

impl BlockBehaviour for RootsBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for RootsBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block_below = block_accessor.get_block(pos);
        if block_below == &Block::WARPED_ROOTS {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_ROOTS)
        } else {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_ROOTS)
        }
    }

    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        self.can_plant_on_top(block_accessor, &block_pos.down())
    }
}

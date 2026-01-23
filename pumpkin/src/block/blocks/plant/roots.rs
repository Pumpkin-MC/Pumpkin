use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::BlockFuture;
use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::WARPED_ROOTS.name, Block::CRIMSON_ROOTS.name]
    }
}

impl BlockBehaviour for RootsBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
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
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block_below = block_accessor.get_block(pos).await;
        block_below.has_tag(&tag::Block::MINECRAFT_NYLIUM)
            || block_below == &Block::SOUL_SOIL
            || block_below.has_tag(&tag::Block::MINECRAFT_DIRT)
            || block_below == &Block::FARMLAND
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        self.can_plant_on_top(block_accessor, &block_pos.down())
            .await
    }
}

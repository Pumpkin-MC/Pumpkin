use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

#[pumpkin_block("minecraft:seagrass")]
pub struct SeaGrassBlock;

impl BlockBehaviour for SeaGrassBlock {
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

impl PlantBlockBase for SeaGrassBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos).await;
        let block_state = block_accessor.get_block_state(pos).await;
        block_state.is_side_solid(BlockDirection::Up) && block != &Block::MAGMA_BLOCK
    }
}

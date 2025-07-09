use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::{
    blocks::plant::PlantBlockBase,
    pumpkin_block::{CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock},
};

#[pumpkin_block("minecraft:seagrass")]
pub struct SeaGrassBlock;

#[async_trait]
impl PumpkinBlock for SeaGrassBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
        .await
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

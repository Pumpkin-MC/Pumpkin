use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{GetStateForNeighborUpdateArgs, blocks::plant::PlantBlockBase};

use crate::block::{BlockBehaviour, CanPlaceAtArgs, OnEntityCollisionArgs};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

#[async_trait]
impl BlockBehaviour for LilyPadBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        // Proberbly not the best solution, but works
        if args
            .entity
            .get_entity()
            .entity_type
            .resource_name
            .ends_with("_boat")
        {
            args.world
                .break_block(args.position, None, BlockFlags::empty())
                .await;
        }
    }

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

impl PlantBlockBase for LilyPadBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        let above_fluid = block_accessor.get_block(&pos.up()).await;
        (block == &Block::WATER || block == &Block::ICE)
            && (above_fluid != &Block::WATER && above_fluid != &Block::LAVA)
    }
}

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;

#[pumpkin_block("minecraft:cactus_flower")]
pub struct CactusFlowerBlock;

impl BlockBehaviour for CactusFlowerBlock {
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

impl PlantBlockBase for CactusFlowerBlock {
    async fn can_place_at(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let support_pos = pos.down();
        let (support_block, support_block_state) =
            block_accessor.get_block_and_state(&support_pos).await;
        if supports_cactus_flower(support_block, support_block_state) {
            return true;
        }
        false
    }
    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            return Block::AIR.default_state.id;
        }
        block_state
    }
}
fn supports_cactus_flower(block: &Block, block_state: &BlockState) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_SUPPORT_OVERRIDE_CACTUS_FLOWER)
        || block_state.is_center_solid(BlockDirection::Up)
}

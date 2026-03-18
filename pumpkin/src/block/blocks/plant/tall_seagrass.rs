use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::{PlantBlockBase, seagrass::supports_seagrass},
};
#[pumpkin_block("minecraft:tall_seagrass")]
pub struct TallSeaGrassBlock;
impl BlockBehaviour for TallSeaGrassBlock {
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

impl PlantBlockBase for TallSeaGrassBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let (support_block, support_block_state) = block_accessor.get_block_and_state(pos).await;
        let replacing_block = block_accessor.get_block(&pos.up()).await;
        if replacing_block != &Block::WATER && replacing_block != &Block::TALL_SEAGRASS {
            return false;
        }

        if replacing_block == &Block::TALL_SEAGRASS {
            //only for blockupdate
            let block_above = block_accessor.get_block(&pos.up_height(2)).await;
            let is_support_seagrass_block = support_block == &Block::TALL_SEAGRASS;
            let is_above_seagrass_block = block_above == &Block::TALL_SEAGRASS;
            match (is_support_seagrass_block, is_above_seagrass_block) {
                (true, true) | (false, false) => return false,
                _ => {}
            }
        }
        if support_block == &Block::TALL_SEAGRASS {
            return true;
        }
        if supports_seagrass(support_block, support_block_state) {
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
            return Block::WATER.default_state.id;
        }
        block_state
    }
}

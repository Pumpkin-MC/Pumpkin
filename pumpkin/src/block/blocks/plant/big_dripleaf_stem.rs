use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::big_dripleaf::can_plant_dripleaf_on_top;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
};
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BigDripleafLikeProperties, BlockProperties, LadderLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block("minecraft:big_dripleaf_stem")]
pub struct BigDripleafStemBlock;

impl BlockBehaviour for BigDripleafStemBlock {
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
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move { handle_big_dripleaf_breaking(args.world, args.position).await })
    }
}
impl PlantBlockBase for BigDripleafStemBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let support_block = block_accessor.get_block(pos).await;
        can_plant_dripleaf_on_top(support_block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            let block = block_accessor.get_block(block_pos).await;

            let dripleaf_stem_props = LadderLikeProperties::from_state_id(block_state, block);
            if dripleaf_stem_props.waterlogged {
                return Block::WATER.default_state.id;
            }
            return Block::AIR.default_state.id;
        }
        block_state
    }
}
pub async fn handle_big_dripleaf_breaking(world: &Arc<World>, position: &BlockPos) {
    let support_pos = position.down();
    let (support_block, support_state_id) = world.get_block_and_state_id(&support_pos).await;
    if support_block == &Block::BIG_DRIPLEAF_STEM {
        let dripleaf_stem_props =
            LadderLikeProperties::from_state_id(support_state_id, support_block);

        let mut dripleaf_props = BigDripleafLikeProperties::default(&Block::BIG_DRIPLEAF);
        dripleaf_props.facing = dripleaf_stem_props.facing;
        dripleaf_props.waterlogged = dripleaf_stem_props.waterlogged;
        world
            .set_block_state(
                &support_pos,
                dripleaf_props.to_state_id(&Block::BIG_DRIPLEAF),
                BlockFlags::empty(),
            )
            .await;
    }
}

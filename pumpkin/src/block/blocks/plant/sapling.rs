use pumpkin_data::block_properties::{BlockProperties, Integer0To1};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
};
use crate::world::World;

type SaplingProperties = pumpkin_data::block_properties::OakSaplingLikeProperties;

#[pumpkin_block_from_tag("minecraft:saplings")]
pub struct SaplingBlock;

impl SaplingBlock {
    async fn generate(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let mut props = SaplingProperties::from_state_id(state, block);
        if props.stage == Integer0To1::L0 {
            props.stage = Integer0To1::L1;
            world
                .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        } else {
            //TODO generate tree
        }
    }
}

impl BlockBehaviour for SaplingBlock {
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

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.generate(args.world, args.position).await;
        })
    }
}

impl PlantBlockBase for SaplingBlock {}

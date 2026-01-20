use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{
        BlockProperties as _, EnumVariants as _, Integer0To3, NetherWartLikeProperties,
    },
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng as _;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
        blocks::plant::{PlantBlockBase, crop::CropBlockBase},
    },
    world::World,
};

#[pumpkin_block("minecraft:nether_wart")]
pub struct NetherWartBlock;

impl BlockBehaviour for NetherWartBlock {
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
            <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
        })
    }
}

impl PlantBlockBase for NetherWartBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block == &Block::SOUL_SAND
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }
}

impl CropBlockBase for NetherWartBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, pos).await
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = NetherWartLikeProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = NetherWartLikeProperties::from_state_id(state, block);
        props.age = Integer0To3::from_index(age as u16);
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        if age < self.max_age() && rand::rng().random_range(0..=10) == 0 {
            world
                .set_block_state(
                    pos,
                    self.state_with_age(block, state, age + 1),
                    BlockFlags::NOTIFY_NEIGHBORS,
                )
                .await;
        }
    }
}

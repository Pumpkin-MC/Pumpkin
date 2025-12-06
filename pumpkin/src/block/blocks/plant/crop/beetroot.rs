use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, EnumVariants, Integer0To3, NetherWartLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use rand::Rng;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
};

type BeetrootProperties = NetherWartLikeProperties;

#[pumpkin_block("minecraft:beetroots")]
pub struct BeetrootBlock;

impl BlockBehaviour for BeetrootBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as CropBlockBase>::can_plant_on_top(
                self,
                args.block_accessor,
                &args.position.down(),
            )
            .await
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
            if rand::rng().random_range(0..2) != 0 {
                <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
            }
        })
    }
}

impl PlantBlockBase for BeetrootBlock {}

impl CropBlockBase for BeetrootBlock {
    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = BeetrootProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = BeetrootProperties::from_state_id(state, block);
        props.age = Integer0To3::from_index(age as u16);
        props.to_state_id(block)
    }
}

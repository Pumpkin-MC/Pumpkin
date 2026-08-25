use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, NetherWartLikeProperties};
use pumpkin_macros::pumpkin_block;
use rand::RngExt;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
};

type BeetrootProperties = NetherWartLikeProperties;

#[pumpkin_block("minecraft:beetroots")]
pub struct BeetrootBlock;

impl BlockBehaviour for BeetrootBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::is_valid_bonemeal_target(self, args.world, args.position)
    }

    fn perform_bonemeal<'a>(&'a self, args: crate::block::BonemealArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            <Self as CropBlockBase>::perform_bonemeal(self, args.world, args.position).await;
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_plant_on_top(self, args.block_accessor, &args.position.down())
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
            if rand::rng().random_range(0..3) == 0 {
                <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
            }
        })
    }
}

impl PlantBlockBase for BeetrootBlock {}

impl CropBlockBase for BeetrootBlock {
    fn bonemeal_age_increase(&self) -> i32 {
        rand::rng().random_range(2..=5) / 3
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: BlockStateId, block: &Block) -> i32 {
        let props = BeetrootProperties::from_state_id(state, block);
        i32::from(props.age)
    }

    fn state_with_age(&self, block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = BeetrootProperties::from_state_id(state, block);
        props.age = age as u8;
        props.to_state_id(block)
    }
}

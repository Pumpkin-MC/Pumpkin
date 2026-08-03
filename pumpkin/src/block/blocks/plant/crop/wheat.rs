use pumpkin_data::BlockStateId;
use pumpkin_macros::pumpkin_block;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    RandomTickArgs,
};

#[pumpkin_block("minecraft:wheat")]
pub struct WheatBlock;

impl BlockBehaviour for WheatBlock {
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
            <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
        })
    }

    fn is_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_bonemeal(self, args.state_id, args.block)
    }

    fn perform_bonemeal<'a>(&'a self, args: BonemealArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            <Self as CropBlockBase>::grow_from_bonemeal(self, args.world, args.position).await;
        })
    }
}

impl PlantBlockBase for WheatBlock {}

impl CropBlockBase for WheatBlock {}

use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::crop::CropBlockBase;
use crate::block::{BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs};

#[pumpkin_block("minecraft:wheat")]
pub struct WheatBlock;

#[async_trait]
impl BlockBehaviour for WheatBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as CropBlockBase>::can_plant_on_top(self, args.block_accessor, &args.position.down())
            .await
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

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
    }
}

impl PlantBlockBase for WheatBlock {}

impl CropBlockBase for WheatBlock {}

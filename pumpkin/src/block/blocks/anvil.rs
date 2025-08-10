use crate::block::blocks::falling::FallingBlock;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:anvil")]
pub struct AnvilBlock;

#[async_trait]
impl BlockBehaviour for AnvilBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let dir = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .rotate_clockwise();

        let mut props = WallTorchLikeProperties::default(args.block);

        props.facing = dir;
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        FallingBlock::placed(&FallingBlock, args).await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        FallingBlock::get_state_for_neighbor_update(&FallingBlock, args).await
    }
    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        FallingBlock::on_scheduled_tick(&FallingBlock, args).await;
    }
}

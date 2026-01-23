use crate::block::blocks::falling::FallingBlock;
use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
    PlacedArgs,
};
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:anvil")]
pub struct AnvilBlock;

impl BlockBehaviour for AnvilBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let dir = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .rotate_clockwise();

            let mut props = WallTorchLikeProperties::default(args.block);

            props.facing = dir;
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FallingBlock::placed(&FallingBlock, args).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(
            async move { FallingBlock::get_state_for_neighbor_update(&FallingBlock, args).await },
        )
    }
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FallingBlock::on_scheduled_tick(&FallingBlock, args).await;
        })
    }
}

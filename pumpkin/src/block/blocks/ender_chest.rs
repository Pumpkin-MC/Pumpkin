use crate::block::pumpkin_block::{OnPlaceArgs, PumpkinBlock};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, LadderLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:ender_chest")]
pub struct EnderChestBlock;

#[async_trait]
impl PumpkinBlock for EnderChestBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LadderLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }
}

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{CanPlaceAtArgs, OnEntityCollisionArgs, PumpkinBlock};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

#[async_trait]
impl PumpkinBlock for LilyPadBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        // Proberbly not the best solution, but works
        if args
            .entity
            .get_entity()
            .entity_type
            .resource_name
            .ends_with("_boat")
        {
            args.world
                .break_block(args.location, None, BlockFlags::empty())
                .await;
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below == &Block::WATER || block_below == &Block::ICE
    }
}

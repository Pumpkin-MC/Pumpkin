use async_trait::async_trait;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, CanPlaceAtArgs};

#[pumpkin_block("minecraft:bamboo")]
pub struct BambooBlock;

#[async_trait]
impl BlockBehaviour for BambooBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        block_below.has_tag(&tag::Block::MINECRAFT_BAMBOO_PLANTABLE_ON)
    }
}

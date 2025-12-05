use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs};

#[pumpkin_block("minecraft:bamboo")]
pub struct BambooBlock;

impl BlockBehaviour for BambooBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block_below = args.block_accessor.get_block(&args.position.down()).await;
            block_below.has_tag(&tag::Block::MINECRAFT_BAMBOO_PLANTABLE_ON)
        })
    }
}

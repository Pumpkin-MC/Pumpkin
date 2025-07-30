use async_trait::async_trait;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

use crate::block::{BlockBehaviour, BlockMetadata, CanPlaceAtArgs};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::WARPED_ROOTS.name, Block::CRIMSON_ROOTS.name]
    }
}

#[async_trait]
impl BlockBehaviour for RootsBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_NYLIUM)
            || block_below == &Block::SOUL_SOIL
            || block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_DIRT)
            || block_below == &Block::FARMLAND
    }
}

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::Tagable;

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

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
impl PumpkinBlock for RootsBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:nylium").unwrap()
            || block_below == &Block::SOUL_SOIL
            || block_below.is_tagged_with("minecraft:dirt").unwrap()
            || block_below == &Block::FARMLAND
    }
}

use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, tag};

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct ShortPlantBlock;

impl BlockMetadata for ShortPlantBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["short_grass", "fern"]
    }
}

#[async_trait]
impl PumpkinBlock for ShortPlantBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_DIRT)
            || block_below == &Block::FARMLAND
    }
}

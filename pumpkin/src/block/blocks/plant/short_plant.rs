use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::Tagable;

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
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == &Block::FARMLAND
    }
}

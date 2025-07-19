use async_trait::async_trait;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct DryVegetationBlock;

impl BlockMetadata for DryVegetationBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["dead_bush", "tall_dry_grass", "short_dry_grass"]
    }
}

#[async_trait]
impl PumpkinBlock for DryVegetationBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_DRY_VEGETATION_MAY_PLACE_ON)
    }
}

use async_trait::async_trait;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct MushroomPlantBlock;

impl BlockMetadata for MushroomPlantBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["brown_mushroom", "red_mushroom"]
    }
}

#[async_trait]
impl PumpkinBlock for MushroomPlantBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        if block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_MUSHROOM_GROW_BLOCK) {
            return true;
        }
        // TODO: Check light level and isOpaqueFullCube
        false
    }
}

use async_trait::async_trait;
use pumpkin_data::tag::Tagable;

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
        if block_below
            .is_tagged_with("minecraft:mushroom_grow_block")
            .unwrap()
        {
            return true;
        }
        // TODO: Check light level and isOpaqueFullCube
        false
    }
}

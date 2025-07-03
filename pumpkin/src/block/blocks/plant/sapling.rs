use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::{RegistryKey, Tagable, get_tag_values};

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct SaplingBlock;

impl BlockMetadata for SaplingBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:saplings").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SaplingBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == &Block::FARMLAND
    }
}

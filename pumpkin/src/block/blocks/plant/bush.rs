use async_trait::async_trait;
use pumpkin_data::Block;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct BushBlock;

impl BlockMetadata for BushBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::BUSH.name, Block::FIREFLY_BUSH.name]
    }
}

#[async_trait]
impl PumpkinBlock for BushBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
    }
}

impl PlantBlockBase for BushBlock {}

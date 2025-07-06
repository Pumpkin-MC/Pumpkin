use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::Tagable;

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock};

pub struct TallPlantBlock;

impl BlockMetadata for TallPlantBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            "tall_grass",
            "large_fern",
            "pitcher_plant",
            // TallFlowerBlocks
            "sunflower",
            "lilac",
            "peony",
            "rose_bush",
        ]
    }
}

#[async_trait]
impl PumpkinBlock for TallPlantBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let (block, state) = args
            .block_accessor
            .get_block_and_block_state(args.location)
            .await;
        if let Some(props) = block.properties(state.id).map(|s| s.to_props()) {
            if props
                .iter()
                .any(|(key, value)| key == "half" && value == "upper")
            {
                let (block, below_state) = args
                    .block_accessor
                    .get_block_and_block_state(&args.location.down())
                    .await;
                if let Some(props) = block.properties(below_state.id).map(|s| s.to_props()) {
                    let is_lower = props
                        .iter()
                        .any(|(key, value)| key == "half" && value == "lower");
                    return below_state.id == state.id && is_lower;
                }
            }
        }
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == &Block::FARMLAND
    }
}

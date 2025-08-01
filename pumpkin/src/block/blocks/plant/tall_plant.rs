use async_trait::async_trait;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

use crate::block::{BlockBehaviour, BlockMetadata, CanPlaceAtArgs};

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
impl BlockBehaviour for TallPlantBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let (block, state) = args.block_accessor.get_block_and_state(args.position).await;
        if let Some(props) = block.properties(state.id).map(|s| s.to_props()) {
            if props
                .iter()
                .any(|(key, value)| key == "half" && value == "upper")
            {
                let (block, below_state) = args
                    .block_accessor
                    .get_block_and_state(&args.position.down())
                    .await;
                if let Some(props) = block.properties(below_state.id).map(|s| s.to_props()) {
                    let is_lower = props
                        .iter()
                        .any(|(key, value)| key == "half" && value == "lower");
                    return below_state.id == state.id && is_lower;
                }
            }
        }
        let block_below = args.block_accessor.get_block(&args.position.down()).await;
        block_below.is_tagged_with_by_tag(&tag::Block::MINECRAFT_DIRT)
            || block_below == &Block::FARMLAND
    }
}

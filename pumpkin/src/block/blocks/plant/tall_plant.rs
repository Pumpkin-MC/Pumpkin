use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

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
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let (block, state) = block_accessor.get_block_and_block_state(block_pos).await;
        if let Some(props) = block.properties(state.id).map(|s| s.to_props()) {
            if props
                .iter()
                .any(|(key, value)| key == "half" && value == "upper")
            {
                let (block, below_state) = block_accessor
                    .get_block_and_block_state(&block_pos.down())
                    .await;
                if let Some(props) = block.properties(below_state.id).map(|s| s.to_props()) {
                    let is_lower = props
                        .iter()
                        .any(|(key, value)| key == "half" && value == "lower");
                    return below_state.id == state.id && is_lower;
                }
            }
        }
        let block_below = block_accessor.get_block(&block_pos.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == Block::FARMLAND
    }
}

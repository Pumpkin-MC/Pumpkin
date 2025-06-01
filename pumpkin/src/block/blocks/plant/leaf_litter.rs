use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

pub struct LeafLitterBlock;

impl BlockMetadata for LeafLitterBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &["leaf_litter"]
    }
}

#[async_trait]
impl PumpkinBlock for LeafLitterBlock {
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
        let block_below = block_accessor.get_block_state(&block_pos.down()).await;
        block_below.is_side_solid(BlockDirection::Up)
    }
}

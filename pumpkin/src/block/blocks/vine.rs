use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

#[pumpkin_block("minecraft:vine")]
pub struct VineBlock;

#[async_trait]
impl PumpkinBlock for VineBlock {
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
        // TODO: This is bad and not vanilla, just a "hotfix"
        for dir in BlockDirection::all() {
            if !block_accessor
                .get_block_state(&block_pos.offset(dir.to_offset()))
                .await
                .is_air()
            {
                return true;
            }
        }
        false
    }
}

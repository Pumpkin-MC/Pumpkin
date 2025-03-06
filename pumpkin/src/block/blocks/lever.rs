use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::{block::CardinalDirection, item::Item};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_data::block::Block;

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    server::Server,
    world::World,
};

#[pumpkin_block("minecraft:lever")]
pub struct LeverBlock;

#[async_trait]
impl PumpkinBlock for LeverBlock {
    async fn on_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &CardinalDirection,
        other: bool,
    ) -> u16 {
        let face = match face {
            BlockDirection::Bottom | BlockDirection::Top => *face,
            _ => face.opposite(),
        };

        block.default_state_id
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        _location: BlockPos,
        _item: &Item,
        _server: &Server,
    ) -> BlockActionResult {
        BlockActionResult::Consume
    }
}

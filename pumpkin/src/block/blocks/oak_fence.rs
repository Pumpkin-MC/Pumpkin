use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::OakFenceProps;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_data::{block::CardinalDirection, item::Item};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    server::Server,
    world::World,
};

#[pumpkin_block("minecraft:oak_fence")]
pub struct OakFenceBlock;

#[async_trait]
impl PumpkinBlock for OakFenceBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,

        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &CardinalDirection,
        other: bool,
    ) -> u16 {
        let mut block_properties = OakFenceProps::from_state_id(block.default_state_id).unwrap();

        block_properties.west = Boolean::True;
        block_properties.east = Boolean::True;
        block_properties.waterlogged = Boolean::True;

        block_properties.to_state_id()
    }
}

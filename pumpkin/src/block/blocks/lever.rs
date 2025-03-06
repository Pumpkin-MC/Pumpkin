use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::{
    block::{AttachmentFace, BlockProperties, CardinalDirection, LeverProps},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

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
        _server: &Server,
        _world: &World,
        block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &CardinalDirection,
        _other: bool,
    ) -> u16 {
        let mut lever_props = LeverProps::from_state_id(block.default_state_id).unwrap();

        match face {
            BlockDirection::Up => lever_props.face = AttachmentFace::Ceiling,
            BlockDirection::Down => lever_props.face = AttachmentFace::Floor,
            _ => lever_props.face = AttachmentFace::Wall,
        }

        if face == &BlockDirection::Up || face == &BlockDirection::Down {
            lever_props.facing = *player_direction;
        } else {
            lever_props.facing = player_direction.opposite();
        };

        lever_props.to_state_id()
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

use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{
    BlockDirection,
    registry::{Block, State},
};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("type")]
pub enum ChestType {
    Single,
    Left,
    Right,
}

#[async_trait]
impl BlockProperty for ChestType {
    async fn on_place(
        &self,
        _world: &World,
        _block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        // TODO: determine if should combine
        Self::Single.value()
    }

    async fn can_update(
        &self,
        value: String,
        _block: &Block,
        _block_state: &State,
        _face: &BlockDirection,
        _use_item_on: &SUseItemOn,
        _other: bool,
    ) -> bool {
        value == Self::Single.value()
    }
}

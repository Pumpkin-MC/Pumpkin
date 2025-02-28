use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("axis")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[async_trait]
impl BlockProperty for Axis {
    async fn on_place(
        &self,
        _world: &World,
        _block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        match face {
            BlockDirection::North | BlockDirection::South => Self::Z.value(),
            BlockDirection::East | BlockDirection::West => Self::X.value(),
            BlockDirection::Top | BlockDirection::Bottom => Self::Y.value(),
        }
    }
}

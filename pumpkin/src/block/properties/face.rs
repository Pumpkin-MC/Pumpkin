use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("face")]
pub enum Face {
    Ceiling,
    Floor,
    Wall,
}

#[async_trait]
impl BlockProperty for Face {
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
        let face = match face {
            BlockDirection::Top => Self::Ceiling,
            BlockDirection::Bottom => Self::Floor,
            _ => Self::Wall,
        };
        face.value()
    }
}

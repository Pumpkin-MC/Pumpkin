use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{registry::Block, BlockDirection};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("waterlogged")]
pub struct Waterlogged(bool);

#[async_trait]
impl BlockProperty for Waterlogged {
    async fn on_place(
        &self,
        _world: &World,
        block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        if block.name == "water" {
            return Self::True().value();
        }
        Self::False().value()
    }
}

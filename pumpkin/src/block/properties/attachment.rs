use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("attachment")]
pub enum Attachment {
    Floor,
    Ceiling,
    SingleWall,
    DoubleWall,
}

#[async_trait]
impl BlockProperty for Attachment {
    async fn on_place(
        &self,
        world: &World,
        _block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        match face {
            BlockDirection::Top => Self::Ceiling.value(),
            BlockDirection::Bottom => Self::Floor.value(),
            _ => {
                let other_side_block = BlockPos(block_pos.0.sub(&face.to_offset()));
                let block = world.get_block(&other_side_block).await.unwrap();
                if block.id != 0 {
                    return Self::DoubleWall.value();
                }
                Self::SingleWall.value()
            }
        }
    }
}

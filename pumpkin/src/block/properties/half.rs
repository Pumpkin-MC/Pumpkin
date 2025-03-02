use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("half")]
pub enum Half {
    Top,
    Bottom,
}

#[async_trait]
impl BlockProperty for Half {
    async fn on_place(
        &self,
        _world: &World,
        _block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        evaluate_half(*face, use_item_on)
    }
}

pub fn evaluate_half(face: BlockDirection, use_item_on: &SUseItemOn) -> String {
    match face {
        BlockDirection::Bottom => Half::Bottom.value(),
        BlockDirection::Top => Half::Top.value(),
        _ => {
            if use_item_on.cursor_pos.y > 0.5 {
                Half::Top.value()
            } else {
                Half::Bottom.value()
            }
        }
    }
}

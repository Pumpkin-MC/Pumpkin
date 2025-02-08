use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{
    registry::{Block, State},
    BlockDirection,
};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("type")]
pub enum SlabType {
    Top,
    Bottom,
    Double,
}

#[async_trait]
impl BlockProperty for SlabType {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        other: bool,
    ) -> String {
        let clicked_block = world.get_block(block_pos).await.unwrap();

        if block.id == clicked_block.id && !other {
            return SlabType::Double.value();
        }

        let y_pos = use_item_on.cursor_pos.y;
        if (y_pos > 0.5 && face != &BlockDirection::Bottom) || face == &BlockDirection::Top {
            return SlabType::Top.value();
        }

        SlabType::Bottom.value()
    }

    async fn can_update(
        &self,
        value: String,
        _block: &Block,
        _block_state: &State,
        face: &BlockDirection,
        use_item_on: &SUseItemOn,
        other: bool,
    ) -> bool {
        if other {
            let y = use_item_on.cursor_pos.y;
            match face {
                BlockDirection::Top => return value == SlabType::Bottom.value(),
                BlockDirection::Bottom => return value == SlabType::Top.value(),
                _ => {
                    if y < 0.5 {
                        return value == SlabType::Top.value();
                    } else {
                        return value == SlabType::Bottom.value();
                    }
                }
            }
        } else {
            if value == SlabType::Double.value() {
                return false;
            }
            match face {
                BlockDirection::Top => value == SlabType::Bottom.value(),
                BlockDirection::Bottom => value == SlabType::Top.value(),
                _ => false,
            }
        }
    }
}

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

// Those which requires custom names to values can be defined like this
#[block_property("layers", [1, 2, 3, 4, 5, 6, 7, 8])]
pub enum Layers {
    Lay1,
    Lay2,
    Lay3,
    Lay4,
    Lay5,
    Lay6,
    Lay7,
    Lay8,
}

#[async_trait]
impl BlockProperty for Layers {
    async fn on_place(
        &self,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        properties: &BlockProperties,
        _other: bool,
    ) -> String {
        let clicked_block = world.get_block(block_pos).await.unwrap();
        let clicked_block_state = world.get_block_state(block_pos).await.unwrap();
        let state =
            &properties.property_mappings[&(clicked_block_state.id - clicked_block.states[0].id)];
        for property in state {
            if block.id == clicked_block.id {
                // bro its is so hacky :crying:
                let mut layer: u8 = property.parse().unwrap();
                // lets add a new layer
                layer += 1;
                return Self::from_value(layer.to_string()).value();
            }
        }

        Self::Lay1.value()
    }

    async fn can_update(
        &self,
        value: String,
        block: &Block,
        _block_state: &State,
        face: &BlockDirection,
        _use_item_on: &SUseItemOn,
        other: bool,
    ) -> bool {
        if value == Self::Lay8.value() {
            return false;
        }
        if value == Self::Lay1.value() {
            return true;
        }
        if !other {
            match face {
                BlockDirection::Top => return block.name == "snow",
                _ => return false,
            }
        }
        true
    }
}

use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{registry::Block, BlockDirection};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("facing")]
pub enum Facing {
    North,
    South,
    East,
    West,
}

#[async_trait]
impl BlockProperty for Facing {
    async fn on_place(
        &self,
        _world: &World,
        _block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        // Some blocks have also facing with top and bottom
        let facing = match face {
            BlockDirection::North => Facing::North,
            BlockDirection::South => Facing::South,
            BlockDirection::East => Facing::East,
            BlockDirection::West => Facing::West,
            BlockDirection::Top | BlockDirection::Bottom => match player_direction {
                Direction::North => Facing::North,
                Direction::South => Facing::South,
                Direction::East => Facing::East,
                Direction::West => Facing::West,
            },
        };
        facing.value()
    }
}

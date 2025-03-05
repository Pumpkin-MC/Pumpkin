use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

// #[block_property("rotation")]
#[block_property("rotation", [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])]
pub enum Rotation {
    Rotation0,
    Rotation1,
    Rotation2,
    Rotation3,
    Rotation4,
    Rotation5,
    Rotation6,
    Rotation7,
    Rotation8,
    Rotation9,
    Rotation10,
    Rotation11,
    Rotation12,
    Rotation13,
    Rotation14,
    Rotation15,
}

#[async_trait]
impl BlockProperty for Rotation {
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
        // TODO: Player direction should be extended for additional angles
        let rotation = match face {
            BlockDirection::Bottom => match player_direction {
                Direction::North => Self::Rotation8,
                Direction::East => Self::Rotation12,
                Direction::South => Self::Rotation0,
                Direction::West => Self::Rotation4,
            },
            _ => Self::Rotation0,
        };
        rotation.value()
    }
}

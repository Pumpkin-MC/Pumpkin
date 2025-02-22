use crate::world::World;
use async_trait::async_trait;
use pumpkin_macros::block_property;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::block::{BlockDirection, registry::Block};

use super::{BlockProperties, BlockProperty, BlockPropertyMetadata, Direction};

#[block_property("signal_fire")]
pub struct SignalFire(bool);

#[async_trait]
impl BlockProperty for SignalFire {
    async fn on_place(
        &self,
        world: &World,
        _block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        let other_side_block = BlockPos(block_pos.0.sub(&Vector3::new(0, 1, 0)));
        let block = world.get_block(&other_side_block).await.unwrap();
        if block.name == "hay_block" {
            return Self::True().value();
        }
        Self::False().value()
    }
}

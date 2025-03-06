use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::OakFenceProps;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::server::Server;
use crate::world::World;

pub async fn fence_state(world: &World, block: &Block, block_pos: &BlockPos) -> u16 {
    let mut block_properties = OakFenceProps::from_state_id(block.default_state_id).unwrap();

    for direction in BlockDirection::horizontal() {
        let offset = block_pos.offset(direction.to_offset());
        let other_block = world.get_block(&offset).await.unwrap_or(Block::AIR);

        if block.id == other_block.id {
            match direction {
                BlockDirection::North => block_properties.north = Boolean::True,
                BlockDirection::South => block_properties.south = Boolean::True,
                BlockDirection::West => block_properties.west = Boolean::True,
                BlockDirection::East => block_properties.east = Boolean::True,
                _ => {}
            }
        }
    }

    block_properties.to_index()
}

#[pumpkin_block("minecraft:oak_fence")]
pub struct OakFenceBlock;

#[async_trait]
impl PumpkinBlock for OakFenceBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &CardinalDirection,
        _other: bool,
    ) -> u16 {
        OakFenceProps::from_index(fence_state(world, block, block_pos).await).to_state_id()
    }
}

use crate::block::BlockIsReplacing;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;
type FenceLikeProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::server::Server;
use crate::world::World;

fn connects_to(from: &Block, to: &Block, to_state: &BlockState, direction: BlockDirection) -> bool {
    if from.id == to.id {
        return true;
    }

    if to.is_tagged_with("c:fence_gates").unwrap() {
        let fence_gate_props = FenceGateProperties::from_state_id(to_state.id, to);
        if BlockDirection::from_cardinal_direction(fence_gate_props.facing).to_axis()
            == direction.rotate_clockwise().to_axis()
        {
            return true;
        }
    }

    // If the block is not a wooden fence, it cannot connect to a wooden fence
    if !from.is_tagged_with("c:fences/wooden").unwrap() {
        return false;
    }

    to.is_tagged_with("c:fences/wooden").unwrap()
        || (to_state.is_solid() && to_state.is_full_cube())
}

/// This returns an index and not a state id making it so all fences can use the same state calculation function
pub async fn fence_state(world: &World, block: &Block, block_pos: &BlockPos) -> u16 {
    let mut block_properties = FenceLikeProperties::default(block);

    for direction in BlockDirection::horizontal() {
        let offset = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) =
            world.get_block_and_block_state(&offset).await.unwrap();

        if connects_to(block, &other_block, &other_block_state, direction) {
            match direction {
                BlockDirection::North => block_properties.north = true,
                BlockDirection::South => block_properties.south = true,
                BlockDirection::West => block_properties.west = true,
                BlockDirection::East => block_properties.east = true,
                _ => {}
            }
        }
    }

    block_properties.to_state_id(block)
}

pub struct FenceBlock;
impl BlockMetadata for FenceBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:fences").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for FenceBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player: &Player,
        _replacing: BlockIsReplacing,
    ) -> u16 {
        fence_state(world, block, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        _state: BlockStateId,
        block_pos: &BlockPos,
        _direction: &BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        fence_state(world, block, block_pos).await
    }
}

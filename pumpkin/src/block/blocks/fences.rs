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
type FenceProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::server::Server;
use crate::world::World;

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
        _face: BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player: &Player,
        replacing: BlockIsReplacing,
    ) -> u16 {
        let mut fence_props = FenceProperties::default(block);
        fence_props.waterlogged = replacing.water_source();

        compute_fence_state(fence_props, world, block, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let fence_props = FenceProperties::from_state_id(state_id, block);
        compute_fence_state(fence_props, world, block, block_pos).await
    }
}

pub async fn compute_fence_state(
    mut fence_props: FenceProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let Ok((other_block, other_block_state)) =
            world.get_block_and_block_state(&other_block_pos).await
        else {
            continue;
        };

        let connected = connects_to(block, &other_block, &other_block_state, direction);
        match direction {
            BlockDirection::North => fence_props.north = connected,
            BlockDirection::South => fence_props.south = connected,
            BlockDirection::West => fence_props.west = connected,
            BlockDirection::East => fence_props.east = connected,
            _ => {}
        }
    }

    fence_props.to_state_id(block)
}

fn connects_to(from: &Block, to: &Block, to_state: &BlockState, direction: BlockDirection) -> bool {
    if from == to {
        return true;
    }

    if to_state.is_solid() && to_state.is_full_cube() {
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

    *from != Block::NETHER_BRICK_FENCE && to.is_tagged_with("c:fences/wooden").unwrap()
}

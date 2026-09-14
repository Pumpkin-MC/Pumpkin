use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
};
use crate::world::World;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, HorizontalFacingExt, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;

type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;
type FenceProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

#[pumpkin_block_from_tag("minecraft:fences")]
pub struct FenceBlock;

impl BlockBehaviour for FenceBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut fence_props = FenceProperties::default(args.block);
        fence_props.waterlogged = args.replacing.water_source();

        compute_fence_state(fence_props, args.world, args.block, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let fence_props = FenceProperties::from_state_id(args.state_id);
        compute_fence_state(fence_props, args.world, args.block, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

pub fn compute_fence_state(
    mut fence_props: FenceProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> BlockStateId {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) = world.get_block_and_state(&other_block_pos);

        let connected = connects_to(
            block,
            other_block,
            other_block_state,
            direction.to_block_direction(),
        );
        match direction {
            HorizontalFacing::North => fence_props.north = connected,
            HorizontalFacing::South => fence_props.south = connected,
            HorizontalFacing::West => fence_props.west = connected,
            HorizontalFacing::East => fence_props.east = connected,
        }
    }

    fence_props.to_state_id(block)
}

fn connects_to(from: &Block, to: &Block, to_state: &BlockState, direction: BlockDirection) -> bool {
    if from == to {
        return true;
    }

    if to_state.is_side_solid(direction.opposite()) {
        return true;
    }

    if to.has_tag(&tag::Block::C_FENCE_GATES) {
        let fence_gate_props = FenceGateProperties::from_state_id(to_state.id);
        if BlockDirection::from_cardinal_direction(fence_gate_props.facing).to_axis()
            == direction.rotate_clockwise().to_axis()
        {
            return true;
        }
    }

    *from != Block::NETHER_BRICK_FENCE && to.has_tag(&tag::Block::C_FENCES_WOODEN)
}

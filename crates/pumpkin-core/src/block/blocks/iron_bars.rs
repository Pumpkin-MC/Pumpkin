use crate::block::blocks::weathering_copper::change_over_time;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
    RandomTickArgs, is_exception_for_connection,
};
use crate::world::World;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId, HorizontalFacingExt, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;

type IronBarsProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

#[pumpkin_block_from_tag("minecraft:bars")]
pub struct IronBarsBlock;

impl BlockBehaviour for IronBarsBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bars_props = IronBarsProperties::default(args.block);
        bars_props.waterlogged = args.replacing.water_source();

        compute_bars_state(bars_props, args.world, args.block, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let bars_props = IronBarsProperties::from_state_id(args.state_id);
        if bars_props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        compute_bars_state(bars_props, args.world, args.block, args.position)
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

pub fn compute_bars_state(
    mut bars_props: IronBarsProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> BlockStateId {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let (other_block, other_block_state) = world.get_block_and_state(&other_block_pos);

        let connected = other_block == block
            || other_block.has_tag(&tag::Block::MINECRAFT_BARS)
            || (!is_exception_for_connection(other_block)
                && other_block_state.is_side_solid(direction.opposite().to_block_direction()))
            || other_block.has_tag(&tag::Block::C_GLASS_PANES)
            || other_block.has_tag(&tag::Block::MINECRAFT_WALLS);

        match direction {
            HorizontalFacing::North => bars_props.north = connected,
            HorizontalFacing::South => bars_props.south = connected,
            HorizontalFacing::West => bars_props.west = connected,
            HorizontalFacing::East => bars_props.east = connected,
        }
    }

    bars_props.to_state_id(block)
}

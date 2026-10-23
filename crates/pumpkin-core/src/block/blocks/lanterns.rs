use crate::block::blocks::weathering_copper::change_over_time;
use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PathComputationType, RandomTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::LanternLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockDirection, BlockState, BlockStateId, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:lanterns")]
pub struct LanternBlock;

impl BlockBehaviour for LanternBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LanternLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();

        let block_up_state = args.world.get_block_state(&args.position.up());
        if block_up_state.is_center_solid(BlockDirection::Down) {
            props.r#hanging = true;
        }

        props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| can_place_at(world, args.position))
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = LanternLikeProperties::from_state_id(args.state_id);
        if props.r#waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        if !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn can_place_at(world: &World, position: &BlockPos) -> bool {
    if world
        .get_block(&position.down())
        .has_tag(&tag::Block::C_FENCE_GATES)
    {
        let fence_gate_props =
            pumpkin_data::block_properties::OakFenceGateLikeProperties::from_state_id(
                world.get_block_state_id(&position.down()),
            );

        if fence_gate_props.open {
            return false;
        }
    }
    let (block_down, block_down_state) = world.get_block_and_state(&position.down());
    let block_up_state = world.get_block_state(&position.up());
    block_down_state.is_center_solid(BlockDirection::Up)
        || block_up_state.is_center_solid(BlockDirection::Down)
        || block_down.has_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER)
}

use pumpkin_data::block_properties::MangroveRootsLikeProperties as HangingRootsLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;

use crate::block::{BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs};

#[pumpkin_block("minecraft:hanging_roots")]
pub struct HangingRootsBlock;

impl HangingRootsBlock {
    #[must_use]
    pub fn can_survive(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let above_pos = pos.up();
        let (above_block, above_state) = world.get_block_and_state(&above_pos);
        above_state.is_side_solid(BlockDirection::Down) && above_block.is_solid()
    }
}

impl BlockBehaviour for HangingRootsBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = HangingRootsLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::can_survive(args.world, args.position) {
            return Block::AIR.default_state.id;
        }

        let props = HangingRootsLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }

        args.state_id
    }
}

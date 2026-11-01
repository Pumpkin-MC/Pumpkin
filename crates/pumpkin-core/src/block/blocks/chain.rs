use crate::block::{
    BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
};
use pumpkin_data::block_properties::{Axis, IronChainLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{BlockDirection, BlockId, BlockState, BlockStateId};
use pumpkin_world::tick::TickPriority;

pub struct ChainBlock;

impl BlockMetadata for ChainBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::IRON_CHAIN,
            BlockId::WAXED_COPPER_CHAIN,
            BlockId::WAXED_EXPOSED_COPPER_CHAIN,
            BlockId::WAXED_WEATHERED_COPPER_CHAIN,
            BlockId::WAXED_OXIDIZED_COPPER_CHAIN,
        ]
        .into()
    }
}

impl BlockBehaviour for ChainBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = IronChainLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();
        props.r#axis = match args.direction {
            BlockDirection::East | BlockDirection::West => Axis::X,
            BlockDirection::Up | BlockDirection::Down => Axis::Y,
            BlockDirection::North | BlockDirection::South => Axis::Z,
        };

        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = IronChainLikeProperties::from_state_id(args.state_id);
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

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

use crate::block::{
    BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType,
};
use pumpkin_data::block_properties::MangroveRootsLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId};
use pumpkin_world::tick::TickPriority;

pub struct MangroveRootsBlock;

impl BlockMetadata for MangroveRootsBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::MANGROVE_ROOTS, BlockId::HEAVY_CORE].into()
    }
}

impl BlockBehaviour for MangroveRootsBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = MangroveRootsLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = MangroveRootsLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        props.to_state_id(args.block)
    }

    fn is_pathfindable(&self, state: &BlockState, _computation_type: PathComputationType) -> bool {
        Block::from_state_id(state.id) != &Block::HEAVY_CORE
    }
}

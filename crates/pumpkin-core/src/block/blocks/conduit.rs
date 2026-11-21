use crate::block::entities::conduit::ConduitBlockEntity;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, PathComputationType, PlacedArgs,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use std::sync::Arc;

#[pumpkin_block("minecraft:conduit")]
pub struct ConduitBlock;

impl BlockBehaviour for ConduitBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            pumpkin_data::block_properties::MangroveRootsLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();

        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = ConduitBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.state_id.is_waterlogged() {
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

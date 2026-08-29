use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    BlockProperties, MangroveRootsLikeProperties as BarrierLikeProperties,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;

#[pumpkin_block("minecraft:barrier")]
pub struct BarrierBlock;

impl BlockBehaviour for BarrierBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BarrierLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = BarrierLikeProperties::from_state_id(args.state_id, args.block);
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
}

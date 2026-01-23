use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::block_properties::{
    BlockProperties, MangroveRootsLikeProperties as BarrierLikeProperties,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;

#[pumpkin_block("minecraft:barrier")]
pub struct BarrierBlock;

impl BlockBehaviour for BarrierBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = BarrierLikeProperties::default(args.block);
            props.waterlogged = args.replacing.water_source();
            props.to_state_id(args.block)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = BarrierLikeProperties::from_state_id(args.state_id, args.block);
            if props.waterlogged {
                args.world
                    .schedule_fluid_tick(
                        &Fluid::WATER,
                        *args.position,
                        Fluid::WATER.flow_speed as u8,
                        TickPriority::Normal,
                    )
                    .await;
            }
            props.to_state_id(args.block)
        })
    }
}


use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs};
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, MangroveRootsLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:mangrove_roots")]
pub struct MangroveRootsBlock;

#[async_trait]
impl BlockBehaviour for MangroveRootsBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = MangroveRootsLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = MangroveRootsLikeProperties::from_state_id(args.state_id, args.block);
        if props.waterlogged {
            args.world
                .schedule_fluid_tick(
                    &Fluid::WATER,
                    *args.position,
                    Fluid::WATER.flow_speed as u8,
                    TickPriority::Normal
                )
                .await;
        }
        props.to_state_id(args.block)
    }
}

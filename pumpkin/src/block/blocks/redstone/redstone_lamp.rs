use crate::block::pumpkin_block::{OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs};
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::block::pumpkin_block::PumpkinBlock;

use super::block_receives_redstone_power;

type RedstoneLampProperties = pumpkin_data::block_properties::RedstoneOreLikeProperties;

#[pumpkin_block("minecraft:redstone_lamp")]
pub struct RedstoneLamp;

#[async_trait]
impl PumpkinBlock for RedstoneLamp {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = RedstoneLampProperties::default(args.block);
        props.lit = block_receives_redstone_power(args.world, args.location).await;
        props.to_state_id(args.block)
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let mut props = RedstoneLampProperties::from_state_id(state.id, args.block);
        let is_lit = props.lit;
        let is_receiving_power = block_receives_redstone_power(args.world, args.location).await;

        if is_lit != is_receiving_power {
            if is_lit {
                args.world
                    .schedule_block_tick(args.block, *args.location, 4, TickPriority::Normal)
                    .await;
            } else {
                props.lit = !props.lit;
                args.world
                    .set_block_state(
                        args.location,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        }
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let mut props = RedstoneLampProperties::from_state_id(state.id, args.block);
        let is_lit = props.lit;
        let is_receiving_power = block_receives_redstone_power(args.world, args.location).await;

        if is_lit && !is_receiving_power {
            props.lit = !props.lit;
            args.world
                .set_block_state(
                    args.location,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }
}

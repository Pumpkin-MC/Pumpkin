use crate::block::{BlockFuture, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs};
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};

use crate::block::BlockBehaviour;

use super::block_receives_redstone_power;

type RedstoneLampProperties = pumpkin_data::block_properties::RedstoneOreLikeProperties;

#[pumpkin_block("minecraft:redstone_lamp")]
pub struct RedstoneLamp;

impl BlockBehaviour for RedstoneLamp {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = RedstoneLampProperties::default(args.block);
            props.lit = block_receives_redstone_power(args.world, args.position).await;
            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut props = RedstoneLampProperties::from_state_id(state.id, args.block);
            let is_lit = props.lit;
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;

            if is_lit != is_receiving_power {
                if is_lit {
                    args.world
                        .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal)
                        .await;
                } else {
                    props.lit = !props.lit;
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut props = RedstoneLampProperties::from_state_id(state.id, args.block);
            let is_lit = props.lit;
            let is_receiving_power = block_receives_redstone_power(args.world, args.position).await;

            if is_lit && !is_receiving_power {
                props.lit = !props.lit;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }
}

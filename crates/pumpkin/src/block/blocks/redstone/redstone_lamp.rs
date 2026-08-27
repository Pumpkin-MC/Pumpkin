use crate::block::{OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::block::BlockBehaviour;

use super::block_receives_redstone_power;

type RedstoneLampProperties = pumpkin_data::block_properties::RedstoneOreLikeProperties;

#[pumpkin_block("minecraft:redstone_lamp")]
pub struct RedstoneLamp;

impl BlockBehaviour for RedstoneLamp {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = RedstoneLampProperties::default(args.block);
        props.lit = block_receives_redstone_power(args.world, args.position);
        props.to_state_id(args.block)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state = args.world.get_block_state(args.position);
            let mut props = RedstoneLampProperties::from_state_id(state.id, args.block);
            let is_lit = props.lit;
            let is_receiving_power = block_receives_redstone_power(args.world, args.position);

            if is_lit != is_receiving_power {
                if is_lit {
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        4,
                        TickPriority::Normal,
                    );
                } else {
                    props.lit = !props.lit;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    );
                }
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let props = RedstoneLampProperties::from_state_id(state.id, args.block);
        let is_lit = props.lit;
        let is_receiving_power = block_receives_redstone_power(args.world, args.position);

        if is_lit && !is_receiving_power {
            let block = args.world.get_block(args.position);
            let mut props = RedstoneLampProperties::from_state_id(state.id, block);
            props.lit = !props.lit;
            args.world.set_block_state(
                args.position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }
}

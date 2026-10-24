//! Shared defaults for vanilla `SimpleWaterloggedBlock` and waterlogged neighbor updates.

use pumpkin_data::{fluid::Fluid, item::Item};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use super::{CanPlaceLiquidArgs, GetStateForNeighborUpdateArgs, PickupFluidArgs, PlaceLiquidArgs};

pub(super) fn can_place_liquid(args: CanPlaceLiquidArgs<'_>) -> bool {
    args.fluid == &Fluid::WATER
}

// Shared default for SimpleWaterloggedBlock.placeLiquid.
pub(super) fn place_liquid(args: PlaceLiquidArgs<'_>) -> bool {
    if args.fluid != &Fluid::WATER {
        return false;
    }
    let Some(state_id) = args.block.set_waterlogged(args.state_id, true) else {
        return false;
    };
    args.world
        .set_block_state(args.position, state_id, BlockFlags::NOTIFY_ALL);
    args.world.schedule_fluid_tick(
        args.fluid,
        *args.position,
        args.fluid.flow_speed as u8,
        TickPriority::Normal,
    );
    true
}

/// Removes contained water, breaks unsupported blocks, and returns a water bucket.
pub(super) fn pickup_fluid(args: PickupFluidArgs<'_>) -> Option<&'static Item> {
    let state_id = args.block.set_waterlogged(args.state_id, false)?;
    args.world
        .set_block_state_if(args.position, state_id, BlockFlags::NOTIFY_ALL, |current| {
            current == args.state_id
        })?;
    // SimpleWaterloggedBlock.pickupBlock checks the original state after draining.
    if !args.world.block_registry.can_place_at(
        None,
        Some(args.world),
        args.world.as_ref(),
        None,
        args.block,
        args.state_id.to_state(),
        args.position,
        None,
        None,
    ) {
        args.world
            .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
    }
    Some(&Item::WATER_BUCKET)
}

/// Schedules a water tick if the block is waterlogged.
pub(super) fn schedule_water_tick(args: &GetStateForNeighborUpdateArgs<'_>) {
    if args.state_id.is_waterlogged() {
        args.world.schedule_fluid_tick(
            &Fluid::WATER,
            *args.position,
            Fluid::WATER.flow_speed as u8,
            TickPriority::Normal,
        );
    }
}

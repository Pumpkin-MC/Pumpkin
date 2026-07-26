use std::sync::Arc;

/**
 * This implementation is heavily based on <https://github.com/MCHPR/MCHPRS>
 * Updated to fit pumpkin by 4lve
 */
use pumpkin_data::{Block, BlockDirection, BlockState, HorizontalFacingExt};
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

pub mod bell;
pub mod buttons;
pub mod comparator;
pub mod copper_bulb;
pub mod crafter;
pub mod daylight_detector;
pub mod dropper;
pub mod lever;
pub mod lightning_rod;
pub mod observer;
pub mod pressure_plate;
pub mod rails;
pub mod redstone_block;
pub mod redstone_lamp;
pub mod redstone_ore;
pub mod redstone_torch;
pub mod redstone_wire;
pub mod repeater;
pub mod sculk_sensor;
pub mod target_block;
pub mod tripwire;
pub mod tripwire_hook;
// Vanilla 26.2 neighbor pipeline
pub mod neighbor_updater;
pub mod orientation;

// abstract
pub mod abstract_redstone_gate;
pub mod dispenser;

pub async fn update_wire_neighbors(world: &Arc<World>, pos: &BlockPos) {
    // `RedStoneWireBlock.onPlace` only sends vertical notifications here; the
    // regular setBlock path already notified direct horizontal neighbors.
    for direction in BlockDirection::vertical() {
        notify_wire_neighbors_at(world, pos.offset(direction.to_offset())).await;
    }

    update_neighbors_of_neighboring_wires(world, pos).await;
}

/// `RedStoneWireBlock.affectNeighborsAfterRemoval` first notifies the six
/// adjacent positions, then its evaluator and corner-shape notifications run.
pub(crate) async fn notify_removed_wire_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for direction in BlockDirection::all() {
        notify_wire_neighbors_at(world, pos.offset(direction.to_offset())).await;
    }
}

async fn notify_wire_neighbors_at(world: &Arc<World>, pos: BlockPos) {
    world
        .neighbor_updater
        .update_neighbors_at_except(world, pos, &Block::REDSTONE_WIRE, None, None)
        .await;
}

async fn check_corner_change_at(world: &Arc<World>, pos: BlockPos) {
    if world.get_block(&pos) != &Block::REDSTONE_WIRE {
        return;
    }

    notify_wire_neighbors_at(world, pos).await;
    for direction in BlockDirection::all() {
        notify_wire_neighbors_at(world, pos.offset(direction.to_offset())).await;
    }
}

pub(crate) async fn update_neighbors_of_neighboring_wires(
    world: &Arc<World>,
    pos: &BlockPos,
) {
    for direction in BlockDirection::horizontal() {
        let direction = direction.to_block_direction();
        check_corner_change_at(world, pos.offset(direction.to_offset())).await;
    }

    for direction in BlockDirection::horizontal() {
        let direction = direction.to_block_direction();
        let neighbor_pos = pos.offset(direction.to_offset());
        let corner_pos = if world.get_block_state(&neighbor_pos).is_solid_block() {
            neighbor_pos.up()
        } else {
            neighbor_pos.down()
        };
        check_corner_change_at(world, corner_pos).await;
    }
}

/// Vanilla `DefaultRedstoneWireEvaluator.updatePowerStrength` after `setBlock(..., 2)`:
/// for each of `pos` and its 6 neighbors, call `level.updateNeighborsAt(p, wire)`.
///
/// Without this, lamps / repeaters / torches only re-read power when dust is broken
/// (which runs [`update_wire_neighbors`]) — pulses and lever flips look "stuck".
pub async fn notify_after_wire_power_change(world: &Arc<World>, pos: &BlockPos) {
    // Each call keeps the wire as the source block. `World::update_neighbors`
    // derives its source from the target position, which is not what
    // DefaultRedstoneWireEvaluator does for the six adjacent positions.
    world
        .neighbor_updater
        .update_neighbors_at_except(world, *pos, &Block::REDSTONE_WIRE, None, None)
        .await;
    for direction in BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        world
            .neighbor_updater
            .update_neighbors_at_except(world, neighbor_pos, &Block::REDSTONE_WIRE, None, None)
            .await;
    }
    // Do NOT flush here. Vanilla marks dirty via setBlock → chunkSource.blockChanged,
    // then ServerChunkCache.broadcastChangedChunks once per tick. Per-wire flush was
    // O(wires × players) and not vanilla.
}

pub async fn is_emitting_redstone_power(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: &BlockPos,
    facing: BlockDirection,
) -> bool {
    get_redstone_power(block, state, world, pos, facing).await > 0
}

pub async fn get_redstone_power(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: &BlockPos,
    facing: BlockDirection,
) -> u8 {
    if state.is_solid_block() {
        return std::cmp::max(
            get_max_strong_power(world, pos, true).await,
            get_weak_power(block, state, world, pos, facing, true).await,
        );
    }
    get_weak_power(block, state, world, pos, facing, true).await
}

async fn get_redstone_power_no_dust(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: BlockPos,
    facing: BlockDirection,
) -> u8 {
    if state.is_solid_block() {
        return std::cmp::max(
            get_max_strong_power(world, &pos, false).await,
            get_weak_power(block, state, world, &pos, facing, false).await,
        );
    }
    get_weak_power(block, state, world, &pos, facing, false).await
}

async fn get_max_strong_power(world: &World, pos: &BlockPos, dust_power: bool) -> u8 {
    let mut max_power = 0;
    for side in BlockDirection::all() {
        let (block, state) = world.get_block_and_state(&pos.offset(side.to_offset()));
        max_power = max_power.max(
            get_strong_power(
                block,
                state,
                world,
                &pos.offset(side.to_offset()),
                side,
                dust_power,
            )
            .await,
        );
    }
    max_power
}

async fn get_max_weak_power(world: &World, pos: &BlockPos, dust_power: bool) -> u8 {
    let mut max_power = 0;
    for side in BlockDirection::all() {
        let (block, state) = world.get_block_and_state(&pos.offset(side.to_offset()));
        max_power = max_power.max(
            get_weak_power(
                block,
                state,
                world,
                &pos.offset(side.to_offset()),
                side,
                dust_power,
            )
            .await,
        );
    }
    max_power
}

async fn get_weak_power(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: &BlockPos,
    side: BlockDirection,
    dust_power: bool,
) -> u8 {
    if !dust_power && block == &Block::REDSTONE_WIRE {
        return 0;
    }
    world
        .block_registry
        .get_weak_redstone_power(block, world, pos, state, side)
        .await
}

async fn get_strong_power(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: &BlockPos,
    side: BlockDirection,
    dust_power: bool,
) -> u8 {
    if !dust_power && block == &Block::REDSTONE_WIRE {
        return 0;
    }
    world
        .block_registry
        .get_strong_redstone_power(block, world, pos, state, side)
        .await
}

pub async fn block_receives_redstone_power(world: &World, pos: &BlockPos) -> bool {
    for facing in BlockDirection::all() {
        let neighbor_pos = pos.offset(facing.to_offset());
        let (block, state) = world.get_block_and_state(&neighbor_pos);
        if is_emitting_redstone_power(block, state, world, &neighbor_pos, facing).await {
            return true;
        }
    }
    false
}

#[must_use]
pub fn is_diode(block: &Block) -> bool {
    block == &Block::REPEATER || block == &Block::COMPARATOR
}

pub async fn diode_get_input_strength(world: &World, pos: &BlockPos, facing: BlockDirection) -> u8 {
    let input_pos = pos.offset(facing.to_offset());
    let (input_block, input_state) = world.get_block_and_state(&input_pos);
    let power: u8 = get_redstone_power(input_block, input_state, world, &input_pos, facing).await;
    if power == 0 && input_state.is_solid_block() {
        return get_max_weak_power(world, &input_pos, true).await;
    }
    power
}

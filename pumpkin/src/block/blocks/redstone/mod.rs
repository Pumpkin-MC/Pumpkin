use std::sync::Arc;

/**
 * This implementation is heavily based on <https://github.com/MCHPR/MCHPRS>
 * Updated to fit pumpkin by 4lve
 */
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

pub mod buttons;
pub mod comparator;
pub mod copper_bulb;
pub mod dropper;
pub mod lever;
pub mod observer;
pub mod pressure_plate;
pub mod rails;
pub mod redstone_block;
pub mod redstone_lamp;
pub mod redstone_torch;
pub mod redstone_wire;
pub mod repeater;
pub mod target_block;
pub mod tripwire;
pub mod tripwire_hook;
pub mod turbo;

// abstract
pub mod abstract_redstone_gate;
pub mod dispenser;

pub async fn update_wire_neighbors(world: &Arc<World>, pos: &BlockPos) {
    // Use vanilla update order (W, E, D, U, N, S) for parity with technical redstone builds.
    // The order matters because some contraptions depend on which neighbor gets updated first.
    for direction in BlockDirection::update_order() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let block = world.get_block(&neighbor_pos).await;
        world
            .block_registry
            .on_neighbor_update(world, block, &neighbor_pos, block, true)
            .await;

        for n_direction in BlockDirection::update_order() {
            let n_neighbor_pos = neighbor_pos.offset(n_direction.to_offset());
            let block = world.get_block(&n_neighbor_pos).await;
            world
                .block_registry
                .on_neighbor_update(world, block, &n_neighbor_pos, block, true)
                .await;
        }
    }
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
        let (block, state) = world
            .get_block_and_state(&pos.offset(side.to_offset()))
            .await;
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
        let (block, state) = world
            .get_block_and_state(&pos.offset(side.to_offset()))
            .await;
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
        let (block, state) = world.get_block_and_state(&neighbor_pos).await;
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
    let (input_block, input_state) = world.get_block_and_state(&input_pos).await;
    let power: u8 = get_redstone_power(input_block, input_state, world, &input_pos, facing).await;
    if power == 0 && input_state.is_solid_block() {
        return get_max_weak_power(world, &input_pos, true).await;
    }
    power
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::BlockDirection;

    #[test]
    fn test_is_diode_repeater() {
        assert!(is_diode(&Block::REPEATER));
    }

    #[test]
    fn test_is_diode_comparator() {
        assert!(is_diode(&Block::COMPARATOR));
    }

    #[test]
    fn test_is_diode_non_diode_blocks() {
        assert!(!is_diode(&Block::REDSTONE_WIRE));
        assert!(!is_diode(&Block::REDSTONE_TORCH));
        assert!(!is_diode(&Block::REDSTONE_BLOCK));
        assert!(!is_diode(&Block::OBSERVER));
        assert!(!is_diode(&Block::PISTON));
        assert!(!is_diode(&Block::LEVER));
        assert!(!is_diode(&Block::STONE));
        assert!(!is_diode(&Block::AIR));
    }

    /// Vanilla update order is West, East, Down, Up, North, South.
    /// This differs from BlockDirection::all() which is Down, Up, North, South, West, East.
    /// Technical redstone builds depend on this specific order.
    #[test]
    fn test_vanilla_update_order() {
        let order = BlockDirection::update_order();
        assert_eq!(order[0], BlockDirection::West);
        assert_eq!(order[1], BlockDirection::East);
        assert_eq!(order[2], BlockDirection::Down);
        assert_eq!(order[3], BlockDirection::Up);
        assert_eq!(order[4], BlockDirection::North);
        assert_eq!(order[5], BlockDirection::South);
    }

    /// Verify that all() and update_order() contain the same directions
    /// but in different order. This is important for signal propagation:
    /// - all() is used for power queries (order doesn't matter, max is taken)
    /// - update_order() is used for neighbor notifications (order matters)
    #[test]
    fn test_all_vs_update_order_same_elements() {
        let mut all: Vec<BlockDirection> = BlockDirection::all().to_vec();
        let mut update: Vec<BlockDirection> = BlockDirection::update_order().to_vec();
        all.sort_by_key(|d| d.to_index());
        update.sort_by_key(|d| d.to_index());
        assert_eq!(all, update);
    }
}

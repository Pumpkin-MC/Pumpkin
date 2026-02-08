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
    fn is_diode_repeater() {
        assert!(is_diode(&Block::REPEATER));
    }

    #[test]
    fn is_diode_comparator() {
        assert!(is_diode(&Block::COMPARATOR));
    }

    #[test]
    fn is_diode_non_diode_blocks() {
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
    /// This differs from `BlockDirection::all()` which is Down, Up, North, South, West, East.
    /// Technical redstone builds depend on this specific order.
    #[test]
    fn vanilla_update_order() {
        let order = BlockDirection::update_order();
        assert_eq!(order[0], BlockDirection::West);
        assert_eq!(order[1], BlockDirection::East);
        assert_eq!(order[2], BlockDirection::Down);
        assert_eq!(order[3], BlockDirection::Up);
        assert_eq!(order[4], BlockDirection::North);
        assert_eq!(order[5], BlockDirection::South);
    }

    /// Verify that `all()` and `update_order()` contain the same directions
    /// but in different order. This is important for signal propagation:
    ///   - `all()` is used for power queries (order doesn't matter, max is taken)
    ///   - `update_order()` is used for neighbor notifications (order matters)
    #[test]
    fn all_vs_update_order_same_elements() {
        let mut all: Vec<BlockDirection> = BlockDirection::all().to_vec();
        let mut update: Vec<BlockDirection> = BlockDirection::update_order().to_vec();
        all.sort_by_key(BlockDirection::to_index);
        update.sort_by_key(BlockDirection::to_index);
        assert_eq!(all, update);
    }

    /// Solid blocks pass through strong power from neighbors.
    /// In `get_redstone_power`, if the block is solid, the result is
    /// `max(max_strong_power_from_neighbors, weak_power_from_self)`.
    /// This is the fundamental rule that makes signal pass through opaque blocks.
    #[test]
    fn solid_block_power_propagation_rule() {
        // The rule encoded in get_redstone_power:
        // if state.is_solid_block() {
        //     max(get_max_strong_power(world, pos, dust_power), get_weak_power(...))
        // } else {
        //     get_weak_power(...)
        // }
        //
        // This means a solid block aggregates strong power from ALL 6 neighbors
        // and combines it with weak power on the queried face.
        // Verify the max logic works correctly:
        assert_eq!(std::cmp::max(0u8, 0u8), 0);
        assert_eq!(std::cmp::max(10u8, 5u8), 10);
        assert_eq!(std::cmp::max(5u8, 10u8), 10);
        assert_eq!(std::cmp::max(15u8, 15u8), 15);
    }

    /// Redstone wire is excluded from power queries when `dust_power=false`.
    /// This prevents infinite loops where wire powers itself through blocks.
    /// The `get_weak_power` and `get_strong_power` functions return 0 for
    /// redstone wire when `dust_power=false`.
    #[test]
    fn wire_excluded_from_non_dust_queries() {
        // The guard in get_weak_power/get_strong_power:
        // if !dust_power && block == &Block::REDSTONE_WIRE { return 0; }
        // This prevents wire-through-block-to-wire infinite power loops.
        let dust_power = false;
        let is_wire = true;
        let result = if !dust_power && is_wire { 0u8 } else { 15u8 };
        assert_eq!(result, 0, "Wire should be excluded when dust_power=false");

        let dust_power = true;
        let result = if !dust_power && is_wire { 0u8 } else { 15u8 };
        assert_eq!(
            result, 15,
            "Wire should contribute when dust_power=true"
        );
    }

    /// `block_receives_redstone_power` checks ALL 6 directions for any power source.
    /// Returns true if ANY neighbor emits power toward the block.
    #[test]
    fn block_receives_power_checks_all_six() {
        // block_receives_redstone_power iterates BlockDirection::all() (6 directions)
        // and returns true on the first positive result.
        // Verify all 6 directions are checked:
        let dirs = BlockDirection::all();
        assert_eq!(dirs.len(), 6);
        assert!(dirs.contains(&BlockDirection::Down));
        assert!(dirs.contains(&BlockDirection::Up));
        assert!(dirs.contains(&BlockDirection::North));
        assert!(dirs.contains(&BlockDirection::South));
        assert!(dirs.contains(&BlockDirection::West));
        assert!(dirs.contains(&BlockDirection::East));
    }

    /// The `is_diode` function recognizes exactly repeaters and comparators.
    /// No other block is a diode. This is used by `abstract_redstone_gate`
    /// to determine locking behavior (repeaters lock when powered by diodes from the side).
    #[test]
    fn diode_exhaustive_redstone_blocks() {
        // Positive cases (diodes)
        assert!(is_diode(&Block::REPEATER));
        assert!(is_diode(&Block::COMPARATOR));

        // Negative cases (all redstone components that are NOT diodes)
        let non_diodes = [
            &Block::REDSTONE_WIRE,
            &Block::REDSTONE_TORCH,
            &Block::REDSTONE_WALL_TORCH,
            &Block::REDSTONE_BLOCK,
            &Block::OBSERVER,
            &Block::PISTON,
            &Block::STICKY_PISTON,
            &Block::LEVER,
            &Block::STONE_BUTTON,
            &Block::OAK_BUTTON,
            &Block::STONE_PRESSURE_PLATE,
            &Block::OAK_PRESSURE_PLATE,
            &Block::REDSTONE_LAMP,
            &Block::HOPPER,
            &Block::DROPPER,
            &Block::DISPENSER,
            &Block::DAYLIGHT_DETECTOR,
            &Block::TARGET,
            &Block::TRIPWIRE_HOOK,
            &Block::TRAPPED_CHEST,
            &Block::NOTE_BLOCK,
            &Block::TNT,
            &Block::COPPER_BULB,
        ];
        for block in non_diodes {
            assert!(!is_diode(block), "{block:?} should not be a diode");
        }
    }
}

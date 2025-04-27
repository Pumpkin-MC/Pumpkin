use std::sync::Arc;

/**
 * This implementation is heavily based on <https://github.com/MCHPR/MCHPRS>
 * Updated to fit pumpkin by 4lve
 */
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::world::World;

pub(crate) mod buttons;
pub(crate) mod lever;
pub(crate) mod observer;
pub(crate) mod piston;
pub(crate) mod rails;
pub(crate) mod redstone_block;
pub(crate) mod redstone_lamp;
pub(crate) mod redstone_torch;
pub(crate) mod redstone_wire;
pub(crate) mod repeater;
pub(crate) mod target_block;
pub(crate) mod turbo;

pub async fn update_wire_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for direction in BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let block = world.get_block(&neighbor_pos).await.unwrap();
        world
            .block_registry
            .on_neighbor_update(world, &block, &neighbor_pos, &block, true)
            .await;

        for n_direction in BlockDirection::all() {
            let n_neighbor_pos = neighbor_pos.offset(n_direction.to_offset());
            let block = world.get_block(&n_neighbor_pos).await.unwrap();
            world
                .block_registry
                .on_neighbor_update(world, &block, &n_neighbor_pos, &block, true)
                .await;
        }
    }
}

pub async fn get_redstone_power(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: BlockPos,
    facing: BlockDirection,
) -> u8 {
    if state.is_solid() {
        return std::cmp::max(
            get_max_strong_power(world, &pos, true).await,
            get_weak_power(block, state, world, &pos, facing, true).await,
        );
    }
    get_weak_power(block, state, world, &pos, facing, true).await
}

async fn get_redstone_power_no_dust(
    block: &Block,
    state: &BlockState,
    world: &World,
    pos: BlockPos,
    facing: BlockDirection,
) -> u8 {
    if state.is_solid() {
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
            .get_block_and_block_state(&pos.offset(side.to_offset()))
            .await
            .unwrap();
        max_power = max_power.max(
            get_strong_power(
                &block,
                &state,
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
            .get_block_and_block_state(&pos.offset(side.to_offset()))
            .await
            .unwrap();
        max_power = max_power.max(
            get_weak_power(
                &block,
                &state,
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
    for face in BlockDirection::all() {
        let neighbor_pos = pos.offset(face.to_offset());
        let (block, state) = world
            .get_block_and_block_state(&neighbor_pos)
            .await
            .unwrap();
        if get_redstone_power(&block, &state, world, neighbor_pos, face).await > 0 {
            return true;
        }
    }
    false
}

pub fn is_diode(block: &Block) -> bool {
    block == &Block::REPEATER || block == &Block::COMPARATOR
}

pub async fn diode_get_input_strength(world: &World, pos: &BlockPos, facing: BlockDirection) -> u8 {
    let input_pos = pos.offset(facing.to_offset());
    let (input_block, input_state) = world.get_block_and_block_state(&input_pos).await.unwrap();
    let power: u8 = get_redstone_power(&input_block, &input_state, world, input_pos, facing).await;
    if power == 0 && input_state.is_solid() {
        return get_max_weak_power(world, &input_pos, true).await;
    }
    power
}

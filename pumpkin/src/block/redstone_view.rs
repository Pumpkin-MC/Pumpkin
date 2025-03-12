use pumpkin_data::block::{Block, BlockProperties, EnumVariants, RedstoneWireLikeProperties};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::world::World;

async fn get_strong_redstone_power(
    world: &World,
    block_pos: &BlockPos,
    direction: &BlockDirection,
) -> u8 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();
    world
        .block_registry
        .get_strong_redstone_power(&block, world, block_pos, &state, direction)
        .await
}

pub async fn get_received_strong_redstone_power(world: &World, block_pos: &BlockPos) -> u8 {
    let mut power = 0;

    for direction in BlockDirection::all() {
        power = std::cmp::max(
            power,
            get_strong_redstone_power(world, &block_pos.offset(direction.to_offset()), &direction)
                .await,
        );
        if power >= 15 {
            break;
        }
    }

    power
}

pub async fn get_emitted_redstone_power_with_gate(
    world: &World,
    block_pos: &BlockPos,
    direction: &BlockDirection,
    only_from_gate: bool,
) -> u8 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();

    if only_from_gate {
        // TODO: return AbstractRedstoneGateBlock.isRedstoneGate(lv) ? this.getStrongRedstonePower(pos, direction) : 0;
    } else if block == Block::REDSTONE_BLOCK {
        return 15;
    } else if block == Block::REDSTONE_WIRE {
        let wire_props = RedstoneWireLikeProperties::from_state_id(state.id, &block);
        return wire_props.power.to_index() as u8;
    } else if world
        .block_registry
        .emits_redstone_power(&block, &state)
        .await
    {
        return get_strong_redstone_power(world, block_pos, direction).await;
    }

    0
}

pub async fn is_emitting_redstone_power(
    world: &World,
    block_pos: &BlockPos,
    direction: &BlockDirection,
) -> bool {
    get_emitted_redstone_power(world, block_pos, direction).await > 0
}

pub async fn get_emitted_redstone_power(
    world: &World,
    block_pos: &BlockPos,
    direction: &BlockDirection,
) -> u8 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();
    let power = world
        .block_registry
        .get_weak_redstone_power(&block, world, block_pos, &state, direction)
        .await;

    if state.is_solid {
        std::cmp::max(
            power,
            get_received_strong_redstone_power(world, block_pos).await,
        )
    } else {
        power
    }
}

pub async fn is_receiving_redstone_power(world: &World, block_pos: &BlockPos) -> bool {
    for direction in BlockDirection::all() {
        if get_emitted_redstone_power(world, block_pos, &direction).await > 0 {
            return true;
        }
    }

    false
}

pub async fn get_received_redstone_power(world: &World, block_pos: &BlockPos) -> u8 {
    let mut power = 0;

    for direction in BlockDirection::all() {
        power = std::cmp::max(
            power,
            get_emitted_redstone_power(world, &block_pos.offset(direction.to_offset()), &direction)
                .await,
        );
        if power >= 15 {
            break;
        }
    }

    power
}

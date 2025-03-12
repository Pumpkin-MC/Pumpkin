use pumpkin_data::block::{
    Block, BlockProperties, BlockState, EnumVariants, Integer0To15, RedstoneWireLikeProperties,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::world::{BlockFlags, World};

async fn get_strong_power_at(world: &World, block_pos: &BlockPos) -> u8 {
    world
        .block_registry
        .get_pumpkin_block(&Block::REDSTONE_WIRE)
        .unwrap()
        .get_strong_power(world, block_pos)
        .await
}

async fn get_wire_power_at(world: &World, block_pos: &BlockPos) -> u8 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();
    if block == Block::REDSTONE_WIRE {
        let wire_props = RedstoneWireLikeProperties::from_state_id(state.id, &block);
        return wire_props.power.to_index() as u8;
    }
    0
}

async fn calculate_wire_power_at(world: &World, block_pos: &BlockPos) -> u8 {
    // Using i8 here so we don't panic if we get negative power
    let mut power: i8 = 0;

    for direction in BlockDirection::horizontal() {
        let other_pos = block_pos.offset(direction.to_offset());
        let other_block_state = world.get_block_state(&other_pos).await.unwrap();
        let up_pos = block_pos.up();
        let up_block_state = world.get_block_state(&up_pos).await.unwrap();
        power = std::cmp::max(power, get_wire_power_at(world, &other_pos).await as i8);

        if other_block_state.is_solid && !up_block_state.is_solid {
            let other_up_pos = other_pos.up();
            power = std::cmp::max(power, get_wire_power_at(world, &other_up_pos).await as i8);
        } else if !other_block_state.is_solid {
            let other_down_pos = other_pos.down();
            power = std::cmp::max(power, get_wire_power_at(world, &other_down_pos).await as i8);
        }
    }

    std::cmp::max(power - 1, 0) as u8
}

pub async fn update(
    world: &World,
    block_pos: &BlockPos,
    block: &Block,
    state: &BlockState,
    _wire_orientation: Option<BlockDirection>,
    _block_added: bool,
) {
    let power = calculate_total_power_at(world, block_pos).await;
    let mut wire_props = RedstoneWireLikeProperties::from_state_id(state.id, block);

    if wire_props.power.to_index() as u8 != power {
        if world.get_block_state(block_pos).await.unwrap().id == state.id {
            wire_props.power = Integer0To15::from_index(u16::from(power));
            world
                .set_block_state(
                    block_pos,
                    wire_props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }

        let mut positions_to_update = Vec::new();
        positions_to_update.push(*block_pos);

        for direction in BlockDirection::all() {
            positions_to_update.push(block_pos.offset(direction.to_offset()));
        }

        for position in positions_to_update {
            world.update_neighbors(&position, None).await;
        }
    }
}

async fn calculate_total_power_at(world: &World, block_pos: &BlockPos) -> u8 {
    let power = get_strong_power_at(world, block_pos).await;

    if power == 15 {
        return power;
    }

    std::cmp::max(power, calculate_wire_power_at(world, block_pos).await)
}

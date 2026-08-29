use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockState;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;
use rand::RngExt;

use crate::world::World;

pub mod coral_block;
pub mod coral_fan;
pub mod coral_plant;

pub fn scan_for_water(world: &Arc<World>, pos: &BlockPos) -> bool {
    for direction in BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let block = world.get_fluid(&neighbor_pos);
        if block.has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER) {
            return true;
        }
    }
    false
}
fn is_dead_coral(block: &Block) -> bool {
    block == &Block::DEAD_BRAIN_CORAL
        || block == &Block::DEAD_BUBBLE_CORAL
        || block == &Block::DEAD_FIRE_CORAL
        || block == &Block::DEAD_HORN_CORAL
        || block == &Block::DEAD_TUBE_CORAL
        || block == &Block::DEAD_BRAIN_CORAL_BLOCK
        || block == &Block::DEAD_BUBBLE_CORAL_BLOCK
        || block == &Block::DEAD_FIRE_CORAL_BLOCK
        || block == &Block::DEAD_HORN_CORAL_BLOCK
        || block == &Block::DEAD_TUBE_CORAL_BLOCK
        || block == &Block::DEAD_BRAIN_CORAL_FAN
        || block == &Block::DEAD_BRAIN_CORAL_WALL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_FAN
        || block == &Block::DEAD_BUBBLE_CORAL_WALL_FAN
        || block == &Block::DEAD_FIRE_CORAL_FAN
        || block == &Block::DEAD_FIRE_CORAL_WALL_FAN
        || block == &Block::DEAD_HORN_CORAL_FAN
        || block == &Block::DEAD_HORN_CORAL_WALL_FAN
        || block == &Block::DEAD_TUBE_CORAL_FAN
        || block == &Block::DEAD_TUBE_CORAL_WALL_FAN
}
/// Vanilla `BaseCoralPlantTypeBlock.canSurvive`: `below.isFaceSturdy(level, below, UP)`.
#[must_use]
pub fn floor_coral_can_survive(world: &World, coral_pos: &BlockPos) -> bool {
    world.is_face_sturdy(&coral_pos.down(), BlockDirection::Up)
}

/// Cached `SupportType.FULL` when there is no `World` (worldgen `BlockAccessor`).
#[must_use]
const fn floor_coral_can_survive_state(support: &BlockState) -> bool {
    support.is_side_solid(BlockDirection::Up)
}

#[must_use]
pub fn floor_coral_can_survive_at(
    world: Option<&World>,
    accessor: &dyn BlockAccessor,
    pos: &BlockPos,
) -> bool {
    world.map_or_else(
        || floor_coral_can_survive_state(accessor.get_block_state(&pos.down())),
        |world| floor_coral_can_survive(world, pos),
    )
}

/// `from_state_id` panics on air if a piston already popped this cell.
#[must_use]
pub fn coral_still_present(world: &World, pos: &BlockPos, block: &Block) -> bool {
    Block::from_state_id(world.get_block_state(pos).id) == block
}

pub fn try_schedule_die_tick(block: &Block, world: &Arc<World>, pos: &BlockPos) {
    let tick_delay = 60 + rand::rng().random_range(0..40);
    world.schedule_block_tick(
        block,
        *pos,
        tick_delay as u8,
        pumpkin_world::tick::TickPriority::Normal,
    );
}

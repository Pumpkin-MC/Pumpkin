use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use super::super::memory::{MemoryModuleType, MemoryStatus, types};
use super::one_shot::OneShot;
use super::utils::set_walk_and_look_target_memories_to_block;

#[must_use]
pub fn go_to_target_location(
    location_memory: MemoryModuleType<BlockPos>,
    close_enough_dist: i32,
    speed_modifier: f32,
) -> OneShot {
    OneShot::with_required(
        "GoToTargetLocation",
        vec![
            (location_memory.id(), MemoryStatus::ValuePresent),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
        ],
        vec![types::LOOK_TARGET.id()],
        move |tick| {
            let Some(location) = tick.brain.get(location_memory).copied() else {
                return false;
            };
            if location.squared_distance(&tick.mob.get_entity().block_pos.load())
                < close_enough_dist * close_enough_dist
            {
                return true;
            }
            let mut rng = tick.mob.get_random();
            let nearby = BlockPos::new(
                location.0.x + rng.random_range(0..3) - 1,
                location.0.y,
                location.0.z + rng.random_range(0..3) - 1,
            );
            set_walk_and_look_target_memories_to_block(
                tick.brain,
                nearby,
                speed_modifier,
                close_enough_dist,
            );
            true
        },
    )
}

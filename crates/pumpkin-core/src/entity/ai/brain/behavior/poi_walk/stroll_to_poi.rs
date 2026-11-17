use super::super::OneShot;
use super::closer_to_center_than;
use crate::entity::ai::brain::memory::{
    GlobalPos, MemoryModuleType, MemoryStatus, types, walk_target::WalkTarget,
};

const STROLL_INTERVAL: i64 = 80;

#[must_use]
pub fn create(
    memory_type: MemoryModuleType<GlobalPos>,
    speed_modifier: f32,
    close_enough_dist: i32,
    max_distance_from_poi: i32,
) -> OneShot {
    let mut next_ok_start_time = 0i64;
    OneShot::new(
        "StrollToPoi",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (memory_type.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            let Some(pos) = tick.state.get(memory_type).copied() else {
                return false;
            };
            if pos.dimension.id != tick.world.dimension.id
                || !closer_to_center_than(
                    pos.pos,
                    tick.mob.get_entity().pos.load(),
                    f64::from(max_distance_from_poi),
                )
            {
                return false;
            }
            if tick.time <= next_ok_start_time {
                return true;
            }
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::from_block_pos(pos.pos, speed_modifier, close_enough_dist),
            );
            next_ok_start_time = tick.time + STROLL_INTERVAL;
            true
        },
    )
}

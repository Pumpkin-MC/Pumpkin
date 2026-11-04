use rand::RngExt;

use super::super::OneShot;
use super::closer_to_center_than;
use crate::entity::ai::brain::memory::{
    GlobalPos, MemoryModuleType, MemoryStatus, types, walk_target::WalkTarget,
};

const STROLL_INTERVAL: i64 = 100;

#[must_use]
pub fn create(
    stroll_to_memory_type: MemoryModuleType<Vec<GlobalPos>>,
    speed_modifier: f32,
    close_enough_dist: i32,
    max_distance_from_poi: i32,
    must_be_close_to_memory_type: MemoryModuleType<GlobalPos>,
) -> OneShot {
    let mut next_ok_start_time = 0i64;
    OneShot::new(
        "StrollToPoiList",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (stroll_to_memory_type.id(), MemoryStatus::ValuePresent),
            (
                must_be_close_to_memory_type.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let Some(stroll_to) = tick.state.get(stroll_to_memory_type).cloned() else {
                return false;
            };
            let Some(stay_close_to) = tick.state.get(must_be_close_to_memory_type).copied() else {
                return false;
            };
            if stroll_to.is_empty() {
                return false;
            }
            let mut rng = tick.mob.get_random();
            let index = rng.random_range(0..stroll_to.len());
            let Some(target_pos) = stroll_to.get(index).copied() else {
                return false;
            };
            if target_pos.dimension.id != tick.world.dimension.id
                || !closer_to_center_than(
                    stay_close_to.pos,
                    tick.mob.get_entity().pos.load(),
                    f64::from(max_distance_from_poi),
                )
            {
                return false;
            }
            if tick.time > next_ok_start_time {
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_block_pos(target_pos.pos, speed_modifier, close_enough_dist),
                );
                next_ok_start_time = tick.time + STROLL_INTERVAL;
            }
            true
        },
    )
}

use super::super::OneShot;
use super::closer_to_center_than;
use crate::entity::ai::brain::memory::{
    GlobalPos, MemoryModuleType, MemoryStatus, types, walk_target::WalkTarget,
};
use crate::entity::ai::util::land_random_pos;

const MIN_TIME_BETWEEN_STROLLS: i64 = 180;
const STROLL_MAX_XZ_DIST: i32 = 8;
const STROLL_MAX_Y_DIST: i32 = 6;

#[must_use]
pub fn create(
    memory_type: MemoryModuleType<GlobalPos>,
    speed_modifier: f32,
    max_distance_from_poi: i32,
) -> OneShot {
    let mut next_ok_start_time = 0i64;
    OneShot::new(
        "StrollAroundPoi",
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
            let land_pos =
                land_random_pos::get_pos(tick.mob, STROLL_MAX_XZ_DIST, STROLL_MAX_Y_DIST);
            tick.state.set_optional(
                types::WALK_TARGET,
                land_pos.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 1)),
            );
            next_ok_start_time = tick.time + MIN_TIME_BETWEEN_STROLLS;
            true
        },
    )
}

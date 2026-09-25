use super::super::memory::global_pos::GlobalPos;
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleType, MemoryStatus, types};
use super::one_shot::OneShot;
use crate::entity::ai::brain::memory::position_tracker::{BlockPosTracker, PositionTracker};
use crate::entity::ai::util::land_random_pos;
use std::sync::Arc;

const MIN_TIME_BETWEEN_STROLLS: i64 = 180;
const TIME_BETWEEN_WALKS: i64 = 80;
const STROLL_MAX_XZ_DIST: i32 = 8;
const STROLL_MAX_Y_DIST: i32 = 6;

#[must_use]
pub fn stroll_to_poi(
    memory: MemoryModuleType<GlobalPos>,
    speed_modifier: f32,
    close_enough_dist: i32,
    max_distance_from_poi: i32,
) -> OneShot {
    let max_distance_squared = f64::from(max_distance_from_poi * max_distance_from_poi);
    let mut next_ok_start_time = 0;
    OneShot::with_required(
        "StrollToPoi",
        vec![(memory.id(), MemoryStatus::ValuePresent)],
        vec![types::WALK_TARGET.id()],
        move |tick| {
            let Some(poi) = tick.brain.get(memory).copied() else {
                return false;
            };
            if !in_range(tick, poi, max_distance_squared) {
                return false;
            }
            if tick.time <= next_ok_start_time {
                return true;
            }
            tick.brain.set(
                types::WALK_TARGET,
                WalkTarget::new(
                    Arc::new(BlockPosTracker::new(poi.pos)) as Arc<dyn PositionTracker>,
                    speed_modifier,
                    close_enough_dist,
                ),
            );
            next_ok_start_time = tick.time + TIME_BETWEEN_WALKS;
            true
        },
    )
}

#[must_use]
pub fn stroll_around_poi(
    memory: MemoryModuleType<GlobalPos>,
    speed_modifier: f32,
    max_distance_from_poi: i32,
) -> OneShot {
    let max_distance_squared = f64::from(max_distance_from_poi * max_distance_from_poi);
    let mut next_ok_start_time = 0;
    OneShot::with_required(
        "StrollAroundPoi",
        vec![(memory.id(), MemoryStatus::ValuePresent)],
        vec![types::WALK_TARGET.id()],
        move |tick| {
            let Some(poi) = tick.brain.get(memory).copied() else {
                return false;
            };
            if !in_range(tick, poi, max_distance_squared) {
                return false;
            }
            if tick.time <= next_ok_start_time {
                return true;
            }
            let target = land_random_pos::get_pos(tick.mob, STROLL_MAX_XZ_DIST, STROLL_MAX_Y_DIST)
                .map(|pos| WalkTarget::from_vec(pos, speed_modifier, 1));
            tick.brain.set_optional(types::WALK_TARGET, target);
            next_ok_start_time = tick.time + MIN_TIME_BETWEEN_STROLLS;
            true
        },
    )
}

fn in_range(tick: &super::super::BrainTick<'_>, poi: GlobalPos, max_distance_squared: f64) -> bool {
    if tick.world.dimension.id != poi.dimension.id {
        return false;
    }
    let body_pos = tick.mob.get_entity().pos.load();
    poi.pos
        .dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
        < max_distance_squared
}

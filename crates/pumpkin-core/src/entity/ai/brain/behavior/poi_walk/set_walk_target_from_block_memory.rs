use std::f64::consts::FRAC_PI_2;

use pumpkin_util::math::position::BlockPos;

use super::super::OneShot;
use super::release_poi::release_poi;
use crate::entity::ai::brain::memory::{
    GlobalPos, MemoryModuleType, MemoryStatus, types, walk_target::WalkTarget,
};
use crate::entity::ai::util::default_random_pos;

const MAX_TRIES: i32 = 1000;

#[must_use]
pub fn create(
    memory_type: MemoryModuleType<GlobalPos>,
    speed_modifier: f32,
    close_enough_dist: i32,
    too_far_distance: i32,
    too_long_unreachable_duration: i64,
) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromBlockMemory",
        vec![
            (
                types::CANT_REACH_WALK_TARGET_SINCE.id(),
                MemoryStatus::Registered,
            ),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (memory_type.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            let Some(target_pos) = tick.state.get(memory_type).copied() else {
                return false;
            };
            let cant_reach_target_since =
                tick.state.get(types::CANT_REACH_WALK_TARGET_SINCE).copied();
            let body_pos = tick.mob.get_entity().block_pos.load();
            if target_pos.dimension.id == tick.world.dimension.id
                && cant_reach_target_since
                    .is_none_or(|since| tick.time - since <= too_long_unreachable_duration)
            {
                if target_pos.pos.manhattan_distance(body_pos) > too_far_distance {
                    let mut towards_target_pos = None;
                    let mut tries = 0;
                    loop {
                        if towards_target_pos.is_some_and(|pos| {
                            BlockPos::floored_v(pos).manhattan_distance(body_pos)
                                <= too_far_distance
                        }) {
                            break;
                        }
                        towards_target_pos = default_random_pos::get_pos_towards(
                            tick.mob,
                            15,
                            7,
                            target_pos.pos.to_f64(),
                            FRAC_PI_2,
                        );
                        tries += 1;
                        if tries == MAX_TRIES {
                            release_poi(tick.state, tick.mob, memory_type);
                            tick.state.erase(memory_type.id());
                            tick.state
                                .set(types::CANT_REACH_WALK_TARGET_SINCE, tick.time);
                            return true;
                        }
                    }
                    if let Some(towards_target_pos) = towards_target_pos {
                        tick.state.set(
                            types::WALK_TARGET,
                            WalkTarget::from_vec(
                                towards_target_pos,
                                speed_modifier,
                                close_enough_dist,
                            ),
                        );
                    }
                } else if target_pos.pos.manhattan_distance(body_pos) > close_enough_dist {
                    tick.state.set(
                        types::WALK_TARGET,
                        WalkTarget::from_block_pos(
                            target_pos.pos,
                            speed_modifier,
                            close_enough_dist,
                        ),
                    );
                }
            } else {
                release_poi(tick.state, tick.mob, memory_type);
                tick.state.erase(memory_type.id());
                tick.state
                    .set(types::CANT_REACH_WALK_TARGET_SINCE, tick.time);
            }

            true
        },
    )
}

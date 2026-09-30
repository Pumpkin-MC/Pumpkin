use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::entity::ai::util::land_random_pos;

use super::super::BrainTick;
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleType, MemoryStatus, types};
use super::one_shot::OneShot;

const FLEE_ATTEMPTS: i32 = 10;
const FLEE_HORIZONTAL_DIST: i32 = 16;
const FLEE_VERTICAL_DIST: i32 = 7;

#[must_use]
pub fn pos(
    memory: MemoryModuleType<BlockPos>,
    speed_modifier: f32,
    desired_distance: i32,
    interrupt_current_walk: bool,
) -> OneShot {
    create(
        memory.id(),
        speed_modifier,
        desired_distance,
        interrupt_current_walk,
        move |tick| {
            tick.brain.get(memory).map(|pos| {
                Vector3::new(
                    f64::from(pos.0.x) + 0.5,
                    f64::from(pos.0.y),
                    f64::from(pos.0.z) + 0.5,
                )
            })
        },
    )
}

#[must_use]
pub fn entity(
    memory: MemoryModuleType<Arc<dyn EntityBase>>,
    speed_modifier: f32,
    desired_distance: i32,
    interrupt_current_walk: bool,
) -> OneShot {
    create(
        memory.id(),
        speed_modifier,
        desired_distance,
        interrupt_current_walk,
        move |tick| {
            tick.brain
                .get(memory)
                .map(|entity| entity.get_entity().pos.load())
        },
    )
}

fn create(
    walk_away_from: super::super::memory::MemoryModuleId,
    speed_modifier: f32,
    desired_distance: i32,
    interrupt_current_walk: bool,
    avoid_position: impl Fn(&BrainTick<'_>) -> Option<Vector3<f64>> + Send + Sync + 'static,
) -> OneShot {
    let desired_squared = f64::from(desired_distance * desired_distance);
    OneShot::with_required(
        "SetWalkTargetAwayFrom",
        vec![(walk_away_from, MemoryStatus::ValuePresent)],
        vec![types::WALK_TARGET.id()],
        move |tick| {
            let has_walk_target = tick.brain.has_memory_value(types::WALK_TARGET.id());
            if has_walk_target && !interrupt_current_walk {
                return false;
            }
            let Some(avoid_pos) = avoid_position(tick) else {
                return false;
            };
            let body_pos = tick.mob.get_entity().pos.load();
            if body_pos.squared_distance_to_vec(&avoid_pos) >= desired_squared {
                return false;
            }

            let heading_the_same_way = tick
                .brain
                .get(types::WALK_TARGET)
                .filter(|current| current.speed_modifier() == speed_modifier)
                .is_some_and(|current| {
                    let current_direction = current.target().current_position() - body_pos;
                    let avoid_direction = avoid_pos - body_pos;
                    current_direction.x * avoid_direction.x
                        + current_direction.y * avoid_direction.y
                        + current_direction.z * avoid_direction.z
                        < 0.0
                });
            if heading_the_same_way {
                return false;
            }

            for _ in 0..FLEE_ATTEMPTS {
                if let Some(flee_to) = land_random_pos::get_pos_away(
                    tick.mob,
                    FLEE_HORIZONTAL_DIST,
                    FLEE_VERTICAL_DIST,
                    avoid_pos,
                ) {
                    tick.brain.set(
                        types::WALK_TARGET,
                        WalkTarget::from_vec(flee_to, speed_modifier, 0),
                    );
                    break;
                }
            }
            true
        },
    )
}

use std::sync::Arc;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TargetMemory {
    #[default]
    Adult,
    Player,
}

#[must_use]
pub fn create(min_range: i32, max_range: i32, speed_modifier: f32) -> OneShot {
    create_with(
        min_range,
        max_range,
        move |_| speed_modifier,
        TargetMemory::Adult,
        false,
    )
}

#[must_use]
pub fn create_with(
    min_range: i32,
    max_range: i32,
    speed_modifier: impl Fn(&dyn Mob) -> f32 + Send + Sync + 'static,
    target_memory: TargetMemory,
    target_eye: bool,
) -> OneShot {
    let memory_id = match target_memory {
        TargetMemory::Adult => types::NEAREST_VISIBLE_ADULT.id(),
        TargetMemory::Player => types::NEAREST_VISIBLE_PLAYER.id(),
    };
    OneShot::new(
        "BabyFollowAdult",
        vec![
            (memory_id, MemoryStatus::ValuePresent),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
        ],
        move |tick| {
            let living = &tick.mob.get_mob_entity().living_entity;
            if !living.is_baby() {
                return false;
            }
            let target: Arc<dyn EntityBase> = match target_memory {
                TargetMemory::Adult => {
                    let Some(adult) = tick.state.get(types::NEAREST_VISIBLE_ADULT).cloned() else {
                        return false;
                    };
                    adult
                }
                TargetMemory::Player => {
                    let Some(player) = tick.state.get(types::NEAREST_VISIBLE_PLAYER).cloned()
                    else {
                        return false;
                    };
                    player as Arc<dyn EntityBase>
                }
            };
            let distance = living
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());
            let max = f64::from(max_range + 1);
            let min = f64::from(min_range);
            if distance >= max * max || distance < min * min {
                return false;
            }
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::with_target_eye_height(
                    target.clone(),
                    true,
                    target_eye,
                )) as Arc<dyn PositionTracker>,
            );
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(
                    Arc::new(EntityTracker::with_target_eye_height(
                        target, target_eye, target_eye,
                    )),
                    speed_modifier(tick.mob),
                    min_range - 1,
                ),
            );
            true
        },
    )
}

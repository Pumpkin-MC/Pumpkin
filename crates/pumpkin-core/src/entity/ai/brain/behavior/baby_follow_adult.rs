use std::sync::Arc;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;

#[must_use]
pub fn create(min_range: i32, max_range: i32, speed_modifier: f32) -> OneShot {
    OneShot::new(
        "BabyFollowAdult",
        vec![
            (
                types::NEAREST_VISIBLE_ADULT.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
        ],
        move |tick| {
            let living = &tick.mob.get_mob_entity().living_entity;
            if !living.is_baby() {
                return false;
            }
            let Some(adult) = tick.state.get(types::NEAREST_VISIBLE_ADULT).cloned() else {
                return false;
            };
            let distance = living
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&adult.get_entity().pos.load());
            let max = f64::from(max_range + 1);
            let min = f64::from(min_range);
            if distance >= max * max || distance < min * min {
                return false;
            }
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(adult.clone(), true)) as Arc<dyn PositionTracker>,
            );
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(
                    Arc::new(EntityTracker::new(adult, false)),
                    speed_modifier,
                    min_range - 1,
                ),
            );
            true
        },
    )
}

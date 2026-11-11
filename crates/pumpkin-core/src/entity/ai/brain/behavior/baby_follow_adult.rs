use std::sync::Arc;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::mob::Mob;

#[must_use]
pub fn create(min_range: i32, max_range: i32, speed_modifier: f32) -> OneShot {
    create_with(min_range, max_range, move |_| speed_modifier)
}

/// Vanilla `BabyFollowAdult.create(followRange, speedModifier, ...)` with a speed that depends on
/// the mob, as the function overload has it.
#[must_use]
pub fn create_with(
    min_range: i32,
    max_range: i32,
    speed_modifier: impl Fn(&dyn Mob) -> f32 + Send + Sync + 'static,
) -> OneShot {
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
                    speed_modifier(tick.mob),
                    min_range - 1,
                ),
            );
            true
        },
    )
}

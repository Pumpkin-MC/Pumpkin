use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::OneShot;
use crate::entity::mob::Mob;

#[must_use]
pub fn create(speed_modifier: f32, close_enough_distance: i32) -> OneShot {
    create_with(|_| true, move |_| speed_modifier, close_enough_distance)
}

#[must_use]
pub fn create_with(
    can_set_walk_target: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    speed_modifier: impl Fn(&dyn Mob) -> f32 + Send + Sync + 'static,
    close_enough_distance: i32,
) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromLookTarget",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            if !can_set_walk_target(tick) {
                return false;
            }
            let Some(target) = tick.state.get(types::LOOK_TARGET).cloned() else {
                return false;
            };
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(target, speed_modifier(tick.mob), close_enough_distance),
            );
            true
        },
    )
}

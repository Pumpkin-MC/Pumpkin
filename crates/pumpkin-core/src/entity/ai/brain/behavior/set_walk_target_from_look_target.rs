use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::OneShot;

#[must_use]
pub fn create(speed_modifier: f32, close_enough_distance: i32) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromLookTarget",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            let Some(target) = tick.state.get(types::LOOK_TARGET).cloned() else {
                return false;
            };
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(target, speed_modifier, close_enough_distance),
            );
            true
        },
    )
}

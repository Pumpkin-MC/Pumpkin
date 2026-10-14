use crate::entity::ai::util::land_random_pos;

use super::super::memory::{MemoryStatus, WalkTarget, types};
use super::OneShot;

#[must_use]
pub fn stroll(speed_modifier: f32, horizontal_distance: i32, vertical_distance: i32) -> OneShot {
    OneShot::new(
        "RandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let position =
                land_random_pos::get_pos(tick.mob, horizontal_distance, vertical_distance);
            tick.brain.set_optional(
                types::WALK_TARGET,
                position.map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0)),
            );
            true
        },
    )
}

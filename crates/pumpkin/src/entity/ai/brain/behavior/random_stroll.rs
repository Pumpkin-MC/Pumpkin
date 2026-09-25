use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;
use crate::entity::ai::util::land_random_pos;

const MAX_XZ_DIST: i32 = 10;
const MAX_Y_DIST: i32 = 7;

#[must_use]
pub fn stroll(speed_modifier: f32, may_stroll_from_water: bool) -> OneShot {
    stroll_with_range(
        speed_modifier,
        MAX_XZ_DIST,
        MAX_Y_DIST,
        may_stroll_from_water,
    )
}

#[must_use]
pub fn stroll_with_range(
    speed_modifier: f32,
    max_horizontal_distance: i32,
    max_vertical_distance: i32,
    may_stroll_from_water: bool,
) -> OneShot {
    OneShot::new(
        "RandomStroll",
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            if !may_stroll_from_water && tick.mob.get_entity().is_in_water() {
                return false;
            }
            let target =
                land_random_pos::get_pos(tick.mob, max_horizontal_distance, max_vertical_distance)
                    .map(|pos| WalkTarget::from_vec(pos, speed_modifier, 0));
            tick.brain.set_optional(types::WALK_TARGET, target);
            true
        },
    )
}

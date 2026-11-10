use super::super::memory::{MemoryStatus, types};
use super::{OneShot, PositionGetter, utils};
use crate::entity::ai::brain::BrainTick;

#[must_use]
pub fn create(
    target_position_getter: PositionGetter,
    should_run: fn(&mut BrainTick<'_>) -> bool,
    close_enough: i32,
    too_far: i32,
    speed_modifier: f32,
) -> OneShot {
    OneShot::new(
        "StayCloseToTarget",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
        ],
        move |tick| {
            let Some(target) = target_position_getter(tick) else {
                return false;
            };
            if !should_run(tick)
                || target
                    .current_position()
                    .squared_distance_to_vec(&tick.mob.get_entity().pos.load())
                    < f64::from(too_far * too_far)
            {
                return false;
            }
            utils::set_walk_and_look_target_memories(
                tick.state,
                target,
                speed_modifier,
                close_enough,
            );
            true
        },
    )
}

use std::sync::Arc;

use pumpkin_util::math::rotate_if_necessary;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn back_up_if_too_close(too_close_distance: i32, strafe_speed: f32) -> OneShot {
    let too_close_squared = f64::from(too_close_distance * too_close_distance);
    OneShot::with_required(
        "BackUpIfTooClose",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        vec![types::LOOK_TARGET.id()],
        move |tick| {
            let body_pos = tick.mob.get_entity().pos.load();
            let target = {
                let ctx = tick.visibility();
                let Some(target) = ctx.brain.get(types::ATTACK_TARGET) else {
                    return false;
                };
                let too_close = target
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_pos)
                    < too_close_squared;
                let visible = ctx
                    .brain
                    .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                    .is_some_and(|visible| visible.contains(target.as_ref(), &ctx));
                (too_close && visible).then(|| Arc::clone(target))
            };
            let Some(target) = target else {
                return false;
            };

            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
            );
            let mob_entity = tick.mob.get_mob_entity();
            mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .strafe(-strafe_speed, 0.0);
            let entity = tick.mob.get_entity();
            entity.set_rotation(
                rotate_if_necessary(entity.yaw.load(), entity.head_yaw.load(), 0.0),
                entity.pitch.load(),
            );
            true
        },
    )
}

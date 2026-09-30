use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::EntityBase;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;
use super::utils::has_passenger;

#[must_use]
pub fn set_entity_look_target(
    predicate: impl Fn(&Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
    max_dist: f32,
) -> OneShot {
    let max_dist_squared = f64::from(max_dist * max_dist);
    OneShot::new(
        "SetEntityLookTarget",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let body_pos = tick.mob.get_entity().pos.load();
            let target = {
                let ctx = tick.visibility();
                let Some(visible) = ctx.brain.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
                    return false;
                };
                visible.find_closest(&ctx, |entity| {
                    predicate(entity)
                        && entity
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&body_pos)
                            <= max_dist_squared
                        && !has_passenger(ctx.mob.get_entity(), entity.as_ref())
                })
            };
            let Some(target) = target else {
                return false;
            };
            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
            );
            true
        },
    )
}

#[must_use]
pub fn set_entity_look_target_of_type(entity_type: &'static EntityType, max_dist: f32) -> OneShot {
    set_entity_look_target(
        move |entity| entity.get_entity().entity_type == entity_type,
        max_dist,
    )
}

#[must_use]
pub fn set_entity_look_target_any(max_dist: f32) -> OneShot {
    set_entity_look_target(|_| true, max_dist)
}

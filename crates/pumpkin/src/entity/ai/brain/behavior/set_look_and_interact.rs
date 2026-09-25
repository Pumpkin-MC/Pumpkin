use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn set_look_and_interact(entity_type: &'static EntityType, interaction_range: i32) -> OneShot {
    let range_squared = f64::from(interaction_range * interaction_range);
    OneShot::with_required(
        "SetLookAndInteract",
        vec![
            (types::INTERACTION_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        vec![types::LOOK_TARGET.id()],
        move |tick| {
            let body_pos = tick.mob.get_entity().pos.load();
            let closest = {
                let ctx = tick.visibility();
                let Some(visible) = ctx.brain.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
                    return false;
                };
                visible.find_closest(&ctx, |entity| {
                    entity
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&body_pos)
                        <= range_squared
                        && entity.get_entity().entity_type == entity_type
                })
            };
            let Some(target) = closest else {
                return false;
            };
            tick.brain
                .set(types::INTERACTION_TARGET, Arc::clone(&target));
            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
            );
            true
        },
    )
}

use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;

#[must_use]
pub fn create(entity_type: &'static EntityType, interaction_range: i32) -> OneShot {
    let interaction_range_sqr = f64::from(interaction_range * interaction_range);
    OneShot::new(
        "SetLookAndInteract",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::INTERACTION_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let position = tick.mob.get_entity().pos.load();
            let closest = {
                let ctx = tick.visibility();
                tick.state
                    .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                    .and_then(|entities| {
                        entities.find_closest(&ctx, |entity| {
                            entity.get_entity().entity_type == entity_type
                                && entity
                                    .get_entity()
                                    .pos
                                    .load()
                                    .squared_distance_to_vec(&position)
                                    <= interaction_range_sqr
                        })
                    })
            };
            let Some(closest) = closest else {
                return false;
            };
            tick.state.set(types::INTERACTION_TARGET, closest.clone());
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(closest, true)) as Arc<dyn PositionTracker>,
            );
            true
        },
    )
}

use std::sync::Arc;

use pumpkin_data::entity::{EntityType, MobCategory};

use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use crate::entity::EntityBase;

#[must_use]
pub fn for_type(entity_type: &'static EntityType, max_dist: f32) -> OneShot {
    create(
        move |entity: &Arc<dyn EntityBase>| entity.get_entity().entity_type == entity_type,
        max_dist,
    )
}

#[must_use]
pub fn for_category(category: &'static MobCategory, max_dist: f32) -> OneShot {
    create(
        move |entity: &Arc<dyn EntityBase>| entity.get_entity().entity_type.category == category,
        max_dist,
    )
}

fn create(
    predicate: impl Fn(&Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
    max_dist: f32,
) -> OneShot {
    let max_dist_sqr = f64::from(max_dist * max_dist);
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
            let position = tick.mob.get_entity().pos.load();
            let target = {
                let ctx = tick.visibility();
                tick.state
                    .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                    .and_then(|entities| {
                        entities.find_closest(&ctx, |entity| {
                            predicate(entity)
                                && entity
                                    .get_entity()
                                    .pos
                                    .load()
                                    .squared_distance_to_vec(&position)
                                    <= max_dist_sqr
                                && !tick.mob.has_passenger(entity.as_ref())
                        })
                    })
            };
            let Some(target) = target else {
                return false;
            };
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
            );
            true
        },
    )
}

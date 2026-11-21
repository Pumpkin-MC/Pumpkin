use std::sync::Arc;

use pumpkin_data::attributes::Attributes;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, NearestVisibleLivingEntities, types};
use super::{Sensor, is_living_alive};
use crate::entity::EntityBase;

const REQUIRES: [MemoryModuleId; 2] = [
    types::NEAREST_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
];

pub struct NearestLivingEntitySensor;

impl Sensor for NearestLivingEntitySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        let body = &living.entity;
        let follow_range = living.get_attribute_value(&Attributes::FOLLOW_RANGE);
        let search = body
            .bounding_box
            .load()
            .expand(follow_range, follow_range, follow_range);
        let position = body.pos.load();
        let mut nearby: Vec<Arc<dyn EntityBase>> = tick
            .world
            .get_all_at_box(&search)
            .into_iter()
            .filter(|entity| {
                entity.get_entity().entity_id != body.entity_id && is_living_alive(entity.as_ref())
            })
            .collect();
        nearby.sort_by(|a, b| {
            position
                .squared_distance_to_vec(&a.get_entity().pos.load())
                .total_cmp(&position.squared_distance_to_vec(&b.get_entity().pos.load()))
        });
        tick.state
            .set(types::NEAREST_LIVING_ENTITIES, nearby.clone());
        tick.state.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(nearby),
        );
    }
}

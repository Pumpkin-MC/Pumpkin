use crate::entity::EntityBase;
use pumpkin_data::attributes::Attributes;
use std::sync::Arc;

use super::Sensor;
use crate::entity::ai::brain::{
    BrainTick,
    memory::{MemoryModuleId, NearestVisibleLivingEntities, types},
};

pub struct NearestLivingEntitySensor;
const REQUIRED_MEMORIES: [MemoryModuleId; 2] = [
    types::NEAREST_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
];

impl Sensor for NearestLivingEntitySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRED_MEMORIES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let range = living.get_attribute_value(&Attributes::FOLLOW_RANGE);
        let bounds = entity.bounding_box.load().expand_all(range);
        let position = entity.pos.load();
        let mut nearby = tick.world.get_entities_at_box(&bounds);
        nearby.extend(
            tick.world
                .get_players_at_box(&bounds)
                .into_iter()
                .map(|player| player as Arc<dyn EntityBase>),
        );
        nearby.retain(|other| {
            other.get_entity().entity_id != entity.entity_id
                && other
                    .get_living_entity()
                    .is_some_and(|living| living.health.load() > 0.0)
                && other.get_entity().is_alive()
        });
        nearby.sort_by(|left, right| {
            position
                .squared_distance_to_vec(&left.get_entity().pos.load())
                .total_cmp(&position.squared_distance_to_vec(&right.get_entity().pos.load()))
        });
        tick.brain
            .set(types::NEAREST_LIVING_ENTITIES, nearby.clone());
        tick.brain.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(nearby),
        );
    }
}

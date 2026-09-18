use pumpkin_data::attributes::Attributes;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, NearestVisibleLivingEntities, types};
use super::{Sensor, entities_in_inflated_box};

const REQUIRES: &[MemoryModuleId] = &[
    types::NEAREST_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
];

pub struct NearestLivingEntitySensor;

impl Sensor for NearestLivingEntitySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let follow_range = tick
            .mob
            .get_mob_entity()
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        let nearby = entities_in_inflated_box(
            tick.world,
            tick.mob.get_entity(),
            follow_range,
            follow_range,
            follow_range,
            |entity| {
                entity.get_living_entity().is_some()
                    && super::super::behavior::utils::is_alive(entity.as_ref())
            },
        );

        tick.brain
            .set(types::NEAREST_LIVING_ENTITIES, nearby.clone());
        tick.brain.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(nearby),
        );
    }
}

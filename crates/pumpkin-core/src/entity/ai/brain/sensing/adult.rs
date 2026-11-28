use pumpkin_data::tag::Taggable;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;
use crate::entity::living::LivingEntity;

const REQUIRES: [MemoryModuleId; 2] = [
    types::NEAREST_VISIBLE_ADULT.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
];

/// The nearest visible adult of the mob's own type.
pub struct AdultSensor;

impl Sensor for AdultSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(visible) = tick.state.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
            return;
        };
        let entity_type = tick.mob.get_entity().entity_type;
        let adult = visible.find_closest(&tick.visibility(), |entity| {
            entity.get_entity().entity_type == entity_type
                && !entity
                    .get_living_entity()
                    .is_some_and(LivingEntity::is_baby)
        });
        tick.state.set_optional(types::NEAREST_VISIBLE_ADULT, adult);
    }
}

/// The nearest visible adult mob that matches `minecraft:followable_friendly_mobs`.
pub struct AdultSensorAnyType;

impl Sensor for AdultSensorAnyType {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(visible) = tick.state.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
            return;
        };
        let adult = visible.find_closest(&tick.visibility(), |entity| {
            entity
                .get_entity()
                .entity_type
                .has_tag(&pumpkin_data::tag::EntityType::MINECRAFT_FOLLOWABLE_FRIENDLY_MOBS)
                && !entity
                    .get_living_entity()
                    .is_some_and(LivingEntity::is_baby)
        });
        tick.state.set_optional(types::NEAREST_VISIBLE_ADULT, adult);
    }
}

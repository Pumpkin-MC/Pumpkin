use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::EntityBase;
use crate::entity::mob::piglin_ai;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, NearestVisibleLivingEntities, types};
use super::Sensor;

const REQUIRES: &[MemoryModuleId] = &[
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_NEMESIS.id(),
    types::NEARBY_ADULT_PIGLINS.id(),
];

pub struct PiglinBruteSpecificSensor;

impl Sensor for PiglinBruteSpecificSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let nemesis = {
            let ctx = tick.visibility();
            let empty = NearestVisibleLivingEntities::empty();
            let visible = ctx
                .brain
                .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                .unwrap_or(&empty);
            visible.find_closest(&ctx, |entity: &Arc<dyn EntityBase>| {
                let entity_type = entity.get_entity().entity_type;
                entity_type == &EntityType::WITHER_SKELETON || entity_type == &EntityType::WITHER
            })
        };
        let nearby_adults = piglin_ai::find_nearby_adult_piglins(tick.brain);

        tick.brain
            .set_optional(types::NEAREST_VISIBLE_NEMESIS, nemesis);
        tick.brain.set(types::NEARBY_ADULT_PIGLINS, nearby_adults);
    }
}

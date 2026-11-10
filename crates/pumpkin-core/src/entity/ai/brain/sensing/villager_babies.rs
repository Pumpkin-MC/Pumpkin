use pumpkin_data::entity::EntityType;

use crate::entity::living::LivingEntity;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 1] = [types::VISIBLE_VILLAGER_BABIES.id()];

pub struct VillagerBabiesSensor;

impl Sensor for VillagerBabiesSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let babies = tick
            .state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
            .map(|visible| {
                visible
                    .find_all(&tick.visibility(), |entity| {
                        entity.get_entity().entity_type == &EntityType::VILLAGER
                            && entity
                                .get_living_entity()
                                .is_some_and(LivingEntity::is_baby)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        tick.state.set(types::VISIBLE_VILLAGER_BABIES, babies);
    }
}

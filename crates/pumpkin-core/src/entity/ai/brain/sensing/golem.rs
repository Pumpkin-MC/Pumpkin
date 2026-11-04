use pumpkin_data::entity::EntityType;

use super::super::BrainState;
use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 2] = [
    types::NEAREST_LIVING_ENTITIES.id(),
    types::GOLEM_DETECTED_RECENTLY.id(),
];

const MEMORY_TIME_TO_LIVE: i64 = 599;

pub fn golem_detected(state: &mut BrainState) {
    state.set_with_expiry(types::GOLEM_DETECTED_RECENTLY, true, MEMORY_TIME_TO_LIVE);
}

pub struct GolemSensor;

impl Sensor for GolemSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(living_entities) = tick.state.get(types::NEAREST_LIVING_ENTITIES) else {
            return;
        };
        let golem_present = living_entities
            .iter()
            .any(|entity| entity.get_entity().entity_type == &EntityType::IRON_GOLEM);
        if golem_present {
            golem_detected(tick.state);
        }
    }
}

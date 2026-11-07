use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;

const REQUIRES: [MemoryModuleId; 1] = [types::IS_IN_WATER.id()];

pub struct IsInWaterSensor;

impl Sensor for IsInWaterSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        if tick.mob.get_entity().is_in_water() {
            tick.state.set(types::IS_IN_WATER, ());
        } else {
            tick.state.erase(types::IS_IN_WATER.id());
        }
    }
}

use super::super::BrainTick;
use super::super::memory::MemoryModuleId;
use super::Sensor;

pub struct DummySensor;

impl Sensor for DummySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &[]
    }

    fn do_tick(&mut self, _tick: &mut BrainTick<'_>) {}
}

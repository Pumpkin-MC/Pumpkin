use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;
use crate::entity::passive::armadillo::{ArmadilloEntity, SCARE_CHECK_INTERVAL};

const REQUIRES: [MemoryModuleId; 1] = [types::NEAREST_LIVING_ENTITIES.id()];

/// Vanilla `MobSensor(5, Armadillo::isScaredBy, Armadillo::canStayRolledUp, DANGER_DETECTED_RECENTLY, 80)`.
pub struct ArmadilloScareDetectedSensor;

impl Sensor for ArmadilloScareDetectedSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(armadillo) = tick.mob.cast_any().downcast_ref::<ArmadilloEntity>() else {
            return;
        };
        // The panic state comes from `tick.state`: the brain is already borrowed for this tick.
        let panicking = tick.state.has_memory_value(types::IS_PANICKING.id());
        if !armadillo.can_stay_rolled_up_while(panicking) {
            tick.state.erase(types::DANGER_DETECTED_RECENTLY.id());
            return;
        }
        let scared = tick
            .state
            .get(types::NEAREST_LIVING_ENTITIES)
            .is_some_and(|entities| {
                entities
                    .iter()
                    .any(|entity| armadillo.is_scared_by(entity.as_ref()))
            });
        if scared {
            tick.state.set_with_expiry(
                types::DANGER_DETECTED_RECENTLY,
                true,
                i64::from(SCARE_CHECK_INTERVAL),
            );
        }
    }
}

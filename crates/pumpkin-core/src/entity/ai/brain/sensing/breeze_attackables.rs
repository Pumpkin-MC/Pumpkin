use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{NearestLivingEntitySensor, Sensor, is_entity_attackable};
use crate::entity::predicate::EntityPredicate;

const REQUIRES: &[MemoryModuleId] = &[
    types::NEAREST_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_ATTACKABLE.id(),
];

pub(super) struct BreezeAttackEntitySensor;

impl Sensor for BreezeAttackEntitySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        NearestLivingEntitySensor.do_tick(tick);
        let ctx = tick.visibility();
        let nearest = tick
            .state
            .get(types::NEAREST_LIVING_ENTITIES)
            .and_then(|entities| {
                entities.iter().find(|entity| {
                    EntityPredicate::ExceptCreativeOrSpectator.test(entity.get_entity())
                        && is_entity_attackable(&ctx, entity.as_ref())
                })
            })
            .cloned();
        tick.state.set_optional(types::NEAREST_ATTACKABLE, nearest);
    }
}

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, is_entity_attackable};
use crate::entity::passive::frog;

const REQUIRES: [MemoryModuleId; 3] = [
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_ATTACKABLE.id(),
    types::UNREACHABLE_TONGUE_TARGETS.id(),
];

pub const TARGET_DETECTION_DISTANCE: f64 = 10.0;

/// The nearest visible mob a frog can eat and has not recently failed to reach.
pub struct FrogAttackablesSensor;

impl Sensor for FrogAttackablesSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let position = tick.mob.get_entity().pos.load();
        let ctx = tick.visibility();
        let target = tick
            .state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
            .and_then(|visible| {
                visible.find_closest(&ctx, |entity| {
                    is_entity_attackable(&ctx, entity.as_ref())
                        && frog::can_eat(entity.as_ref())
                        && !tick
                            .state
                            .get(types::UNREACHABLE_TONGUE_TARGETS)
                            .is_some_and(|targets| {
                                targets.contains(&entity.get_entity().entity_uuid)
                            })
                        && entity
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&position)
                            < TARGET_DETECTION_DISTANCE * TARGET_DETECTION_DISTANCE
                })
            });
        tick.state.set_optional(types::NEAREST_ATTACKABLE, target);
    }
}

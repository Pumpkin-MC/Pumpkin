use super::super::OneShot;
use crate::entity::ai::brain::memory::{MemoryModuleId, MemoryStatus, types};

const SAFE_DISTANCE_FROM_DANGER: i32 = 36;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 3] = [
    (types::HURT_BY.id(), MemoryStatus::Registered),
    (types::HURT_BY_ENTITY.id(), MemoryStatus::Registered),
    (types::NEAREST_HOSTILE.id(), MemoryStatus::Registered),
];

#[must_use]
pub fn create() -> OneShot {
    OneShot::new("VillagerCalmDown", ENTRY_CONDITIONS.to_vec(), |tick| {
        let body_position = tick.mob.get_entity().pos.load();
        let feel_scared = tick.state.get(types::HURT_BY).is_some()
            || tick.state.get(types::NEAREST_HOSTILE).is_some()
            || tick.state.get(types::HURT_BY_ENTITY).is_some_and(|entity| {
                entity
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_position)
                    <= f64::from(SAFE_DISTANCE_FROM_DANGER)
            });
        if !feel_scared {
            tick.state.erase(types::HURT_BY.id());
            tick.state.erase(types::HURT_BY_ENTITY.id());
            let pos = tick.mob.get_entity().block_pos.load();
            tick.state
                .update_activity_from_schedule(tick.world, tick.time, &pos);
        }
        true
    })
}

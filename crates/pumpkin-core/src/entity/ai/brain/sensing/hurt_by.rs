use std::sync::Arc;

use super::Sensor;
use crate::entity::ai::brain::{
    BrainTick,
    memory::{MemoryModuleId, types, value::DamageSourceMemory},
};

pub struct HurtBySensor;
const REQUIRED_MEMORIES: [MemoryModuleId; 2] = [types::HURT_BY.id(), types::HURT_BY_ENTITY.id()];

impl Sensor for HurtBySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRED_MEMORIES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        if let Some(damage_type) = living.get_last_damage_type() {
            let attacker = living.get_last_damage_attacker();
            if let Some(attacker) = attacker
                .as_ref()
                .filter(|entity| entity.get_living_entity().is_some())
            {
                tick.brain.set(types::HURT_BY_ENTITY, attacker.clone());
            }
            tick.brain.set(
                types::HURT_BY,
                DamageSourceMemory {
                    damage_type,
                    attacker,
                },
            );
        } else {
            tick.brain.erase(types::HURT_BY.id());
        }
        if tick
            .brain
            .get(types::HURT_BY_ENTITY)
            .is_some_and(|attacker| {
                !attacker.get_entity().is_alive()
                    || tick
                        .world
                        .get_entity_by_id(attacker.get_entity().entity_id)
                        .is_none()
                    || attacker
                        .get_living_entity()
                        .is_some_and(|living| living.health.load() <= 0.0)
                    || !Arc::ptr_eq(&attacker.get_entity().world.load_full(), tick.world)
            })
        {
            tick.brain.erase(types::HURT_BY_ENTITY.id());
        }
    }
}

use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::{DamageSourceMemory, MemoryModuleId, types};
use super::Sensor;

const REQUIRES: &[MemoryModuleId] = &[types::HURT_BY.id(), types::HURT_BY_ENTITY.id()];

pub struct HurtBySensor;

impl Sensor for HurtBySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        if let Some(damage_type) = living.get_last_damage_type() {
            let attacker = tick
                .world
                .get_entity_by_id(
                    living
                        .last_attacker_id
                        .load(std::sync::atomic::Ordering::Relaxed),
                )
                .filter(|attacker| attacker.get_living_entity().is_some());
            tick.brain.set(
                types::HURT_BY,
                DamageSourceMemory {
                    damage_type,
                    attacker: attacker.clone(),
                },
            );
            if let Some(attacker) = attacker {
                tick.brain.set(types::HURT_BY_ENTITY, attacker);
            }
        } else {
            tick.brain.erase(types::HURT_BY.id());
        }

        let departed = tick
            .brain
            .get(types::HURT_BY_ENTITY)
            .is_some_and(|attacker| {
                !super::super::behavior::utils::is_alive(attacker.as_ref())
                    || !Arc::ptr_eq(&attacker.get_entity().world.load_full(), tick.world)
            });
        if departed {
            tick.brain.erase(types::HURT_BY_ENTITY.id());
        }
    }
}

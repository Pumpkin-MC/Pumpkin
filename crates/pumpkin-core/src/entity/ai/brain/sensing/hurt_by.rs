use super::super::BrainTick;
use super::super::memory::value::DamageSourceMemory;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, is_living_alive};

const REQUIRES: [MemoryModuleId; 2] = [types::HURT_BY.id(), types::HURT_BY_ENTITY.id()];

pub struct HurtBySensor;

impl Sensor for HurtBySensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        if let Some((damage_type, attacker_id)) = living.get_last_damage_source() {
            let attacker = (attacker_id != 0)
                .then(|| tick.world.get_entity_by_id(attacker_id))
                .flatten();
            tick.state.set(
                types::HURT_BY,
                DamageSourceMemory {
                    damage_type,
                    attacker: attacker.clone(),
                },
            );
            if let Some(attacker) =
                attacker.filter(|attacker| attacker.get_living_entity().is_some())
            {
                tick.state.set(types::HURT_BY_ENTITY, attacker);
            }
        } else {
            tick.state.erase(types::HURT_BY.id());
        }

        let stale = tick
            .state
            .get(types::HURT_BY_ENTITY)
            .is_some_and(|attacker| {
                !is_living_alive(attacker.as_ref())
                    || attacker.get_entity().world.load().uuid != tick.world.uuid
            });
        if stale {
            tick.state.erase(types::HURT_BY_ENTITY.id());
        }
    }
}

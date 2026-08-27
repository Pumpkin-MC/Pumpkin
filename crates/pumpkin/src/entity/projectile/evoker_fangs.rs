use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::Sound;

use crate::{
    entity::{Entity, EntityBase},
    server::Server,
};

pub struct EvokerFangsEntity {
    pub entity: Entity,
    pub warmup_ticks: AtomicU32,
    pub life_ticks: AtomicU32,
    pub owner_id: Option<i32>,
    pub has_bitten: AtomicBool,
}

impl EvokerFangsEntity {
    #[must_use]
    pub fn new(entity: Entity, warmup_ticks: u32, yaw: f32, owner_id: Option<i32>) -> Self {
        entity.set_rotation(yaw, 0.0);
        Self {
            entity,
            warmup_ticks: AtomicU32::new(warmup_ticks),
            life_ticks: AtomicU32::new(0),
            owner_id,
            has_bitten: AtomicBool::new(false),
        }
    }
}

impl EntityBase for EvokerFangsEntity {
    fn write_custom_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_int("Warmup", self.warmup_ticks.load(Ordering::Relaxed) as i32);
    }

    fn read_custom_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if let Some(warmup) = nbt.get_int("Warmup") {
            self.warmup_ticks.store(warmup as u32, Ordering::Relaxed);
        }
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let world = entity.world.load();

        let warmup = self.warmup_ticks.load(Ordering::Relaxed);
        let life = self.life_ticks.fetch_add(1, Ordering::Relaxed) + 1;

        if life >= warmup {
            if !self.has_bitten.swap(true, Ordering::SeqCst) {
                entity.play_sound(Sound::EntityEvokerFangsAttack);

                let bb = entity.bounding_box.load().expand(0.2, 0.2, 0.2);
                let candidates = world.get_entities_at_box(&bb);

                for cand in candidates {
                    let cand_ent = cand.get_entity();
                    if Some(cand_ent.entity_id) == self.owner_id {
                        continue;
                    }

                    if cand_ent.entity_id != entity.entity_id {
                        let _ = cand.damage(cand.as_ref(), 6.0, DamageType::MAGIC);
                    }
                }
            }

            if life > warmup + 20 {
                entity.remove();
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

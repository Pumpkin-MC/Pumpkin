use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::Sound;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NbtFuture},
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
    fn write_custom_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_int("Warmup", self.warmup_ticks.load(Ordering::Relaxed) as i32);
        })
    }

    fn read_custom_nbt<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(warmup) = nbt.get_int("Warmup") {
                self.warmup_ticks.store(warmup as u32, Ordering::Relaxed);
            }
        })
    }

    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
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
                            let cand_clone = cand.clone();
                            tokio::spawn(async move {
                                let _ = cand_clone
                                    .damage(cand_clone.as_ref(), 6.0, DamageType::MAGIC)
                                    .await;
                            });
                        }
                    }
                }

                if life > warmup + 20 {
                    entity.remove().await;
                }
            }
        })
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

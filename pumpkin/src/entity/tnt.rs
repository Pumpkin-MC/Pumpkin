use super::{ArcEntityBaseFuture, Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::world::WorldExplosionArgs;
use crate::world::damage_source::DamageSource;
use crate::world::explosion::ExplosionInteraction;
use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::{Block, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
use pumpkin_util::math::vector3::Vector3;
use std::{
    f64::consts::TAU,
    sync::{
        Arc,
        atomic::{
            AtomicU32,
            Ordering::{self, Relaxed},
        },
    },
};

pub struct TNTEntity {
    entity: Entity,
    power: f32,
    fuse: AtomicU32,
}

impl TNTEntity {
    pub const fn new(entity: Entity, power: f32, fuse: u32) -> Self {
        Self {
            entity,
            power,
            fuse: AtomicU32::new(fuse),
        }
    }
}

impl NBTStorage for TNTEntity {}

impl EntityBase for TNTEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            let original_velo = entity.velocity.load();

            let mut velo = original_velo;
            velo.y -= self.get_gravity();

            entity.move_entity(caller.clone(), velo).await;
            entity.tick_block_collisions(&caller, server).await;
            entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));

            if entity.on_ground.load(Ordering::Relaxed) {
                entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
            }

            if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
                entity.send_pos_rot().await;
                entity.send_velocity().await;
            }

            // FIX: Prevent fuse underflow (vanilla parity)
            let fuse = self.fuse.load(Relaxed);

            if fuse <= 1 {
                // TNT explodes now
                if let Some(entity) = self
                    .get_entity()
                    .world
                    .load()
                    .get_entity_by_id(self.get_entity().entity_id)
                {
                    entity.clone().explode(entity.get_entity().pos.load()).await;
                }
            } else {
                // Safe decrement
                self.fuse.store(fuse - 1, Relaxed);
                entity.update_fluid_state(&caller).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let pos: f64 = rand::random::<f64>() * TAU;

            self.entity
                .set_velocity(Vector3::new(-pos.sin() * 0.02, 0.2, -pos.cos() * 0.02))
                .await;

            self.entity
                .send_meta_data(&[
                    Metadata::new(
                        TrackedData::DATA_FUSE,
                        MetaDataType::INTEGER,
                        VarInt(self.fuse.load(Relaxed) as i32),
                    ),
                    Metadata::new(
                        TrackedData::DATA_BLOCK_STATE,
                        MetaDataType::BLOCK_STATE,
                        VarInt(i32::from(Block::TNT.default_state.id)),
                    ),
                ])
                .await;
        })
    }

    fn explode(self: Arc<Self>, position: Vector3<f64>) -> ArcEntityBaseFuture<()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            world
                .explode_with(WorldExplosionArgs {
                    source_entity: Some(self.clone()),
                    damage_source: Some(DamageSource::explosion_from_direct(
                        &world,
                        Some(self.clone()),
                    )),
                    damage_calculator: None,
                    power: self.power,
                    pos: position,
                    fire: false,
                    explosion_interaction: ExplosionInteraction::Tnt,
                    small_particle: Particle::Explosion,
                    large_particle: Particle::ExplosionEmitter,
                    sound: Sound::EntityGenericExplode,
                })
                .await;
            self.entity.remove().await;
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }
}

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::{
    entity::EntityBaseFuture,
    server::Server,
    world::{
        explosion::Explosion,
        explosion_behavior::{EntityBasedExplosion, ExplosionInteraction},
    },
};
use core::f32;
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

const EXPLOSION_Y_OFFSET: f64 = 0.0625;

pub struct TNTEntity {
    entity: Entity,
    power: f32,
    fuse: AtomicU32,
    owner_id: Option<i32>,
}

impl TNTEntity {
    pub const fn new(entity: Entity, power: f32, fuse: u32, owner: Option<&Entity>) -> Self {
        Self {
            entity,
            power,
            fuse: AtomicU32::new(fuse),
            owner_id: match owner {
                Some(e) => Some(e.entity_id),
                None => None,
            },
        }
    }
}

impl NBTStorage for TNTEntity {}

impl EntityBase for TNTEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            let original_velo = entity.velocity.load();

            let mut velo = original_velo;
            velo.y -= self.get_gravity();

            entity.move_entity(caller, velo).await;
            entity.tick_block_collisions(caller, server).await;
            entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));

            if entity.on_ground.load(Ordering::Relaxed) {
                entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
            }

            if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
                entity.send_pos_rot();
                entity.send_velocity();
            }

            // FIX: Prevent fuse underflow (vanilla parity)
            let fuse = self.fuse.load(Relaxed);

            if fuse <= 1 {
                // TNT explodes now
                let world = self.entity.world.load();
                let height = self.entity.entity_dimension.load().height as f64;
                let pos =
                    self.entity.pos.load() + Vector3::new(0., height * EXPLOSION_Y_OFFSET, 0.);
                let explosion = Explosion {
                    source_id: Some(self.entity.entity_id),
                    cause_id: self.owner_id,
                    behavior: Arc::new(EntityBasedExplosion),
                    block_interaction: ExplosionInteraction::TNT.resolve(&world),
                    ..Explosion::new(self.power, pos)
                };

                world.explode(&explosion).await;
                self.entity.remove().await;
                // TODO: Prevent nether portals from TNT explosions after teleportation
            } else {
                // Safe decrement
                self.fuse.store(fuse - 1, Relaxed);
                entity.update_fluid_state(caller).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            let pos: f64 = rand::random::<f64>() * TAU;

            self.entity
                .set_velocity(Vector3::new(-pos.sin() * 0.02, 0.2, -pos.cos() * 0.02));

            self.entity.send_meta_data(&[
                Metadata::new(
                    TrackedData::FUSE_ID,
                    MetaDataType::INTEGER,
                    VarInt(self.fuse.load(Relaxed) as i32),
                ),
                Metadata::new(
                    TrackedData::BLOCK_STATE_ID,
                    MetaDataType::BLOCK_STATE,
                    VarInt(i32::from(Block::TNT.default_state.id)),
                ),
            ]);
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

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }
}

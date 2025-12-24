use super::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::Block;
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::client::play::{MetaDataType, Metadata},
};
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
    pub fn new(entity: Entity, power: f32, fuse: u32) -> Self {
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
            let velocity_dirty = entity.velocity_dirty.swap(false, Ordering::SeqCst);

            if velocity_dirty {
                entity.send_pos_rot().await;

                entity.send_velocity().await;
            }

            let fuse = self.fuse.fetch_sub(1, Relaxed);
            if fuse == 0 {
                self.entity.remove().await;
                self.entity
                    .world
                    .explode(self.entity.pos.load(), self.power)
                    .await;
            } else {
                entity.update_fluid_state(&caller).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            // TODO: Yes, this is the wrong function, but we need to send this after spawning the entity.
            let pos: f64 = rand::random::<f64>() * TAU;

            self.entity
                .set_velocity(Vector3::new(-pos.sin() * 0.02, 0.2, -pos.cos() * 0.02))
                .await;
            // We can merge multiple `Metadata`s into one meta packet.
            self.entity
                .send_meta_data(&[
                    Metadata::new(
                        8,
                        MetaDataType::Integer,
                        VarInt(self.fuse.load(Relaxed) as i32),
                    ),
                    Metadata::new(
                        9,
                        MetaDataType::BlockState,
                        VarInt(i32::from(Block::TNT.default_state.id)),
                    ),
                ])
                .await;
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
}

use super::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use async_trait::async_trait;
use core::f32;
use pumpkin_data::{Block, damage::DamageType};
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

#[async_trait]
impl EntityBase for TNTEntity {
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
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
                .read()
                .await
                .explode(server, self.entity.pos.load(), self.power)
                .await;
        } else {
            entity.update_fluid_state(&caller).await;
        }
    }

    async fn init_data_tracker(&self) {
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
    }

    async fn damage_with_context(
        &self,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        false
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
}

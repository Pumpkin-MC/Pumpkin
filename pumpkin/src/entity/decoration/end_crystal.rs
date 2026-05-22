use core::f32;
use std::sync::Arc;

use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity},
    world::{
        explosion::Explosion,
        explosion_behavior::{EntityBasedExplosion, ExplosionInteraction},
    },
};
use pumpkin_data::{
    damage::DamageType,
    meta_data_type::MetaDataType,
    tag::{self, Taggable},
    tracked_data::TrackedData,
};
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

pub struct EndCrystalEntity {
    entity: Entity,
}

impl EndCrystalEntity {
    pub const fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl EndCrystalEntity {
    pub fn set_show_bottom(&self, show_bottom: bool) {
        self.entity.send_meta_data(&[Metadata::new(
            TrackedData::SHOW_BOTTOM,
            MetaDataType::BOOLEAN,
            show_bottom,
        )]);
    }
}

impl NBTStorage for EndCrystalEntity {}

impl EntityBase for EndCrystalEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if !damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION) {
                let world = self.entity.world.load();
                let explosion = Explosion {
                    source_id: Some(self.entity.entity_id),
                    cause_id: cause.map(|e| e.get_entity().entity_id),
                    behavior: Arc::new(EntityBasedExplosion),
                    block_interaction: ExplosionInteraction::Block.resolve(&world),
                    ..Explosion::new(6.0, self.entity.pos.load())
                };

                world.explode(&explosion).await;
            }

            self.entity.remove().await;

            // TODO
            true
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

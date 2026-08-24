use core::f32;

use crate::entity::{Entity, EntityBase, EntityBaseFuture, living::LivingEntity};
use pumpkin_data::{
    damage::DamageType,
    tag::{self, Taggable},
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
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::end_crystal::SHOW_BOTTOM,
                show_bottom,
            )],
            None,
        );
    }
}

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
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.entity.remove().await;
            if !damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION) {
                self.entity
                    .world
                    .load()
                    .explode(
                        self.entity.pos.load(),
                        6.0,
                        crate::world::ExplosionInteraction::Block,
                    )
                    .await;
            }

            // TODO
            true
        })
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

use core::f32;

use crate::entity::{Entity, EntityBase, living::LivingEntity};
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

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.entity.remove();
        if !damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION) {
            let world = self.entity.world.load();
            let pos = self.entity.pos.load();
            world.explode(pos, 6.0, crate::world::ExplosionInteraction::Block);
        }

        // TODO
        true
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

use core::f32;

use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity};
use pumpkin_data::{damage::DamageType, meta_data_type::MetaDataType, tracked_data::TrackedData};
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
                TrackedData::SHOW_BOTTOM,
                MetaDataType::BOOLEAN,
                show_bottom,
            )],
            None,
        );
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

    fn is_pickable(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if source.is_some_and(|source| {
                source.get_entity().entity_type == &pumpkin_data::entity::EntityType::ENDER_DRAGON
            }) {
                return false;
            }

            self.entity.remove().await;
            if !is_explosion_damage(&damage_type) {
                self.entity
                    .world
                    .load()
                    .explode(self.entity.pos.load(), 6.0)
                    .await;
            }

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

const fn is_explosion_damage(damage_type: &DamageType) -> bool {
    damage_type.id == DamageType::FIREWORKS.id
        || damage_type.id == DamageType::EXPLOSION.id
        || damage_type.id == DamageType::PLAYER_EXPLOSION.id
        || damage_type.id == DamageType::BAD_RESPAWN_POINT.id
}

#[cfg(test)]
mod tests {
    use super::is_explosion_damage;
    use pumpkin_data::damage::DamageType;

    #[test]
    fn all_vanilla_explosion_sources_are_recognized() {
        assert!(is_explosion_damage(&DamageType::FIREWORKS));
        assert!(is_explosion_damage(&DamageType::EXPLOSION));
        assert!(is_explosion_damage(&DamageType::PLAYER_EXPLOSION));
        assert!(is_explosion_damage(&DamageType::BAD_RESPAWN_POINT));
        assert!(!is_explosion_damage(&DamageType::MAGIC));
    }
}

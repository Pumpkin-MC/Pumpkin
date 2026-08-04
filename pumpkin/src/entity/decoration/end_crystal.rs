use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity};
use pumpkin_data::{
    damage::DamageType,
    meta_data_type::MetaDataType,
    tag::{self, Taggable},
    tracked_data::TrackedData,
};
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct EndCrystalEntity {
    entity: Entity,
    pub time: std::sync::atomic::AtomicI32,
}

impl EndCrystalEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            time: std::sync::atomic::AtomicI32::new(rand::random_range(0..100000)),
        }
    }

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

    pub fn set_beam_target(&self, target: Option<BlockPos>) {
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BEAM_TARGET,
                MetaDataType::OPTIONAL_BLOCK_POS,
                target,
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

    fn can_hit(&self) -> bool {
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
            let pos = self.entity.pos.load();
            let world = self.entity.world.load();

            self.entity.remove().await;
            if !damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION) {
                world.explode(pos, 6.0).await;
            }

            if let Some(fight_mutex) = &world.dragon_fight {
                let attacker = source.and_then(crate::entity::EntityBase::get_player);
                let dragon_uuid = fight_mutex
                    .lock()
                    .await
                    .on_crystal_destroyed(&world, self.entity.entity_uuid, pos, attacker)
                    .await;

                if let Some(dragon_uuid) = dragon_uuid
                    && let Some(dragon_base) = world
                        .entities
                        .load()
                        .iter()
                        .find(|e| e.get_entity().entity_uuid == dragon_uuid)
                    && let Some(dragon) = dragon_base
                        .cast_any()
                        .downcast_ref::<crate::entity::boss::ender_dragon::EnderDragonEntity>(
                    )
                {
                    dragon
                        .crystal_destroyed(
                            dragon_base.as_ref(),
                            self.entity.entity_uuid,
                            pos,
                            attacker,
                        )
                        .await;
                }
            }

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

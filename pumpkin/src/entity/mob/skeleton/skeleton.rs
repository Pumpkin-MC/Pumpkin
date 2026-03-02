use std::sync::Arc;

use crate::entity::mob::SunSensitive;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct SkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl SkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl NBTStorage for SkeletonEntity {}

impl Mob for SkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        self.sun_sensitive_tick()
    }
}

impl SunSensitive for SkeletonEntity {}

use std::sync::Arc;

use crate::entity::mob::SunSensitive;
use crate::entity::{mob::{skeleton::SkeletonEntityBase, Mob, MobEntity}, Entity, EntityBase, EntityBaseFuture, NBTStorage};

pub struct StraySkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl StraySkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let stray = Self { entity };
        Arc::new(stray)
    }
}

impl NBTStorage for StraySkeletonEntity {}

impl Mob for StraySkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        self.sun_sensitive_tick()
    }
}

impl SunSensitive for StraySkeletonEntity {}
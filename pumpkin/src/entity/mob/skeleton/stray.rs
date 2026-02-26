use std::sync::Arc;

use crate::entity::{mob::{skeleton::SkeletonEntityBase, Mob, MobEntity}, Entity, EntityBase, EntityBaseFuture, NBTStorage};

pub struct StraySkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl StraySkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for StraySkeletonEntity {}

impl Mob for StraySkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.sunburn().await })
    }
}

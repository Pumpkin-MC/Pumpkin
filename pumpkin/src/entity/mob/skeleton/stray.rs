use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

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
}

use std::sync::Arc;

use crate::entity::{
    mob::{skeleton::SkeletonEntityBase, Mob, MobEntity}, Entity,
    NBTStorage,
};

pub struct WitherSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl WitherSkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for WitherSkeletonEntity {}

impl Mob for WitherSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

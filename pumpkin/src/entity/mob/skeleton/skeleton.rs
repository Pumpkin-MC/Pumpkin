use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct SkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl SkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for SkeletonEntity {}

impl Mob for SkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

use std::sync::Arc;

use crate::entity::{
    mob::{skeleton::SkeletonEntityBase, Mob, MobEntity}, Entity,
    NBTStorage,
};

pub struct ParchedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl ParchedSkeletonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity).await;
        let parched = Self { entity };
        Arc::new(parched)
    }
}

impl NBTStorage for ParchedSkeletonEntity {}

impl Mob for ParchedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

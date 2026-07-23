use std::sync::Arc;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct StraySkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl StraySkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let stray = Self { entity };
        Arc::new(stray)
    }
}

impl NBTStorage for StraySkeletonEntity {}

impl Mob for StraySkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        self.entity.mob_init_data_tracker()
    }
}

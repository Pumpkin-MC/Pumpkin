use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};
use std::sync::Arc;

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let bogged = Self { entity };
        Arc::new(bogged)
    }
}

impl NBTStorage for BoggedSkeletonEntity {}

impl Mob for BoggedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        self.entity.mob_init_data_tracker()
    }
}

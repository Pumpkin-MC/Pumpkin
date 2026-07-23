use std::sync::Arc;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct WitherSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl WitherSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Wither skeletons use melee swords, not bows (vanilla).
        let entity = SkeletonEntityBase::with_combat(entity, false);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl NBTStorage for WitherSkeletonEntity {}

impl Mob for WitherSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        self.entity.mob_init_data_tracker()
    }
}

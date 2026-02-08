use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

/// Bogged — a poisonous skeleton variant found in swamps and trial chambers.
///
/// Delegates to `SkeletonEntityBase` for base AI behavior.
/// Tipped arrow (poison) is a future addition.
pub struct BoggedEntity {
    skeleton: Arc<SkeletonEntityBase>,
}

impl BoggedEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let skeleton = SkeletonEntityBase::make(entity).await;
        Arc::new(Self { skeleton })
    }
}

impl NBTStorage for BoggedEntity {}

impl Mob for BoggedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.skeleton.mob_entity
    }
}

use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

/// Stray — a frozen variant of the Skeleton.
///
/// Inherits skeleton AI (player targeting, look around).
pub struct StrayEntity {
    skeleton: Arc<SkeletonEntityBase>,
}

impl StrayEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let skeleton = SkeletonEntityBase::make(entity).await;
        Arc::new(Self { skeleton })
    }
}

impl NBTStorage for StrayEntity {}

impl Mob for StrayEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.skeleton.mob_entity
    }
}

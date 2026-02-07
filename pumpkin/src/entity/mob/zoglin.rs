use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, hoglin::HoglinEntity},
};

/// Zoglin — a zombified variant of the Hoglin.
///
/// Delegates to HoglinEntity for base AI behavior.
/// Attacks all non-creeper mobs indiscriminately.
pub struct ZoglinEntity {
    hoglin: Arc<HoglinEntity>,
}

impl ZoglinEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let hoglin = HoglinEntity::new(entity).await;
        Arc::new(Self { hoglin })
    }
}

impl NBTStorage for ZoglinEntity {}

impl Mob for ZoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.hoglin.mob_entity
    }
}

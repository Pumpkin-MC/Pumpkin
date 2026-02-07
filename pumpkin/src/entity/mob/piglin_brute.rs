use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, piglin::PiglinEntity},
};

/// Piglin Brute — a stronger piglin that always attacks players.
///
/// Delegates to PiglinEntity for base AI behavior.
pub struct PiglinBruteEntity {
    piglin: Arc<PiglinEntity>,
}

impl PiglinBruteEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let piglin = PiglinEntity::new(entity).await;
        Arc::new(Self { piglin })
    }
}

impl NBTStorage for PiglinBruteEntity {}

impl Mob for PiglinBruteEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.piglin.mob_entity
    }
}

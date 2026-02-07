use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, guardian::GuardianEntity},
};

/// Elder Guardian — a larger, more powerful variant of the Guardian.
///
/// Delegates to GuardianEntity for base AI behavior.
/// Mining fatigue effect is a future addition.
pub struct ElderGuardianEntity {
    guardian: Arc<GuardianEntity>,
}

impl ElderGuardianEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let guardian = GuardianEntity::new(entity).await;
        Arc::new(Self { guardian })
    }
}

impl NBTStorage for ElderGuardianEntity {}

impl Mob for ElderGuardianEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.guardian.mob_entity
    }
}

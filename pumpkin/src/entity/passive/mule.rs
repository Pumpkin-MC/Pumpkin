use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
    passive::horse::HorseEntity,
};

/// Mule — a cross between a horse and donkey that can carry chests.
///
/// Delegates to HorseEntity for base AI behavior.
pub struct MuleEntity {
    horse: Arc<HorseEntity>,
}

impl MuleEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let horse = HorseEntity::new(entity).await;
        Arc::new(Self { horse })
    }
}

impl NBTStorage for MuleEntity {}

impl Mob for MuleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.horse.mob_entity
    }
}

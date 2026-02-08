use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
    passive::horse::HorseEntity,
};

/// Donkey — a smaller rideable animal that can carry chests.
///
/// Delegates to `HorseEntity` for base AI behavior.
pub struct DonkeyEntity {
    horse: Arc<HorseEntity>,
}

impl DonkeyEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let horse = HorseEntity::new(entity).await;
        Arc::new(Self { horse })
    }
}

impl NBTStorage for DonkeyEntity {}

impl Mob for DonkeyEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.horse.mob_entity
    }
}

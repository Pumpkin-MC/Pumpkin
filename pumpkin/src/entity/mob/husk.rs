use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, zombie::ZombieEntity},
};

/// Husk — a desert variant of the Zombie that doesn't burn in sunlight.
///
/// Inherits zombie AI (melee attack, player targeting, egg destruction).
pub struct HuskEntity {
    zombie: Arc<ZombieEntity>,
}

impl HuskEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let zombie = ZombieEntity::new(entity).await;
        Arc::new(Self { zombie })
    }
}

impl NBTStorage for HuskEntity {}

impl Mob for HuskEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.zombie.mob_entity
    }
}

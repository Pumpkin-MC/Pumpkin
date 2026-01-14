use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, zombie::ZombieEntity},
};

pub struct DrownedEntity {
    entity: Arc<ZombieEntity>,
}

impl DrownedEntity {
    pub async fn make(entity: Entity) -> Arc<Self> {
        Arc::new(Self {
            entity: ZombieEntity::make(entity).await,
        })
    }
}

impl NBTStorage for DrownedEntity {}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

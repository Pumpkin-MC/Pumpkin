use super::{Mob, MobEntity};
use crate::entity::mob::zombie::ZombieEntity;
use crate::entity::{Entity, NBTStorage};
use std::sync::Arc;

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntity>,
}

impl ZombieVillagerEntity {
    pub async fn make(entity: Entity) -> Arc<Self> {
        let mob_entity = ZombieEntity::make(entity).await;
        let zombie = Self { mob_entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for ZombieVillagerEntity {}

impl Mob for ZombieVillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity.mob_entity
    }
}

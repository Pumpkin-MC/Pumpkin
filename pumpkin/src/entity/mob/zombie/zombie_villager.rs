use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity, SunSensitive};
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage};
use std::sync::Arc;

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntityBase>,
}

impl ZombieVillagerEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = ZombieEntityBase::new(entity).await;
        let zombie = Self { mob_entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for ZombieVillagerEntity {}

impl Mob for ZombieVillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        self.sun_sensitive_tick()
    }
}

impl SunSensitive for ZombieVillagerEntity {}
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity, SunSensitive};
use crate::entity::{EntityBase, EntityBaseFuture, NBTStorage};
use std::sync::Arc;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
}

impl ZombieEntity {
    pub async fn new(entity: crate::entity::Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity).await;
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for ZombieEntity {}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        self.sun_sensitive_tick()
    }
}

impl SunSensitive for ZombieEntity {}

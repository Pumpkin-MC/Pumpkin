use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{EntityBase, EntityBaseFuture, NBTStorage};
use std::sync::Arc;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
}

impl ZombieEntity {
    pub async fn new(entity: crate::entity::Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity).await;
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        
        mob_arc
    }
}

impl NBTStorage for ZombieEntity {}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.sunburn().await })
    }
}
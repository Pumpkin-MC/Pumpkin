use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{Entity, NBTStorage};
use std::sync::Arc;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
}

impl ZombieEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        Arc::new(zombie)
    }
}

impl NBTStorage for ZombieEntity {}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

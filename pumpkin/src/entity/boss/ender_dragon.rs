use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
};

pub struct EnderDragonEntity {
    pub mob_entity: MobEntity,
}

impl EnderDragonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let dragon = Self { mob_entity };
        Arc::new(dragon)
    }
}

impl NBTStorage for EnderDragonEntity {}

impl Mob for EnderDragonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

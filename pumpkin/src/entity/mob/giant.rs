use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
};

pub struct GiantEntity {
    pub mob_entity: MobEntity,
}

impl GiantEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        // Vanilla `Giant` (and its `Monster` parent) never overrides `registerGoals`,
        // so Giants have no AI goals at all: they stand still and never attack.
        let mob_entity = MobEntity::new(entity);
        Arc::new(Self { mob_entity })
    }
}

impl NBTStorage for GiantEntity {}

impl Mob for GiantEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

use std::sync::Arc;

use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
};

/// Drowned — inherits zombie goals; prefers water pathfinding (vanilla InWater).
pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        // Prefer swimming: water is free, dry land is costly (opposite of golem).
        {
            let mut nav = entity.mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
            // Slight penalty on open dry land encourages staying near water.
            nav.set_pathfinding_malus(PathType::Open, 2.0);
        }
        Arc::new(Self { entity })
    }
}

impl NBTStorage for DrownedEntity {}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

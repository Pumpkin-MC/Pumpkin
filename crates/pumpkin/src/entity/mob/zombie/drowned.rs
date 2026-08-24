use std::sync::Arc;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::{
    Entity,
    mob::{Mob, MobEntity},
};

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        // Fix duplicated since already in ZombieEntity::new()
        {
            //let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

            // TODO
            // target_selector.add_goal(
            //     2,
            //     ActiveTargetGoal::with_default(
            //         &mob_arc.entity.mob_entity,
            //         &EntityType::PLAYER,
            //         true,
            //     ),
            // );
        };

        mob_arc
    }
}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

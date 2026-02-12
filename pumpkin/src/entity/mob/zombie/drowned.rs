use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, zombie::ZombieEntity},
};

pub struct DrownedEntity {
    entity: Arc<ZombieEntity>,
}

impl DrownedEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntity::new(entity).await;
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        // Fix duplicated since already in ZombieEntity::new()
        {
            //let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().await;

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

impl NBTStorage for DrownedEntity {}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

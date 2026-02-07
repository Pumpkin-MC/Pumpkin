use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::active_target::ActiveTargetGoal,
    mob::{Mob, MobEntity},
};

/// Endermite — a small hostile mob that spawns from ender pearls.
pub struct EndermiteEntity {
    pub mob_entity: MobEntity,
}

impl EndermiteEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let endermite = Self { mob_entity };
        let mob_arc = Arc::new(endermite);

        {
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for EndermiteEntity {}

impl Mob for EndermiteEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

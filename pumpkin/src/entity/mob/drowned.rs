use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{active_target::ActiveTargetGoal, ranged_attack::RangedAttackGoal},
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

        {
            let mut goal_selector = mob_arc.entity.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(
                2,
                RangedAttackGoal::new(1.0, 20, 10.0),
            );

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::PLAYER,
                    true,
                ),
            );
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

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal},
    mob::{Mob, MobEntity},
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
}

impl WolfEntity {
    pub async fn make(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self { mob_entity };
        let mob_arc = Arc::new(wolf);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            // TODO
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for WolfEntity {}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

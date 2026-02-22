use std::sync::Arc;

use pumpkin_data::sound::Sound;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::fish_swim::FishSwimGoal,
    mob::{Mob, MobEntity},
};

pub struct SalmonEntity {
    pub mob_entity: MobEntity,
}

impl SalmonEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let salmon = Self {
            mob_entity: MobEntity::new(entity),
        };
        let mob_arc = Arc::new(salmon);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(4, Box::new(FishSwimGoal::new(Sound::EntitySalmonFlop)));
        };

        mob_arc
    }
}

impl NBTStorage for SalmonEntity {}

impl Mob for SalmonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

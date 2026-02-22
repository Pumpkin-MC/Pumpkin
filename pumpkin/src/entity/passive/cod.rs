use std::sync::Arc;

use pumpkin_data::sound::Sound;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::fish_swim::FishSwimGoal,
    mob::{Mob, MobEntity},
};

pub struct CodEntity {
    pub mob_entity: MobEntity,
}

impl CodEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let cod = Self {
            mob_entity: MobEntity::new(entity),
        };
        let mob_arc = Arc::new(cod);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(4, Box::new(FishSwimGoal::new(Sound::EntityCodFlop)));
        };

        mob_arc
    }
}

impl NBTStorage for CodEntity {}

impl Mob for CodEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

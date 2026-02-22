use std::sync::Arc;

use pumpkin_data::sound::Sound;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::fish_swim::FishSwimGoal,
    mob::{Mob, MobEntity},
};

pub struct TropicalFishEntity {
    pub mob_entity: MobEntity,
}

impl TropicalFishEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let tropical_fish = Self {
            mob_entity: MobEntity::new(entity),
        };
        let mob_arc = Arc::new(tropical_fish);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(
                4,
                Box::new(FishSwimGoal::new(Sound::EntityTropicalFishFlop)),
            );
        };

        mob_arc
    }
}

impl NBTStorage for TropicalFishEntity {}

impl Mob for TropicalFishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

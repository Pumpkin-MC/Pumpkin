use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::swim::SwimGoal,
    mob::{Mob, MobEntity},
};

/// Tadpole — a baby frog that swims in water.
///
/// Growth into frog based on biome is a future addition.
pub struct TadpoleEntity {
    pub mob_entity: MobEntity,
}

impl TadpoleEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(0, SwimGoal::new());
        };

        mob_arc
    }
}

impl NBTStorage for TadpoleEntity {}

impl Mob for TadpoleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

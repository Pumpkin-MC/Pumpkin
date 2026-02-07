use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::swim::SwimGoal,
    mob::{Mob, MobEntity},
};

/// Squid — a passive water mob.
///
/// Squids swim around in water. In vanilla they have specialized
/// water movement; for now they use the basic SwimGoal.
pub struct SquidEntity {
    pub mob_entity: MobEntity,
}

impl SquidEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let squid = Self { mob_entity };
        let mob_arc = Arc::new(squid);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(0, SwimGoal::new());
        };

        mob_arc
    }
}

impl NBTStorage for SquidEntity {}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

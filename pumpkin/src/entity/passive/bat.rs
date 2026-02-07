use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::look_around::LookAroundGoal,
    mob::{Mob, MobEntity},
};

/// Bat — a small ambient mob that hangs from ceilings and flies erratically.
///
/// Bats are ambient mobs with minimal AI. In vanilla they have special
/// flying/hanging behavior; for now they use basic look-around.
pub struct BatEntity {
    pub mob_entity: MobEntity,
}

impl BatEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bat = Self { mob_entity };
        let mob_arc = Arc::new(bat);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for BatEntity {}

impl Mob for BatEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

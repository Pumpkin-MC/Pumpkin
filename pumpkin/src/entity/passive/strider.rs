use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal, panic::PanicGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Strider — a passive mob that walks on lava in the Nether.
///
/// Currently has basic AI: panics, wanders, looks at players.
/// Lava walking, saddle control, and shivering are future additions.
pub struct StriderEntity {
    pub mob_entity: MobEntity,
}

impl StriderEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(1, PanicGoal::new(1.65));
            goal_selector.add_goal(6, WanderAroundGoal::new(0.6));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for StriderEntity {}

impl Mob for StriderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal,
    },
    mob::{Mob, MobEntity},
};

/// Shulker — a box-like hostile mob found in End cities.
///
/// Currently has basic AI: targets players.
/// Shulker bullet projectile and hiding in shell are future additions.
pub struct ShulkerEntity {
    pub mob_entity: MobEntity,
}

impl ShulkerEntity {
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
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for ShulkerEntity {}

impl Mob for ShulkerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Slime — a bouncing hostile mob that splits into smaller slimes on death.
///
/// Currently has basic AI: targets players, wanders, swims.
/// Splitting behavior and size variants are future additions.
pub struct SlimeEntity {
    pub mob_entity: MobEntity,
}

impl SlimeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self { mob_entity };
        let mob_arc = Arc::new(slime);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.2, false)));
            goal_selector.add_goal(6, WanderAroundGoal::new(0.8));
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

impl NBTStorage for SlimeEntity {}

impl Mob for SlimeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

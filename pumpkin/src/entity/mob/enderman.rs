use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
        zombie_attack::ZombieAttackGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct EndermanEntity {
    pub mob_entity: MobEntity,
}

impl EndermanEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let enderman = Self { mob_entity };
        let mob_arc = Arc::new(enderman);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(2, ZombieAttackGoal::new(0.1, false));
            goal_selector.add_goal(7, WanderAroundGoal::new(1.0));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));

            // Endermen only target players who look at them in vanilla;
            // for now we use basic player targeting as a placeholder
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for EndermanEntity {}

impl Mob for EndermanEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

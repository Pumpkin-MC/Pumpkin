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

pub struct SpiderEntity {
    pub mob_entity: MobEntity,
}

impl SpiderEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let spider = Self { mob_entity };
        let mob_arc = Arc::new(spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            // Spiders use the same melee attack pattern as zombies
            goal_selector.add_goal(2, ZombieAttackGoal::new(0.1, false));
            goal_selector.add_goal(6, WanderAroundGoal::new(0.8));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SpiderEntity {}

impl Mob for SpiderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

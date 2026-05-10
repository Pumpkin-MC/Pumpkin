use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct SnowGolemEntity {
    pub mob_entity: MobEntity,
}

impl SnowGolemEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let snow_golem = Self { mob_entity };
        let mob_arc = Arc::new(snow_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            // TODO: SnowballAttackGoal
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::ZOMBIE, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SnowGolemEntity {}

impl Mob for SnowGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct EvokerEntity {
    pub mob_entity: MobEntity,
}

impl EvokerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let evoker = Self { mob_entity };
        let mob_arc = Arc::new(evoker);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Evokers have complex spell AI, but for now basic movement
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 0.6, 1.0)),
            );
            goal_selector.add_goal(
                3,
                Box::new(AvoidEntityGoal::new(&EntityType::CREAKING, 8.0, 0.6, 1.0)),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut player_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true);
            player_goal.set_unseen_memory_ticks(300);
            target_selector.add_goal(1, player_goal);
            let mut villager_goal =
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true);
            villager_goal
                .set_target_types(vec![&EntityType::VILLAGER, &EntityType::WANDERING_TRADER]);
            villager_goal.set_unseen_memory_ticks(300);
            target_selector.add_goal(2, villager_goal);
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for EvokerEntity {}

impl Mob for EvokerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

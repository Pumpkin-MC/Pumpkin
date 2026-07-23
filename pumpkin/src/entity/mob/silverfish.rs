use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, join_anger::JoinAngerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        silverfish_merge::SilverfishMergeWithStoneGoal, silverfish_wake::SilverfishWakeFriendsGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Silverfish — pack anger, wake friends, merge into stone.
pub struct SilverfishEntity {
    entity: Arc<MobEntity>,
}

impl SilverfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let silverfish = Self { entity };
        let mob_arc = Arc::new(silverfish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // Crack nearby infested stone while fighting (vanilla wake friends).
            goal_selector.add_goal(3, SilverfishWakeFriendsGoal::new());
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            // Enter host stone when idle (vanilla merge).
            goal_selector.add_goal(5, SilverfishMergeWithStoneGoal::new());
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // Wake friends when one is hurt.
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::SILVERFISH));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SilverfishEntity {}

impl Mob for SilverfishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}

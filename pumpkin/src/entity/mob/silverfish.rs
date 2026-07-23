use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, join_anger::JoinAngerGoal, melee_attack::MeleeAttackGoal,
        revenge::RevengeGoal, silverfish_merge::SilverfishMergeWithStoneGoal,
        silverfish_wake::SilverfishWakeFriendsGoal, swim::SwimGoal,
    },
    mob::{Mob, MobEntity},
};

/// Silverfish — pack anger, wake friends, merge into stone (vanilla 26.2).
pub struct SilverfishEntity {
    entity: Arc<MobEntity>,
}

impl SilverfishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let silverfish = Self { entity };
        let mob_arc = Arc::new(silverfish);

        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Silverfish.registerGoals (no look/wander).
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            // ClimbOnTopOfPowderSnowGoal TODO
            goal_selector.add_goal(3, SilverfishWakeFriendsGoal::new());
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, SilverfishMergeWithStoneGoal::new());

            // HurtByTarget.setAlertOthers() + NearestAttackableTarget(Player)
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(1, JoinAngerGoal::new(&EntityType::SILVERFISH));
            target_selector.add_goal(
                2,
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

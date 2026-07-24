use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, join_anger::JoinAngerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Hoglin — **Brain** in vanilla (`HoglinAi`); GoalSelector stand-in only.
///
/// Brain activities: hunt players/piglins, flee when outnumbered, breed, etc.
pub struct HoglinEntity {
    pub mob_entity: MobEntity,
}

impl HoglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let hoglin = Self { mob_entity };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.4));
            // Flee many piglins (brain flee stand-in).
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PIGLIN, 8.0, 1.0, 1.2)),
            );
            goal_selector.add_goal(3, BreedGoal::new(0.6));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.4)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::HOGLIN));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PIGLIN, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for HoglinEntity {}

impl Mob for HoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        snowball_attack::SnowballAttackGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

// Vanilla ItemTags.LLAMA_TEMPT_ITEMS — hay block primary.
const TEMPT_ITEMS: &[&Item] = &[&Item::HAY_BLOCK];

/// Llama — vanilla 26.2: spit (snowball stand-in), panic, breed, tempt, hunt wolves.
///
/// Decompile: RangedAttackGoal interval 40; LlamaAttackWolfGoal; caravan TODO.
pub struct LlamaEntity {
    pub mob_entity: MobEntity,
}

impl LlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self { mob_entity };
        let mob_arc = Arc::new(llama);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // RunAroundLikeCrazyGoal TODO (untamed)
            goal_selector.add_goal(3, SnowballAttackGoal::new(1.25)); // RangedAttack spit
            goal_selector.add_goal(3, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(4, BreedGoal::new(1.0));
            goal_selector.add_goal(5, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(6, Box::new(FollowParentGoal::new(1.0)));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WOLF, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for LlamaEntity {}

impl Mob for LlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

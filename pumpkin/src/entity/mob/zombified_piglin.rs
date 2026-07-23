use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        join_anger::JoinAngerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Zombified piglin — neutral until hurt; pack joins anger via `JoinAngerGoal`.
pub struct ZombifiedPiglinEntity {
    pub mob_entity: MobEntity,
}

impl ZombifiedPiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self { mob_entity };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            // Personal revenge first, then pack anger from nearby allies.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(2, JoinAngerGoal::new(&EntityType::ZOMBIFIED_PIGLIN));
            // Vanilla Zombie HurtByTargetGoal.setAlertOthers(ZombifiedPiglin):
            // also join when a nearby zombie is attacked.
            target_selector.add_goal(3, JoinAngerGoal::new(&EntityType::ZOMBIE));
            target_selector.add_goal(3, JoinAngerGoal::new(&EntityType::HUSK));
            target_selector.add_goal(3, JoinAngerGoal::new(&EntityType::DROWNED));
            target_selector.add_goal(3, JoinAngerGoal::new(&EntityType::ZOMBIE_VILLAGER));
        };

        mob_arc
    }
}

impl NBTStorage for ZombifiedPiglinEntity {}

impl Mob for ZombifiedPiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

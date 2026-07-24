use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        join_anger::JoinAngerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::BAMBOO];

/// Panda — breed/tempt bamboo; personality roll/sit TODO.
pub struct PandaEntity {
    pub mob_entity: MobEntity,
}

impl PandaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let panda = Self { mob_entity };
        let mob_arc = Arc::new(panda);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Panda — avoid player/monsters when worried; sit/roll TODO.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(MeleeAttackGoal::new(1.2, true)));
            goal_selector.add_goal(4, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(
                6,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 2.0, 2.0)),
            );
            for ty in [
                &EntityType::ZOMBIE,
                &EntityType::SKELETON,
                &EntityType::CREEPER,
                &EntityType::SPIDER,
            ] {
                goal_selector.add_goal(6, Box::new(AvoidEntityGoal::new(ty, 4.0, 2.0, 2.0)));
            }
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(14, Box::new(WanderAroundGoal::new(1.0)));

            // PandaHurtByTarget.setAlertOthers()
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(1, JoinAngerGoal::new(&EntityType::PANDA));
        };

        mob_arc
    }
}

impl NBTStorage for PandaEntity {}

impl Mob for PandaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

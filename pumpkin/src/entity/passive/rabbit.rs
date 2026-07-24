use std::sync::{Arc, Weak};

use pumpkin_data::{entity::EntityType, item::Item};

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::CARROT, &Item::GOLDEN_CARROT, &Item::DANDELION];

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
}

impl RabbitEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let rabbit = Self { mob_entity };
        let mob_arc = Arc::new(rabbit);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Vanilla 26.2: avoid Player/Wolf/Monster; RaidGarden TODO.
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 2.2, 2.2)),
            );
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::WOLF, 10.0, 2.2, 2.2)),
            );
            // Monster.class stand-in: common hostiles
            for ty in [
                &EntityType::ZOMBIE,
                &EntityType::SKELETON,
                &EntityType::SPIDER,
                &EntityType::CREEPER,
                &EntityType::HUSK,
                &EntityType::STRAY,
            ] {
                goal_selector.add_goal(
                    4,
                    Box::new(AvoidEntityGoal::new(ty, 4.0, 2.2, 2.2)),
                );
            }
            goal_selector.add_goal(2, BreedGoal::new(0.8));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
        };

        mob_arc
    }
}

impl NBTStorage for RabbitEntity {}

impl Mob for RabbitEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

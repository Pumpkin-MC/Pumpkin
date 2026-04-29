use std::sync::{Arc, Weak};

use pumpkin_data::{entity::EntityType, item::Item};

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::CARROT, &Item::GOLDEN_CARROT, &Item::DANDELION];

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
}

impl RabbitEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let rabbit = Self { mob_entity };
        let mob_arc = Arc::new(rabbit);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(2, BreedGoal::new(0.8));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(0.8)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(11, Box::new(RandomLookAroundGoal::default()));
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

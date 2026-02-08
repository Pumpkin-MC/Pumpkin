use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        breed, follow_owner::FollowOwnerGoal, follow_parent,
        look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct WolfEntity {
    pub mob_entity: MobEntity,
}

impl WolfEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self { mob_entity };
        let mob_arc = Arc::new(wolf);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(3, tempt::TemptGoal::new(1.0, tempt::TEMPT_WOLF, 10.0));
            goal_selector.add_goal(4, breed::BreedGoal::new(1.0));
            goal_selector.add_goal(5, follow_parent::FollowParentGoal::new(1.1));
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0));
            goal_selector.add_goal(7, WanderAroundGoal::new(1.0));
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for WolfEntity {}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

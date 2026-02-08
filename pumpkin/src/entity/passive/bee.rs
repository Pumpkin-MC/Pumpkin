use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        breed, follow_parent, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, panic::PanicGoal, swim::SwimGoal, tempt,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Bee — a neutral flying mob that pollinates flowers and produces honey.
pub struct BeeEntity {
    pub mob_entity: MobEntity,
}

impl BeeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bee = Self { mob_entity };
        let mob_arc = Arc::new(bee);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(1, PanicGoal::new(2.0));
            goal_selector.add_goal(3, tempt::TemptGoal::new(0.6, tempt::TEMPT_BEE, 10.0));
            goal_selector.add_goal(4, breed::BreedGoal::new(1.0));
            goal_selector.add_goal(5, follow_parent::FollowParentGoal::new(1.1));
            goal_selector.add_goal(6, WanderAroundGoal::new(1.0));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for BeeEntity {}

impl Mob for BeeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

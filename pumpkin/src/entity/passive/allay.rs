use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, follow_player::FollowPlayerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        random_float::RandomFloatGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Allay — follows nearest player; item collect TODO.
pub struct AllayEntity {
    pub mob_entity: MobEntity,
}

impl AllayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let allay = Self { mob_entity };
        let mob_arc = Arc::new(allay);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            // Hover between follows (item-collect / dance TODO).
            goal_selector.add_goal(1, RandomFloatGoal::new());
            goal_selector.add_goal(2, FollowPlayerGoal::new(1.0));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 16.0),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for AllayEntity {}

impl Mob for AllayEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.04
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Allay — follows/looks at players; item collect TODO.
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
            // Prefer staying near players (look + wander slowly).
            goal_selector.add_goal(
                1,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 16.0),
            );
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
            let _ = mob_weak;
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

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        follow_flock_leader::FollowFlockLeaderGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Represents a Cod, a common passive aquatic mob.
///
/// Wiki: <https://minecraft.wiki/w/Cod>
pub struct CodEntity {
    pub mob_entity: MobEntity,
}

impl CodEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cod = Self { mob_entity };
        let mob_arc = Arc::new(cod);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.25));
            // Vanilla `AbstractFish.registerGoals`: flee players within 8 blocks.
            // The vanilla goal also skips spectators, which `AvoidEntityGoal` cannot do yet.
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new_with_interval(1.0, 40)));
            goal_selector.add_goal(5, FollowFlockLeaderGoal::new());
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for CodEntity {}

impl Mob for CodEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

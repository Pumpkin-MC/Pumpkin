use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

/// Cod — school fish; flee players.
pub struct CodEntity {
    pub mob_entity: MobEntity,
}

impl CodEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let cod = Self { mob_entity };
        let mob_arc = Arc::new(cod);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            // Vanilla AbstractFish: AvoidEntity(Player only) — not axolotl.
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
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

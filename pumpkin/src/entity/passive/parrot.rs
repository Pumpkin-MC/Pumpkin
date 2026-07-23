use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        follow_player::FollowPlayerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[
    &Item::WHEAT_SEEDS,
    &Item::MELON_SEEDS,
    &Item::PUMPKIN_SEEDS,
    &Item::BEETROOT_SEEDS,
];

/// Parrot — flees hostiles, follows player, tempt seeds; shoulder sit TODO.
pub struct ParrotEntity {
    pub mob_entity: MobEntity,
}

impl ParrotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let parrot = Self { mob_entity };
        let mob_arc = Arc::new(parrot);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            // Vanilla flees many mobs; cover common hostiles.
            for ty in [
                &EntityType::CREEPER,
                &EntityType::SKELETON,
                &EntityType::ZOMBIE,
                &EntityType::SPIDER,
                &EntityType::BLAZE,
                &EntityType::WITCH,
            ] {
                goal_selector.add_goal(
                    1,
                    Box::new(AvoidEntityGoal::new(ty, 8.0, 1.2, 1.4)),
                );
            }
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            // Stand-in for flying to nearby player (shoulder perch TODO).
            goal_selector.add_goal(3, FollowPlayerGoal::new(1.0));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for ParrotEntity {}

impl Mob for ParrotEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.05
    }
}

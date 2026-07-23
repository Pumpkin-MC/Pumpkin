use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal,
        escape_danger::EscapeDangerGoal, leap_at_target::LeapAtTargetGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

/// Ocelot — shy; avoid players, hunt chickens, tempt with fish.
pub struct OcelotEntity {
    pub mob_entity: MobEntity,
}

impl OcelotEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ocelot = Self { mob_entity };
        let mob_arc = Arc::new(ocelot);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 10.0, 0.8, 1.33)),
            );
            goal_selector.add_goal(3, Box::new(TemptGoal::new(0.6, TEMPT_ITEMS)));
            goal_selector.add_goal(3, Box::new(LeapAtTargetGoal::new(0.3)));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::CHICKEN, true),
            );
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::TURTLE, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for OcelotEntity {}

impl Mob for OcelotEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

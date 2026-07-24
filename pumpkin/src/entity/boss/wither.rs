use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, random_float::RandomFloatGoal, revenge::RevengeGoal,
        wither_skull::WitherSkullGoal,
    },
    mob::{Mob, MobEntity},
};

/// Wither boss — flying skull shooter stand-in (full head AI / block break TODO).
pub struct WitherEntity {
    pub mob_entity: MobEntity,
}

impl WitherEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        mob_entity
            .living_entity
            .set_attribute_base(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE, 40.0);
        let wither = Self { mob_entity };
        let mob_arc = Arc::new(wither);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(2, WitherSkullGoal::new());
            goal_selector.add_goal(5, RandomFloatGoal::new());
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // Wither targets living non-undead; player is primary.
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SNOW_GOLEM, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for WitherEntity {}

impl Mob for WitherEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}

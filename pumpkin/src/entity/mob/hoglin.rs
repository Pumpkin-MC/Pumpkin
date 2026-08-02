use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct HoglinEntity {
    pub mob_entity: MobEntity,
}

impl HoglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let hoglin = Self { mob_entity };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            // Vanilla: adult hoglins flee visible piglins within 8 blocks while idle
            // (HoglinAi.java:74, DESIRED_DISTANCE_FROM_PIGLIN_WHEN_IDLING=8, speed
            // 0.4F) and flee harder once actually hit (initRetreatActivity, distance
            // 15, speed SPEED_MULTIPLIER_WHEN_RETREATING=1.3F). Pumpkin's
            // `AvoidEntityGoal` has a single close/far speed model rather than two
            // separate Brain activities, so this merges both into one goal; the
            // adult/baby and `isPacified`/repellent gating from vanilla are also not
            // reproduced here.
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PIGLIN, 8.0, 0.4, 1.3)),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for HoglinEntity {}

impl Mob for HoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

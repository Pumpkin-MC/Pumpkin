use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct GiantEntity {
    pub mob_entity: MobEntity,
}

impl GiantEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut attributes = mob_entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(health) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::MAX_HEALTH.id)
            {
                health.base_value = 100.0;
                health
                    .dirty
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            if let Some(speed) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED.id)
            {
                speed.base_value = 0.5;
                speed
                    .dirty
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            if let Some(damage) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::ATTACK_DAMAGE.id)
            {
                damage.base_value = 50.0;
                damage
                    .dirty
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        mob_entity.living_entity.health.store(100.0);

        let giant = Self { mob_entity };
        let mob_arc = Arc::new(giant);

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl Mob for GiantEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

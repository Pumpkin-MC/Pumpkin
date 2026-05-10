use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Represents a Polar Bear, a neutral mob found in cold biomes.
///
/// Wiki: <https://minecraft.wiki/w/Polar_Bear>
pub struct PolarBearEntity {
    pub mob_entity: MobEntity,
}

impl PolarBearEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let polar_bear = Self { mob_entity };
        let mob_arc = Arc::new(polar_bear);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;
            // Polar bears are neutral but aggressive towards foxes
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::FOX, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for PolarBearEntity {}

impl Mob for PolarBearEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

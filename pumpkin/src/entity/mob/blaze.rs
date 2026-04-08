use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct BlazeEntity {
    entity: Arc<MobEntity>,
}

impl BlazeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.entity.target_selector.lock().await;

            // TODO
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for BlazeEntity {}

impl Mob for BlazeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Weak};

    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::world_seed::Seed;
    use pumpkin_world::{level::Level, world_info::LevelData};
    use tempfile::tempdir;
    use uuid::Uuid;

    use crate::block::registry::default_registry;
    use crate::entity::EntityBase;
    use crate::world::World;

    fn test_world() -> Arc<World> {
        let temp_dir = tempdir().unwrap();
        let block_registry = default_registry();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            temp_dir.keep(),
            block_registry.clone(),
            0,
            Dimension::OVERWORLD,
        );
        let level_info = Arc::new(ArcSwap::new(Arc::new(LevelData::default(Seed(0)))));

        Arc::new(World::load(
            level,
            level_info,
            Dimension::OVERWORLD,
            block_registry,
            Weak::new(),
        ))
    }

    async fn make_blaze(world: Arc<World>) -> Arc<BlazeEntity> {
        let entity = Entity::from_uuid(
            Uuid::new_v4(),
            world,
            Vector3::new(0.0, 80.0, 0.0),
            &EntityType::BLAZE,
        );
        BlazeEntity::new(entity).await
    }

    #[tokio::test]
    async fn blaze_uses_airborne_movement_hooks_instead_of_ground_mob_defaults() {
        let blaze = make_blaze(test_world()).await;

        assert_eq!(blaze.get_gravity(), 0.0);
        assert_eq!(blaze.get_y_velocity_drag(), Some(0.6));

        let falling_velocity = -0.4;
        let blaze_next_y = (falling_velocity - blaze.get_gravity())
            * blaze
                .get_y_velocity_drag()
                .expect("blaze should override Y drag");
        let generic_ground_mob_next_y = (falling_velocity - 0.08) * 0.98;

        assert!(
            blaze_next_y > generic_ground_mob_next_y,
            "blaze should retain more upward/hovering motion than a gravity-bound mob"
        );
    }
}

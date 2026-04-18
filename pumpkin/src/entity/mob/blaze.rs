use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct BlazeEntity {
    entity: Arc<MobEntity>,
    allowed_height_offset: AtomicCell<f32>,
    next_height_offset_change_tick: AtomicI32,
}

impl BlazeEntity {
    fn next_triangular(mode: f64, deviation: f64) -> f64 {
        deviation.mul_add(rand::random::<f64>() - rand::random::<f64>(), mode)
    }

    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let blaze = Self {
            entity,
            allowed_height_offset: AtomicCell::new(0.5),
            next_height_offset_change_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(blaze);
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

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.next_height_offset_change_tick.fetch_sub(1, Relaxed) <= 0 {
                self.next_height_offset_change_tick.store(100, Relaxed);
                self.allowed_height_offset
                    .store(Self::next_triangular(0.5, 6.891) as f32);
            }

            let Some(target) = self.entity.target.lock().await.clone() else {
                return;
            };

            if !target.get_entity().is_alive() {
                return;
            }

            let entity = &self.entity.living_entity.entity;
            let blaze_eye_y = entity.pos.load().y + entity.get_eye_height();
            let target_entity = target.get_entity();
            let target_eye_y = target_entity.pos.load().y + target_entity.get_eye_height();

            if target_eye_y <= blaze_eye_y + f64::from(self.allowed_height_offset.load()) {
                return;
            }

            let mut velocity = entity.velocity.load();
            velocity.y += (0.300_000_011_920_928_96 - velocity.y) * 0.300_000_011_920_928_96;
            entity.velocity.store(velocity);
        })
    }

    fn post_tick(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = &self.entity.living_entity.entity;
            if entity.on_ground.load(Relaxed) {
                return;
            }

            let mut velocity = entity.velocity.load();
            if velocity.y >= 0.0 {
                return;
            }

            velocity.y *= 0.6;
            entity.velocity.store(velocity);
        })
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
            None,
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

    async fn make_blaze(world: Arc<World>, pos: Vector3<f64>) -> Arc<BlazeEntity> {
        let entity = Entity::from_uuid(Uuid::new_v4(), world, pos, &EntityType::BLAZE);
        BlazeEntity::new(entity).await
    }

    #[tokio::test]
    async fn blaze_damps_only_falling_velocity_like_vanilla() {
        let blaze = make_blaze(test_world(), Vector3::new(0.0, 80.0, 0.0)).await;
        let entity = &blaze.entity.living_entity.entity;

        entity.on_ground.store(false, Relaxed);
        entity.velocity.store(Vector3::new(0.1, -0.5, -0.2));

        blaze.post_tick().await;

        let velocity = entity.velocity.load();
        assert_eq!(velocity.x, 0.1);
        assert_eq!(velocity.z, -0.2);
        assert!((velocity.y - -0.3).abs() < f64::EPSILON);
        assert_eq!(blaze.get_gravity(), 0.08);
        assert_eq!(blaze.get_y_velocity_drag(), None);
    }

    #[tokio::test]
    async fn blaze_lifts_toward_higher_targets_like_vanilla() {
        let world = test_world();
        let blaze = make_blaze(world.clone(), Vector3::new(0.0, 80.0, 0.0)).await;
        let target = make_blaze(world, Vector3::new(0.0, 90.0, 0.0)).await;

        blaze.allowed_height_offset.store(0.5);
        blaze.next_height_offset_change_tick.store(100, Relaxed);
        blaze
            .set_mob_target(Some(target.clone() as Arc<dyn EntityBase>))
            .await;

        let entity = &blaze.entity.living_entity.entity;
        entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));

        blaze
            .mob_tick(&(blaze.clone() as Arc<dyn EntityBase>))
            .await;

        let velocity = entity.velocity.load();
        assert!((velocity.y - 0.090_000_007_152_557_37).abs() < 1.0e-12);
    }
}

use super::{Entity, EntityBase, NBTStorage, ai::path::Navigator, living::LivingEntity};
use crate::entity::EntityBaseFuture;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::server::Server;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use tokio::sync::Mutex;

pub mod zombie;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals_selector: Mutex<GoalSelector>,
    pub target_selector: Mutex<GoalSelector>,
    pub navigator: Mutex<Navigator>,
    pub target: Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: Mutex<LookControl>,
    pub position_target: AtomicCell<BlockPos>,
    pub position_target_range: AtomicI32,
}

impl MobEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            goals_selector: Mutex::new(GoalSelector::default()),
            target_selector: Mutex::new(GoalSelector::default()),
            navigator: Mutex::new(Navigator::default()),
            target: Mutex::new(None),
            look_control: Mutex::new(LookControl::default()),
            position_target: AtomicCell::new(BlockPos::ZERO),
            position_target_range: AtomicI32::new(-1),
        }
    }
    pub fn is_in_position_target_range(&self) -> bool {
        self.is_in_position_target_range_pos(self.living_entity.entity.block_pos.load())
    }

    pub fn is_in_position_target_range_pos(&self, block_pos: BlockPos) -> bool {
        let position_target_range = self.position_target_range.load(Relaxed);
        if position_target_range == -1 {
            true
        } else {
            self.position_target.load().squared_distance(block_pos)
                < position_target_range * position_target_range
        }
    }

    pub fn set_attacking(&self, _attacking: bool) {
        // TODO: set to data tracker
    }
}

// This trait contains all overridable functions
pub trait Mob: EntityBase + Send + Sync {
    fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }

    fn get_max_look_yaw_change(&self) -> i32 {
        10
    }

    fn get_max_look_pitch_change(&self) -> i32 {
        40
    }

    fn get_max_head_rotation(&self) -> i32 {
        75
    }

    fn get_mob_entity(&self) -> &MobEntity;

    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        None
    }
}

impl<T: Mob + Send + 'static> EntityBase for T {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            mob_entity.living_entity.tick(caller, server).await;

            let age = mob_entity.living_entity.entity.age.load(Relaxed);
            if (age + mob_entity.living_entity.entity.entity_id) % 2 != 0 && age > 1 {
                mob_entity
                    .target_selector
                    .lock()
                    .await
                    .tick_goals(self, false)
                    .await;
                mob_entity
                    .goals_selector
                    .lock()
                    .await
                    .tick_goals(self, false)
                    .await;
            } else {
                mob_entity.target_selector.lock().await.tick(self).await;
                mob_entity.goals_selector.lock().await.tick(self).await;
            }

            let mut navigator = mob_entity.navigator.lock().await;
            navigator.tick(&mob_entity.living_entity).await;
            drop(navigator);

            let mut look_control = mob_entity.look_control.lock().await;
            look_control.tick(self).await;
            drop(look_control);
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.get_mob_entity()
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.get_mob_entity().living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.get_mob_entity().living_entity)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        self.get_mob_entity().living_entity.get_gravity()
    }
}

#[allow(dead_code)]
const DEFAULT_PATHFINDING_FAVOR: f32 = 0.0;

pub trait PathAwareEntity: Mob + Send + Sync {
    fn get_pathfinding_favor(&self, _block_pos: BlockPos, _world: Arc<World>) -> f32 {
        0.0
    }

    // TODO: missing SpawnReason attribute
    fn can_spawn(&self, world: Arc<World>) -> bool {
        self.get_pathfinding_favor(
            self.get_mob_entity().living_entity.entity.block_pos.load(),
            world,
        ) >= 0.0
    }

    fn is_navigation<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async {
            let navigator = self.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    // TODO: implement
    fn is_panicking(&self) -> bool {
        false
    }

    fn should_follow_leash(&self) -> bool {
        true
    }

    fn on_short_leash_tick(&self) {
        // TODO: implement
    }

    fn before_leash_tick(&self) {
        // TODO: implement
    }

    fn get_follow_leash_speed(&self) -> f32 {
        1.0
    }
}

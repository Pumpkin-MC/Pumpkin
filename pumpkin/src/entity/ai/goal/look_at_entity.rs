use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use async_trait::async_trait;
use rand::Rng;
use tokio::sync::Mutex;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::{mob::MobEntity, player::Player, Entity, EntityBase};
use crate::entity::ai::target_predicate;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::predicate::EntityPredicate;
use crate::world::World;
use super::Goal;

pub struct LookAtEntityGoal {
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    range: f64,
    look_time: AtomicI32,
    chance: f64,
    look_forward: bool,
    target_type: EntityType,
    target_predicate: TargetPredicate,
}

impl LookAtEntityGoal {
    #[must_use]
    pub fn new(entity: &Entity, target_type: EntityType, range: f64, chance: f64, look_forward: bool) -> Self {
        let target_predicate = Self::create_target_predicate(entity, target_type, range);
        Self {
            target: Mutex::new(None),
            range,
            look_time: AtomicI32::new(0),
            chance,
            look_forward,
            target_type,
            target_predicate,
        }
    }
    
    pub fn default(entity: &Entity, target_type: EntityType, range: f64) -> Self {
        let target_predicate = Self::create_target_predicate(entity, target_type, range);
        Self {
            target: Mutex::new(None),
            range,
            look_time: AtomicI32::new(0),
            chance: 0.02,
            look_forward: false,
            target_type,
            target_predicate,
        }
    }
    
    fn create_target_predicate(entity: &Entity, target_type: EntityType, range: f64) -> TargetPredicate {
        if target_type == EntityType::PLAYER {
            let mut target_predicate = TargetPredicate::non_attackable();
            target_predicate.base_max_distance = range;
            target_predicate.predicate = Some(Arc::new(move |living_entity: Arc<LivingEntity>, world: Arc<World>| {
                Box::pin(async move {
                    EntityPredicate::Rides(entity).test(&living_entity.entity).await
                })
            }));
            target_predicate
        } else {
            let mut target_predicate = TargetPredicate::non_attackable();
            target_predicate.base_max_distance = range;
            target_predicate
        }
    }
}

#[async_trait]
impl Goal for LookAtEntityGoal {
    async fn can_start(&self, mob: &MobEntity) -> bool {
        if mob.get_random().random::<f64>() >= self.chance {
            return false;
        }
        
        let mut target = self.target.lock().await;
        
        let mob_target = mob.target.lock().await;
        if mob_target.is_some() {
            *target = mob_target
                .clone()
                .map(|living_entity: Arc<LivingEntity>| living_entity as Arc<dyn EntityBase>);
        }
        drop(mob_target);
        
        if self.target_type == EntityType::PLAYER {
            *target = mob
                .living_entity
                .entity
                .world
                .read()
                .await
                .get_closest_player(mob.living_entity.entity.pos.load(), self.range)
                .await
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
        } else {
            *target = mob
                .living_entity
                .entity
                .world
                .read()
                .await
                .get_closest_entity(mob.living_entity.entity.pos.load(), self.range, Some(&[self.target_type]))
                .await;
        }
        
        target.is_some()
    }

    async fn should_continue(&self, mob: &MobEntity) -> bool {
        if let Some(target) = self.target.lock().await.as_ref() {
            if !target.get_entity().is_alive() {
                return false;
            }
            let mob_pos = mob.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(target_pos) > (self.range * self.range) {
                return false;
            }
            return self.look_time.load(Relaxed) > 0
        }
        false
    }

    async fn start(&self, mob: &MobEntity) {
        
    }

    async fn stop(&self, mob: &MobEntity) {
        *self.target.lock().await = None;
    }

    async fn tick(&self, mob: &MobEntity) {
        if let Some(target) = self.target.lock().await.as_ref() {
            if target.get_entity().is_alive() {
                let d = if self.look_forward { mob.living_entity.entity.get_eye_y() } else { target.get_entity().get_eye_y() };
                let target_pos = target.get_entity().pos.load();
                mob.living_entity.entity.look_at(Vector3::new(target_pos.x, d, target_pos.y)).await;
                self.look_time.fetch_sub(1, Relaxed);
            }
        }
    }
}

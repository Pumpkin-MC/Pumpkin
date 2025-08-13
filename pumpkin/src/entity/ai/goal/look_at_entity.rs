use super::{Control, Goal, GoalControl};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, player::Player};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use rand::Rng;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct LookAtEntityGoal {
    goal_control: GoalControl,
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    range: f64,
    look_time: AtomicI32,
    chance: f64,
    look_forward: bool,
    target_type: &'static EntityType,
    target_predicate: TargetPredicate,
}

impl LookAtEntityGoal {
    #[must_use]
    pub fn new(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f64,
        chance: f64,
        look_forward: bool,
    ) -> Self {
        let target_predicate = Self::create_target_predicate(mob_weak, target_type, range);
        Self {
            goal_control: GoalControl::from_array(&[Control::Look]),
            target: Mutex::new(None),
            range,
            look_time: AtomicI32::new(0),
            chance,
            look_forward,
            target_type,
            target_predicate,
        }
    }

    #[must_use]
    pub fn with_default(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f64,
    ) -> Self {
        Self::new(mob_weak, target_type, range, 0.02, false)
    }

    fn create_target_predicate(
        mob_weak: Weak<dyn Mob>,
        target_type: &'static EntityType,
        range: f64,
    ) -> TargetPredicate {
        let mut target_predicate = TargetPredicate::non_attackable();
        target_predicate.base_max_distance = range;
        if target_type == &EntityType::PLAYER {
            target_predicate.set_predicate(move |living_entity, _world| {
                let mob_weak = mob_weak.clone();
                async move {
                    if let Some(mob_arc) = mob_weak.upgrade() {
                        let predicate = EntityPredicate::Rides(mob_arc.get_entity());
                        predicate.test(&living_entity.entity).await
                    } else {
                        // MobEntity is destroyed
                        false
                    }
                }
            });
        }
        target_predicate
    }
}

#[async_trait]
impl Goal for LookAtEntityGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        if mob.get_random().random::<f64>() >= self.chance {
            return false;
        }

        let mob_entity = mob.get_mob_entity();
        let mut target = self.target.lock().await;

        let mob_target = mob_entity.target.lock().await;
        if mob_target.is_some() {
            (*target).clone_from(&mob_target);
        }
        drop(mob_target);

        let world = mob_entity.living_entity.entity.world.read().await;
        if self.target_type == &EntityType::PLAYER {
            *target = world
                .get_closest_player(mob_entity.living_entity.entity.pos.load(), self.range)
                .await
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
        } else {
            *target = world
                .get_closest_entity(
                    mob_entity.living_entity.entity.pos.load(),
                    self.range,
                    Some(&[self.target_type]),
                )
                .await;
        }

        target.is_some()
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        let mob = mob.get_mob_entity();
        if let Some(target) = self.target.lock().await.as_ref() {
            if !target.get_entity().is_alive() {
                return false;
            }
            let mob_pos = mob.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            if mob_pos.squared_distance_to_vec(target_pos) > (self.range * self.range) {
                return false;
            }
            return self.look_time.load(Relaxed) > 0;
        }
        false
    }

    async fn start(&self, mob: &dyn Mob) {
        let tick_count = self.get_tick_count(40 + mob.get_random().random_range(0..40));
        self.look_time.store(tick_count, Relaxed);
    }

    async fn stop(&self, _mob: &dyn Mob) {
        *self.target.lock().await = None;
    }

    async fn tick(&self, mob: &dyn Mob) {
        let mob = mob.get_mob_entity();
        if let Some(target) = self.target.lock().await.as_ref()
            && target.get_entity().is_alive()
        {
            let target_pos = target.get_entity().pos.load();
            mob.living_entity.entity.look_at(target_pos).await;
            self.look_time.fetch_sub(1, Relaxed);
        }
    }

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}

use std::sync::Arc;

use super::{Goal, to_goal_ticks};
use crate::entity::ai::goal::track_target_goal::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use rand::Rng;
use tokio::sync::Mutex;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

#[allow(dead_code)]
pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    reciprocal_chance: i32,
    target_type: EntityType,
    target_predicate: TargetPredicate,
}

impl ActiveTargetGoal {
    pub fn new<F, Fut>(
        mob: &MobEntity,
        target_type: EntityType,
        reciprocal_chance: i32,
        check_visibility: bool,
        check_can_navigate: bool,
        predicate: Option<F>,
    ) -> Self
    where
        F: Fn(Arc<LivingEntity>, Arc<World>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        let track_target_goal = TrackTargetGoal::new(check_visibility, check_can_navigate);
        let mut target_predicate = TargetPredicate::attackable();
        target_predicate.base_max_distance = TrackTargetGoal::get_follow_range(mob);
        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }
        Self {
            track_target_goal,
            target: Mutex::new(None),
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type,
            target_predicate,
        }
    }

    #[must_use]
    pub fn with_default(mob: &MobEntity, target_type: EntityType) -> Self {
        let track_target_goal = TrackTargetGoal::with_default(true);
        let mut target_predicate = TargetPredicate::attackable();
        target_predicate.base_max_distance = TrackTargetGoal::get_follow_range(mob);
        Self {
            track_target_goal,
            target: Mutex::new(None),
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type,
            target_predicate,
        }
    }

    async fn find_closest_target(&self, mob: &MobEntity) {
        let mut target = self.target.lock().await;
        let world = mob.living_entity.entity.world.read().await;
        if self.target_type == EntityType::PLAYER {
            *target = world
                .get_closest_player(
                    mob.living_entity.entity.pos.load(),
                    TrackTargetGoal::get_follow_range(mob),
                )
                .await
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
        } else {
            *target = world
                .get_closest_entity(
                    mob.living_entity.entity.pos.load(),
                    TrackTargetGoal::get_follow_range(mob),
                    Some(&[self.target_type]),
                )
                .await;
        }
    }
}

#[async_trait]
impl Goal for ActiveTargetGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        if self.reciprocal_chance > 0
            && mob.get_random().random_range(0..self.reciprocal_chance) != 0
        {
            return false;
        }
        self.find_closest_target(mob.get_mob_entity()).await;
        self.target.lock().await.is_some()
    }
    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob).await
    }

    async fn start(&self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let mut mob_target = mob_entity.target.lock().await;
        let target = self.target.lock().await.clone();
        (*mob_target).clone_from(&target);

        self.track_target_goal.start(mob).await;
    }

    async fn stop(&self, mob: &dyn Mob) {
        self.track_target_goal.stop(mob).await;
    }

    async fn tick(&self, _mob: &dyn Mob) {}
}

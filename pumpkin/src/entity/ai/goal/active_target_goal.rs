use std::sync::{Arc, Weak};

use super::{Goal, to_goal_ticks};
use crate::entity::ai::goal::track_target_goal::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use rand::Rng;
use tokio::sync::Mutex;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

pub struct ActiveTargetGoal {
    mob: Weak<MobEntity>,
    track_target_goal: TrackTargetGoal,
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    reciprocal_chance: i32,
    target_type: EntityType,
    target_predicate: TargetPredicate,
}

impl ActiveTargetGoal {
    #[must_use]
    pub fn new<F, Fut>(
        mob: Weak<MobEntity>,
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
        let track_target_goal =
            TrackTargetGoal::new(mob.clone(), check_visibility, check_can_navigate);
        let mut target_predicate = TargetPredicate::attackable();
        target_predicate.base_max_distance = track_target_goal.get_follow_range();
        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }
        Self {
            mob: mob.clone(),
            track_target_goal,
            target: Mutex::new(None),
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type,
            target_predicate,
        }
    }

    pub fn with_default(mob: Weak<MobEntity>, target_type: EntityType) -> Self {
        let track_target_goal = TrackTargetGoal::with_default(mob.clone(), true);
        let mut target_predicate = TargetPredicate::attackable();
        target_predicate.base_max_distance = track_target_goal.get_follow_range();
        Self {
            mob: mob.clone(),
            track_target_goal,
            target: Mutex::new(None),
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type,
            target_predicate,
        }
    }

    async fn find_closest_target(&self) {
        if let Some(mob) = self.mob.upgrade() {
            let mut target = self.target.lock().await;
            let world = mob.living_entity.entity.world.read().await;
            if self.target_type == EntityType::PLAYER {
                *target = world
                    .get_closest_player(
                        mob.living_entity.entity.pos.load(),
                        self.track_target_goal.get_follow_range(),
                    )
                    .await
                    .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
            } else {
                *target = world
                    .get_closest_entity(
                        mob.living_entity.entity.pos.load(),
                        self.track_target_goal.get_follow_range(),
                        Some(&[self.target_type]),
                    )
                    .await;
            }
        }
    }
}

#[async_trait]
impl Goal for ActiveTargetGoal {
    async fn can_start(&self, mob: &MobEntity) -> bool {
        if self.reciprocal_chance > 0
            && mob.get_random().random_range(0..self.reciprocal_chance) != 0
        {
            return false;
        }
        self.find_closest_target().await;
        self.target.lock().await.is_some()
    }
    async fn should_continue(&self, mob: &MobEntity) -> bool {
        self.track_target_goal.should_continue(mob).await
    }

    async fn start(&self, mob: &MobEntity) {
        let mut mob_target = mob.target.lock().await;
        let target = self.target.lock().await.clone();
        *mob_target = target.clone();

        self.track_target_goal.start(mob).await;
    }

    async fn stop(&self, mob: &MobEntity) {
        self.track_target_goal.stop(mob).await;
    }

    async fn tick(&self, _mob: &MobEntity) {}
}

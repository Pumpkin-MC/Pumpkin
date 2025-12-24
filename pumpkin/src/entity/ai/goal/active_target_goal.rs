use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target_goal::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use pumpkin_data::entity::EntityType;
use rand::Rng;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

#[allow(dead_code)]
pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    target_type: &'static EntityType,
    target_predicate: TargetPredicate,
}

impl ActiveTargetGoal {
    pub fn new<F, Fut>(
        mob: &MobEntity,
        target_type: &'static EntityType,
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
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type,
            target_predicate,
        }
    }

    #[must_use]
    pub fn with_default(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let mut target_predicate = TargetPredicate::attackable();
        target_predicate.base_max_distance = TrackTargetGoal::get_follow_range(mob);
        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type,
            target_predicate,
        })
    }

    async fn find_closest_target(&mut self, mob: &MobEntity) {
        let world = &mob.living_entity.entity.world;
        if self.target_type == &EntityType::PLAYER {
            self.target = world
                .get_closest_player(
                    mob.living_entity.entity.pos.load(),
                    TrackTargetGoal::get_follow_range(mob).into(),
                )
                .await
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);
        } else {
            self.target = world
                .get_closest_entity(
                    mob.living_entity.entity.pos.load(),
                    TrackTargetGoal::get_follow_range(mob).into(),
                    Some(&[self.target_type]),
                )
                .await;
        }
    }
}

impl Goal for ActiveTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.reciprocal_chance > 0
                && mob.get_random().random_range(0..self.reciprocal_chance) != 0
            {
                return false;
            }
            self.find_closest_target(mob.get_mob_entity()).await;
            self.target.is_some()
        })
    }
    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob_entity = mob.get_mob_entity();
            let mut mob_target = mob_entity.target.lock().await;
            (*mob_target).clone_from(&self.target);

            self.track_target_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.track_target_goal.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}

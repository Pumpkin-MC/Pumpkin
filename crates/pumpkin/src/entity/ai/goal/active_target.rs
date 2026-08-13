use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity};
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    target_types: Vec<&'static EntityType>,
    target_predicate: TargetPredicate,
    only_when_untamed: bool,
}

impl ActiveTargetGoal {
    pub fn new<F>(
        mob: &MobEntity,
        target_type: &'static EntityType,
        reciprocal_chance: i32,
        check_visibility: bool,
        check_can_navigate: bool,
        predicate: Option<F>,
    ) -> Self
    where
        F: for<'a> Fn(
                &'a LivingEntity,
                &'a World,
            ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        let track_target_goal = TrackTargetGoal::new(check_visibility, check_can_navigate);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }

        Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_types: vec![target_type],
            target_predicate,
            only_when_untamed: false,
        }
    }

    #[must_use]
    pub fn with_default(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_types: vec![target_type],
            target_predicate,
            only_when_untamed: false,
        })
    }

    pub fn set_target_types(&mut self, target_types: Vec<&'static EntityType>) {
        self.target_types = target_types;
    }

    pub const fn set_max_distance(&mut self, max_distance: f64) {
        self.target_predicate.base_max_distance = max_distance;
    }

    pub const fn set_only_when_untamed(&mut self, only_when_untamed: bool) {
        self.only_when_untamed = only_when_untamed;
    }

    pub const fn set_unseen_memory_ticks(&mut self, ticks: i32) {
        self.track_target_goal = self.track_target_goal.set_unseen_memory_ticks(ticks);
    }

    pub fn set_predicate<F>(&mut self, predicate: F)
    where
        F: for<'a> Fn(
                &'a LivingEntity,
                &'a World,
            ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        self.target_predicate.set_predicate(predicate);
    }

    pub fn set_target(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target = target;
    }

    async fn find_closest_target(&mut self, mob: &MobEntity) {
        let follow_range = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        // Vanilla updates the target conditions with the current follow distance on every search
        self.target_predicate.base_max_distance = follow_range;

        let world = mob.living_entity.entity.world.load();

        // Vanilla searches using getEyeY(), so we offset the position by the eye height
        let mut search_pos = mob.living_entity.entity.pos.load();
        search_pos.y += mob.living_entity.entity.entity_dimension.load().eye_height as f64;

        let search_box = mob
            .living_entity
            .entity
            .bounding_box
            .load()
            .expand_all(follow_range);
        let mut candidates = Vec::new();
        world.extend_entities_in_box_where(&mut candidates, usize::MAX, search_box, |entity| {
            self.target_types.contains(&entity.get_entity().entity_type)
        });

        let mut closest: Option<Arc<dyn EntityBase>> = None;
        let mut closest_distance = f64::INFINITY;
        for candidate in candidates {
            if candidate
                .get_entity()
                .get_player()
                .is_some_and(|player| player.is_spectator() || player.is_creative())
            {
                continue;
            }
            let Some(living) = candidate.get_living_entity() else {
                continue;
            };
            if !self
                .target_predicate
                .test(&world, Some(&mob.living_entity), living)
                .await
            {
                continue;
            }
            let distance = search_pos.squared_distance_to_vec(&candidate.get_entity().pos.load());
            if distance < closest_distance {
                closest_distance = distance;
                closest = Some(candidate);
            }
        }
        self.target = closest;
    }
}

impl Goal for ActiveTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.only_when_untamed && mob.is_tame() {
                return false;
            }
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
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            let Some(_living_target) = target.get_living_entity() else {
                return false;
            };

            self.track_target_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;
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

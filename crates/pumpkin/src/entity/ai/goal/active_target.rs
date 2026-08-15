use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::future::Future;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    target_type: Option<&'static EntityType>,
    target_types: Option<&'static [&'static EntityType]>,
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
            target_type: Some(target_type),
            target_types: None,
            target_predicate,
        }
    }

    pub fn new_types<F, Fut>(
        mob: &MobEntity,
        target_types: &'static [&'static EntityType],
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
            target_type: None,
            target_types: Some(target_types),
            target_predicate,
        }
    }

    #[must_use]
    pub fn with_default(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
    ) -> Box<Self> {
        Self::with_default_and_memory(mob, target_type, check_visibility, 60)
    }

    #[must_use]
    pub fn with_default_and_memory(
        mob: &MobEntity,
        target_type: &'static EntityType,
        check_visibility: bool,
        unseen_memory_ticks: i32,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let track_target_goal = track_target_goal.set_unseen_memory_ticks(unseen_memory_ticks);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type: Some(target_type),
            target_types: None,
            target_predicate,
        })
    }

    #[must_use]
    pub fn with_default_types(
        mob: &MobEntity,
        target_types: &'static [&'static EntityType],
        check_visibility: bool,
    ) -> Box<Self> {
        Self::with_default_types_and_memory(mob, target_types, check_visibility, 60)
    }

    #[must_use]
    pub fn with_default_types_and_memory(
        mob: &MobEntity,
        target_types: &'static [&'static EntityType],
        check_visibility: bool,
        unseen_memory_ticks: i32,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::with_default(check_visibility);
        let track_target_goal = track_target_goal.set_unseen_memory_ticks(unseen_memory_ticks);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type: None,
            target_types: Some(target_types),
            target_predicate,
        })
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

        let mut candidates: Vec<Arc<dyn EntityBase>> =
            if self.target_type == Some(&EntityType::PLAYER) && self.target_types.is_none() {
                world
                    .get_nearby_players(search_pos, follow_range)
                    .into_iter()
                    .map(|player: Arc<Player>| player as Arc<dyn EntityBase>)
                    .collect()
            } else if let Some(target_types) = self.target_types {
                world
                    .get_nearby_entities(search_pos, follow_range)
                    .into_values()
                    .filter(|entity| target_types.contains(&entity.get_entity().entity_type))
                    .collect()
            } else if let Some(target_type) = self.target_type {
                world
                    .get_nearby_entities(search_pos, follow_range)
                    .into_values()
                    .filter(|entity| entity.get_entity().entity_type == target_type)
                    .collect()
            } else {
                world
                    .get_nearby_entities(search_pos, follow_range)
                    .into_values()
                    .collect()
            };

        candidates.sort_by(|a, b| {
            a.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&search_pos)
                .total_cmp(
                    &b.get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&search_pos),
                )
        });

        for potential_entity in candidates {
            if let Some(living) = potential_entity.get_living_entity()
                && self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
                    .await
            {
                self.target = Some(potential_entity);
                return;
            }
        }
        self.target = None;
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

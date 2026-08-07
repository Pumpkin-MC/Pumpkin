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
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::future::Future;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

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
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
            target_type,
            target_predicate,
        })
    }

    pub fn set_target(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target = target;
    }

    async fn find_closest_target(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let follow_range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        // Vanilla updates the target conditions with the current follow distance on every search
        self.target_predicate.base_max_distance = follow_range;

        let world = mob_entity.living_entity.entity.world.load();

        // Vanilla searches using getEyeY(), so we offset the position by the eye height
        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;

        // Vanilla evaluates the target conditions per candidate inside the search, so the result is
        // the nearest *valid* target. Testing only the nearest candidate would make a single
        // invalid entity (for example an invulnerable creative player) hide every target behind it.
        // The predicate is async, so candidates are gathered first and tested in order.
        let sort_by_distance = |a: &Vector3<f64>, b: &Vector3<f64>| {
            a.squared_distance_to_vec(&search_pos)
                .partial_cmp(&b.squared_distance_to_vec(&search_pos))
                .unwrap()
        };

        self.target = if self.target_type == &EntityType::PLAYER {
            let mut candidates = world.get_nearby_players(search_pos, follow_range);
            candidates.sort_by(|a, b| {
                sort_by_distance(&a.get_entity().pos.load(), &b.get_entity().pos.load())
            });
            let mut result = None;
            for player in candidates {
                // Vanilla `TargetingConditions.test` (combat branch, `TargetingConditions.java:78`)
                // consults `targeter.canAttack(target)` before the rest of the predicate.
                if mob.can_attack(player.get_entity())
                    && self
                        .target_predicate
                        .test(
                            &world,
                            Some(&mob_entity.living_entity),
                            &player.living_entity,
                        )
                        .await
                {
                    result = Some(player as Arc<dyn EntityBase>);
                    break;
                }
            }
            result
        } else {
            let mut candidates: Vec<Arc<dyn EntityBase>> = world
                .get_nearby_entities(search_pos, follow_range)
                .into_values()
                .filter(|entity| entity.get_entity().entity_type == self.target_type)
                .collect();
            candidates.sort_by(|a, b| {
                sort_by_distance(&a.get_entity().pos.load(), &b.get_entity().pos.load())
            });
            let mut result = None;
            for entity in candidates {
                if let Some(living) = entity.get_living_entity()
                    && mob.can_attack(entity.get_entity())
                    && self
                        .target_predicate
                        .test(&world, Some(&mob_entity.living_entity), living)
                        .await
                {
                    result = Some(entity);
                    break;
                }
            }
            result
        };
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
            self.find_closest_target(mob).await;
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
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

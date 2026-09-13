use super::{Controls, Goal, to_goal_ticks};

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity, player::Player};
use crate::world::World;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use rand::RngExt;
use std::sync::Arc;

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

pub struct ActiveTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    /// `None` targets the closest entity of any type passing the predicate,
    /// matching vanilla's class-agnostic `NearestAttackableTargetGoal`;
    /// `Some` restricts the search to a single type.
    target_type: Option<&'static EntityType>,
    target_predicate: TargetPredicate,
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
        F: Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static,
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
            target_type: Some(target_type),
            target_predicate,
        })
    }

    /// Targets the closest entity of any type passing `predicate`, like vanilla's
    /// class-agnostic `NearestAttackableTargetGoal` (e.g. iron golems targeting
    /// every `Enemy` but creepers). Candidates are all tested, so an invalid
    /// entity nearest to the mob cannot block a valid one farther away.
    pub fn predicated(
        mob: &MobEntity,
        reciprocal_chance: i32,
        check_visibility: bool,
        predicate: impl Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static,
    ) -> Box<Self> {
        let track_target_goal = TrackTargetGoal::new(check_visibility, false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        target_predicate.set_predicate(predicate);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            target_type: None,
            target_predicate,
        })
    }

    pub fn set_target(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target = target;
    }

    fn find_closest_target(&mut self, mob: &MobEntity) {
        let follow_range = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        // Vanilla updates the target conditions with the current follow distance on every search
        self.target_predicate.base_max_distance = follow_range;

        let world = mob.living_entity.entity.world.load();

        // Vanilla searches using getEyeY(), so we offset the position by the eye height
        let mut search_pos = mob.living_entity.entity.pos.load();
        search_pos.y += mob.living_entity.entity.entity_dimension.load().eye_height as f64;

        if self.target_type == Some(&EntityType::PLAYER) {
            let potential_player = world
                .get_closest_player(search_pos, follow_range)
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>);

            if let Some(potential_entity) = potential_player
                && let Some(living) = potential_entity.get_living_entity()
                && self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
            {
                self.target = Some(potential_entity);
                return;
            }
        } else if let Some(target_type) = self.target_type {
            let potential_entity =
                world.get_closest_entity(search_pos, follow_range, Some(&[target_type]));

            if let Some(potential_entity) = potential_entity
                && let Some(living) = potential_entity.get_living_entity()
                && self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
            {
                self.target = Some(potential_entity);
                return;
            }
        } else {
            // Class-agnostic search: test every candidate and keep the closest
            // passing one, like vanilla's `getNearestEntity` with conditions.
            let entities = world.get_nearby_entities(search_pos, follow_range);
            let mut best: Option<(f64, Arc<dyn EntityBase>)> = None;
            for entity in entities.into_values() {
                let Some(living) = entity.get_living_entity() else {
                    continue;
                };
                if !self
                    .target_predicate
                    .test(&world, Some(&mob.living_entity), living)
                {
                    continue;
                }
                let dist_sq = search_pos.squared_distance_to_vec(&living.entity.pos.load());
                if best
                    .as_ref()
                    .is_none_or(|(best_dist, _)| dist_sq < *best_dist)
                {
                    best = Some((dist_sq, entity));
                }
            }
            self.target = best.map(|(_, entity)| entity);
            return;
        }
        self.target = None;
    }
}

impl Goal for ActiveTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.reciprocal_chance > 0
            && mob.get_random().random_range(0..self.reciprocal_chance) != 0
        {
            return false;
        }
        self.find_closest_target(mob.get_mob_entity());
        self.target.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());
        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}

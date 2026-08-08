use std::future::Future;
use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::{EntityBase, player::Player};

const DEFAULT_RECIPROCAL_CHANCE: i32 = 10;

/// Shared by `WolfEntity` and `CatEntity`: vanilla `Turtle.BABY_ON_LAND_SELECTOR` target type.
pub(crate) const TURTLE_TYPES: &[&EntityType] = &[&EntityType::TURTLE];

/// Vanilla `Turtle.BABY_ON_LAND_SELECTOR`: `target.isBaby() && !target.isInWater()`.
pub(crate) async fn baby_turtle_on_land(
    target: Arc<LivingEntity>,
    _world: Arc<crate::world::World>,
) -> bool {
    let entity = &target.entity;
    entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0
        && !entity
            .touching_water
            .load(std::sync::atomic::Ordering::SeqCst)
}

/// Makes a *non-tamed* `TamableAnimal` (wolf, cat) target a nearby entity of one of the given
/// types as prey.
///
/// Subject to an optional extra predicate; once the owner tames the mob this goal can never
/// `can_start` again. Vanilla source:
/// `net/minecraft/world/entity/ai/goal/target/NonTameRandomTargetGoal.java`
/// (`NearestAttackableTargetGoal` with `reciprocalChance = 10` plus the tamed check).
///
/// This is a thin `!is_tamed()` + multi-type wrapper around the same search/tracking machinery
/// as `ActiveTargetGoal` -- `ActiveTargetGoal` itself is left untouched since other mobs depend
/// on its exact (single-type) signature.
pub struct NonTameRandomTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    target_types: &'static [&'static EntityType],
    target_predicate: TargetPredicate,
    reciprocal_chance: i32,
}

impl NonTameRandomTargetGoal {
    pub fn new<F, Fut>(
        mob: &MobEntity,
        target_types: &'static [&'static EntityType],
        check_visibility: bool,
        predicate: Option<F>,
    ) -> Box<Self>
    where
        F: Fn(Arc<LivingEntity>, Arc<crate::world::World>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        let track_target_goal = TrackTargetGoal::new(check_visibility, false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        if let Some(predicate) = predicate {
            target_predicate.set_predicate(predicate);
        }

        Box::new(Self {
            track_target_goal,
            target: None,
            target_types,
            target_predicate,
            reciprocal_chance: to_goal_ticks(DEFAULT_RECIPROCAL_CHANCE),
        })
    }

    /// Convenience constructor for the common `subselector == null` case (e.g.
    /// `new NonTameRandomTargetGoal<>(this, Rabbit.class, false, null)`), which would otherwise
    /// need an explicit turbofish to disambiguate `None::<F>`.
    #[must_use]
    pub fn without_predicate(
        mob: &MobEntity,
        target_types: &'static [&'static EntityType],
        check_visibility: bool,
    ) -> Box<Self> {
        async fn always_true(_target: Arc<LivingEntity>, _world: Arc<crate::world::World>) -> bool {
            true
        }

        Self::new(mob, target_types, check_visibility, Some(always_true))
    }

    async fn find_closest_target(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let follow_range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        self.target_predicate.base_max_distance = follow_range;

        let world = mob_entity.living_entity.entity.world.load();
        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;

        let potential_entity = if self.target_types == [&EntityType::PLAYER].as_slice() {
            world
                .get_closest_player(search_pos, follow_range)
                .map(|p: Arc<Player>| p as Arc<dyn EntityBase>)
        } else {
            world.get_closest_entity(search_pos, follow_range, Some(self.target_types))
        };

        if let Some(potential_entity) = potential_entity
            && let Some(living) = potential_entity.get_living_entity()
            && mob.can_attack(potential_entity.get_entity())
            && self
                .target_predicate
                .test(&world, Some(&mob_entity.living_entity), living)
                .await
        {
            self.target = Some(potential_entity);
            return;
        }
        self.target = None;
    }
}

impl Goal for NonTameRandomTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Vanilla: `NonTameRandomTargetGoal.canUse` -- `!tamableMob.isTame() && super.canUse()`.
            if mob.get_mob_entity().is_tamed() {
                return false;
            }

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

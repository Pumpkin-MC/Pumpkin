use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityType, MobCategory};
use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::{Mob, MobEntity};

/// Vanilla `IronGolem.registerGoals`: `targetSelector.addGoal(3, new
/// NearestAttackableTargetGoal<>(this, Mob.class, 5, ...))` -- `reducedTickDelay(5)`.
const IRON_GOLEM_RECIPROCAL_CHANCE: i32 = 5;

/// Vanilla `SnowGolem.registerGoals`: `targetSelector.addGoal(1, new
/// NearestAttackableTargetGoal<>(this, Mob.class, 10, true, false, ...))`.
const SNOW_GOLEM_RECIPROCAL_CHANCE: i32 = 10;

/// Vanilla predicate base: `target instanceof Enemy`.
///
/// Pumpkin has no `Enemy` marker trait (see `armadillo_curl_up.rs`, which sets the same
/// precedent), so `MobCategory::MONSTER` stands in for it. This is broader than vanilla's
/// `Enemy` interface (e.g. it would also match a hostile-categorized mob that doesn't
/// implement `Enemy`, if one existed), but every current `MobCategory::MONSTER` entity in
/// Pumpkin's data does implement `Enemy` in vanilla, so the sets coincide today.
#[must_use]
pub fn is_hostile_target(entity_type: &EntityType) -> bool {
    entity_type.category == &MobCategory::MONSTER
}

/// Iron Golem's predicate additionally excludes creepers: `target instanceof Enemy &&
/// !(target instanceof Creeper)` (`IronGolem.java`).
#[must_use]
pub fn is_hostile_target_excluding_creeper(entity_type: &EntityType) -> bool {
    is_hostile_target(entity_type) && entity_type != &EntityType::CREEPER
}

/// Makes a mob attack the nearest hostile mob (an "any `Enemy`" target, as opposed to
/// `ActiveTargetGoal`'s single fixed `EntityType`).
///
/// Mirrors vanilla's `NearestAttackableTargetGoal<Mob>` with a `target instanceof Enemy`
/// predicate. Iron Golem additionally excludes creepers; Snow Golem does not.
///
/// Iron Golem's usage notably has **no village-proximity condition** in vanilla -- it
/// attacks any nearby hostile mob within follow range regardless of whether it is near a
/// village. The "defends the village" framing comes from where golems are usually found
/// (villages), not from a check in this goal.
///
/// This is a thin category-based sibling of `ActiveTargetGoal`, following the same
/// precedent as `non_tame_random_target.rs`: `ActiveTargetGoal` itself is left untouched
/// since ~30 other mobs depend on its exact single-`EntityType` matching semantics, and a
/// `Mob.class`-style "any hostile" filter doesn't fit that shape.
pub struct NearestHostileTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    exclude_creeper: bool,
    target_predicate: TargetPredicate,
}

impl NearestHostileTargetGoal {
    /// Iron Golem: `reciprocalChance = 5`, `checkSight = false`, excludes creepers.
    #[must_use]
    pub fn new(mob: &MobEntity) -> Box<Self> {
        Self::with_params(mob, IRON_GOLEM_RECIPROCAL_CHANCE, false, true)
    }

    /// Snow Golem: `reciprocalChance = 10`, `checkSight = true`, includes creepers.
    #[must_use]
    pub fn new_for_snow_golem(mob: &MobEntity) -> Box<Self> {
        Self::with_params(mob, SNOW_GOLEM_RECIPROCAL_CHANCE, true, false)
    }

    #[must_use]
    pub fn with_params(
        mob: &MobEntity,
        reciprocal_chance: i32,
        check_visibility: bool,
        exclude_creeper: bool,
    ) -> Box<Self> {
        // Vanilla: `checkCanNavigate = false` for both current callers.
        let track_target_goal = TrackTargetGoal::new(check_visibility, false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(reciprocal_chance),
            exclude_creeper,
            target_predicate,
        })
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

        // Gather-sort-iterate, same shape as `ActiveTargetGoal::find_closest_target`: the
        // target conditions are evaluated per candidate in distance order so a single
        // invalid nearby entity (e.g. an invulnerable creative player, though not relevant
        // for `Mob` targets specifically) can't hide a valid target behind it.
        let exclude_creeper = self.exclude_creeper;
        let mut candidates: Vec<Arc<dyn EntityBase>> = world
            .get_nearby_entities(search_pos, follow_range)
            .into_values()
            .filter(|entity| {
                let entity_type = entity.get_entity().entity_type;
                if exclude_creeper {
                    is_hostile_target_excluding_creeper(entity_type)
                } else {
                    is_hostile_target(entity_type)
                }
            })
            .collect();
        candidates.sort_by(|a, b| {
            let sq_dist = |e: &Arc<dyn EntityBase>| {
                e.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&search_pos)
            };
            sq_dist(a).partial_cmp(&sq_dist(b)).unwrap()
        });

        let mut result = None;
        for entity in candidates {
            // Vanilla `TargetingConditions.test` (combat branch, `TargetingConditions.java:78`)
            // consults `targeter.canAttack(target)` before the rest of the predicate.
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
        self.target = result;
    }
}

impl Goal for NearestHostileTargetGoal {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_monsters_are_targeted() {
        assert!(is_hostile_target(&EntityType::ZOMBIE));
        assert!(is_hostile_target(&EntityType::SKELETON));
    }

    #[test]
    fn creeper_is_hostile_but_excluded_for_iron_golem() {
        assert!(is_hostile_target(&EntityType::CREEPER));
        assert!(!is_hostile_target_excluding_creeper(&EntityType::CREEPER));
    }

    #[test]
    fn non_monsters_are_not_targeted() {
        assert!(!is_hostile_target(&EntityType::VILLAGER));
        assert!(!is_hostile_target(&EntityType::IRON_GOLEM));
        assert!(!is_hostile_target(&EntityType::COW));
        assert!(!is_hostile_target_excluding_creeper(&EntityType::VILLAGER));
    }
}

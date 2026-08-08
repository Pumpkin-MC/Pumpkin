use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::player::Player;

/// Vanilla `Ghast.registerGoals`: `targetSelector.addGoal(1, new NearestAttackableTargetGoal<>(
/// this, Player.class, 10, true, false, (target, level) -> Math.abs(target.getY() - this.getY())
/// <= 4.0))` (Ghast.java:60-61).
const RECIPROCAL_CHANCE: i32 = 10;
const MAX_VERTICAL_DISTANCE: f64 = 4.0;

/// Vanilla predicate: `Math.abs(target.getY() - this.getY()) <= 4.0` (Ghast.java:61).
#[must_use]
pub fn within_vertical_range(target_y: f64, ghast_y: f64) -> bool {
    (target_y - ghast_y).abs() <= MAX_VERTICAL_DISTANCE
}

/// Same precedent as `nearest_hostile_target.rs`/`non_tame_random_target.rs`.
///
/// `ActiveTargetGoal`'s custom-predicate plumbing (`TargetPredicate::set_predicate`) takes
/// closures over `Arc<LivingEntity>`, but candidates walked in `find_closest_target` are only
/// ever borrowed (`Player.living_entity` is a plain field, not `Arc<LivingEntity>`), so a
/// predicate set through it can never actually run. `ActiveTargetGoal` itself is left
/// untouched since ~30 other mobs depend on its exact single-`EntityType` matching semantics;
/// this is a thin, Ghast-specific sibling with the vertical-distance check applied directly
/// against borrowed data instead.
///
/// `within_vertical_range` is applied only at acquisition (`find_closest_target`), not on every
/// tick a target is held. This matches vanilla: `NearestAttackableTargetGoal` never overrides
/// `canContinueToUse`, and the base `TargetGoal.canContinueToUse`
/// (`net/minecraft/world/entity/ai/goal/target/TargetGoal.java:36-72`) checks only
/// null/`canAttack`/team/follow-distance/line-of-sight memory -- it does not re-run the
/// `TargetingConditions.Selector` (the y-distance predicate here). A player who acquires as a
/// target and then flies far above/below the ghast is *not* vanilla-dropped as a target for
/// that reason alone.
pub struct GhastNearestPlayerTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    reciprocal_chance: i32,
    target_predicate: TargetPredicate,
}

impl GhastNearestPlayerTargetGoal {
    #[must_use]
    pub fn new(mob: &MobEntity) -> Box<Self> {
        // Vanilla: `checkSight = true, checkCanNavigate = false`.
        let track_target_goal = TrackTargetGoal::new(true, false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        Box::new(Self {
            track_target_goal,
            target: None,
            reciprocal_chance: to_goal_ticks(RECIPROCAL_CHANCE),
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
        let ghast_y = mob_entity.living_entity.entity.pos.load().y;

        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;

        let mut candidates = world.get_nearby_players(search_pos, follow_range);
        candidates.sort_by(|a, b| {
            let sq_dist = |p: &Arc<Player>| {
                p.get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&search_pos)
            };
            sq_dist(a).partial_cmp(&sq_dist(b)).unwrap()
        });

        let mut result = None;
        for player in candidates {
            if within_vertical_range(player.get_entity().pos.load().y, ghast_y)
                && mob.can_attack(player.get_entity())
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
        self.target = result;
    }
}

impl Goal for GhastNearestPlayerTargetGoal {
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
    use super::within_vertical_range;

    #[test]
    fn within_four_blocks_is_a_valid_target() {
        assert!(within_vertical_range(10.0, 6.0));
        assert!(within_vertical_range(10.0, 14.0));
        assert!(within_vertical_range(10.0, 10.0));
    }

    #[test]
    fn beyond_four_blocks_is_rejected() {
        assert!(!within_vertical_range(10.0, 5.9));
        assert!(!within_vertical_range(10.0, 14.1));
    }
}

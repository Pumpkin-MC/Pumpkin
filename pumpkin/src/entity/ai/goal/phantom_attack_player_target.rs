//! Port of `Phantom.PhantomAttackPlayerTargetGoal` (`Phantom.java:216-249`).
//!
//! Extends `Goal` directly in vanilla (not `TargetGoal`), so no `Controls::TARGET` flag is
//! set here either.

use std::sync::{Arc, Weak};

use crate::entity::EntityBase;
use crate::entity::ai::goal::{Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::phantom::PhantomEntity;
use crate::entity::player::Player;

/// Vanilla: `TargetingConditions.forCombat().range(64.0)`.
const ATTACK_RANGE: f64 = 64.0;
/// Search radius passed to `getNearbyPlayers`. Vanilla filters by an asymmetric box
/// (`getBoundingBox().inflate(16.0, 64.0, 16.0)`); Pumpkin's player search is spherical, so
/// candidates are gathered with a generous superset radius and then filtered exactly against
/// that box below.
const CANDIDATE_SEARCH_RADIUS: f64 = 96.0;

pub struct PhantomAttackPlayerTargetGoal {
    phantom: Weak<PhantomEntity>,
    next_scan_tick: i32,
}

impl PhantomAttackPlayerTargetGoal {
    #[must_use]
    pub const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            next_scan_tick: to_goal_ticks(20),
        }
    }
}

/// Vanilla sorts candidates by Y descending and picks the first that passes the (looser,
/// unlimited-range) `canAttack` check - i.e. "highest valid player", not "nearest".
fn sort_players_by_height_descending(players: &mut [Arc<Player>]) {
    players.sort_by(|a, b| {
        b.get_entity()
            .pos
            .load()
            .y
            .partial_cmp(&a.get_entity().pos.load().y)
            .unwrap()
    });
}

impl Goal for PhantomAttackPlayerTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.next_scan_tick > 0 {
                self.next_scan_tick -= 1;
                return false;
            }
            self.next_scan_tick = to_goal_ticks(60);

            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            let entity = &phantom.mob_entity.living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let search_box = entity.bounding_box.load().expand(16.0, 64.0, 16.0);

            let attack_targeting =
                TargetPredicate::create_attackable().set_base_max_distance(ATTACK_RANGE);
            let default_targeting = TargetPredicate::create_attackable();

            let mut candidates: Vec<Arc<Player>> = Vec::new();
            for player in world.get_nearby_players(pos, CANDIDATE_SEARCH_RADIUS) {
                if !search_box.intersects(&player.get_entity().bounding_box.load()) {
                    continue;
                }
                if attack_targeting
                    .test(
                        &world,
                        Some(&phantom.mob_entity.living_entity),
                        &player.living_entity,
                    )
                    .await
                {
                    candidates.push(player);
                }
            }

            if candidates.is_empty() {
                return false;
            }

            sort_players_by_height_descending(&mut candidates);

            for player in candidates {
                if default_targeting
                    .test(
                        &world,
                        Some(&phantom.mob_entity.living_entity),
                        &player.living_entity,
                    )
                    .await
                {
                    let _ = mob;
                    phantom
                        .set_mob_target(Some(player as Arc<dyn EntityBase>))
                        .await;
                    return true;
                }
            }

            false
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(phantom) = self.phantom.upgrade() else {
                return false;
            };
            let target = phantom.mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            let Some(target_living) = target.get_living_entity() else {
                return false;
            };
            let world = phantom.mob_entity.living_entity.entity.world.load_full();
            TargetPredicate::create_attackable()
                .test(
                    &world,
                    Some(&phantom.mob_entity.living_entity),
                    target_living,
                )
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_cooldowns_match_vanilla_reduced_tick_delays() {
        assert_eq!(to_goal_ticks(20), 10);
        assert_eq!(to_goal_ticks(60), 30);
    }
}

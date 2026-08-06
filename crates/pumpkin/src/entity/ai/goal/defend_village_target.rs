use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::entity::EntityType;
use pumpkin_util::GameMode;

use super::track_target::TrackTargetGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::passive::iron_golem::IronGolemEntity;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::player::Player;

/// Vanilla `DefendVillageTargetGoal` (`target/DefendVillageTargetGoal.java`).
///
/// Makes an Iron Golem attack a player that any nearby villager holds a reputation of -100 or
/// lower against (`Villager.getPlayerReputation` <= -100, itself just
/// `gossips.getReputation(uuid, t -> true)` -- `Villager.java:675-677`).
///
/// Despite the class name this is not actually gated on `isCloseToVillage`/POI density at all --
/// vanilla only scans a small box around the golem itself (`getBoundingBox().inflate(10.0, 8.0,
/// 10.0)`, `DefendVillageTargetGoal.java:28`) for nearby villagers and players. The 64-block
/// `attackTargeting` range on the candidate-gathering conditions is inert here since the inflated
/// box is always smaller than that; it only matters for `TargetGoal.getFollowDistance()`
/// (`Attributes.FOLLOW_RANGE`, not overridden by this goal -- `TargetGoal.java:70-72`) gating
/// whether the goal keeps running once started.
///
/// `mustReach = true` in vanilla (`super(golem, false, true)`) would normally drop the target
/// once `TargetGoal`'s reach-cache decides it's unreachable. Pumpkin's `can_navigate_to_entity`
/// stub always answers "unreachable" (see `track_target.rs`), so passing that through as
/// `check_can_navigate: true` would drop the target one tick after every `start()` -- inverting
/// vanilla's normal-case outcome instead of approximating it. `false` is used instead, skipping
/// the check entirely, matching the outcome in the common case (golem and player both pathable)
/// until a real navigation hook exists for `TrackTargetGoal`.
pub struct DefendVillageTargetGoal {
    track_target_goal: TrackTargetGoal,
    potential_target: Option<Arc<dyn EntityBase>>,
}

impl DefendVillageTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            track_target_goal: TrackTargetGoal::new(false, false),
            potential_target: None,
        })
    }
}

/// Whether a player with the given reputation and gamemode is a valid `DefendVillageTargetGoal`
/// target for a golem that is not player-created (`IronGolemEntity::player_created` is checked
/// once per `can_start`, not per candidate, since it doesn't vary between candidates).
///
/// Pure decision core of `DefendVillageTargetGoal.canUse` (`DefendVillageTargetGoal.java:37-44`):
/// `reputation <= -100`, and never a spectator or creative player.
#[must_use]
const fn should_defend_against(reputation: i32, gamemode: GameMode) -> bool {
    if matches!(gamemode, GameMode::Spectator | GameMode::Creative) {
        return false;
    }
    reputation <= -100
}

impl Goal for DefendVillageTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.potential_target = None;

            // `IronGolem.canAttack`'s player-created gate (`IronGolem.java:136-141`) applies
            // to every player target this goal could ever pick, so check it once up front
            // rather than per-candidate.
            let player_created = mob
                .cast_any()
                .downcast_ref::<IronGolemEntity>()
                .is_some_and(|golem| golem.player_created.load(Ordering::Relaxed));
            if player_created {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let world = entity.world.load();

            let grow = entity.bounding_box.load().expand(10.0, 8.0, 10.0);

            let villagers: Vec<Arc<dyn EntityBase>> = world
                .get_entities_at_box(&grow)
                .into_iter()
                .filter(|e| {
                    e.get_entity().is_alive() && e.get_entity().entity_type == &EntityType::VILLAGER
                })
                .collect();
            if villagers.is_empty() {
                return false;
            }

            let players: Vec<Arc<Player>> = world
                .get_players_at_box(&grow)
                .into_iter()
                .filter(|p| p.get_entity().is_alive())
                .collect();
            if players.is_empty() {
                return false;
            }

            // Vanilla iterates every (villager, player) pair and keeps overwriting
            // `potentialTarget` on each match, so the *last* qualifying player in
            // iteration order wins, not the first (`DefendVillageTargetGoal.java:33-42`).
            let mut potential_target: Option<Arc<Player>> = None;
            for villager_entity in &villagers {
                let Some(villager) = villager_entity.cast_any().downcast_ref::<VillagerEntity>()
                else {
                    continue;
                };
                for player in &players {
                    let reputation = villager
                        .gossips
                        .lock()
                        .await
                        .get_reputation(player.gameprofile.id, |_| true);
                    if should_defend_against(reputation, player.gamemode.load()) {
                        potential_target = Some(player.clone());
                    }
                }
            }

            self.potential_target = potential_target.map(|player| player as Arc<dyn EntityBase>);
            self.potential_target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.potential_target.take()).await;
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
    use super::should_defend_against;
    use pumpkin_util::GameMode;

    #[test]
    fn reputation_at_threshold_is_a_valid_target() {
        assert!(should_defend_against(-100, GameMode::Survival));
    }

    #[test]
    fn reputation_above_threshold_is_not_a_valid_target() {
        assert!(!should_defend_against(-99, GameMode::Survival));
    }

    #[test]
    fn spectator_and_creative_players_are_never_valid_targets() {
        assert!(!should_defend_against(-100, GameMode::Spectator));
        assert!(!should_defend_against(-100, GameMode::Creative));
    }

    #[test]
    fn adventure_and_survival_players_can_be_valid_targets() {
        assert!(should_defend_against(-100, GameMode::Adventure));
        assert!(should_defend_against(-100, GameMode::Survival));
    }
}

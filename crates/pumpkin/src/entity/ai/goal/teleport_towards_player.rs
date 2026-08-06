use std::sync::Arc;

use super::track_target::TrackTargetGoal;
use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::{EndermanEntity, PLAYER_EYE_HEIGHT};
use crate::entity::player::Player;
use pumpkin_data::attributes::Attributes;

const STARE_CLOSE_DISTANCE_SQ: f64 = 16.0;
const TELEPORT_FAR_DISTANCE_SQ: f64 = 256.0;

pub struct TeleportTowardsPlayerGoal {
    enderman: Arc<EndermanEntity>,
    track_target_goal: TrackTargetGoal,
    target_player: Option<Arc<dyn EntityBase>>,
    committed_target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
    warmup: i32,
    unseen_ticks: i32,
}

impl TeleportTowardsPlayerGoal {
    pub fn new(enderman: Arc<EndermanEntity>) -> Self {
        let track_target_goal = TrackTargetGoal::with_default(false);
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = enderman
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        Self {
            enderman,
            track_target_goal,
            target_player: None,
            committed_target: None,
            target_predicate,
            warmup: 0,
            unseen_ticks: 0,
        }
    }

    async fn find_staring_player(&self) -> Option<Arc<Player>> {
        let entity = &self.enderman.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let follow_range = self
            .enderman
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        // The target conditions are evaluated per candidate so that a nearer but invalid player
        // (a spectator or an invulnerable creative player) does not hide a valid one behind it.
        // The predicate is async, so candidates are gathered first and tested in distance order.
        let mut candidates = world.get_nearby_players(pos, follow_range);
        candidates.sort_by(|a, b| {
            a.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&pos)
                .partial_cmp(&b.get_entity().pos.load().squared_distance_to_vec(&pos))
                .unwrap()
        });

        for player in candidates {
            if !self
                .target_predicate
                .test(
                    &world,
                    Some(&self.enderman.mob_entity.living_entity),
                    &player.living_entity,
                )
                .await
            {
                continue;
            }

            if self.enderman.is_player_staring(&player).await || self.enderman.is_angry() {
                return Some(player);
            }
        }

        None
    }
}

impl Goal for TeleportTowardsPlayerGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(player) = self.find_staring_player().await else {
                return false;
            };
            self.target_player = Some(player);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if let Some(target) = &self.target_player
                && let Some(player) = target.get_player()
            {
                if !self.enderman.is_player_staring(player).await && !self.enderman.is_angry() {
                    return false;
                }
                let player_pos = player.get_entity().pos.load();
                let mut look_control = mob.get_mob_entity().look_control.lock().unwrap();
                look_control.look_at_with_range(
                    player_pos.x,
                    player_pos.y + PLAYER_EYE_HEIGHT,
                    player_pos.z,
                    10.0,
                    10.0,
                );
                true
            } else if self.target_player.is_some() {
                false
            } else if let Some(target) = &self.committed_target {
                if !target.get_entity().is_alive() {
                    return false;
                }
                let mob_entity = mob.get_mob_entity();
                let dist_sq = mob_entity
                    .living_entity
                    .entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&target.get_entity().pos.load());
                let follow_range = mob_entity
                    .living_entity
                    .get_attribute_value(&Attributes::FOLLOW_RANGE);
                if dist_sq > follow_range * follow_range {
                    return false;
                }
                let needs_reset = mob_entity.target.lock().await.is_none();
                if needs_reset {
                    mob.set_mob_target(Some(target.clone())).await;
                }
                true
            } else {
                self.track_target_goal.should_continue(mob).await
            }
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.warmup = to_goal_ticks(5);
            self.unseen_ticks = 0;
            self.enderman.set_provoked(true);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let external_target = mob.get_mob_entity().target.lock().await.clone();
            if external_target.is_none()
                && self.target_player.is_none()
                && self.committed_target.is_none()
            {
                return;
            }

            if self.target_player.is_some() {
                self.warmup -= 1;
                if self.warmup <= 0 {
                    let target = self.target_player.take();
                    self.committed_target.clone_from(&target);
                    self.enderman.set_target(target).await;
                    self.track_target_goal.start(mob).await;
                }
                return;
            }

            let target = self.committed_target.clone().or(external_target);
            let Some(target) = target else {
                return;
            };

            let entity = &mob.get_mob_entity().living_entity.entity;
            let pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = pos.squared_distance_to_vec(&target_pos);

            if let Some(player) = target.get_player()
                && self.enderman.is_player_staring(player).await
            {
                if dist_sq < STARE_CLOSE_DISTANCE_SQ {
                    self.enderman.teleport_randomly();
                }
                self.unseen_ticks = 0;
            } else if dist_sq > TELEPORT_FAR_DISTANCE_SQ {
                self.unseen_ticks += 1;
                if self.unseen_ticks >= to_goal_ticks(30) {
                    self.enderman.teleport_towards(target.as_ref());
                    self.unseen_ticks = 0;
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target_player = None;
            self.committed_target = None;
            self.enderman.set_target(None).await;
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}

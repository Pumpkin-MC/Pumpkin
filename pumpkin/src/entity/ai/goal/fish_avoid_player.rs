use std::cmp::Ordering;
use std::sync::Arc;

use pumpkin_util::math::vector3::Vector3;

use super::{
    Controls, Goal, GoalFuture,
    fish_helpers::{find_water_target_away_from, has_reached_target, is_in_water, set_move_target},
};
use crate::entity::{EntityBase, mob::Mob, player::Player};

const DEFAULT_FLEE_DISTANCE: f64 = 8.0;
const FAST_DISTANCE_SQ: f64 = 49.0;
const SLOW_SPEED: f64 = 0.16;
const FAST_SPEED: f64 = 0.14;

pub struct FishAvoidPlayerGoal {
    goal_control: Controls,
    target: Option<Vector3<f64>>,
    threat_id: Option<i32>,
}

impl FishAvoidPlayerGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            target: None,
            threat_id: None,
        }
    }

    fn find_threat(mob: &dyn Mob) -> Option<Arc<Player>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let position = entity.pos.load();
        let world = entity.world.load();
        let players = world.get_nearby_players(position, DEFAULT_FLEE_DISTANCE);

        players
            .into_iter()
            .filter(|player| !player.is_spectator())
            .min_by(|a, b| {
                let a_dist_sq = a
                    .living_entity
                    .entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&position);
                let b_dist_sq = b
                    .living_entity
                    .entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&position);
                a_dist_sq.partial_cmp(&b_dist_sq).unwrap_or(Ordering::Equal)
            })
    }

    fn get_tracked_threat(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let threat_id = self.threat_id?;
        world
            .get_player_by_id(threat_id)
            .filter(|p| !p.is_spectator())
    }
}

impl Default for FishAvoidPlayerGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for FishAvoidPlayerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !is_in_water(mob) {
                return false;
            }

            let Some(threat) = Self::find_threat(mob) else {
                return false;
            };

            let threat_pos = threat.living_entity.entity.pos.load();
            self.target = find_water_target_away_from(mob, threat_pos, 16.0, 7, 10).await;
            self.threat_id = Some(threat.entity_id());
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !is_in_water(mob) {
                return false;
            }

            let Some(target) = self.target else {
                return false;
            };
            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            !has_reached_target(position, target)
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(threat) = self
                .get_tracked_threat(mob)
                .or_else(|| Self::find_threat(mob))
            else {
                self.target = None;
                self.threat_id = None;
                return;
            };
            self.threat_id = Some(threat.entity_id());

            let threat_pos = threat.living_entity.entity.pos.load();
            let position = mob.get_mob_entity().living_entity.entity.pos.load();

            if self
                .target
                .is_none_or(|target| has_reached_target(position, target))
            {
                self.target = find_water_target_away_from(mob, threat_pos, 16.0, 7, 10).await;
            }

            let Some(target) = self.target else {
                return;
            };

            let distance_sq = position.squared_distance_to_vec(&threat_pos);
            let speed = if distance_sq < FAST_DISTANCE_SQ {
                FAST_SPEED
            } else {
                SLOW_SPEED
            };
            set_move_target(mob, target, speed).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.threat_id = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

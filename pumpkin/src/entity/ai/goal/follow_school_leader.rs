//! Vanilla `FollowFlockLeaderGoal` — schooling fish trail a leader.
//!
//! Simplification: instead of the vanilla leader/follower registration (shared
//! mutable school state), a fish only follows the nearest same-type fish with a
//! lower entity id. That yields the same emergent schooling while staying
//! cycle-free without cross-entity bookkeeping.

use pumpkin_data::entity::EntityType;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

const SEARCH_RADIUS: f64 = 8.0;
/// Vanilla stops following beyond 121 (11 blocks squared).
const MAX_FOLLOW_DISTANCE_SQ: f64 = 121.0;
const MIN_FOLLOW_DISTANCE_SQ: f64 = 3.0 * 3.0;

pub struct FollowSchoolLeaderGoal {
    speed: f64,
    leader_id: Option<i32>,
    cooldown: i32,
    repath_delay: i32,
}

impl FollowSchoolLeaderGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            leader_id: None,
            cooldown: 0,
            repath_delay: 0,
        }
    }

    fn find_leader(mob: &dyn Mob) -> Option<i32> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let own_type: &'static EntityType = entity.entity_type;
        let own_id = entity.entity_id;

        let mut best: Option<(i32, f64)> = None;
        for (_, other) in world.get_nearby_entities(pos, SEARCH_RADIUS) {
            let other_entity = other.get_entity();
            // Lower ids lead: keeps the school hierarchy acyclic.
            if other_entity.entity_type != own_type
                || other_entity.entity_id >= own_id
                || !other_entity.is_alive()
            {
                continue;
            }
            let distance = other_entity.pos.load().squared_distance_to_vec(&pos);
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((other_entity.entity_id, distance));
            }
        }
        best.map(|(id, _)| id)
    }
}

impl Goal for FollowSchoolLeaderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            self.leader_id = Self::find_leader(mob);
            if self.leader_id.is_none() {
                // Vanilla nextInt(200) recheck pacing for leaderless fish.
                self.cooldown = to_goal_ticks(200);
                return false;
            }
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(leader_id) = self.leader_id else {
                return false;
            };
            let entity = mob.get_entity();
            let world = entity.world.load();
            let Some(leader) = world.get_entity_by_id(leader_id) else {
                return false;
            };
            if !leader.get_entity().is_alive() {
                return false;
            }
            let distance = entity
                .pos
                .load()
                .squared_distance_to_vec(&leader.get_entity().pos.load());
            (MIN_FOLLOW_DISTANCE_SQ..=MAX_FOLLOW_DISTANCE_SQ).contains(&distance)
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.repath_delay = 0;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.leader_id = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.repath_delay -= 1;
            if self.repath_delay > 0 {
                return;
            }
            self.repath_delay = to_goal_ticks(10);
            let Some(leader_id) = self.leader_id else {
                return;
            };
            let entity = mob.get_entity();
            let world = entity.world.load();
            let Some(leader) = world.get_entity_by_id(leader_id) else {
                return;
            };
            let mob_pos = entity.pos.load();
            let leader_pos = leader.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(mob_pos, leader_pos, self.speed));
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::{EntityBase, mob::Mob};

/// Vanilla: `FollowFlockLeaderGoal.INTERVAL_TICKS` -- base cooldown between school-forming
/// attempts (`200 + random.nextInt(200) % 20`).
const INTERVAL_TICKS: i32 = 200;
/// Search radius used both to find nearby same-species fish and (approximated) for
/// `AbstractSchoolingFish.inRangeOfLeader` (vanilla: `distanceToSqr(leader) <= 121.0`, i.e. 11
/// blocks -- reused here as the search radius too instead of a separate 8-block box).
const SCHOOL_RANGE: f64 = 8.0;
const IN_RANGE_OF_LEADER_SQ: f64 = 121.0;
/// Vanilla school sizes vary per species (`getMaxSchoolSize`, 4-8); approximated with a single
/// constant since Pumpkin doesn't track a `schoolSize` counter per fish.
const MAX_SCHOOL_SIZE: usize = 4;

/// Makes schooling fish (cod, salmon, tropical fish) cluster with, and follow, one "leader" fish
/// of the same species picked from nearby individuals.
///
/// Vanilla source: `net/minecraft/world/entity/ai/goal/FollowFlockLeaderGoal.java`, backed by
/// per-fish state in `net/minecraft/world/entity/animal/fish/AbstractSchoolingFish.java`
/// (`leader`/`schoolSize`/`isFollower`/`canBeFollowed`).
///
/// Scope reduction: vanilla's leader/follower bookkeeping is mutual and shared across every fish
/// in the school (`addFollowers` mutates the chosen leader's `schoolSize` and every follower's
/// `leader` reference, and a fish can itself become a leader others follow). Pumpkin's `Goal`s
/// only have access to their own mob, so this port approximates the same visual clustering with
/// a per-fish, independently-computed pseudo-leader: each fish picks the lowest-`entity_id` fish
/// of its own species within `SCHOOL_RANGE`, and follows it if that isn't itself. There is no
/// shared `schoolSize` cap enforcement beyond `MAX_SCHOOL_SIZE` counted on each tick's search,
/// and no `canBeFollowed`/`hasFollowers` leader-side gating.
pub struct FollowFlockLeaderGoal {
    goal_control: Controls,
    leader: Mutex<Option<Weak<dyn EntityBase>>>,
    next_start_tick: AtomicI32,
    time_to_recalc_path: AtomicI32,
}

impl FollowFlockLeaderGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE,
            leader: Mutex::new(None),
            next_start_tick: AtomicI32::new(0),
            time_to_recalc_path: AtomicI32::new(0),
        })
    }

    fn roll_next_start_tick(mob: &dyn Mob) -> i32 {
        to_goal_ticks(INTERVAL_TICKS + mob.get_random().random_range(0..200) % 20)
    }

    fn find_leader(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let self_entity = mob.get_entity();
        let self_uuid = self_entity.entity_uuid;
        let self_id = self_entity.entity_id;
        let self_type = self_entity.entity_type;
        let pos = self_entity.pos.load();
        let world = self_entity.world.load();

        let mut best: Option<Arc<dyn EntityBase>> = None;
        let mut best_id = self_id;
        let mut count = 1usize;

        for (uuid, candidate) in world.get_nearby_entities(pos, SCHOOL_RANGE) {
            if uuid == self_uuid {
                continue;
            }
            let candidate_entity = candidate.get_entity();
            if candidate_entity.entity_type != self_type || !candidate_entity.is_alive() {
                continue;
            }

            count += 1;
            if candidate_entity.entity_id < best_id {
                best_id = candidate_entity.entity_id;
                best = Some(candidate);
            }
        }

        if count > MAX_SCHOOL_SIZE || best_id == self_id {
            return None;
        }

        best
    }
}

impl Default for FollowFlockLeaderGoal {
    fn default() -> Self {
        *Self::new()
    }
}

impl Goal for FollowFlockLeaderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let remaining = self.next_start_tick.load(Ordering::Relaxed);
            if remaining > 0 {
                self.next_start_tick.fetch_sub(1, Ordering::Relaxed);
                return false;
            }
            self.next_start_tick
                .store(Self::roll_next_start_tick(mob), Ordering::Relaxed);

            let Some(leader) = Self::find_leader(mob) else {
                return false;
            };

            *self.leader.lock().await = Some(Arc::downgrade(&leader));
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(leader) = self.leader.lock().await.as_ref().and_then(Weak::upgrade) else {
                return false;
            };
            if !leader.get_entity().is_alive() {
                return false;
            }

            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&leader.get_entity().pos.load());
            dist_sq <= IN_RANGE_OF_LEADER_SQ
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.time_to_recalc_path.store(0, Ordering::Relaxed);
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            *self.leader.lock().await = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let remaining = self.time_to_recalc_path.fetch_sub(1, Ordering::Relaxed) - 1;
            if remaining > 0 {
                return;
            }
            self.time_to_recalc_path
                .store(to_goal_ticks(10), Ordering::Relaxed);

            let Some(leader) = self.leader.lock().await.as_ref().and_then(Weak::upgrade) else {
                return;
            };

            let pos = mob.get_entity().pos.load();
            let target: Vector3<f64> = leader.get_entity().pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal::new(pos, target, 1.0));
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

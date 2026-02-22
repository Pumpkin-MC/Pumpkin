use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use rand::RngExt;

use super::{
    Controls, Goal, GoalFuture,
    fish_helpers::{is_in_water, set_move_target},
    to_goal_ticks,
};
use crate::entity::{EntityBase, mob::Mob};

const SEARCH_RADIUS: f64 = 8.0;
const MAX_LEADER_DISTANCE_SQ: f64 = 121.0;
const RECALC_TICKS: i32 = 10;
const FOLLOW_SPEED: f64 = 0.1;

pub struct FollowSchoolLeaderGoal {
    goal_control: Controls,
    school_leader_id: Arc<AtomicI32>,
    time_to_recalc_path: i32,
    next_start_tick: i32,
}

impl FollowSchoolLeaderGoal {
    #[must_use]
    pub fn new(school_leader_id: Arc<AtomicI32>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            school_leader_id,
            time_to_recalc_path: 0,
            next_start_tick: to_goal_ticks(200),
        }
    }

    fn next_start_tick(mob: &dyn Mob) -> i32 {
        to_goal_ticks(200 + mob.get_random().random_range(0..200) % 20)
    }

    fn clear_leader(&self) {
        self.school_leader_id.store(0, Relaxed);
    }

    fn tracked_leader(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let leader_id = self.school_leader_id.load(Relaxed);
        if leader_id == 0 {
            return None;
        }

        let leader = world.get_entity_by_id(leader_id)?;
        if leader.get_entity().entity_type != entity.entity_type {
            return None;
        }
        if leader
            .get_living_entity()
            .is_some_and(|living| living.dead.load(Relaxed))
        {
            return None;
        }
        Some(leader)
    }

    fn find_nearest_same_type_leader(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();

        world
            .get_nearby_entities(pos, SEARCH_RADIUS)
            .into_values()
            .filter(|other| {
                other.get_entity().entity_id != entity.entity_id
                    && other.get_entity().entity_type == entity.entity_type
                    && other
                        .get_living_entity()
                        .is_some_and(|living| !living.dead.load(Relaxed))
            })
            .min_by(|a, b| {
                let a_dist_sq = a.get_entity().pos.load().squared_distance_to_vec(&pos);
                let b_dist_sq = b.get_entity().pos.load().squared_distance_to_vec(&pos);
                a_dist_sq
                    .partial_cmp(&b_dist_sq)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

impl Goal for FollowSchoolLeaderGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !is_in_water(mob) {
                return false;
            }

            if self.tracked_leader(mob).is_some() {
                return true;
            }

            if self.next_start_tick > 0 {
                self.next_start_tick -= 1;
                return false;
            }

            self.next_start_tick = Self::next_start_tick(mob);

            let Some(leader) = self.find_nearest_same_type_leader(mob) else {
                return false;
            };

            self.school_leader_id
                .store(leader.get_entity().entity_id, Relaxed);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !is_in_water(mob) {
                return false;
            }

            let Some(leader) = self.tracked_leader(mob) else {
                return false;
            };

            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            let leader_pos = leader.get_entity().pos.load();
            position.squared_distance_to_vec(&leader_pos) <= MAX_LEADER_DISTANCE_SQ
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.time_to_recalc_path = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.time_to_recalc_path -= 1;
            if self.time_to_recalc_path > 0 {
                return;
            }
            self.time_to_recalc_path = to_goal_ticks(RECALC_TICKS);

            let Some(leader) = self.tracked_leader(mob) else {
                self.clear_leader();
                return;
            };

            let target = leader.get_entity().pos.load();
            set_move_target(mob, target, FOLLOW_SPEED).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.clear_leader();
            self.time_to_recalc_path = 0;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

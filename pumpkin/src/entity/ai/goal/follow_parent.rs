use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};

const SEARCH_RADIUS: f64 = 8.0;
const MIN_DISTANCE_SQ: f64 = 9.0;
const MAX_DISTANCE_SQ: f64 = 256.0;

pub struct FollowParentGoal {
    goal_control: Controls,
    speed: f64,
    parent: Option<Arc<dyn EntityBase>>,
    delay: i32,
}

impl FollowParentGoal {
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            parent: None,
            delay: 0,
        }
    }

    fn find_parent(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let my_type = entity.entity_type;
        let world = entity.world.load();

        let nearby = world.get_nearby_entities(pos, SEARCH_RADIUS);
        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;

        for candidate in nearby.values() {
            let c_entity = candidate.get_entity();
            if c_entity.entity_type != my_type {
                continue;
            }
            if c_entity.age.load(Relaxed) < 0 {
                continue;
            }
            let dist_sq = pos.squared_distance_to_vec(&c_entity.pos.load());
            if dist_sq < MIN_DISTANCE_SQ {
                continue;
            }
            if closest.as_ref().is_none_or(|(d, _)| dist_sq < *d) {
                closest = Some((dist_sq, candidate.clone()));
            }
        }

        closest.map(|(_, e)| e)
    }
}

impl Goal for FollowParentGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let age = mob.get_mob_entity().living_entity.entity.age.load(Relaxed);
            if age >= 0 {
                return false;
            }
            self.parent = Self::find_parent(mob);
            self.parent.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let age = mob.get_mob_entity().living_entity.entity.age.load(Relaxed);
            if age >= 0 {
                return false;
            }
            let Some(parent) = &self.parent else {
                return false;
            };
            let parent_entity = parent.get_entity();
            if !parent_entity.is_alive() {
                return false;
            }
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let parent_pos = parent_entity.pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&parent_pos);
            (MIN_DISTANCE_SQ..=MAX_DISTANCE_SQ).contains(&dist_sq)
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.delay = 0;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.delay -= 1;
            if self.delay > 0 {
                return;
            }
            self.delay = to_goal_ticks(10);
            if let Some(parent) = &self.parent {
                let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let parent_pos = parent.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(mob_pos, parent_pos, self.speed));
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.parent = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

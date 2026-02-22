use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use rand::RngExt;

use super::{
    Controls, Goal, GoalFuture,
    fish_helpers::{find_random_water_target, has_reached_target, is_in_water, set_move_target},
};
use crate::entity::mob::Mob;

const SWIM_SPEED: f64 = 0.1;
const START_INTERVAL: i32 = 40;

pub struct FishSwimGoal {
    goal_control: Controls,
    school_leader_id: Arc<AtomicI32>,
    target: Option<pumpkin_util::math::vector3::Vector3<f64>>,
}

impl FishSwimGoal {
    #[must_use]
    pub fn new(school_leader_id: Arc<AtomicI32>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            school_leader_id,
            target: None,
        }
    }

    fn can_random_swim(&self) -> bool {
        self.school_leader_id.load(Relaxed) == 0
    }
}

impl Goal for FishSwimGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !self.can_random_swim() || !is_in_water(mob) {
                return false;
            }

            if mob.get_random().random_range(0..START_INTERVAL) != 0 {
                return false;
            }

            self.target = find_random_water_target(mob, 10.0, 7, 10).await;
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !self.can_random_swim() || !is_in_water(mob) {
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
            let Some(target) = self.target else {
                return;
            };
            let position = mob.get_mob_entity().living_entity.entity.pos.load();
            if has_reached_target(position, target) {
                self.target = None;
                return;
            }
            set_move_target(mob, target, SWIM_SPEED).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

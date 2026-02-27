use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const RANGE: i32 = 5;
const RECENT_DAMAGE_TICKS: i32 = 100;
const TARGET_ATTEMPTS: usize = 10;

pub struct EscapeDangerGoal {
    speed: f64,
    goal_control: Controls,
    target: Option<Vector3<f64>>,
}

impl EscapeDangerGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        Box::new(Self {
            speed,
            goal_control: Controls::MOVE,
            target: None,
        })
    }

    fn is_in_danger(mob: &dyn Mob) -> bool {
        let living = &mob.get_mob_entity().living_entity;

        if living.entity.fire_ticks.load(Relaxed) > 0 {
            return true;
        }

        let last_attacked = living.last_attacked_time.load(Relaxed);
        if last_attacked == 0 {
            return false;
        }
        let age = living.entity.age.load(Relaxed);
        age - last_attacked < RECENT_DAMAGE_TICKS
    }

    fn find_escape_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        for _ in 0..TARGET_ATTEMPTS {
            let dx = rng.random_range(-RANGE..=RANGE);
            let dz = rng.random_range(-RANGE..=RANGE);
            if dx == 0 && dz == 0 {
                continue;
            }
            return Some(Vector3::new(pos.x + dx as f64, pos.y, pos.z + dz as f64));
        }

        None
    }
}

impl Goal for EscapeDangerGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !Self::is_in_danger(mob) {
                return false;
            }
            self.target = Self::find_escape_target(mob);
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let navigator = mob.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target) = self.target {
                let pos = mob.get_mob_entity().living_entity.entity.pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal::new(pos, target, self.speed));
            }
        })
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

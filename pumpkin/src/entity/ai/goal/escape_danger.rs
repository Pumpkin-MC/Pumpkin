use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

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
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let fire_ticks = entity.fire_ticks.load(Relaxed);
        if fire_ticks > 0 {
            return true;
        }
        let age = entity.age.load(Relaxed);
        let last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
        last_attacked_time > 0 && (age - last_attacked_time) < 100
    }

    fn find_escape_target(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let dx = rng.random_range(-5.0..=5.0);
            let dz = rng.random_range(-5.0..=5.0);
            if dx == 0.0 && dz == 0.0 {
                continue;
            }
            return Some(Vector3::new(pos.x + dx, pos.y, pos.z + dz));
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

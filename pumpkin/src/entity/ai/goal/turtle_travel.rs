use std::sync::Weak;
use std::sync::atomic::Ordering::SeqCst;

use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, passive::turtle::TurtleEntity};

/// Vanilla: `Turtle.TurtleTravelGoal` (`Turtle.java:577-649`). While swimming, an adult turtle
/// occasionally roams out to a distant point instead of just drifting near its home beach.
///
/// Simplification: vanilla walks toward the travel point through `DefaultRandomPos`-biased
/// waypoints and aborts (`stuck = true`) if the target chunk region (`hasChunksAt`, a 69x69
/// area) isn't loaded. This port navigates directly to the travel point and never gets stuck,
/// since there's no equivalent "is this area loaded" check wired up for goals here.
pub struct TurtleTravelGoal {
    turtle: Weak<TurtleEntity>,
    speed: f64,
    travel_pos: Option<Vector3<f64>>,
}

impl TurtleTravelGoal {
    #[must_use]
    pub fn new(turtle: Weak<TurtleEntity>, speed: f64) -> Box<Self> {
        Box::new(Self {
            turtle,
            speed,
            travel_pos: None,
        })
    }
}

impl Goal for TurtleTravelGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return false;
            };
            if turtle.is_going_home() || turtle.has_egg() {
                return false;
            }
            mob.get_entity().touching_water.load(SeqCst)
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return false;
            };
            if turtle.is_going_home() || turtle.has_egg() {
                return false;
            }
            if turtle.get_mob_entity().is_in_love() {
                return false;
            }
            !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let my_pos = entity.pos.load();
            let world = entity.world.load();
            let mut rng = mob.get_random();

            let xt = f64::from(rng.random_range(-512i32..=512));
            let mut yt = f64::from(rng.random_range(-4i32..=4));
            let zt = f64::from(rng.random_range(-512i32..=512));

            if yt + my_pos.y > f64::from(world.sea_level - 1) {
                yt = 0.0;
            }

            self.travel_pos = Some(Vector3::new(my_pos.x + xt, my_pos.y + yt, my_pos.z + zt));
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.travel_pos = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.travel_pos else {
                return;
            };

            let navigator_idle = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
            if navigator_idle {
                let my_pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(my_pos, target, self.speed));
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

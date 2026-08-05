use std::sync::Weak;

use rand::RngExt;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::{
    ageable::AgeableMob, ai::pathfinder::NavigatorGoal, mob::Mob, passive::turtle::TurtleEntity,
};

/// `Turtle.TurtleGoHomeGoal.GIVE_UP_TICKS`.
const GIVE_UP_TICKS: i32 = 600;
/// Chance-per-tick roll (`reducedTickDelay(700)`) that decides whether a wandering turtle
/// starts heading home once it's far enough away.
const HOME_CHECK_INTERVAL: i32 = 700;
const FAR_FROM_HOME_DISTANCE: f64 = 64.0;
const CLOSE_TO_HOME_DISTANCE: f64 = 7.0;

/// Vanilla: `Turtle.TurtleGoHomeGoal` (`Turtle.java:329-396`). Makes the turtle path back to
/// its home beach after wandering far away, or immediately once it's carrying an egg.
///
/// Simplification: vanilla picks intermediate waypoints biased towards home via
/// `DefaultRandomPos.getPosTowards` (falling back through progressively wider search cones,
/// and giving up with `stuck = true` if none are found). This port instead re-issues a direct
/// navigator target at the home position whenever the navigator goes idle; it never gets
/// "stuck" since there's no pathfinding-candidate search to exhaust.
pub struct TurtleGoHomeGoal {
    turtle: Weak<TurtleEntity>,
    speed: f64,
    close_to_home_try_ticks: i32,
}

impl TurtleGoHomeGoal {
    #[must_use]
    pub fn new(turtle: Weak<TurtleEntity>, speed: f64) -> Box<Self> {
        Box::new(Self {
            turtle,
            speed,
            close_to_home_try_ticks: 0,
        })
    }

    fn distance_sq_to_home(mob: &dyn Mob, turtle: &TurtleEntity) -> Option<f64> {
        let home = turtle.get_home()?;
        let my_pos = mob.get_entity().pos.load();
        Some(my_pos.squared_distance_to_vec(&home.to_centered_f64()))
    }
}

impl Goal for TurtleGoHomeGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return false;
            };
            if turtle.is_baby() {
                return false;
            }
            if turtle.has_egg() {
                return true;
            }

            if mob
                .get_random()
                .random_range(0..to_goal_ticks(HOME_CHECK_INTERVAL))
                != 0
            {
                return false;
            }
            let Some(dist_sq) = Self::distance_sq_to_home(mob, &turtle) else {
                return false;
            };
            dist_sq > FAR_FROM_HOME_DISTANCE * FAR_FROM_HOME_DISTANCE
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return false;
            };
            let Some(dist_sq) = Self::distance_sq_to_home(mob, &turtle) else {
                return false;
            };
            dist_sq > CLOSE_TO_HOME_DISTANCE * CLOSE_TO_HOME_DISTANCE
                && self.close_to_home_try_ticks <= to_goal_ticks(GIVE_UP_TICKS)
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(turtle) = self.turtle.upgrade() {
                turtle.set_going_home(true);
            }
            self.close_to_home_try_ticks = 0;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(turtle) = self.turtle.upgrade() {
                turtle.set_going_home(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(turtle) = self.turtle.upgrade() else {
                return;
            };
            let Some(home) = turtle.get_home() else {
                return;
            };

            if Self::distance_sq_to_home(mob, &turtle).is_some_and(|d| d < 16.0 * 16.0) {
                self.close_to_home_try_ticks += 1;
            }

            let navigator_idle = mob.get_mob_entity().navigator.lock().unwrap().is_idle();
            if navigator_idle {
                let my_pos = mob.get_entity().pos.load();
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                navigator.set_progress(NavigatorGoal::new(
                    my_pos,
                    home.to_centered_f64(),
                    self.speed,
                ));
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

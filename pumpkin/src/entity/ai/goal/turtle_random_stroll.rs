use std::sync::Weak;
use std::sync::atomic::Ordering::SeqCst;

use super::wander_around::WanderAroundGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::{mob::Mob, passive::turtle::TurtleEntity};

/// Vanilla: `Turtle.TurtleRandomStrollGoal` (`Turtle.java:563-575`), a `RandomStrollGoal`
/// (interval 100) that only runs while the turtle is out of water, not heading home, and not
/// carrying an egg.
pub struct TurtleRandomStrollGoal {
    turtle: Weak<TurtleEntity>,
    inner: WanderAroundGoal,
}

impl TurtleRandomStrollGoal {
    #[must_use]
    pub fn new(turtle: Weak<TurtleEntity>, speed: f64) -> Box<Self> {
        Box::new(Self {
            turtle,
            inner: WanderAroundGoal::new_with_interval(speed, 100),
        })
    }

    fn guard_passes(&self, mob: &dyn Mob) -> bool {
        if mob.get_entity().touching_water.load(SeqCst) {
            return false;
        }
        let Some(turtle) = self.turtle.upgrade() else {
            return false;
        };
        !turtle.is_going_home() && !turtle.has_egg()
    }
}

impl Goal for TurtleRandomStrollGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !self.guard_passes(mob) {
                return false;
            }
            self.inner.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.should_continue(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.stop(mob)
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

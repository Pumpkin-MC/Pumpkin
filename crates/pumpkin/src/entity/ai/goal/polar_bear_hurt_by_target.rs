use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::mob::Mob;

/// `PolarBear.PolarBearHurtByTargetGoal` (PolarBear.java:282-302): a baby bear that gets hurt
/// alerts nearby adult bears (via `RevengeGoal::alert_only_adults`) but never fights back
/// itself.
///
/// An adult that gets hurt never alerts (the base `HurtByTargetGoal` only alerts when
/// `setAlertOthers()` was called, which this goal never does).
pub struct PolarBearHurtByTargetGoal {
    inner: RevengeGoal,
}

impl PolarBearHurtByTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            inner: RevengeGoal::new(true)
                .alert_only_adults()
                .alert_only_when_self_is_baby(),
        })
    }
}

impl Goal for PolarBearHurtByTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.can_start(mob)
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.should_continue(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.inner.start(mob).await;
            if mob.get_entity().age.load(Relaxed) < 0 {
                self.inner.stop(mob).await;
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.stop(mob)
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

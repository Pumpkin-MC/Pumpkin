use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::llama::llama_data_of;

/// `Llama.LlamaHurtByTargetGoal` (`Llama.java:474-488`).
///
/// A plain `HurtByTargetGoal` that refuses
/// to continue once the llama has just spit at something (the spit itself already picked a
/// target through the ranged-attack goal, so revenge-targeting the same hit shouldn't also fire).
pub struct LlamaHurtByTargetGoal {
    inner: RevengeGoal,
}

impl LlamaHurtByTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            inner: RevengeGoal::new(true),
        })
    }
}

impl Goal for LlamaHurtByTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.can_start(mob)
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if let Some(data) = llama_data_of(mob as &dyn EntityBase)
                && data.did_spit.swap(false, Relaxed)
            {
                return false;
            }
            self.inner.should_continue(mob).await
        })
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

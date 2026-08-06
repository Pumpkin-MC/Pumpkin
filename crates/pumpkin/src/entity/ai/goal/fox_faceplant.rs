use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::passive::fox::FoxEntity;

const FACEPLANT_TICKS: i32 = 40;

/// `Fox.FaceplantGoal`: holds the fox immobile for 40 ticks after a snow faceplant (set by
/// `FoxPounceGoal`'s landing check), then clears `isFaceplanted` back to `false`.
///
/// Without this goal nothing ever clears the flag PR-1 introduced -- `FoxPounceGoal` can set it
/// but never unsets it.
///
/// Uses the literal tick count rather than `get_tick_count`/`to_goal_ticks` -- this codebase's
/// `GoalSelector::tick` always calls `tick_goals(mob, true)` regardless of
/// `should_run_every_tick()`, so every running goal is ticked every real game tick with no
/// half-rate path to compensate for; `FoxSleepGoal`'s existing `WAIT_TIME_BEFORE_SLEEP` already
/// follows this same literal-ticks precedent.
pub struct FoxFaceplantGoal {
    countdown: i32,
}

impl FoxFaceplantGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self { countdown: 0 })
    }
}

impl Goal for FoxFaceplantGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.cast_any()
                .downcast_ref::<FoxEntity>()
                .is_some_and(FoxEntity::is_faceplanted)
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let faceplanted = mob
                .cast_any()
                .downcast_ref::<FoxEntity>()
                .is_some_and(FoxEntity::is_faceplanted);
            faceplanted && self.countdown > 0
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.countdown = FACEPLANT_TICKS;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(fox) = mob.cast_any().downcast_ref::<FoxEntity>() {
                fox.set_faceplanted(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.countdown -= 1;
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

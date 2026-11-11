//! Vanilla `Panda.PandaLieOnBackGoal`: a lazy panda now and then lies down on its back.

use std::sync::atomic::Ordering::Relaxed;

use rand::RngExt;

use super::{PandaEntity, as_panda};
use crate::entity::EntityBase;
use crate::entity::ai::goal::{Goal, GoalReach, to_goal_ticks};
use crate::entity::mob::Mob;

/// `canUse` starts on one in this many `nextInt` rolls.
const START_BOUND: i32 = 400;
/// `canContinueToUse` ends a non-lazy panda's lie on one in this many rolls.
const NOT_LAZY_END_BOUND: i32 = 600;
/// `canContinueToUse` ends any panda's lie on one in this many rolls.
const END_BOUND: i32 = 2000;
/// Server ticks the panda waits after lying down before it may do so again.
const COOLDOWN_TICKS: i32 = 200;

/// Vanilla `PandaLieOnBackGoal.canUse` after the cooldown, laziness and `canPerformAction`
/// checks. `next_int(bound)` stands for vanilla's `RandomSource.nextInt`; the bound goes
/// through `reducedTickDelay` because the selector asks every second tick.
fn rolls_lie_on_back(mut next_int: impl FnMut(i32) -> i32) -> bool {
    next_int(to_goal_ticks(START_BOUND)) == 1
}

/// Vanilla `PandaLieOnBackGoal.canContinueToUse`: a panda in the water gets up, a lazy panda
/// only rolls the long odds and any other panda rolls the shorter ones first. The rolls
/// short-circuit like vanilla's, so a panda that is out wins no extra draw.
fn keeps_lying(in_water: bool, lazy: bool, mut next_int: impl FnMut(i32) -> i32) -> bool {
    if in_water || (!lazy && next_int(to_goal_ticks(NOT_LAZY_END_BOUND)) == 1) {
        return false;
    }
    next_int(to_goal_ticks(END_BOUND)) != 1
}

/// Vanilla `PandaLieOnBackGoal` (goal priority 8, no control flags): the panda lies on its
/// back until the goal ends. `PandaMoveControl` keeps it from walking meanwhile.
#[derive(Default)]
pub struct PandaLieOnBackGoal {
    /// Vanilla `cooldown`: the `tick_count` the panda has to pass before it lies down again.
    cooldown: i32,
}

impl PandaLieOnBackGoal {
    /// The checks of vanilla `canUse` before the random roll.
    fn may_lie_down(&self, panda: &PandaEntity) -> bool {
        self.cooldown < panda.get_entity().tick_count.load(Relaxed)
            && panda.is_lazy()
            && panda.can_perform_action()
    }
}

impl Goal for PandaLieOnBackGoal {
    /// Vanilla `PandaLieOnBackGoal.canUse`: cooled down, lazy, able to act and a won roll.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        if !self.may_lie_down(panda) {
            return false;
        }
        let mut rng = mob.get_random();
        rolls_lie_on_back(|bound| rng.random_range(0..bound))
    }

    /// Vanilla `PandaLieOnBackGoal.canContinueToUse`.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        let mut rng = mob.get_random();
        keeps_lying(panda.get_entity().is_in_water(), panda.is_lazy(), |bound| {
            rng.random_range(0..bound)
        })
    }

    /// Vanilla `PandaLieOnBackGoal.start`: lie down and clear the cooldown.
    fn start(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob) {
            panda.set_on_back(true);
        }
        self.cooldown = 0;
    }

    /// Vanilla `PandaLieOnBackGoal.stop`: get up and wait before lying down again.
    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob) {
            panda.set_on_back(false);
            self.cooldown = panda.get_entity().tick_count.load(Relaxed) + COOLDOWN_TICKS;
        }
    }

    /// The goal only reads and writes the panda itself, so it reaches no other entity.
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach::default())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::scripted;
    use super::*;

    #[test]
    fn the_rolls_are_drawn_in_vanilla_order_and_a_lazy_panda_skips_the_short_one() {
        // `reducedTickDelay` halves the bounds 400, 600 and 2000.
        assert!(scripted(&[(200, 1)], |next| rolls_lie_on_back(next)));
        assert!(!scripted(&[(200, 0)], |next| rolls_lie_on_back(next)));

        let keeps = |lazy: bool, calls: &[(i32, i32)]| {
            scripted(calls, |next| keeps_lying(false, lazy, next))
        };
        assert!(keeps(true, &[(1000, 0)]));
        assert!(!keeps(true, &[(1000, 1)]));
        assert!(!keeps(false, &[(300, 1)]));
        assert!(keeps(false, &[(300, 0), (1000, 0)]));
        assert!(!keeps(false, &[(300, 0), (1000, 1)]));
        assert!(!scripted(&[], |next| keeps_lying(true, false, next)));
    }
}

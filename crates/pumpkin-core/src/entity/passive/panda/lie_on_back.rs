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
    use std::sync::Arc;

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::entity_variant::PandaGene;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn lazy_panda(world: &Arc<World>) -> Arc<PandaEntity> {
        let panda = PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::PANDA,
        ));
        panda.set_main_gene(PandaGene::Lazy);
        panda.set_hidden_gene(PandaGene::Lazy);
        panda
    }

    fn set_tick(panda: &PandaEntity, tick: i32) {
        panda.get_entity().tick_count.store(tick, Relaxed);
    }

    /// Runs a closure on scripted `(bound, result)` random calls and checks all were used.
    fn scripted<T>(calls: &[(i32, i32)], run: impl FnOnce(&mut dyn FnMut(i32) -> i32) -> T) -> T {
        let mut script = calls.iter();
        let mut next_int = |bound| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        };
        let result = run(&mut next_int);
        assert!(script.next().is_none(), "unused scripted random calls");
        result
    }

    #[test]
    fn starting_rolls_one_in_the_reduced_four_hundred() {
        assert!(scripted(&[(200, 1)], |next| rolls_lie_on_back(next)));
        assert!(!scripted(&[(200, 0)], |next| rolls_lie_on_back(next)));
        assert!(!scripted(&[(200, 2)], |next| rolls_lie_on_back(next)));
    }

    #[test]
    fn only_a_lazy_panda_skips_the_shorter_ending_roll_and_the_water_ends_it() {
        // A lazy panda rolls only the long odds.
        assert!(scripted(&[(1000, 0)], |next| keeps_lying(
            false, true, next
        )));
        assert!(!scripted(&[(1000, 1)], |next| keeps_lying(
            false, true, next
        )));
        // Any other panda rolls the shorter odds first; winning them ends the lie at once.
        assert!(!scripted(&[(300, 1)], |next| keeps_lying(
            false, false, next
        )));
        assert!(scripted(&[(300, 0), (1000, 0)], |next| keeps_lying(
            false, false, next
        )));
        assert!(!scripted(&[(300, 0), (1000, 1)], |next| keeps_lying(
            false, false, next
        )));
        // In the water the panda gets up without any roll.
        assert!(!scripted(&[], |next| keeps_lying(true, true, next)));
        assert!(!scripted(&[], |next| keeps_lying(true, false, next)));
    }

    #[tokio::test]
    async fn goal_needs_a_cooled_down_lazy_panda_that_can_act() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = lazy_panda(&world);
        let goal = PandaLieOnBackGoal::default();

        set_tick(&panda, 1);
        assert!(goal.may_lie_down(&panda));
        // The cooldown has to be passed, not just reached.
        let cooling = PandaLieOnBackGoal { cooldown: 5 };
        set_tick(&panda, 5);
        assert!(!cooling.may_lie_down(&panda));
        set_tick(&panda, 6);
        assert!(cooling.may_lie_down(&panda));

        panda.set_sitting(true);
        assert!(!goal.may_lie_down(&panda), "a sitting panda lay down");
        panda.set_sitting(false);
        panda.set_on_back(true);
        assert!(!goal.may_lie_down(&panda), "a panda already on its back");
        panda.set_on_back(false);

        panda.set_main_gene(PandaGene::Normal);
        panda.set_hidden_gene(PandaGene::Normal);
        assert!(
            !goal.may_lie_down(&panda),
            "a panda that is not lazy lay down"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn start_lies_down_and_stop_gets_up_and_waits_two_hundred_ticks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = lazy_panda(&world);
        let mut goal = PandaLieOnBackGoal { cooldown: 99 };
        panda.mob_init_data_tracker();

        goal.start(panda.as_ref());
        assert!(panda.is_on_back());
        assert!(!panda.can_perform_action());
        assert_eq!(goal.cooldown, 0, "start kept the cooldown");

        set_tick(&panda, 1000);
        goal.stop(panda.as_ref());
        assert!(!panda.is_on_back());
        assert!(panda.can_perform_action());
        assert_eq!(goal.cooldown, 1200);
        set_tick(&panda, 1200);
        assert!(!goal.may_lie_down(&panda));
        set_tick(&panda, 1201);
        assert!(goal.may_lie_down(&panda));
        world.level.shutdown().await;
        Ok(())
    }
}

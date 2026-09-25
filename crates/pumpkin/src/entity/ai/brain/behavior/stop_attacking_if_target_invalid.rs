use std::sync::Arc;

use crate::entity::EntityBase;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;
use super::utils::is_alive;

const TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE: i64 = 200;

#[must_use]
pub fn is_tired_of_trying_to_reach_target(time: i64, cant_reach_since: Option<i64>) -> bool {
    cant_reach_since.is_some_and(|since| time - since > TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE)
}

#[must_use]
pub fn stop_attacking_if_target_invalid(
    stop_attacking_when: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
    on_target_erased: impl Fn(&mut BrainTick<'_>, &Arc<dyn EntityBase>) + Send + Sync + 'static,
    can_grow_tired: bool,
) -> OneShot {
    OneShot::with_required(
        "StopAttackingIfTargetInvalid",
        vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
        vec![types::CANT_REACH_WALK_TARGET_SINCE.id()],
        move |tick| {
            let Some(target) = tick.brain.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            let can_attack = target
                .get_living_entity()
                .is_some_and(|living| tick.mob.can_attack(living));
            let tired = can_grow_tired
                && is_tired_of_trying_to_reach_target(
                    tick.time,
                    tick.brain.get(types::CANT_REACH_WALK_TARGET_SINCE).copied(),
                );
            let same_world = Arc::ptr_eq(&target.get_entity().world.load_full(), tick.world);

            if can_attack
                && !tired
                && is_alive(target.as_ref())
                && same_world
                && !stop_attacking_when(tick, &target)
            {
                return true;
            }

            on_target_erased(tick, &target);
            tick.brain.erase(types::ATTACK_TARGET.id());
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use super::is_tired_of_trying_to_reach_target;

    #[test]
    fn a_target_never_out_of_reach_is_never_given_up_on() {
        assert!(!is_tired_of_trying_to_reach_target(10_000, None));
    }

    #[test]
    fn giving_up_takes_more_than_two_hundred_ticks_out_of_reach() {
        assert!(!is_tired_of_trying_to_reach_target(1_199, Some(1_000)));
        assert!(!is_tired_of_trying_to_reach_target(1_200, Some(1_000)));
        assert!(is_tired_of_trying_to_reach_target(1_201, Some(1_000)));
    }
}

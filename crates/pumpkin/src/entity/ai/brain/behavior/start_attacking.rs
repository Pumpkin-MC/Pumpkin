use std::sync::Arc;

use crate::entity::EntityBase;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn start_attacking(
    can_attack: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    find_target: impl Fn(&BrainTick<'_>) -> Option<Arc<dyn EntityBase>> + Send + Sync + 'static,
) -> OneShot {
    OneShot::with_required(
        "StartAttacking",
        vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent)],
        vec![types::CANT_REACH_WALK_TARGET_SINCE.id()],
        move |tick| {
            if !can_attack(tick) {
                return false;
            }
            let Some(target) = find_target(tick) else {
                return false;
            };
            let Some(living) = target.get_living_entity() else {
                return false;
            };
            if !tick.mob.can_attack(living) {
                return false;
            }
            tick.brain.set(types::ATTACK_TARGET, target);
            tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
            true
        },
    )
}

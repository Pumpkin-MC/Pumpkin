use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;

/// Sets the attack target from `target_finder` while `can_attack` holds.
pub fn create(
    can_attack: fn(&BrainTick<'_>) -> bool,
    target_finder: fn(&BrainTick<'_>) -> Option<Arc<dyn EntityBase>>,
) -> OneShot {
    OneShot::new(
        "StartAttacking",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::CANT_REACH_WALK_TARGET_SINCE.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            if !can_attack(tick) {
                return false;
            }
            let Some(target) = target_finder(tick) else {
                return false;
            };
            if !tick.mob.can_attack(target.as_ref()) {
                return false;
            }
            tick.state.set(types::ATTACK_TARGET, target);
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
            true
        },
    )
}

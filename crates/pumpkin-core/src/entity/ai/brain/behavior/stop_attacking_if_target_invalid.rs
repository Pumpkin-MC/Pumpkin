use std::sync::Arc;

use super::super::memory::{MemoryStatus, types};
use super::OneShot;

const TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE: i64 = 200;

/// Drops the attack target once it can no longer be attacked, died, left the world or
/// stayed out of reach for too long.
#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "StopAttackingIfTargetInvalid",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (
                types::CANT_REACH_WALK_TARGET_SINCE.id(),
                MemoryStatus::Registered,
            ),
        ],
        |tick| {
            let Some(target) = tick.state.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            let tired = tick
                .state
                .get(types::CANT_REACH_WALK_TARGET_SINCE)
                .is_some_and(|since| tick.time - since > TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE);
            let still_valid = tick.mob.can_attack(target.as_ref())
                && !tired
                && target
                    .get_living_entity()
                    .is_some_and(crate::entity::living::LivingEntity::is_alive)
                && Arc::ptr_eq(&target.get_entity().world.load_full(), tick.world);
            if !still_valid {
                tick.state.erase(types::ATTACK_TARGET.id());
            }
            true
        },
    )
}

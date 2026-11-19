use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::{OneShot, utils};
use crate::entity::mob::Mob;

const CLOSE_ENOUGH_DISTANCE: i32 = 0;

#[must_use]
pub fn create(speed_modifier: fn(&dyn Mob) -> f32) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromAttackTargetIfTargetOutOfReach",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(target) = tick.state.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            if utils::entity_is_visible(&tick.visibility(), target.as_ref())
                && utils::is_within_attack_range(tick.mob, target.as_ref())
            {
                tick.state.erase(types::WALK_TARGET.id());
            } else {
                utils::look_at_entity(tick.state, target.clone());
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_entity(
                        target,
                        speed_modifier(tick.mob),
                        CLOSE_ENOUGH_DISTANCE,
                    ),
                );
            }
            true
        },
    )
}

use std::sync::Arc;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;
use super::utils::is_within_attack_range;

const PROJECTILE_ATTACK_RANGE_BUFFER: i32 = 1;

#[must_use]
pub fn set_walk_target_from_attack_target_if_target_out_of_reach(speed_modifier: f32) -> OneShot {
    OneShot::with_required(
        "SetWalkTargetFromAttackTargetIfTargetOutOfReach",
        vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
        vec![
            types::WALK_TARGET.id(),
            types::LOOK_TARGET.id(),
            types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
        ],
        move |tick| {
            let (target, in_reach) = {
                let ctx = tick.visibility();
                let Some(target) = ctx.brain.get(types::ATTACK_TARGET).map(Arc::clone) else {
                    return false;
                };
                let visible = ctx
                    .brain
                    .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                    .is_some_and(|visible| visible.contains(target.as_ref(), &ctx));
                let in_reach = visible
                    && is_within_attack_range(
                        ctx.mob,
                        target.as_ref(),
                        PROJECTILE_ATTACK_RANGE_BUFFER,
                    );
                (target, in_reach)
            };

            if in_reach {
                tick.brain.erase(types::WALK_TARGET.id());
            } else {
                tick.brain.set(
                    types::LOOK_TARGET,
                    Arc::new(EntityTracker::new(Arc::clone(&target), true))
                        as Arc<dyn PositionTracker>,
                );
                tick.brain.set(
                    types::WALK_TARGET,
                    WalkTarget::new(
                        Arc::new(EntityTracker::new(target, false)),
                        speed_modifier,
                        0,
                    ),
                );
            }
            true
        },
    )
}

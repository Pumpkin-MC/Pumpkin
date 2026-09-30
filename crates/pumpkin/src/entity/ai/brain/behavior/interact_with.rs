use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::EntityBase;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleType, MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn interact_with(
    entity_type: &'static EntityType,
    interaction_range: i32,
    self_filter: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    target_filter: impl Fn(&Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
    interaction_target: MemoryModuleType<Arc<dyn EntityBase>>,
    speed_modifier: f32,
    stop_distance: i32,
) -> OneShot {
    let range_squared = f64::from(interaction_range * interaction_range);
    OneShot::with_required(
        "InteractWith",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        vec![interaction_target.id(), types::LOOK_TARGET.id()],
        move |tick| {
            if !self_filter(tick) {
                return false;
            }
            let body_pos = tick.mob.get_entity().pos.load();
            let (any_valid, closest) = {
                let ctx = tick.visibility();
                let Some(visible) = ctx.brain.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
                    return false;
                };
                let is_valid = |entity: &Arc<dyn EntityBase>| {
                    entity.get_entity().entity_type == entity_type && target_filter(entity)
                };
                let any_valid = visible.contains_any(&ctx, is_valid);
                let closest = visible.find_closest(&ctx, |entity| {
                    entity
                        .get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&body_pos)
                        <= range_squared
                        && is_valid(entity)
                });
                (any_valid, closest)
            };
            if !any_valid {
                return false;
            }

            if let Some(target) = closest {
                tick.brain.set(interaction_target, Arc::clone(&target));
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
                        stop_distance,
                    ),
                );
            }
            true
        },
    )
}

use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::super::memory::walk_target::WalkTarget;
use super::super::super::memory::{MemoryModuleType, MemoryStatus, types};
use super::super::OneShot;
use crate::entity::EntityBase;
use crate::entity::mob::Mob;

#[must_use]
pub fn of(
    entity_type: &'static EntityType,
    interaction_range: i32,
    interaction_target: MemoryModuleType<Arc<dyn EntityBase>>,
    speed_modifier: f32,
    stop_distance: i32,
) -> OneShot {
    of_filtered(
        entity_type,
        interaction_range,
        |_| true,
        |_| true,
        interaction_target,
        speed_modifier,
        stop_distance,
    )
}

#[must_use]
pub fn of_filtered(
    entity_type: &'static EntityType,
    interaction_range: i32,
    self_filter: fn(&dyn Mob) -> bool,
    target_filter: fn(&dyn EntityBase) -> bool,
    interaction_target: MemoryModuleType<Arc<dyn EntityBase>>,
    speed_modifier: f32,
    stop_distance: i32,
) -> OneShot {
    let interaction_range_sqr = f64::from(interaction_range * interaction_range);
    OneShot::new(
        "InteractWith",
        vec![
            (interaction_target.id(), MemoryStatus::Registered),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            if !self_filter(tick.mob) {
                return false;
            }
            let position = tick.mob.get_entity().pos.load();
            let closest = {
                let ctx = tick.visibility();
                let Some(entities) = tick.state.get(types::NEAREST_VISIBLE_LIVING_ENTITIES) else {
                    return false;
                };
                let is_valid = |target: &Arc<dyn EntityBase>| {
                    target.get_entity().entity_type == entity_type && target_filter(target.as_ref())
                };
                if !entities.contains_any(&ctx, is_valid) {
                    return false;
                }
                entities.find_closest(&ctx, |target| {
                    is_valid(target)
                        && target
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&position)
                            <= interaction_range_sqr
                })
            };
            if let Some(target) = closest {
                tick.state.set(interaction_target, target.clone());
                tick.state.set(
                    types::LOOK_TARGET,
                    Arc::new(EntityTracker::new(target.clone(), true)) as Arc<dyn PositionTracker>,
                );
                tick.state.set(
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

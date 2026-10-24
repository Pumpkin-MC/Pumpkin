use std::sync::Arc;

use super::super::memory::{
    MemoryStatus, position_tracker::EntityTracker, types, walk_target::WalkTarget,
};
use super::{OneShot, utils};

#[must_use]
pub fn create(speed_modifier: f32, interrupt_ongoing_walk: bool, max_dist_to_walk: i32) -> OneShot {
    OneShot::new(
        "GoToWantedItem",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (
                types::WALK_TARGET.id(),
                if interrupt_ongoing_walk {
                    MemoryStatus::Registered
                } else {
                    MemoryStatus::ValueAbsent
                },
            ),
            (
                types::NEAREST_VISIBLE_WANTED_ITEM.id(),
                MemoryStatus::ValuePresent,
            ),
            (
                types::ITEM_PICKUP_COOLDOWN_TICKS.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(item) = tick.state.get(types::NEAREST_VISIBLE_WANTED_ITEM).cloned() else {
                return false;
            };
            let item_entity = item.get_entity();
            let item_block = item_entity.block_pos.load();
            if tick
                .state
                .has_memory_value(types::ITEM_PICKUP_COOLDOWN_TICKS.id())
                || item_entity
                    .pos
                    .load()
                    .squared_distance_to_vec(&tick.mob.get_entity().pos.load())
                    >= f64::from(max_dist_to_walk * max_dist_to_walk)
                || !tick
                    .world
                    .worldborder
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .contains_block(item_block.0.x, item_block.0.z)
                || !tick.mob.can_pick_up_loot(Some(tick.state))
            {
                return false;
            }
            utils::look_at_entity(tick.state, item.clone());
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(Arc::new(EntityTracker::new(item, false)), speed_modifier, 0),
            );
            true
        },
    )
}

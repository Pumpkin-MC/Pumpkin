use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn go_to_wanted_item(
    predicate: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    speed_modifier: f32,
    interrupt_ongoing_walk: bool,
    max_dist_to_walk: i32,
) -> OneShot {
    let max_dist_squared = f64::from(max_dist_to_walk * max_dist_to_walk);
    let walk_condition = if interrupt_ongoing_walk {
        Vec::new()
    } else {
        vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)]
    };
    let mut conditions = vec![(
        types::NEAREST_VISIBLE_WANTED_ITEM.id(),
        MemoryStatus::ValuePresent,
    )];
    conditions.extend(walk_condition);

    OneShot::with_required(
        "GoToWantedItem",
        conditions,
        vec![
            types::LOOK_TARGET.id(),
            types::WALK_TARGET.id(),
            types::ITEM_PICKUP_COOLDOWN_TICKS.id(),
        ],
        move |tick| {
            if tick
                .brain
                .has_memory_value(types::ITEM_PICKUP_COOLDOWN_TICKS.id())
                || !predicate(tick)
                || !tick.mob.get_mob_entity().can_pick_up_loot()
            {
                return false;
            }
            let Some(item) = tick.brain.get(types::NEAREST_VISIBLE_WANTED_ITEM).cloned() else {
                return false;
            };
            let item_entity = item.get_entity();
            let body_pos = tick.mob.get_entity().pos.load();
            if item_entity.pos.load().squared_distance_to_vec(&body_pos) > max_dist_squared {
                return false;
            }
            let item_block_pos = item_entity.block_pos.load();
            if !tick
                .world
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains_block(item_block_pos.0.x, item_block_pos.0.z)
            {
                return false;
            }

            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(Arc::clone(&item), true)) as Arc<dyn PositionTracker>,
            );
            tick.brain.set(
                types::WALK_TARGET,
                WalkTarget::new(Arc::new(EntityTracker::new(item, false)), speed_modifier, 0),
            );
            true
        },
    )
}

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, entities_in_inflated_box};

const XZ_RANGE: f64 = 32.0;
const Y_RANGE: f64 = 16.0;
const MAX_DISTANCE_TO_WANTED_ITEM: f64 = 32.0;

const REQUIRES: &[MemoryModuleId] = &[types::NEAREST_VISIBLE_WANTED_ITEM.id()];

pub struct NearestItemSensor;

impl Sensor for NearestItemSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let mob = tick.mob;
        let brain: &super::super::Brain = tick.brain;
        let body = mob.get_entity();
        let body_pos = body.pos.load();
        let max_distance_squared = MAX_DISTANCE_TO_WANTED_ITEM * MAX_DISTANCE_TO_WANTED_ITEM;

        let items =
            entities_in_inflated_box(tick.world, body, XZ_RANGE, Y_RANGE, XZ_RANGE, |entity| {
                let Some(item) = entity.get_item_entity() else {
                    return false;
                };
                if entity
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_pos)
                    > max_distance_squared
                {
                    return false;
                }
                let stack = item
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                mob.wants_to_pick_up(brain, &stack)
            });

        let nearest = items
            .into_iter()
            .find(|item| mob.has_line_of_sight(item.get_entity()));
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_WANTED_ITEM, nearest);
    }
}

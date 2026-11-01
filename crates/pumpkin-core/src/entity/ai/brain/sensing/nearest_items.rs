use super::super::{
    BrainTick,
    memory::{MemoryModuleId, types},
};
use super::Sensor;
use crate::entity::item::ItemEntity;

const XZ_RANGE: f64 = 32.0;
const Y_RANGE: f64 = 16.0;
const MAX_DISTANCE_TO_WANTED_ITEM: f64 = 32.0;

const REQUIRES: [MemoryModuleId; 1] = [types::NEAREST_VISIBLE_WANTED_ITEM.id()];

pub struct NearestItemSensor;

impl Sensor for NearestItemSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let entity = tick.mob.get_entity();
        let position = entity.pos.load();
        let mut items: Vec<_> = tick
            .world
            .get_entities_at_box(
                &entity
                    .bounding_box
                    .load()
                    .expand(XZ_RANGE, Y_RANGE, XZ_RANGE),
            )
            .into_iter()
            .filter(|entity| entity.cast_any().is::<ItemEntity>())
            .collect();
        items.sort_by(|a, b| {
            position
                .squared_distance_to_vec(&a.get_entity().pos.load())
                .total_cmp(&position.squared_distance_to_vec(&b.get_entity().pos.load()))
        });
        let target = items.into_iter().find(|candidate| {
            candidate
                .cast_any()
                .downcast_ref::<ItemEntity>()
                .is_some_and(|item| {
                    let stack = item
                        .get_item_stack()
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone();
                    tick.mob.wants_to_pick_up(&stack)
                })
                && position.squared_distance_to_vec(&candidate.get_entity().pos.load())
                    < MAX_DISTANCE_TO_WANTED_ITEM * MAX_DISTANCE_TO_WANTED_ITEM
                && tick.mob.has_line_of_sight(candidate.get_entity())
        });
        tick.state
            .set_optional(types::NEAREST_VISIBLE_WANTED_ITEM, target);
    }
}

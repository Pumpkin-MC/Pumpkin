use pumpkin_data::entity::EntityType;
use rand::RngExt;

use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};
use crate::entity::EntityBase;

#[must_use]
pub fn create(max_dist: f32, min_interval: i32, max_interval: i32) -> OneShot {
    create_filtered(max_dist, min_interval, max_interval, |_| true)
}

#[must_use]
pub fn create_for_type(
    entity_type: &'static EntityType,
    max_dist: f32,
    min_interval: i32,
    max_interval: i32,
) -> OneShot {
    create_filtered(max_dist, min_interval, max_interval, move |target| {
        target.get_entity().entity_type == entity_type
    })
}

fn create_filtered(
    max_dist: f32,
    min_interval: i32,
    max_interval: i32,
    predicate: impl Fn(&dyn EntityBase) -> bool + Send + Sync + 'static,
) -> OneShot {
    let max_dist_sqr = f64::from(max_dist * max_dist);
    let mut ticks_until_next_start = 0;
    OneShot::new(
        "SetEntityLookTargetSometimes",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let position = tick.mob.get_entity().pos.load();
            let target = tick
                .state
                .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                .and_then(|visible| {
                    visible.find_closest(&tick.visibility(), |target| {
                        predicate(target.as_ref())
                            && target
                                .get_entity()
                                .pos
                                .load()
                                .squared_distance_to_vec(&position)
                                <= max_dist_sqr
                    })
                });
            let Some(target) = target else {
                return false;
            };
            if ticks_until_next_start == 0 {
                ticks_until_next_start = tick
                    .mob
                    .get_random()
                    .random_range(min_interval..=max_interval)
                    - 1;
                return false;
            }
            ticks_until_next_start -= 1;
            if ticks_until_next_start != 0 {
                return false;
            }
            utils::look_at_entity(tick.state, target);
            true
        },
    )
}

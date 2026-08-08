//! Port of `sensing/NearestItemSensor.java`.

use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::memory::{MemoryKeyId, NearestVisibleWantedItemMemory};
use crate::entity::ai::brain::sensor::{Sensor, SensorFuture, randomly_delayed_start};
use crate::entity::mob::Mob;

/// `NearestItemSensor.XZ_RANGE` / `Y_RANGE` / `MAX_DISTANCE_TO_WANTED_ITEM` (`:15-17`).
const XZ_RANGE: f64 = 32.0;
const Y_RANGE: f64 = 16.0;
const MAX_DISTANCE_TO_WANTED_ITEM: f64 = 32.0;

const REQUIRES: [MemoryKeyId; 1] = [MemoryKeyId::NearestVisibleWantedItem];

pub struct NearestItemSensor {
    ticks_until_scan: i64,
}

impl NearestItemSensor {
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> Box<dyn Sensor> {
        Box::new(Self {
            ticks_until_scan: randomly_delayed_start(20),
        })
    }
}

impl Sensor for NearestItemSensor {
    /// `requires()` (`NearestItemSensor.java:19-22`).
    fn requires(&self) -> &[MemoryKeyId] {
        &REQUIRES
    }

    fn ticks_until_scan(&mut self) -> &mut i64 {
        &mut self.ticks_until_scan
    }

    /// `doTick` (`NearestItemSensor.java:24-34`): collect item entities in an inflated box,
    /// sort by squared distance, keep the first that the mob wants, is within 32 blocks, and
    /// has line of sight to; write it into `NEAREST_VISIBLE_WANTED_ITEM`.
    fn do_tick<'a>(&'a mut self, mob: &'a dyn Mob, brain: &'a Brain) -> SensorFuture<'a> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let mob_pos = entity.pos.load();
            let world = entity.world.load();

            let search_box = entity
                .bounding_box
                .load()
                .expand(XZ_RANGE, Y_RANGE, XZ_RANGE);
            let mut candidates: Vec<(f64, Arc<dyn EntityBase>)> = world
                .get_entities_at_box(&search_box)
                .into_iter()
                .filter_map(|candidate| {
                    let candidate_pos = candidate.get_entity().pos.load();
                    let distance = candidate_pos.squared_distance_to_vec(&mob_pos);
                    (distance < MAX_DISTANCE_TO_WANTED_ITEM * MAX_DISTANCE_TO_WANTED_ITEM)
                        .then_some((distance, candidate))
                })
                .collect();
            candidates.sort_by(|a, b| a.0.total_cmp(&b.0));

            let mut nearest: Option<Arc<dyn EntityBase>> = None;
            for (_, candidate) in candidates {
                let Some(item_entity) = candidate.clone().get_item_entity() else {
                    continue;
                };
                // The item-stack guard is a tokio mutex, so clone the stack out and drop the
                // guard before touching the brain's std mutex.
                let stack = item_entity.get_item_stack().lock().await.clone();
                if !mob.wants_to_pick_up_item(&world, &stack) {
                    continue;
                }

                let has_line_of_sight = world
                    .raycast_collision(
                        entity.get_eye_pos(),
                        candidate.get_eye_pos(),
                        async |block_pos, world| {
                            !world.get_block_state(block_pos).collision_shapes.is_empty()
                        },
                    )
                    .await
                    .is_none();
                if has_line_of_sight {
                    nearest = Some(candidate);
                    break;
                }
            }

            match nearest {
                Some(item) => {
                    brain.set::<NearestVisibleWantedItemMemory>(Arc::downgrade(&item));
                }
                None => brain.erase::<NearestVisibleWantedItemMemory>(),
            }
        })
    }
}

use std::cell::{Cell, RefCell};

use pumpkin_data::poi_type::PoiType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::Occupancy;
use rand::RngExt;
use rustc_hash::FxHashMap;

use super::super::BrainTick;
use super::super::behavior::job_site::acquire_poi::{SCAN_RANGE, find_path_to_pois};
use super::super::memory::{MemoryModuleId, types};
use super::Sensor;
use crate::entity::ai::pathfinder::path::Path;

const CACHE_TIMEOUT: i64 = 40;
const BATCH_SIZE: i32 = 5;
const REQUIRES: [MemoryModuleId; 1] = [types::NEAREST_BED.id()];

#[derive(Default)]
pub struct NearestBedSensor {
    batch_cache: FxHashMap<i64, i64>,
    last_update: i64,
}

impl Sensor for NearestBedSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        if !tick.mob.get_mob_entity().living_entity.is_baby() {
            return;
        }
        let tried_count = Cell::new(0);
        self.last_update = tick.time + i64::from(tick.mob.get_random().random_range(0..20));
        let last_update = self.last_update;
        let pois = {
            let cache = RefCell::new(&mut self.batch_cache);
            let cache_test = |pos: BlockPos| -> bool {
                let key = pos.as_long();
                let mut cache = cache.borrow_mut();
                if cache.contains_key(&key) {
                    return false;
                }
                tried_count.set(tried_count.get() + 1);
                if tried_count.get() >= BATCH_SIZE {
                    return false;
                }
                cache.insert(key, last_update + CACHE_TIMEOUT);
                true
            };
            let center = tick.mob.get_entity().block_pos.load();
            tick.world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .find_all_closest_first_with_type(
                    &|poi| poi == PoiType::Home,
                    &cache_test,
                    center,
                    SCAN_RANGE,
                    Occupancy::Any,
                )
        };
        let path = find_path_to_pois(tick.mob, &pois).filter(Path::can_reach);
        if let Some(path) = path {
            let target = path.get_target();
            let has_type = tick
                .world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_type(&target)
                .is_some();
            if has_type {
                tick.state.set(types::NEAREST_BED, target);
            }
        } else if tried_count.get() < BATCH_SIZE {
            self.batch_cache.retain(|_, expiry| *expiry >= last_update);
        }
    }
}

use pumpkin_data::poi_type::PoiType;
use pumpkin_world::poi::Occupancy;
use rand::RngExt;
use rustc_hash::FxHashMap;

use super::super::OneShot;
use super::super::job_site::find_path_to_pois;
use crate::entity::ai::brain::memory::{MemoryStatus, types, walk_target::WalkTarget};

const CACHE_TIMEOUT: i64 = 40;
const BATCH_SIZE: i32 = 5;
const RATE: i64 = 20;
const OK_DISTANCE_SQR: i32 = 4;

#[must_use]
pub fn create(speed_modifier: f32) -> OneShot {
    let mut batch_cache: FxHashMap<i64, i64> = FxHashMap::default();
    let mut last_update = 0i64;
    OneShot::new(
        "SetClosestHomeAsWalkTarget",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::HOME.id(), MemoryStatus::ValueAbsent),
        ],
        move |tick| {
            if tick.time - last_update < RATE {
                return false;
            }
            let body_pos = tick.mob.get_entity().block_pos.load();
            let closest = tick
                .world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .find_closest(
                    &|poi_type| poi_type == PoiType::Home,
                    &|_| true,
                    body_pos,
                    48,
                    Occupancy::Any,
                );
            let Some(closest) = closest else {
                return false;
            };
            if closest.squared_distance(&body_pos) <= OK_DISTANCE_SQR {
                return false;
            }
            let mut tried_count = 0;
            last_update = tick.time + tick.mob.get_random().random_range(0..RATE);
            let all_pois = tick
                .world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .find_all_closest_first_with_type(
                    &|poi_type| poi_type == PoiType::Home,
                    &|_| true,
                    body_pos,
                    48,
                    Occupancy::Any,
                );
            let mut pois = Vec::new();
            for poi in all_pois {
                let key = poi.1.as_long();
                if batch_cache.contains_key(&key) {
                    continue;
                }
                tried_count += 1;
                if tried_count >= BATCH_SIZE {
                    break;
                }
                batch_cache.insert(key, last_update + CACHE_TIMEOUT);
                pois.push(poi);
            }
            let mut reachable = false;
            if let Some(path) = find_path_to_pois(tick.mob, &pois)
                && path.can_reach()
            {
                reachable = true;
                let target_pos = path.get_target();
                let poi_type = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get_type(&target_pos);
                if poi_type.is_some() {
                    tick.state.set(
                        types::WALK_TARGET,
                        WalkTarget::from_block_pos(target_pos, speed_modifier, 1),
                    );
                }
            }
            if !reachable && tried_count < BATCH_SIZE {
                batch_cache.retain(|_, expiry| *expiry >= last_update);
            }
            true
        },
    )
}

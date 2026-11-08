use std::cell::RefCell;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::poi_type::PoiType;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::poi::Occupancy;
use rand::{Rng, RngExt};
use rustc_hash::FxHashMap;

use super::super::super::memory::{GlobalPos, MemoryModuleType, MemoryStatus};
use super::super::OneShot;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::mob::Mob;
use crate::world::World;

pub const SCAN_RANGE: i32 = 48;

const BATCH_SIZE: usize = 5;

#[must_use]
pub fn create(
    poi_type: impl Fn(PoiType) -> bool + Send + Sync + 'static,
    memory_to_validate: MemoryModuleType<GlobalPos>,
    memory_to_acquire: MemoryModuleType<GlobalPos>,
    only_if_adult: bool,
    on_poi_acquisition_event: Option<fn() -> (EntityStatus, ActorEventID)>,
    valid_poi: fn(&World, BlockPos) -> bool,
) -> OneShot {
    let mut next_scheduled_start = 0i64;
    let mut batch_cache: FxHashMap<i64, JitteredLinearRetry> = FxHashMap::default();
    let mut conditions = vec![(memory_to_acquire.id(), MemoryStatus::ValueAbsent)];
    if memory_to_validate.id() != memory_to_acquire.id() {
        conditions.push((memory_to_validate.id(), MemoryStatus::ValueAbsent));
    }
    OneShot::new("AcquirePoi", conditions, move |tick| {
        if only_if_adult && tick.mob.get_mob_entity().living_entity.is_baby() {
            return false;
        }
        let timestamp = tick.time;
        let mut random = tick.mob.get_random();
        if next_scheduled_start == 0 {
            next_scheduled_start = timestamp + i64::from(random.random_range(0..20));
            return false;
        }
        if timestamp < next_scheduled_start {
            return false;
        }
        next_scheduled_start = timestamp + 20 + i64::from(random.random_range(0..20));
        batch_cache.retain(|_, retry| retry.is_still_valid(timestamp));
        let center = tick.mob.get_entity().block_pos.load();
        let mut poi_positions = {
            let cache = RefCell::new(&mut batch_cache);
            let random_cell = RefCell::new(&mut random);
            let cache_test = |pos: BlockPos| -> bool {
                let mut cache = cache.borrow_mut();
                if let Some(retry) = cache.get_mut(&pos.as_long()) {
                    if !retry.should_retry(timestamp) {
                        return false;
                    }
                    retry.mark_attempt(&mut **random_cell.borrow_mut(), timestamp);
                }
                true
            };
            let mut pois = tick
                .world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            pois.find_all_closest_first_with_type(
                &poi_type,
                &cache_test,
                center,
                SCAN_RANGE,
                Occupancy::HasSpace,
            )
        };
        poi_positions.truncate(BATCH_SIZE);
        poi_positions.retain(|(_, pos)| valid_poi(tick.world, *pos));
        let path = find_path_to_pois(tick.mob, &poi_positions);
        if let Some(path) = path.filter(Path::can_reach) {
            let target_pos = path.get_target();
            let has_type = {
                let mut pois = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.get_type(&target_pos).is_some()
            };
            if has_type {
                let mut pois = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.take(
                    &poi_type,
                    &|_, poi_pos| poi_pos == target_pos,
                    target_pos,
                    1,
                );
                drop(pois);
                if let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) {
                    tick.state
                        .set(memory_to_acquire, GlobalPos::new(dimension, target_pos));
                }
                if let Some(event) = on_poi_acquisition_event {
                    let (status, bedrock_status) = event();
                    tick.world
                        .send_entity_status(tick.mob.get_entity(), status, bedrock_status);
                }
                batch_cache.clear();
            }
        } else {
            for (_, pos) in &poi_positions {
                batch_cache
                    .entry(pos.as_long())
                    .or_insert_with(|| JitteredLinearRetry::new(&mut random, timestamp));
            }
        }
        true
    })
}

#[must_use]
pub fn find_path_to_pois(mob: &dyn Mob, pois: &[(PoiType, BlockPos)]) -> Option<Path> {
    if pois.is_empty() {
        return None;
    }
    let mut max_range = 1;
    let mut targets = Vec::with_capacity(pois.len());
    for (poi, pos) in pois {
        max_range = max_range.max(poi.valid_range());
        targets.push(*pos);
    }
    let mob_entity = mob.get_mob_entity();
    let living = &mob_entity.living_entity;
    let mut navigator = mob_entity
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    navigator.create_path_to_targets(living, &targets, max_range)
}

struct JitteredLinearRetry {
    previous_attempt_timestamp: i64,
    next_scheduled_attempt_timestamp: i64,
    current_delay: i32,
}

impl JitteredLinearRetry {
    const MIN_INTERVAL_INCREASE: i32 = 40;
    const MAX_INTERVAL_INCREASE: i32 = 80;
    const MAX_RETRY_PATHFINDING_INTERVAL: i32 = 400;

    fn new(random: &mut impl Rng, first_attempt_timestamp: i64) -> Self {
        let mut retry = Self {
            previous_attempt_timestamp: 0,
            next_scheduled_attempt_timestamp: 0,
            current_delay: 0,
        };
        retry.mark_attempt(random, first_attempt_timestamp);
        retry
    }

    fn mark_attempt(&mut self, random: &mut impl Rng, timestamp: i64) {
        self.previous_attempt_timestamp = timestamp;
        let suggested_delay = self.current_delay
            + random.random_range(Self::MIN_INTERVAL_INCREASE..Self::MAX_INTERVAL_INCREASE);
        self.current_delay = suggested_delay.min(Self::MAX_RETRY_PATHFINDING_INTERVAL);
        self.next_scheduled_attempt_timestamp = timestamp + i64::from(self.current_delay);
    }

    fn is_still_valid(&self, timestamp: i64) -> bool {
        timestamp - self.previous_attempt_timestamp
            < i64::from(Self::MAX_RETRY_PATHFINDING_INTERVAL)
    }

    const fn should_retry(&self, timestamp: i64) -> bool {
        timestamp >= self.next_scheduled_attempt_timestamp
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn jittered_linear_retry_increases_between_40_and_79_up_to_the_cap() {
        let mut random = StdRng::seed_from_u64(42);
        let mut retry = JitteredLinearRetry::new(&mut random, 0);
        let mut previous = retry.current_delay;
        assert!((40..=79).contains(&previous));
        for step in 1..=4 {
            retry.mark_attempt(&mut random, step * 100);
            let increase = retry.current_delay - previous;
            assert!((40..=79).contains(&increase));
            assert!(retry.current_delay <= 400);
            previous = retry.current_delay;
        }
        for step in 5..=100 {
            retry.mark_attempt(&mut random, step * 100);
            assert!(retry.current_delay <= 400);
        }
    }

    #[test]
    fn jittered_linear_retry_expires_after_400_ticks_and_retries_at_schedule() {
        let mut random = StdRng::seed_from_u64(7);
        let retry = JitteredLinearRetry::new(&mut random, 100);
        let attempt = retry.next_scheduled_attempt_timestamp;
        assert!(retry.is_still_valid(499));
        assert!(!retry.is_still_valid(500));
        assert!(!retry.should_retry(attempt - 1));
        assert!(retry.should_retry(attempt));
    }
}

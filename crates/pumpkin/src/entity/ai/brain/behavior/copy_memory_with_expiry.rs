use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleType, MemoryStatus, value::MemoryValue};
use super::one_shot::OneShot;

#[must_use]
pub fn copy_memory_with_expiry<T: MemoryValue + Clone>(
    name: &'static str,
    mut copy_if: impl FnMut(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    source: MemoryModuleType<T>,
    target: MemoryModuleType<T>,
    min_duration: i32,
    max_duration: i32,
) -> OneShot {
    OneShot::new(
        name,
        vec![
            (source.id(), MemoryStatus::ValuePresent),
            (target.id(), MemoryStatus::ValueAbsent),
        ],
        move |tick| {
            if !copy_if(tick) {
                return false;
            }
            let Some(value) = tick.brain.get(source).cloned() else {
                return false;
            };
            let duration = min_duration
                + tick
                    .mob
                    .get_random()
                    .random_range(0..(max_duration + 1 - min_duration).max(1));
            tick.brain
                .set_with_expiry(target, value, i64::from(duration));
            true
        },
    )
}

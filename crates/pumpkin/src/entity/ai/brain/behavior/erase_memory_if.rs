use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::one_shot::OneShot;

#[must_use]
pub fn erase_memory_if(
    name: &'static str,
    predicate: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    memory: MemoryModuleId,
) -> OneShot {
    OneShot::new(
        name,
        vec![(memory, MemoryStatus::ValuePresent)],
        move |tick| {
            if !predicate(tick) {
                return false;
            }
            tick.brain.erase(memory);
            true
        },
    )
}

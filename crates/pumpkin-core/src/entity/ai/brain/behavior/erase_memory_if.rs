use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::OneShot;

#[must_use]
pub fn create(
    predicate: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    memory: MemoryModuleId,
) -> OneShot {
    OneShot::new(
        "EraseMemoryIf",
        vec![(memory, MemoryStatus::ValuePresent)],
        move |tick| {
            if predicate(tick) {
                tick.state.erase(memory);
                true
            } else {
                false
            }
        },
    )
}

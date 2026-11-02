use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 1] =
    [(types::LOOK_TARGET.id(), MemoryStatus::ValuePresent)];

pub struct LookAtTargetSink {
    min_duration: i32,
    max_duration: i32,
}

impl LookAtTargetSink {
    #[must_use]
    pub const fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            min_duration,
            max_duration,
        }
    }
}

impl Behavior for LookAtTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        self.min_duration
    }

    fn max_duration(&self) -> i32 {
        self.max_duration
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let ctx = tick.visibility();
        tick.state
            .get(types::LOOK_TARGET)
            .is_some_and(|target| target.is_visible_by(&ctx))
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::LOOK_TARGET.id());
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(target) = tick.state.get(types::LOOK_TARGET) {
            let position = target.current_position();
            tick.mob
                .get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_position(tick.mob, position);
        }
    }

    fn debug_name(&self) -> &'static str {
        "LookAtTargetSink"
    }
}

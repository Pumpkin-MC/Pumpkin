use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;

pub struct LookAtTargetSink {
    conditions: [(MemoryModuleId, MemoryStatus); 1],
    min_duration: i32,
    max_duration: i32,
}

impl LookAtTargetSink {
    #[must_use]
    pub const fn new(min_duration: i32, max_duration: i32) -> Self {
        Self {
            conditions: [(types::LOOK_TARGET.id(), MemoryStatus::ValuePresent)],
            min_duration,
            max_duration,
        }
    }
}

impl Behavior for LookAtTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min_duration
    }

    fn max_duration(&self) -> i32 {
        self.max_duration
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        let ctx = tick.visibility();
        ctx.brain
            .get(types::LOOK_TARGET)
            .is_some_and(|target| target.is_visible_by(&ctx))
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(position) = tick
            .brain
            .get(types::LOOK_TARGET)
            .map(|target| target.current_position())
        else {
            return;
        };
        tick.mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_position(tick.mob, position);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.brain.erase(types::LOOK_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "LookAtTargetSink"
    }
}

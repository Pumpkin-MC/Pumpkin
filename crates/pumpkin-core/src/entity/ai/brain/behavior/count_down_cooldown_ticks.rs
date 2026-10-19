use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryModuleType, MemoryStatus};
use super::Behavior;

pub struct CountDownCooldownTicks {
    cooldown_ticks: MemoryModuleType<i32>,
    conditions: [(MemoryModuleId, MemoryStatus); 1],
}

impl CountDownCooldownTicks {
    #[must_use]
    pub const fn new(cooldown_ticks: MemoryModuleType<i32>) -> Self {
        Self {
            cooldown_ticks,
            conditions: [(cooldown_ticks.id(), MemoryStatus::ValuePresent)],
        }
    }
}

impl Behavior for CountDownCooldownTicks {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn timed_out(&self, _time: i64, _end_timestamp: i64) -> bool {
        false
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        tick.brain
            .get(self.cooldown_ticks)
            .is_some_and(|remaining| *remaining > 0)
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(remaining) = tick.brain.get(self.cooldown_ticks).copied() {
            tick.brain.set(self.cooldown_ticks, remaining - 1);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.brain.erase(self.cooldown_ticks.id());
    }

    fn debug_name(&self) -> &'static str {
        "CountDownCooldownTicks"
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::memory::types;
    use super::super::{DEFAULT_DURATION, Timed};
    use super::CountDownCooldownTicks;

    #[test]
    fn cooldown_is_not_stopped_by_the_default_behavior_timeout() {
        let mut behavior = Timed::new(CountDownCooldownTicks::new(types::GAZE_COOLDOWN_TICKS));
        behavior.start_at(0, DEFAULT_DURATION);

        assert!(!behavior.has_timed_out(i64::from(DEFAULT_DURATION) + 1));
    }
}

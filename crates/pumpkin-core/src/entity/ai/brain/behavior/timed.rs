use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::{BehaviorControl, Status, required_memories_of};

pub const DEFAULT_DURATION: i32 = 60;

pub trait Behavior: Send + Sync {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)];

    fn min_duration(&self) -> i32 {
        DEFAULT_DURATION
    }

    fn max_duration(&self) -> i32 {
        DEFAULT_DURATION
    }

    fn check_extra_start_conditions(&mut self, _tick: &BrainTick<'_>) -> bool {
        true
    }

    fn start(&mut self, _tick: &mut BrainTick<'_>) {}

    fn tick(&mut self, _tick: &mut BrainTick<'_>) {}

    fn stop(&mut self, _tick: &mut BrainTick<'_>) {}

    fn can_still_use(&mut self, _tick: &BrainTick<'_>) -> bool {
        false
    }

    fn debug_name(&self) -> &'static str;
}

pub struct Timed<B: Behavior> {
    inner: B,
    required: Box<[MemoryModuleId]>,
    status: Status,
    end_timestamp: i64,
}

impl<B: Behavior> Timed<B> {
    pub fn new(inner: B) -> Self {
        let required = required_memories_of(inner.entry_conditions());
        Self {
            inner,
            required,
            status: Status::Stopped,
            end_timestamp: 0,
        }
    }

    #[must_use]
    pub const fn has_timed_out(&self, time: i64) -> bool {
        time > self.end_timestamp
    }

    pub const fn start_at(&mut self, time: i64, duration: i32) {
        self.status = Status::Running;
        self.end_timestamp = time + duration as i64;
    }

    #[must_use]
    pub const fn inner(&self) -> &B {
        &self.inner
    }
}

impl<B: Behavior> From<B> for Timed<B> {
    fn from(inner: B) -> Self {
        Self::new(inner)
    }
}

impl<B: Behavior> BehaviorControl for Timed<B> {
    fn status(&self) -> Status {
        self.status
    }

    fn required_memories(&self) -> &[MemoryModuleId] {
        &self.required
    }

    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !tick.brain.check_all(self.inner.entry_conditions())
            || !self.inner.check_extra_start_conditions(tick)
        {
            return false;
        }
        let min = self.inner.min_duration();
        let max = self.inner.max_duration();
        let duration = min
            + tick
                .mob
                .get_random()
                .random_range(0..(max + 1 - min).max(1));
        self.start_at(tick.time, duration);
        self.inner.start(tick);
        true
    }

    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>) {
        if !self.has_timed_out(tick.time) && self.inner.can_still_use(tick) {
            self.inner.tick(tick);
        } else {
            self.do_stop(tick);
        }
    }

    fn do_stop(&mut self, tick: &mut BrainTick<'_>) {
        self.status = Status::Stopped;
        self.inner.stop(tick);
    }

    fn debug_string(&self) -> String {
        self.inner.debug_name().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::memory::types;
    use super::*;

    struct Probe {
        conditions: Vec<(MemoryModuleId, MemoryStatus)>,
    }

    impl Behavior for Probe {
        fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
            &self.conditions
        }

        fn debug_name(&self) -> &'static str {
            "Probe"
        }
    }

    fn probe() -> Timed<Probe> {
        Timed::new(Probe {
            conditions: vec![
                (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
                (types::PACIFIED.id(), MemoryStatus::ValueAbsent),
            ],
        })
    }

    #[test]
    fn required_memories_are_the_entry_condition_keys() {
        assert_eq!(
            probe().required_memories(),
            &[types::ATTACK_TARGET.id(), types::PACIFIED.id()]
        );
    }

    #[test]
    fn a_fresh_behavior_is_stopped() {
        assert_eq!(probe().status(), Status::Stopped);
    }

    #[test]
    fn start_at_runs_until_the_end_timestamp_is_passed() {
        let mut behavior = probe();
        behavior.start_at(100, 5);
        assert_eq!(behavior.status(), Status::Running);
        assert!(!behavior.has_timed_out(104));
        assert!(!behavior.has_timed_out(105));
        assert!(behavior.has_timed_out(106));
    }

    #[test]
    fn a_zero_duration_behavior_times_out_on_the_next_tick() {
        let mut behavior = probe();
        behavior.start_at(100, 0);
        assert!(!behavior.has_timed_out(100));
        assert!(behavior.has_timed_out(101));
    }
}

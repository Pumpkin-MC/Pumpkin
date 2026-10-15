use super::BrainTick;
use super::memory::MemoryModuleId;

pub mod do_nothing;
pub mod gate;
pub mod one_shot;
pub mod shuffling_list;
pub mod timed;
pub mod utils;

pub use do_nothing::DoNothing;
pub use gate::{GateBehavior, OrderPolicy, RunningPolicy, run_one, run_one_with_conditions};
pub use one_shot::{OneShot, Trigger};
pub use shuffling_list::ShufflingList;
pub use timed::{Behavior, DEFAULT_DURATION, Timed};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Stopped,
    Running,
}

pub trait BehaviorControl: Send + Sync {
    fn status(&self) -> Status;
    fn required_memories(&self) -> &[MemoryModuleId];
    fn try_start(&mut self, tick: &mut BrainTick<'_>) -> bool;
    fn tick_or_stop(&mut self, tick: &mut BrainTick<'_>);
    fn do_stop(&mut self, tick: &mut BrainTick<'_>);
    fn debug_string(&self) -> String;
}

pub struct BehaviorEntry {
    pub priority: i32,
    pub activity: pumpkin_data::environment_attribute::Activity,
    pub behavior: Box<dyn BehaviorControl>,
}

#[must_use]
pub fn required_memories_of(
    conditions: &[(MemoryModuleId, super::memory::MemoryStatus)],
) -> Box<[MemoryModuleId]> {
    conditions.iter().map(|(id, _)| *id).collect()
}

use std::sync::Arc;

use super::BrainTick;
use super::memory::MemoryModuleId;
use super::memory::position_tracker::PositionTracker;

pub mod animal_make_love;
pub mod animal_panic;
pub mod baby_follow_adult;
pub mod count_down_cooldown_ticks;
pub mod crisis;
pub mod do_nothing;
pub mod door;
pub mod follow_temptation;
pub mod gate;
pub mod go_and_give_items_to_target;
pub mod go_to_wanted_item;
pub mod interact;
pub mod job_site;
pub mod look_at_target_sink;
pub mod move_to_target_sink;
pub mod one_shot;
pub mod poi_walk;
pub mod random_look_around;
pub mod random_stroll;
pub mod rest;
pub mod set_entity_look_target_sometimes;
pub mod set_walk_target_from_look_target;
pub mod shuffling_list;
pub mod stay_close_to_target;
pub mod swim;
pub mod timed;
pub mod update_activity_from_schedule;
pub mod utils;
pub mod work;

pub use animal_make_love::AnimalMakeLove;
pub use animal_panic::AnimalPanic;
pub use count_down_cooldown_ticks::CountDownCooldownTicks;
pub use do_nothing::DoNothing;
pub use follow_temptation::FollowTemptation;
pub use gate::{GateBehavior, OrderPolicy, RunningPolicy, run_one, run_one_with_conditions};
pub use go_and_give_items_to_target::GoAndGiveItemsToTarget;
pub use look_at_target_sink::LookAtTargetSink;
pub use move_to_target_sink::MoveToTargetSink;
pub use one_shot::{OneShot, Trigger};
pub use random_look_around::RandomLookAround;
pub use shuffling_list::ShufflingList;
pub use swim::Swim;
pub use timed::{Behavior, DEFAULT_DURATION, Timed};

pub type PositionGetter = fn(&mut BrainTick<'_>) -> Option<Arc<dyn PositionTracker>>;

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

pub(super) struct ActivityBehaviors {
    pub activity: pumpkin_data::environment_attribute::Activity,
    pub behaviors: Vec<Box<dyn BehaviorControl>>,
}

#[must_use]
pub fn required_memories_of(
    conditions: &[(MemoryModuleId, super::memory::MemoryStatus)],
) -> Box<[MemoryModuleId]> {
    conditions.iter().map(|(id, _)| *id).collect()
}

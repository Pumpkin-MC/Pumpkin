use super::BrainTick;
use super::memory::MemoryModuleId;

pub mod back_up_if_too_close;
pub mod copy_memory_with_expiry;
pub mod crossbow_attack;
pub mod dismount_or_skip_mounting;
pub mod do_nothing;
pub mod erase_memory_if;
pub mod gate;
pub mod go_to_target_location;
pub mod go_to_wanted_item;
pub mod interact_with;
pub mod interact_with_door;
pub mod look_at_target_sink;
pub mod melee_attack;
pub mod mount;
pub mod move_to_target_sink;
pub mod one_shot;
pub mod random_stroll;
pub mod set_entity_look_target;
pub mod set_entity_look_target_sometimes;
pub mod set_look_and_interact;
pub mod set_walk_target_away_from;
pub mod set_walk_target_from_attack_target;
pub mod set_walk_target_from_look_target;
pub mod shuffling_list;
pub mod start_attacking;
pub mod start_celebrating_if_target_dead;
pub mod stop_attacking_if_target_invalid;
pub mod stop_being_angry_if_target_dead;
pub mod stroll_to_poi;
pub mod timed;
pub mod utils;

pub use back_up_if_too_close::back_up_if_too_close;
pub use copy_memory_with_expiry::copy_memory_with_expiry;
pub use crossbow_attack::CrossbowAttack;
pub use dismount_or_skip_mounting::dismount_or_skip_mounting;
pub use do_nothing::DoNothing;
pub use erase_memory_if::erase_memory_if;
pub use gate::{GateBehavior, OrderPolicy, RunningPolicy, run_one, run_one_with_conditions};
pub use go_to_target_location::go_to_target_location;
pub use go_to_wanted_item::go_to_wanted_item;
pub use interact_with::interact_with;
pub use interact_with_door::interact_with_door;
pub use look_at_target_sink::LookAtTargetSink;
pub use melee_attack::melee_attack;
pub use mount::mount;
pub use move_to_target_sink::MoveToTargetSink;
pub use one_shot::{OneShot, Trigger};
pub use random_stroll::{stroll, stroll_with_range};
pub use set_entity_look_target::{
    set_entity_look_target, set_entity_look_target_any, set_entity_look_target_of_type,
};
pub use set_entity_look_target_sometimes::{Ticker, set_entity_look_target_sometimes};
pub use set_look_and_interact::set_look_and_interact;
pub use set_walk_target_away_from::{
    entity as set_walk_target_away_from_entity, pos as set_walk_target_away_from_pos,
};
pub use set_walk_target_from_attack_target::set_walk_target_from_attack_target_if_target_out_of_reach;
pub use set_walk_target_from_look_target::set_walk_target_from_look_target;
pub use shuffling_list::ShufflingList;
pub use start_attacking::start_attacking;
pub use start_celebrating_if_target_dead::start_celebrating_if_target_dead;
pub use stop_attacking_if_target_invalid::stop_attacking_if_target_invalid;
pub use stop_being_angry_if_target_dead::stop_being_angry_if_target_dead;
pub use stroll_to_poi::{stroll_around_poi, stroll_to_poi};
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

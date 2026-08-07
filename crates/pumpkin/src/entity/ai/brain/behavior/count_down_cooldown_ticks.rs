//! Port of `behavior/CountDownCooldownTicks.java`.
//!
//! Generic over the cooldown memory rather than holding a `MemoryModuleType<Integer>` field,
//! so the memory type stays statically known and no downcast leaks into behavior code.

use std::marker::PhantomData;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::ai::brain::memory::{MemoryKey, MemoryStatus};
use crate::entity::mob::Mob;

pub struct CountDownCooldownTicks<K: MemoryKey<Value = i32>> {
    name: &'static str,
    key: PhantomData<K>,
}

impl<K: MemoryKey<Value = i32> + Send + 'static> CountDownCooldownTicks<K> {
    /// `new CountDownCooldownTicks(memory)` (`CountDownCooldownTicks.java:13-16`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::new(
            Self {
                name: K::NAME,
                key: PhantomData,
            },
            vec![(K::ID, MemoryStatus::ValuePresent)],
        ))
    }
}

impl<K: MemoryKey<Value = i32> + Send + 'static> TimedBehavior for CountDownCooldownTicks<K> {
    fn debug_name(&self) -> &'static str {
        self.name
    }

    /// `timedOut` is overridden to `false` (`CountDownCooldownTicks.java:22-25`): the behavior
    /// runs until the counter itself hits zero, never on a duration.
    fn can_time_out(&self) -> bool {
        false
    }

    /// `canStillUse` (`CountDownCooldownTicks.java:27-31`).
    fn can_still_use(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        brain.get::<K>().is_some_and(|ticks| ticks > 0)
    }

    /// `tick` (`CountDownCooldownTicks.java:33-37`).
    fn tick(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        if let Some(ticks) = brain.get::<K>() {
            brain.set::<K>(ticks - 1);
        }
    }

    /// `stop` (`CountDownCooldownTicks.java:39-42`): erases the memory outright, which is what
    /// makes "cooldown memory present" a usable gate elsewhere (`AllayAi.java:129-133`).
    fn stop(&mut self, _mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        brain.memory.lock().unwrap().erase_by_id(K::ID);
    }
}

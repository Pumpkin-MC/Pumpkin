//! Port of `net.minecraft.world.entity.ai.behavior` (26.2 decompile).
//!
//! Two layers, mirroring vanilla:
//!
//! - [`Behavior`] is `BehaviorControl<E>` (`behavior/BehaviorControl.java`), the only thing the
//!   brain runtime knows about. `DoNothing`, `GateBehavior`/`RunOne` and the declarative
//!   one-shots implement it directly, exactly as they do in Java.
//! - [`TimedBehavior`] + [`TimedBehaviorControl`] together are `Behavior<E>`
//!   (`behavior/Behavior.java`), the stateful start/tick/stop base with a randomized duration.
//!   Java gets this via inheritance with `final tryStart`/`tickOrStop`/`doStop`; Rust cannot
//!   express "final method on a supertype", and a blanket `impl<T: TimedBehavior> Behavior for T`
//!   would collide (E0119) with the direct impls above. Wrapping in a concrete generic struct
//!   keeps the lifecycle in exactly one place without the coherence fight.
//!
//! The trait is deliberately **synchronous**. Everything a behavior in this stage touches --
//! `Navigator::set_progress`, `LookControl`, atomics on `Entity` -- is sync, so there is no way
//! to accidentally hold the memory mutex across an `.await`. If a future behavior genuinely
//! needs async, give it its own trait rather than making this one async.

pub mod animal_panic;
pub mod count_down_cooldown_ticks;
pub mod do_nothing;
pub mod gate;
pub mod go_to_wanted_item;
pub mod look_at_target_sink;
pub mod move_to_target_sink;
pub mod random_stroll;
pub mod set_walk_target_from_look_target;
pub mod swim;

use rand::RngExt;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::memory::{MemoryKeyId, MemoryStatus};
use crate::entity::mob::Mob;

/// `Behavior.Status` (`behavior/Behavior.java:109-112`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviorStatus {
    Stopped,
    Running,
}

/// `BehaviorControl<E>` (`behavior/BehaviorControl.java:8-20`).
///
/// No `Controls`-style exclusivity field, on purpose. See `super`'s module comment.
pub trait Behavior: Send {
    /// `getRequiredMemories()`. Used both as the entry gate and, at brain construction, as the
    /// list of memories to register (`Brain.java:363-366`).
    fn required_memories(&self) -> &[(MemoryKeyId, MemoryStatus)];
    fn status(&self) -> BehaviorStatus;
    fn try_start(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) -> bool;
    fn tick_or_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64);
    fn do_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64);
    fn debug_name(&self) -> &'static str;
}

/// The overridable half of vanilla's `Behavior<E>`: the `protected` hooks a subclass fills in.
pub trait TimedBehavior: Send {
    fn debug_name(&self) -> &'static str;

    /// `checkExtraStartConditions` (`behavior/Behavior.java:88-90`).
    fn check_extra_start_conditions(&mut self, _mob: &dyn Mob, _brain: &Brain) -> bool {
        true
    }

    /// `start` (`behavior/Behavior.java:56-57`).
    fn start(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) {}

    /// `tick` (`behavior/Behavior.java:68-69`).
    fn tick(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) {}

    /// `stop` (`behavior/Behavior.java:77-78`).
    fn stop(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) {}

    /// `canStillUse` (`behavior/Behavior.java:80-82`), which defaults to `false` -- a behavior
    /// that does not override it runs for exactly one tick.
    fn can_still_use(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) -> bool {
        false
    }

    /// `timedOut` (`behavior/Behavior.java:84-86`). `CountDownCooldownTicks` overrides it to
    /// `false` (`behavior/CountDownCooldownTicks.java:22-25`), so it never expires on duration.
    fn can_time_out(&self) -> bool {
        true
    }
}

/// The `final` half of vanilla's `Behavior<E>`: entry-condition check, randomized duration,
/// and the `tryStart` / `tickOrStop` / `doStop` state machine (`behavior/Behavior.java:44-75`).
pub struct TimedBehaviorControl<T: TimedBehavior> {
    inner: T,
    entry_condition: Vec<(MemoryKeyId, MemoryStatus)>,
    status: BehaviorStatus,
    end_timestamp: i64,
    min_duration: i32,
    max_duration: i32,
}

impl<T: TimedBehavior> TimedBehaviorControl<T> {
    /// `Behavior(entryCondition)` -- `DEFAULT_DURATION = 60` (`behavior/Behavior.java:12,19-21`).
    pub const fn new(inner: T, entry_condition: Vec<(MemoryKeyId, MemoryStatus)>) -> Self {
        Self::with_duration(inner, entry_condition, 60, 60)
    }

    /// `Behavior(entryCondition, minDuration, maxDuration)` (`behavior/Behavior.java:27-31`).
    pub const fn with_duration(
        inner: T,
        entry_condition: Vec<(MemoryKeyId, MemoryStatus)>,
        min_duration: i32,
        max_duration: i32,
    ) -> Self {
        Self {
            inner,
            entry_condition,
            status: BehaviorStatus::Stopped,
            end_timestamp: 0,
            min_duration,
            max_duration,
        }
    }

    /// `timedOut(timestamp)` (`behavior/Behavior.java:84-86`).
    fn timed_out(&self, game_time: i64) -> bool {
        self.inner.can_time_out() && game_time > self.end_timestamp
    }
}

impl<T: TimedBehavior + 'static> Behavior for TimedBehaviorControl<T> {
    fn required_memories(&self) -> &[(MemoryKeyId, MemoryStatus)] {
        &self.entry_condition
    }

    fn status(&self) -> BehaviorStatus {
        self.status
    }

    /// `tryStart` (`behavior/Behavior.java:43-54`). The randomized duration is rolled once, at
    /// start: `minDuration + random.nextInt(maxDuration + 1 - minDuration)`.
    fn try_start(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) -> bool {
        if !brain.check_memories(&self.entry_condition) {
            return false;
        }
        if !self.inner.check_extra_start_conditions(mob, brain) {
            return false;
        }
        self.status = BehaviorStatus::Running;
        let span = self.max_duration + 1 - self.min_duration;
        let duration = if span > 1 {
            self.min_duration + mob.get_random().random_range(0..span)
        } else {
            self.min_duration
        };
        self.end_timestamp = game_time + i64::from(duration);
        self.inner.start(mob, brain, game_time);
        true
    }

    /// `tickOrStop` (`behavior/Behavior.java:59-66`).
    fn tick_or_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        if !self.timed_out(game_time) && self.inner.can_still_use(mob, brain, game_time) {
            self.inner.tick(mob, brain, game_time);
        } else {
            self.do_stop(mob, brain, game_time);
        }
    }

    /// `doStop` (`behavior/Behavior.java:71-75`).
    fn do_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        self.status = BehaviorStatus::Stopped;
        self.inner.stop(mob, brain, game_time);
    }

    fn debug_name(&self) -> &'static str {
        self.inner.debug_name()
    }
}

/// `OneShot<E>` (`behavior/OneShot.java:7-38`), the base of every `BehaviorBuilder.create`
/// declarative behavior.
///
/// `tryStart` runs a trigger and, if it fires, marks the behavior running
/// for a single tick; `tickOrStop` unconditionally stops it.
pub trait OneShotTrigger: Send {
    fn debug_name(&self) -> &'static str;
    fn trigger(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) -> bool;
}

pub struct OneShot<T: OneShotTrigger> {
    inner: T,
    entry_condition: Vec<(MemoryKeyId, MemoryStatus)>,
    status: BehaviorStatus,
}

impl<T: OneShotTrigger> OneShot<T> {
    pub const fn new(inner: T, entry_condition: Vec<(MemoryKeyId, MemoryStatus)>) -> Self {
        Self {
            inner,
            entry_condition,
            status: BehaviorStatus::Stopped,
        }
    }
}

impl<T: OneShotTrigger + 'static> Behavior for OneShot<T> {
    fn required_memories(&self) -> &[(MemoryKeyId, MemoryStatus)] {
        &self.entry_condition
    }

    fn status(&self) -> BehaviorStatus {
        self.status
    }

    fn try_start(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) -> bool {
        // The declarative builders fold the memory-status group into the trigger; here the
        // group is the entry condition and is checked first, matching
        // `BehaviorBuilder`'s generated `hasRequiredMemories` gate.
        if !brain.check_memories(&self.entry_condition) {
            return false;
        }
        if self.inner.trigger(mob, brain, game_time) {
            self.status = BehaviorStatus::Running;
            true
        } else {
            false
        }
    }

    fn tick_or_stop(&mut self, mob: &dyn Mob, brain: &Brain, game_time: i64) {
        self.do_stop(mob, brain, game_time);
    }

    fn do_stop(&mut self, _mob: &dyn Mob, _brain: &Brain, _game_time: i64) {
        self.status = BehaviorStatus::Stopped;
    }

    fn debug_name(&self) -> &'static str {
        self.inner.debug_name()
    }
}

//! Port of `net.minecraft.world.entity.ai.sensing` (26.2 decompile).
//!
//! Sensors are the only legitimate writers of *sensed world state* memories; behaviors write
//! intent memories (`WALK_TARGET`, `LOOK_TARGET`, ...).

pub mod nearest_item;

use std::pin::Pin;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::memory::MemoryKeyId;
use crate::entity::mob::Mob;

pub type SensorFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

/// `Sensor<E>` (`sensing/Sensor.java:13-64`).
///
/// Unlike [`super::behavior::Behavior`] this is async, because reading nearby entities and
/// their item stacks in Pumpkin goes through `tokio::sync::Mutex`. Implementations MUST copy
/// what they need out of the world first and only then take the brain's memory lock: the
/// memory guard is a `std::sync::Mutex` guard and must never be held across an `.await`.
pub trait Sensor: Send {
    /// `Sensor.requires()` (`sensing/Sensor.java:64`). Every returned memory is registered on
    /// the brain at construction (`Brain.java:87-89`).
    fn requires(&self) -> &[MemoryKeyId];

    /// `Sensor.doTick` (`sensing/Sensor.java:62`).
    fn do_tick<'a>(&'a mut self, mob: &'a dyn Mob, brain: &'a Brain) -> SensorFuture<'a>;

    /// Countdown state backing `Sensor.tick`'s scan-rate gate.
    fn ticks_until_scan(&mut self) -> &mut i64;

    /// `Sensor.DEFAULT_SCAN_RATE = 20` (`sensing/Sensor.java:14,36-38`).
    fn scan_rate(&self) -> i64 {
        20
    }

    /// `Sensor.tick` (`sensing/Sensor.java:44-50`): pre-decrement, fire at `<= 0`, reset to the
    /// full scan rate.
    ///
    /// `updateTargetingConditionRanges` (`:52-60`) is not ported: it mutates shared static
    /// `TargetingConditions` singletons, which has no Rust analogue and is only consulted by
    /// the targeting sensors this stage does not port.
    fn tick<'a>(&'a mut self, mob: &'a dyn Mob, brain: &'a Brain) -> SensorFuture<'a> {
        let scan_rate = self.scan_rate();
        let remaining = self.ticks_until_scan();
        *remaining -= 1;
        if *remaining > 0 {
            return Box::pin(async {});
        }
        *remaining = scan_rate;
        self.do_tick(mob, brain)
    }
}

/// `Sensor.randomlyDelayStart` (`sensing/Sensor.java:40-42`), called once per sensor at brain
/// creation (`Brain.java:84`) so that mobs spawned in the same tick do not all scan on the
/// same tick.
#[must_use]
pub fn randomly_delayed_start(scan_rate: i64) -> i64 {
    use rand::RngExt;
    rand::rng().random_range(0..scan_rate)
}

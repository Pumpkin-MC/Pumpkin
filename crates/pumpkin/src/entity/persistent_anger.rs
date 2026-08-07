use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;
use tokio::sync::Mutex;
use uuid::Uuid;

// Vanilla `Wolf`/`ZombifiedPiglin`/`Bee`/`PolarBear`/`IronGolem`/`EnderMan`:
// `PERSISTENT_ANGER_TIME = TimeUtil.rangeOfSeconds(20, 39)`.
pub const PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;

/// Shared `NeutralMob`-equivalent state: a timed grudge against a specific entity, surviving reloads.
///
/// Unlike vanilla's absolute `anger_end_time` game tick, this tracks a remaining-tick
/// counter decremented in `tick`.
pub struct PersistentAnger {
    angry_at: Mutex<Option<Uuid>>,
    remaining_ticks: AtomicI32,
}

impl Default for PersistentAnger {
    fn default() -> Self {
        Self {
            angry_at: Mutex::new(None),
            remaining_ticks: AtomicI32::new(0),
        }
    }
}

impl PersistentAnger {
    pub async fn angry_at(&self) -> Option<Uuid> {
        *self.angry_at.lock().await
    }

    #[must_use]
    pub fn is_angry(&self) -> bool {
        self.remaining_ticks.load(Relaxed) > 0
    }

    pub async fn is_angry_at(&self, target: Uuid) -> bool {
        self.is_angry() && *self.angry_at.lock().await == Some(target)
    }

    /// Vanilla `NeutralMob.isAngryAtAllPlayers`: true while angry with no specific grudge
    /// target, gated behind the `universal_anger` game rule (checked by the caller).
    pub async fn is_angry_at_all_players(&self, universal_anger_rule: bool) -> bool {
        universal_anger_rule && self.is_angry() && self.angry_at.lock().await.is_none()
    }

    /// Vanilla `NeutralMob.forgetCurrentTargetAndRefreshUniversalAnger`: `stopBeingAngry()`
    /// then `startPersistentAngerTimer()`, leaving `angry_at` cleared so `isAngryAtAllPlayers`
    /// becomes true for the duration of the new timer.
    pub async fn forget_current_target_and_refresh_universal_anger(&self) {
        self.stop_being_angry().await;
        self.start_timer();
    }

    pub async fn set_angry_at(&self, target: Option<Uuid>) {
        *self.angry_at.lock().await = target;
    }

    /// Vanilla `startPersistentAngerTimer`: `setTimeToRemainAngry(PERSISTENT_ANGER_TIME.sample(random))`.
    pub fn start_timer(&self) {
        let ticks =
            rand::rng().random_range(PERSISTENT_ANGER_MIN_TICKS..=PERSISTENT_ANGER_MAX_TICKS);
        self.remaining_ticks.store(ticks, Relaxed);
    }

    /// Vanilla `stopBeingAngry`: clears target and anger end time (last-hurt-by/target
    /// clearing is the consumer's responsibility since it touches entity/AI state).
    pub async fn stop_being_angry(&self) {
        *self.angry_at.lock().await = None;
        self.remaining_ticks.store(0, Relaxed);
    }

    /// Per-tick decrement, auto-clearing the target once the timer expires.
    pub async fn tick(&self) {
        let prev = self.remaining_ticks.fetch_sub(1, Relaxed);
        if prev <= 0 {
            self.remaining_ticks.store(0, Relaxed);
        } else if prev == 1 {
            *self.angry_at.lock().await = None;
        }
    }

    // Vanilla `NeutralMob.java:44-46` legacy read path: a plain int holding the
    // remaining-ticks count (`AngerTime` -> `setTimeToRemainAngry`), unlike the modern
    // `anger_end_time` long which stores an absolute game tick this primitive doesn't track.
    pub async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("AngerTime", self.remaining_ticks.load(Relaxed).max(0));
        if let Some(uuid) = self.angry_at().await {
            nbt.put_uuid("angry_at", uuid);
        }
    }

    pub async fn read_nbt(&self, nbt: &NbtCompound) {
        self.remaining_ticks
            .store(nbt.get_int("AngerTime").unwrap_or(0).max(0), Relaxed);
        *self.angry_at.lock().await = nbt.get_uuid("angry_at");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fresh_is_not_angry() {
        let anger = PersistentAnger::default();
        assert!(!anger.is_angry());
    }

    #[tokio::test]
    async fn start_timer_sets_angry_in_range() {
        let anger = PersistentAnger::default();
        anger.start_timer();
        assert!(anger.is_angry());
        let ticks = anger.remaining_ticks.load(Relaxed);
        assert!((PERSISTENT_ANGER_MIN_TICKS..=PERSISTENT_ANGER_MAX_TICKS).contains(&ticks));
    }

    #[tokio::test]
    async fn tick_decrements_and_expires() {
        let anger = PersistentAnger::default();
        let target = Uuid::new_v4();
        anger.set_angry_at(Some(target)).await;
        anger.remaining_ticks.store(2, Relaxed);

        anger.tick().await;
        assert!(anger.is_angry());
        assert!(anger.is_angry_at(target).await);

        anger.tick().await;
        assert!(!anger.is_angry());
        assert_eq!(anger.angry_at().await, None);
    }

    #[tokio::test]
    async fn tick_on_expired_timer_is_noop() {
        let anger = PersistentAnger::default();
        anger.tick().await;
        assert!(!anger.is_angry());
    }

    #[tokio::test]
    async fn stop_being_angry_clears_state() {
        let anger = PersistentAnger::default();
        anger.set_angry_at(Some(Uuid::new_v4())).await;
        anger.start_timer();

        anger.stop_being_angry().await;

        assert!(!anger.is_angry());
        assert_eq!(anger.angry_at().await, None);
    }

    #[tokio::test]
    async fn nbt_round_trip() {
        let anger = PersistentAnger::default();
        let target = Uuid::new_v4();
        anger.set_angry_at(Some(target)).await;
        anger.remaining_ticks.store(123, Relaxed);

        let mut nbt = NbtCompound::new();
        anger.write_nbt(&mut nbt).await;

        let restored = PersistentAnger::default();
        restored.read_nbt(&nbt).await;

        assert_eq!(restored.remaining_ticks.load(Relaxed), 123);
        assert_eq!(restored.angry_at().await, Some(target));
    }
}

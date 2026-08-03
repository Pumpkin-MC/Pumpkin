// Port of net.minecraft.world.entity.monster.warden.{AngerLevel, AngerManagement}.
//
// Deliberate scope reduction: vanilla keys suspects by `Entity` (identity/liveness checked
// live against the level) and additionally persists an `angerByUuid` side-map for suspects
// that have unloaded, converting them back via `level.getEntity(uuid)` once they reload
// (`AngerManagement.convertFromUuids`). This port keys suspects by `Uuid` directly and drops
// the unload/reload persistence path entirely (deferred — Pumpkin's entity NBT layer for
// per-mob custom data akin to `addAdditionalSaveData`/`readAdditionalSaveData` would need a
// `Codec`-equivalent; not attempted here). Liveness/targetability is supplied by the caller
// as a predicate at `tick` time, mirroring vanilla's `validEntity` parameter.

use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

/// `Warden.DEFAULT_ANGER`
pub const DEFAULT_ANGER: i32 = 35;
/// `Warden.PROJECTILE_ANGER`
pub const PROJECTILE_ANGER: i32 = 10;
/// `Warden.ON_HURT_ANGER_BOOST`
pub const ON_HURT_ANGER_BOOST: i32 = 20;
/// `Warden.RECENT_PROJECTILE_TICK_THRESHOLD`
pub const RECENT_PROJECTILE_TICK_THRESHOLD: i32 = 100;
/// `Warden.PROJECTILE_ANGER_DISTANCE`
pub const PROJECTILE_ANGER_DISTANCE: f64 = 30.0;
/// `Warden.TOUCH_COOLDOWN_TICKS`
pub const TOUCH_COOLDOWN_TICKS: i32 = 20;
/// `Warden.VIBRATION_COOLDOWN_TICKS`
pub const VIBRATION_COOLDOWN_TICKS: i32 = 40;
/// `Warden.ANGERMANAGEMENT_TICK_DELAY`
pub const ANGERMANAGEMENT_TICK_DELAY: i32 = 20;
/// `Warden.DIGGING_COOLDOWN` / `WardenAi.DIGGING_COOLDOWN`
pub const DIG_COOLDOWN_TICKS: i32 = 1200;
/// `WardenAi.DISTURBANCE_LOCATION_EXPIRY_TIME`
pub const DISTURBANCE_LOCATION_EXPIRY_TICKS: i32 = 100;
/// `net.minecraft.world.entity.ai.behavior.warden.SonicBoom.COOLDOWN`
pub const SONIC_BOOM_COOLDOWN_TICKS: i32 = 40;
/// `Warden.TIME_TO_USE_MELEE_UNTIL_SONIC_BOOM`: the longer grace period set when a new
/// attack target is acquired, distinct from `SonicBoom.COOLDOWN`'s post-use cooldown.
pub const SONIC_BOOM_NEW_TARGET_COOLDOWN_TICKS: i32 = 200;
/// `SonicBoom.DISTANCE_XZ`
pub const SONIC_BOOM_DISTANCE_XZ: f64 = 15.0;
/// `SonicBoom.DISTANCE_Y`
pub const SONIC_BOOM_DISTANCE_Y: f64 = 20.0;
/// `SonicBoom.KNOCKBACK_HORIZONTAL`
pub const SONIC_BOOM_KNOCKBACK_HORIZONTAL: f64 = 2.5;
/// `SonicBoom.KNOCKBACK_VERTICAL`
pub const SONIC_BOOM_KNOCKBACK_VERTICAL: f64 = 0.5;
/// `Warden` `doHurtTarget` damage constant (`ATTACK_DAMAGE` is the melee attribute; sonic
/// boom's own damage is hardcoded in `SonicBoom.tick`: `hurtServer(..., 10.0F)`).
pub const SONIC_BOOM_DAMAGE: f32 = 10.0;
/// `WardenAi.ROAR_DURATION` (`Mth.ceil(84.0F)`).
pub const ROAR_DURATION_TICKS: i32 = 84;
/// `Roar.TICKS_BEFORE_PLAYING_ROAR_SOUND`.
pub const ROAR_SOUND_DELAY_TICKS: i32 = 25;
/// `Roar.ROAR_ANGER_INCREASE`.
pub const ROAR_ANGER_INCREASE: i32 = 20;

/// `AngerManagement.MAX_ANGER`
const MAX_ANGER: i32 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AngerLevel {
    Calm,
    Agitated,
    Angry,
}

impl AngerLevel {
    #[must_use]
    pub const fn minimum_anger(self) -> i32 {
        match self {
            Self::Calm => 0,
            Self::Agitated => 40,
            Self::Angry => 80,
        }
    }

    /// `AngerLevel.byAnger`: highest matching threshold wins (Angry(80) checked before
    /// Agitated(40) before Calm(0)).
    #[must_use]
    pub const fn by_anger(anger: i32) -> Self {
        if anger >= Self::Angry.minimum_anger() {
            Self::Angry
        } else if anger >= Self::Agitated.minimum_anger() {
            Self::Agitated
        } else {
            Self::Calm
        }
    }

    #[must_use]
    pub const fn is_angry(self) -> bool {
        matches!(self, Self::Angry)
    }
}

struct Suspect {
    anger: i32,
    is_player: bool,
}

/// Port of `AngerManagement`. See module doc comment for the uuid-vs-liveness scope
/// reduction.
#[derive(Default)]
pub struct AngerManagement {
    suspects: Vec<Uuid>,
    by_suspect: HashMap<Uuid, Suspect>,
    highest_anger: i32,
}

impl AngerManagement {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `AngerManagement.increaseAnger`. Note vanilla's exact clamp order: the per-suspect
    /// anger is clamped to `MAX_ANGER` first via `computeInt`, but if this is a *newly seen*
    /// suspect, any leftover anger carried over from the (here-unimplemented) uuid side-map
    /// is added back on top *without* re-clamping. This port has no side-map, so that branch
    /// never adds anything — but the clamp-then-conditionally-add structure is kept as
    /// written in `AngerManagement.java` rather than collapsed, so a future add of the
    /// side-map doesn't silently reintroduce over-clamping.
    pub fn increase_anger(&mut self, uuid: Uuid, is_player: bool, increment: i32) -> i32 {
        let new_suspect = !self.by_suspect.contains_key(&uuid);
        let entry = self.by_suspect.entry(uuid).or_insert(Suspect {
            anger: 0,
            is_player,
        });
        entry.anger = (entry.anger + increment).min(MAX_ANGER);
        let anger = entry.anger;
        if new_suspect {
            // No uuid side-map carried anger to add here (see doc comment above); vanilla
            // would add it here, unclamped, after this point and before the `put`.
            self.suspects.push(uuid);
        }
        self.sort_and_update_highest();
        anger
    }

    /// `AngerManagement.clearAnger`
    pub fn clear_anger(&mut self, uuid: Uuid) {
        self.by_suspect.remove(&uuid);
        self.suspects.retain(|u| *u != uuid);
        self.sort_and_update_highest();
    }

    /// `AngerManagement.tick`, minus the uuid side-map decay (see module doc comment).
    /// `valid` mirrors vanilla's `validEntity` predicate (`Warden::canTargetEntity`) plus the
    /// liveness check vanilla does inline (`entity.getRemovalReason() == null`) — the caller
    /// is expected to fold both into one predicate since this module has no entity access.
    pub fn tick(&mut self, valid: impl Fn(Uuid) -> bool) {
        let mut suspects = std::mem::take(&mut self.suspects);
        let mut by_suspect = std::mem::take(&mut self.by_suspect);
        suspects.retain(|uuid| {
            let Some(suspect) = by_suspect.get_mut(uuid) else {
                return false;
            };
            if suspect.anger > 1 && valid(*uuid) {
                suspect.anger -= 1;
                true
            } else {
                by_suspect.remove(uuid);
                false
            }
        });
        self.suspects = suspects;
        self.by_suspect = by_suspect;
        self.sort_and_update_highest();
    }

    /// `AngerManagement.sortAndUpdateHighestAnger` + `Sorter.compare`. Ported branch-for-branch
    /// rather than as "max over all suspects": vanilla only explicitly assigns `highestAnger`
    /// when exactly one suspect remains; for 0 suspects it stays 0 (no comparisons happen),
    /// and for >=2 suspects it is a side effect of the comparator visiting pairs during the
    /// sort (in practice this ends up being the max anger among all suspects, since any
    /// correct sort compares every element at least once, but it is *computed* as a
    /// comparator side channel, not a separate reduction — kept faithful to the original
    /// control flow here in case future edits change the sort algorithm's comparison count).
    fn sort_and_update_highest(&mut self) {
        let mut suspects = std::mem::take(&mut self.suspects);
        let by_suspect = &self.by_suspect;
        let mut highest = 0;
        suspects.sort_by(|a, b| {
            let anger_a = by_suspect.get(a).map_or(0, |s| s.anger);
            let anger_b = by_suspect.get(b).map_or(0, |s| s.anger);
            highest = highest.max(anger_a).max(anger_b);

            let angry_a = AngerLevel::by_anger(anger_a).is_angry();
            let angry_b = AngerLevel::by_anger(anger_b).is_angry();
            if angry_a != angry_b {
                return if angry_a {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }

            let player_a = by_suspect.get(a).is_some_and(|s| s.is_player);
            let player_b = by_suspect.get(b).is_some_and(|s| s.is_player);
            if player_a != player_b {
                return if player_a {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }

            anger_b.cmp(&anger_a)
        });
        self.suspects = suspects;
        self.highest_anger = if self.suspects.len() == 1 {
            self.by_suspect
                .get(&self.suspects[0])
                .map_or(0, |s| s.anger)
        } else {
            highest
        };
    }

    /// `Warden.getActiveAnger` (called through `AngerManagement.getActiveAnger`).
    #[must_use]
    pub fn active_anger(&self, current_target: Option<Uuid>) -> i32 {
        current_target.map_or(self.highest_anger, |uuid| {
            self.by_suspect.get(&uuid).map_or(0, |s| s.anger)
        })
    }

    /// `AngerManagement.getTopSuspect` + `getActiveEntity`, filtered by `valid` (vanilla's
    /// `this.filter`, i.e. `Warden::canTargetEntity`).
    #[must_use]
    pub fn top_suspect(&self, valid: impl Fn(Uuid) -> bool) -> Option<Uuid> {
        self.suspects.iter().find(|u| valid(**u)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anger_level_thresholds_match_vanilla() {
        assert_eq!(AngerLevel::by_anger(0), AngerLevel::Calm);
        assert_eq!(AngerLevel::by_anger(39), AngerLevel::Calm);
        assert_eq!(AngerLevel::by_anger(40), AngerLevel::Agitated);
        assert_eq!(AngerLevel::by_anger(79), AngerLevel::Agitated);
        assert_eq!(AngerLevel::by_anger(80), AngerLevel::Angry);
        assert_eq!(AngerLevel::by_anger(150), AngerLevel::Angry);
        assert!(AngerLevel::Angry.is_angry());
        assert!(!AngerLevel::Agitated.is_angry());
    }

    #[test]
    fn increase_anger_clamps_to_max() {
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        let result = anger.increase_anger(uuid, false, 200);
        assert_eq!(result, 150);
        assert_eq!(anger.active_anger(Some(uuid)), 150);

        let result = anger.increase_anger(uuid, false, 50);
        assert_eq!(result, 150);
    }

    #[test]
    fn default_anger_on_hurt_reaches_angry_threshold() {
        // Warden.hurtServer: increaseAngerAt(attacker, ANGRY.getMinimumAnger() + ON_HURT_ANGER_BOOST, false)
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        let amount = AngerLevel::Angry.minimum_anger() + ON_HURT_ANGER_BOOST;
        assert_eq!(amount, 100);
        let result = anger.increase_anger(uuid, true, amount);
        assert_eq!(result, 100);
        assert!(AngerLevel::by_anger(result).is_angry());
    }

    #[test]
    fn default_anger_from_direct_vibration_does_not_immediately_anger() {
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        let result = anger.increase_anger(uuid, true, DEFAULT_ANGER);
        assert_eq!(result, 35);
        // 35 < AngerLevel::Agitated.minimum_anger() (40), so a single vibration-driven
        // DEFAULT_ANGER hit leaves the Warden merely Calm, not Agitated yet.
        assert_eq!(AngerLevel::by_anger(result), AngerLevel::Calm);
    }

    #[test]
    fn tick_decreases_anger_by_one_and_never_below_one_before_removal() {
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        anger.increase_anger(uuid, false, 5);
        anger.tick(|_| true);
        assert_eq!(anger.active_anger(Some(uuid)), 4);
    }

    #[test]
    fn tick_removes_suspect_once_anger_reaches_one_or_below() {
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        anger.increase_anger(uuid, false, 1);
        anger.tick(|_| true);
        assert_eq!(anger.active_anger(Some(uuid)), 0);
        assert_eq!(anger.top_suspect(|_| true), None);
    }

    #[test]
    fn tick_removes_suspect_that_fails_the_validity_predicate_even_with_anger_remaining() {
        let mut anger = AngerManagement::new();
        let uuid = Uuid::new_v4();
        anger.increase_anger(uuid, false, 50);
        anger.tick(|_| false);
        assert_eq!(anger.active_anger(Some(uuid)), 0);
    }

    #[test]
    fn angry_suspect_sorts_before_non_angry_suspect() {
        let mut anger = AngerManagement::new();
        let calm = Uuid::new_v4();
        let angry = Uuid::new_v4();
        anger.increase_anger(calm, false, 10);
        anger.increase_anger(angry, false, 90);
        assert_eq!(anger.top_suspect(|_| true), Some(angry));
    }

    #[test]
    fn player_suspect_sorts_before_non_player_at_equal_anger_level() {
        let mut anger = AngerManagement::new();
        let mob = Uuid::new_v4();
        let player = Uuid::new_v4();
        // Both agitated (not angry), so the tie-break is "is player".
        anger.increase_anger(mob, false, 50);
        anger.increase_anger(player, true, 50);
        assert_eq!(anger.top_suspect(|_| true), Some(player));
    }

    #[test]
    fn highest_anger_with_no_target_is_the_top_suspects_anger() {
        let mut anger = AngerManagement::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        anger.increase_anger(a, false, 30);
        anger.increase_anger(b, false, 90);
        assert_eq!(anger.active_anger(None), 90);
    }

    #[test]
    fn single_suspect_highest_anger_is_that_suspects_anger() {
        let mut anger = AngerManagement::new();
        let a = Uuid::new_v4();
        anger.increase_anger(a, false, 12);
        assert_eq!(anger.active_anger(None), 12);
    }

    #[test]
    fn clear_anger_removes_the_suspect() {
        let mut anger = AngerManagement::new();
        let a = Uuid::new_v4();
        anger.increase_anger(a, false, 50);
        anger.clear_anger(a);
        assert_eq!(anger.active_anger(Some(a)), 0);
        assert_eq!(anger.top_suspect(|_| true), None);
    }
}

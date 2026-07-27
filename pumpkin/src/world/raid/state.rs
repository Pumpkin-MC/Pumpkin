//! Raid state machine — the pure, world-free half of vanilla `Raid`.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raid.java`.
//!
//! This module owns every predicate and counter that vanilla `Raid` computes
//! from its own fields (status transitions, wave bookkeeping, cooldown ticks,
//! boss-bar progress). Keeping it free of `World`/`Server` makes the transition
//! table directly unit-testable, and lets [`super::Raid`] hold it behind a
//! single lock without ever awaiting while held.

use pumpkin_util::difficulty::Difficulty;

use super::wave::{MAX_RAID_OMEN_LEVEL, absorb_raid_omen_level, num_groups};

/// Vanilla `Raid.RaidStatus` (`Raid.java:711-733`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RaidStatus {
    /// `ONGOING("ongoing")`.
    Ongoing,
    /// `VICTORY("victory")`.
    Victory,
    /// `LOSS("loss")`.
    Loss,
    /// `STOPPED("stopped")`.
    Stopped,
}

impl RaidStatus {
    /// Vanilla `RaidStatus.getSerializedName` (`Raid.java:726-728`), used for the
    /// `status` field of the saved-data codec (`Raid.java:86`).
    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Ongoing => "ongoing",
            Self::Victory => "victory",
            Self::Loss => "loss",
            Self::Stopped => "stopped",
        }
    }

    /// Inverse of [`Self::serialized_name`], for NBT load.
    #[must_use]
    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "ongoing" => Some(Self::Ongoing),
            "victory" => Some(Self::Victory),
            "loss" => Some(Self::Loss),
            "stopped" => Some(Self::Stopped),
            _ => None,
        }
    }
}

/// Vanilla `Raid.DEFAULT_PRE_RAID_TICKS` (`Raid.java:96`) — the initial and
/// between-wave cooldown, also the denominator of the pre-wave boss bar
/// (`Raid.java:296`).
pub const DEFAULT_PRE_RAID_TICKS: i32 = 300;

/// Vanilla `Raid.POST_RAID_TICK_LIMIT` (`Raid.java:95`).
pub const POST_RAID_TICK_LIMIT: i32 = 40;

/// Vanilla `Raid.RAID_TIMEOUT_TICKS` (`Raid.java:90`), checked against
/// `ticksActive` at `Raid.java:276`.
pub const RAID_TIMEOUT_TICKS: i64 = 48000;

/// Vanilla `Raid.MAX_CELEBRATION_TICKS` (`Raid.java:98`).
pub const MAX_CELEBRATION_TICKS: i32 = 600;

/// Vanilla `Raid.HERO_OF_THE_VILLAGE_DURATION` (`Raid.java:105`).
pub const HERO_OF_THE_VILLAGE_DURATION: i32 = 48000;

/// Vanilla `Raid.MAX_NO_ACTION_TIME` (`Raid.java:97`).
pub const MAX_NO_ACTION_TIME: i32 = 2400;

/// Vanilla `Raid.OUTSIDE_RAID_BOUNDS_TIMEOUT` (`Raid.java:99`).
pub const OUTSIDE_RAID_BOUNDS_TIMEOUT: i32 = 30;

/// Vanilla `Raid.VALID_RAID_RADIUS_SQR` (`Raid.java:107`) — the radius within
/// which `getRaidAt` considers a position part of the raid
/// (`ServerLevel.getRaidAt`, `ServerLevel.java:1345`).
pub const VALID_RAID_RADIUS_SQR: i32 = 9216;

/// Vanilla `Raid.RAID_REMOVAL_THRESHOLD_SQR` (`Raid.java:108`), used by
/// `updateRaiders` at `Raid.java:418`.
pub const RAID_REMOVAL_THRESHOLD_SQR: f64 = 12544.0;

/// Vanilla `Raid.VILLAGE_SEARCH_RADIUS` (`Raid.java:89`) — the ring radius used
/// by `findRandomSpawnPos` (`Raid.java:579-580`).
pub const VILLAGE_SEARCH_RADIUS: f32 = 32.0;

/// Vanilla `Raid.VALID_RAID_RADIUS` (`Raid.java:106`) — the vertical bound in
/// `findRandomSpawnPos` (`Raid.java:581`).
pub const VALID_RAID_RADIUS: i32 = 96;

/// Vanilla `Raid.ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD` (`Raid.java:87`),
/// compared against `secondsRemaining` at `Raid.java:583`.
pub const ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD: i32 = 7;

/// Vanilla `Raid.NUM_SPAWN_ATTEMPTS` (`Raid.java:91`), the attempt budget in
/// `Raid.tick` (`Raid.java:333`).
pub const NUM_SPAWN_ATTEMPTS: i32 = 5;

/// Vanilla `Raid.LOW_MOB_THRESHOLD` (`Raid.java:101`) — at or below this many
/// raiders alive the boss bar appends "raiders remaining" (`Raid.java:307`).
pub const LOW_MOB_THRESHOLD: i32 = 2;

/// Vanilla `Raid.SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER` (`Raid.java:88`).
pub const SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER: i32 = 2;

/// The world-independent fields of vanilla `Raid` (`Raid.java:112-126`).
///
/// Deliberately plain data: every method is a direct port of a vanilla
/// predicate, so a test can drive a whole raid without a `World`.
#[derive(Clone, Debug)]
pub struct RaidState {
    /// Vanilla `Raid.started` (`Raid.java:114`).
    pub started: bool,
    /// Vanilla `Raid.active` (`Raid.java:117`); `true` at construction
    /// (`Raid.java:129`).
    pub active: bool,
    /// Vanilla `Raid.ticksActive` (`Raid.java:112`).
    pub ticks_active: i64,
    /// Vanilla `Raid.raidOmenLevel` (`Raid.java:116`).
    pub raid_omen_level: i32,
    /// Vanilla `Raid.groupsSpawned` (`Raid.java:118`).
    pub groups_spawned: i32,
    /// Vanilla `Raid.raidCooldownTicks` (`Raid.java:120`); starts at 300
    /// (`Raid.java:130`).
    pub raid_cooldown_ticks: i32,
    /// Vanilla `Raid.postRaidTicks` (`Raid.java:119`).
    pub post_raid_ticks: i32,
    /// Vanilla `Raid.totalHealth` (`Raid.java:115`).
    pub total_health: f32,
    /// Vanilla `Raid.numGroups` (`Raid.java:123`), fixed at construction from
    /// the difficulty (`Raid.java:133`).
    pub num_groups: i32,
    /// Vanilla `Raid.status` (`Raid.java:124`).
    pub status: RaidStatus,
    /// Vanilla `Raid.celebrationTicks` (`Raid.java:125`).
    pub celebration_ticks: i32,
}

impl RaidState {
    /// Vanilla `Raid(BlockPos, Difficulty)` (`Raid.java:128-135`).
    #[must_use]
    pub const fn new(difficulty: Difficulty) -> Self {
        Self {
            started: false,
            active: true,
            ticks_active: 0,
            raid_omen_level: 0,
            groups_spawned: 0,
            raid_cooldown_ticks: DEFAULT_PRE_RAID_TICKS,
            post_raid_ticks: 0,
            total_health: 0.0,
            num_groups: num_groups(difficulty),
            status: RaidStatus::Ongoing,
            celebration_ticks: 0,
        }
    }

    /// Vanilla `Raid.isOver` (`Raid.java:152-154`).
    #[must_use]
    pub const fn is_over(&self) -> bool {
        self.is_victory() || self.is_loss()
    }

    /// Vanilla `Raid.isStopped` (`Raid.java:164-166`).
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(self.status, RaidStatus::Stopped)
    }

    /// Vanilla `Raid.isVictory` (`Raid.java:168-170`).
    #[must_use]
    pub const fn is_victory(&self) -> bool {
        matches!(self.status, RaidStatus::Victory)
    }

    /// Vanilla `Raid.isLoss` (`Raid.java:172-174`).
    #[must_use]
    pub const fn is_loss(&self) -> bool {
        matches!(self.status, RaidStatus::Loss)
    }

    /// Vanilla `Raid.hasFirstWaveSpawned` (`Raid.java:160-162`).
    #[must_use]
    pub const fn has_first_wave_spawned(&self) -> bool {
        self.groups_spawned > 0
    }

    /// Vanilla `Raid.isBetweenWaves` (`Raid.java:156-158`).
    #[must_use]
    pub const fn is_between_waves(&self, raiders_alive: i32) -> bool {
        self.has_first_wave_spawned() && raiders_alive == 0 && self.raid_cooldown_ticks > 0
    }

    /// Vanilla `Raid.isFinalWave` (`Raid.java:395-397`).
    #[must_use]
    pub const fn is_final_wave(&self) -> bool {
        self.groups_spawned == self.num_groups
    }

    /// Vanilla `Raid.hasBonusWave` (`Raid.java:399-401`).
    #[must_use]
    pub const fn has_bonus_wave(&self) -> bool {
        self.raid_omen_level > 1
    }

    /// Vanilla `Raid.hasSpawnedBonusWave` (`Raid.java:403-405`).
    #[must_use]
    pub const fn has_spawned_bonus_wave(&self) -> bool {
        self.groups_spawned > self.num_groups
    }

    /// Vanilla `Raid.hasMoreWaves` (`Raid.java:388-393`).
    #[must_use]
    pub const fn has_more_waves(&self) -> bool {
        if self.has_bonus_wave() {
            !self.has_spawned_bonus_wave()
        } else {
            !self.is_final_wave()
        }
    }

    /// Vanilla `Raid.shouldSpawnBonusGroup` (`Raid.java:407-409`).
    #[must_use]
    pub const fn should_spawn_bonus_group(&self, raiders_alive: i32) -> bool {
        self.is_final_wave() && raiders_alive == 0 && self.has_bonus_wave()
    }

    /// Vanilla `Raid.shouldSpawnGroup` (`Raid.java:524-526`).
    #[must_use]
    pub const fn should_spawn_group(&self, raiders_alive: i32) -> bool {
        self.raid_cooldown_ticks == 0
            && (self.groups_spawned < self.num_groups
                || self.should_spawn_bonus_group(raiders_alive))
            && raiders_alive == 0
    }

    /// Vanilla `Raid.stop` (`Raid.java:242-246`). Removing the boss-bar players
    /// is the caller's job, since that needs the world.
    pub const fn stop(&mut self) {
        self.active = false;
        self.status = RaidStatus::Stopped;
    }

    /// Vanilla `Raid.absorbRaidOmen` arithmetic (`Raid.java:228-240`).
    ///
    /// `effect_amplifier` is the amplifier of the player's Raid Omen instance.
    /// Returns `true` when this call was the raid's first (i.e. vanilla would
    /// award the `RAID_TRIGGER` stat and fire the `RAID_OMEN` criterion).
    pub const fn absorb_raid_omen(&mut self, effect_amplifier: i32) -> bool {
        self.raid_omen_level = absorb_raid_omen_level(self.raid_omen_level, effect_amplifier);
        !self.has_first_wave_spawned()
    }

    /// Vanilla `Raid.getMaxRaidOmenLevel` (`Raid.java:216-218`).
    #[must_use]
    pub const fn max_raid_omen_level(&self) -> i32 {
        MAX_RAID_OMEN_LEVEL
    }

    /// Boss-bar progress during the pre-wave countdown (`Raid.java:296`):
    /// `clamp((300 - raidCooldownTicks) / 300, 0, 1)`.
    #[must_use]
    pub fn cooldown_progress(&self) -> f32 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "cooldown ticks are small; vanilla does the same float divide"
        )]
        let numerator = (DEFAULT_PRE_RAID_TICKS - self.raid_cooldown_ticks) as f32;
        (numerator / 300.0).clamp(0.0, 1.0)
    }

    /// Boss-bar progress once raiders exist (`Raid.updateBossbar`, `Raid.java:510-512`):
    /// `clamp(healthOfLivingRaiders / totalHealth, 0, 1)`.
    ///
    /// Vanilla divides by zero here when `totalHealth` is 0 and relies on
    /// `Mth.clamp` mapping the resulting NaN; `f32::clamp` panics on NaN inputs
    /// only for the bounds, but NaN values propagate, so the zero case is handled
    /// explicitly.
    #[must_use]
    pub fn health_progress(&self, health_of_living_raiders: f32) -> f32 {
        if self.total_health <= 0.0 {
            return 0.0;
        }
        (health_of_living_raiders / self.total_health).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal_raid() -> RaidState {
        RaidState::new(Difficulty::Normal)
    }

    #[test]
    fn fresh_raid_matches_vanilla_constructor() {
        let raid = normal_raid();
        assert!(!raid.started);
        assert!(raid.active);
        assert_eq!(raid.raid_cooldown_ticks, DEFAULT_PRE_RAID_TICKS);
        assert_eq!(raid.num_groups, 5);
        assert_eq!(raid.status, RaidStatus::Ongoing);
        assert_eq!(raid.raid_omen_level, 0);
        assert!(!raid.has_first_wave_spawned());
    }

    #[test]
    fn status_names_round_trip() {
        for status in [
            RaidStatus::Ongoing,
            RaidStatus::Victory,
            RaidStatus::Loss,
            RaidStatus::Stopped,
        ] {
            let name = status.serialized_name();
            assert_eq!(RaidStatus::from_serialized_name(name), Some(status));
        }
        assert_eq!(RaidStatus::from_serialized_name("nonsense"), None);
    }

    #[test]
    fn no_group_spawns_while_cooldown_is_running() {
        let raid = normal_raid();
        assert!(!raid.should_spawn_group(0));
    }

    #[test]
    fn group_spawns_once_cooldown_hits_zero_and_field_is_clear() {
        let mut raid = normal_raid();
        raid.raid_cooldown_ticks = 0;
        assert!(raid.should_spawn_group(0));
        // A live raider blocks the next wave (vanilla Raid.java:525).
        assert!(!raid.should_spawn_group(1));
    }

    #[test]
    fn waves_run_out_at_num_groups_without_an_omen_bonus() {
        let mut raid = normal_raid();
        raid.raid_omen_level = 1; // hasBonusWave() == false
        raid.groups_spawned = 4;
        assert!(raid.has_more_waves());
        raid.groups_spawned = 5;
        assert!(raid.is_final_wave());
        assert!(!raid.has_more_waves());
        assert!(!raid.should_spawn_bonus_group(0));
    }

    #[test]
    fn omen_level_above_one_grants_exactly_one_bonus_wave() {
        let mut raid = normal_raid();
        raid.raid_omen_level = 2;
        raid.groups_spawned = 5;
        assert!(raid.has_bonus_wave());
        assert!(!raid.has_spawned_bonus_wave());
        assert!(raid.has_more_waves());
        assert!(raid.should_spawn_bonus_group(0));

        raid.groups_spawned = 6;
        assert!(raid.has_spawned_bonus_wave());
        assert!(!raid.has_more_waves());
    }

    #[test]
    fn between_waves_needs_a_spawned_wave_and_a_live_cooldown() {
        let mut raid = normal_raid();
        // Before the first wave the cooldown is running but nothing spawned yet.
        assert!(!raid.is_between_waves(0));
        raid.groups_spawned = 1;
        assert!(raid.is_between_waves(0));
        // Raiders still alive -> not between waves.
        assert!(!raid.is_between_waves(3));
        raid.raid_cooldown_ticks = 0;
        assert!(!raid.is_between_waves(0));
    }

    #[test]
    fn stop_clears_active_and_sets_stopped() {
        let mut raid = normal_raid();
        raid.stop();
        assert!(!raid.active);
        assert!(raid.is_stopped());
        assert!(!raid.is_over(), "stopped is neither victory nor loss");
    }

    #[test]
    fn victory_and_loss_are_both_over() {
        let mut raid = normal_raid();
        raid.status = RaidStatus::Victory;
        assert!(raid.is_over() && raid.is_victory() && !raid.is_loss());
        raid.status = RaidStatus::Loss;
        assert!(raid.is_over() && raid.is_loss() && !raid.is_victory());
    }

    #[test]
    fn absorbing_omen_reports_first_trigger_then_stops() {
        let mut raid = normal_raid();
        // Bad Omen II -> Raid Omen amplifier 1 -> level 2.
        assert!(raid.absorb_raid_omen(1));
        assert_eq!(raid.raid_omen_level, 2);
        // Once a wave has spawned the stat is no longer awarded.
        raid.groups_spawned = 1;
        assert!(!raid.absorb_raid_omen(0));
        assert_eq!(raid.raid_omen_level, 3);
    }

    #[test]
    fn omen_absorption_saturates_at_max_level() {
        let mut raid = normal_raid();
        for _ in 0..10 {
            raid.absorb_raid_omen(4);
        }
        assert_eq!(raid.raid_omen_level, raid.max_raid_omen_level());
    }

    #[test]
    fn cooldown_progress_runs_zero_to_one() {
        let mut raid = normal_raid();
        assert!(raid.cooldown_progress().abs() < f32::EPSILON);
        raid.raid_cooldown_ticks = 150;
        assert!((raid.cooldown_progress() - 0.5).abs() < 1e-6);
        raid.raid_cooldown_ticks = 0;
        assert!((raid.cooldown_progress() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn health_progress_handles_the_empty_raid() {
        let mut raid = normal_raid();
        assert!(raid.health_progress(0.0).abs() < f32::EPSILON);
        raid.total_health = 100.0;
        assert!((raid.health_progress(50.0) - 0.5).abs() < 1e-6);
        // Over-full (a raider healed past its spawn health) clamps to 1.
        assert!((raid.health_progress(500.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn peaceful_raid_has_no_waves_at_all() {
        let raid = RaidState::new(Difficulty::Peaceful);
        assert_eq!(raid.num_groups, 0);
        assert!(raid.is_final_wave());
        assert!(!raid.has_more_waves());
    }

    #[test]
    fn hard_raid_runs_seven_waves() {
        let mut raid = RaidState::new(Difficulty::Hard);
        assert_eq!(raid.num_groups, 7);
        for wave in 0..7 {
            raid.groups_spawned = wave;
            assert!(raid.has_more_waves(), "wave {wave} should not be the last");
        }
        raid.groups_spawned = 7;
        assert!(!raid.has_more_waves());
    }
}

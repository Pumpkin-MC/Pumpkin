//! Vanilla raid system.
//!
//! Ground truth:
//! - `/root/Vanilla/src/net/minecraft/world/entity/raid/Raid.java`
//! - `/root/Vanilla/src/net/minecraft/world/entity/raid/Raids.java`
//! - `/root/Vanilla/src/net/minecraft/world/entity/raid/Raider.java`
//! - `/root/Vanilla/src/net/minecraft/world/level/levelgen/PatrolSpawner.java`
//!
//! # Layout
//!
//! - [`wave`] — the spawn tables and omen arithmetic, pure.
//! - [`state`] — vanilla `Raid`'s own counters and status transitions, pure.
//! - [`village`] — the village-centre approximation, and an explicit statement of
//!   how it differs from vanilla's POI graph.
//! - [`member`] — per-raider raid/patrol fields, kept in a side table keyed by
//!   `Uuid` so no `Arc` cycle can form between a raid and its raiders.
//! - [`tick`] — the world-facing half of `Raid.tick`.
//! - [`spawn`] — wave spawning and spawn-position search.
//! - [`patrol`] — `PatrolSpawner`.
//! - [`registry`] — the per-world `Raids` registry.
//!
//! # Concurrency
//!
//! [`Raid`] keeps all mutable state behind a `std::sync::Mutex`. That is
//! deliberate: the type system then makes it impossible to hold the lock across an
//! `await`. Every tick therefore runs as *decide synchronously, then act
//! asynchronously* — the lock is taken to compute a plan, dropped, and only then
//! are entities spawned, effects applied, and boss-bar packets sent.

use std::sync::Mutex;

use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use rustc_hash::{FxHashMap, FxHashSet};
use uuid::Uuid;

use crate::world::bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags};

pub mod member;
pub mod omen;
pub mod patrol;
pub mod registry;
pub mod spawn;
pub mod state;
pub mod tick;
pub mod village;
pub mod wave;

pub use member::{RaiderMembership, RaiderRegistry};
pub use registry::Raids;
pub use state::{RaidState, RaidStatus};

/// Vanilla `Raid.RAID_NAME_COMPONENT` (`Raid.java:102`).
pub const RAID_NAME_KEY: &str = "event.minecraft.raid";
/// Vanilla `Raid.RAID_BAR_VICTORY_COMPONENT` (`Raid.java:103`).
pub const RAID_BAR_VICTORY_KEY: &str = "event.minecraft.raid.victory.full";
/// Vanilla `Raid.RAID_BAR_DEFEAT_COMPONENT` (`Raid.java:104`).
pub const RAID_BAR_DEFEAT_KEY: &str = "event.minecraft.raid.defeat.full";
/// Vanilla `Raid.RAIDERS_REMAINING` (`Raid.java:93`).
pub const RAIDERS_REMAINING_KEY: &str = "event.minecraft.raid.raiders_remaining";

/// The boss-bar caption a raid currently wants to display.
///
/// Vanilla rebuilds the `Component` inline in several places; naming the three
/// shapes lets the tick compare against what was last sent and skip redundant
/// packets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BarTitle {
    /// Plain `event.minecraft.raid` (`Raid.java:299`, `310`, `313`).
    Raid,
    /// `event.minecraft.raid` + " - " + `raiders_remaining` (`Raid.java:308`),
    /// used at or below `LOW_MOB_THRESHOLD` raiders.
    RaidersRemaining(i32),
    /// `event.minecraft.raid.victory.full` (`Raid.java:367`).
    Victory,
    /// `event.minecraft.raid.defeat.full` (`Raid.java:369`).
    Defeat,
}

impl BarTitle {
    /// Renders the caption. The remaining-raiders variant reproduces vanilla's
    /// `copy().append(" - ").append(translatable(..))` shape (`Raid.java:308`).
    #[must_use]
    pub fn to_component(self) -> TextComponent {
        match self {
            Self::Raid => TextComponent::translate(RAID_NAME_KEY, []),
            Self::RaidersRemaining(count) => TextComponent::translate(RAID_NAME_KEY, [])
                .add_child(TextComponent::text(" - "))
                .add_child(TextComponent::translate(
                    RAIDERS_REMAINING_KEY,
                    [TextComponent::text(count.to_string())],
                )),
            Self::Victory => TextComponent::translate(RAID_BAR_VICTORY_KEY, []),
            Self::Defeat => TextComponent::translate(RAID_BAR_DEFEAT_KEY, []),
        }
    }
}

/// The mutable interior of a [`Raid`].
pub struct RaidInner {
    /// Vanilla `Raid`'s own counters — see [`RaidState`].
    pub state: RaidState,
    /// Vanilla `Raid.center` (`Raid.java:113`).
    pub center: BlockPos,
    /// Vanilla `Raid.groupRaiderMap` (`Raid.java:110`), by wave, holding entity
    /// `Uuid`s rather than `Arc<Raider>` — see [`member`] for why.
    pub group_raiders: FxHashMap<i32, FxHashSet<Uuid>>,
    /// Vanilla `Raid.groupToLeaderMap` (`Raid.java:109`).
    pub group_leaders: FxHashMap<i32, Uuid>,
    /// Vanilla `Raid.heroesOfTheVillage` (`Raid.java:111`).
    pub heroes_of_the_village: FxHashSet<Uuid>,
    /// Vanilla `Raid.waveSpawnPos` (`Raid.java:126`).
    pub wave_spawn_pos: Option<BlockPos>,
    /// Players currently shown the boss bar — vanilla `ServerBossEvent.players`
    /// (`Raid.raidEvent`, `Raid.java:122`).
    pub bossbar_players: Vec<Uuid>,
    /// Last progress value pushed to clients, so unchanged frames send nothing.
    pub last_progress: f32,
    /// Last caption pushed to clients.
    pub last_title: BarTitle,
    /// Whether the boss bar is currently shown — vanilla `setVisible`
    /// (`Raid.java:260`, `364`).
    pub visible: bool,
}

/// A single raid — vanilla `Raid` (`Raid.java:84`).
pub struct Raid {
    /// Registry id, vanilla's `Int2ObjectMap` key in `Raids.raidMap`
    /// (`Raids.java:54`).
    pub id: i32,
    /// Boss-bar identity. Vanilla derives it from
    /// `Mth.createInsecureUUID(this.random)` (`Raid.java:122`).
    pub bossbar_uuid: Uuid,
    inner: Mutex<RaidInner>,
}

impl Raid {
    /// Vanilla `Raid(BlockPos center, Difficulty difficulty)` (`Raid.java:128-135`).
    #[must_use]
    pub fn new(id: i32, center: BlockPos, difficulty: Difficulty) -> Self {
        Self {
            id,
            bossbar_uuid: Uuid::new_v4(),
            inner: Mutex::new(RaidInner {
                state: RaidState::new(difficulty),
                center,
                group_raiders: FxHashMap::default(),
                group_leaders: FxHashMap::default(),
                heroes_of_the_village: FxHashSet::default(),
                wave_spawn_pos: None,
                bossbar_players: Vec::new(),
                // Vanilla sets progress 0 in the constructor (`Raid.java:131`).
                last_progress: 0.0,
                last_title: BarTitle::Raid,
                visible: true,
            }),
        }
    }

    /// Runs `f` against the interior. Never `await` inside `f`: the guard is a
    /// `std::sync::MutexGuard` and is not `Send` across a suspension point.
    pub fn with<R>(&self, f: impl FnOnce(&mut RaidInner) -> R) -> R {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut inner)
    }

    /// Vanilla `Raid.getCenter` (`Raid.java:627-629`).
    #[must_use]
    pub fn center(&self) -> BlockPos {
        self.with(|inner| inner.center)
    }

    /// Vanilla `Raid.isActive` (`Raid.java:676-678`).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.with(|inner| inner.state.active)
    }

    /// Vanilla `Raid.isStopped` (`Raid.java:164-166`).
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.with(|inner| inner.state.is_stopped())
    }

    /// Vanilla `Raid.isStarted` (`Raid.java:188-190`).
    #[must_use]
    pub fn is_started(&self) -> bool {
        self.with(|inner| inner.state.started)
    }

    /// Vanilla `Raid.isOver` (`Raid.java:152-154`).
    #[must_use]
    pub fn is_over(&self) -> bool {
        self.with(|inner| inner.state.is_over())
    }

    /// Vanilla `Raid.isLoss` (`Raid.java:172-174`).
    #[must_use]
    pub fn is_loss(&self) -> bool {
        self.with(|inner| inner.state.is_loss())
    }

    /// Vanilla `Raid.getGroupsSpawned` (`Raid.java:192-194`).
    #[must_use]
    pub fn groups_spawned(&self) -> i32 {
        self.with(|inner| inner.state.groups_spawned)
    }

    /// Vanilla `Raid.getRaidOmenLevel` (`Raid.java:220-222`).
    #[must_use]
    pub fn raid_omen_level(&self) -> i32 {
        self.with(|inner| inner.state.raid_omen_level)
    }

    /// Vanilla `Raid.getMaxRaidOmenLevel` (`Raid.java:216-218`).
    #[must_use]
    pub fn max_raid_omen_level(&self) -> i32 {
        self.with(|inner| inner.state.max_raid_omen_level())
    }

    /// Vanilla `Raid.getEnchantOdds` (`Raid.java:690-705`).
    #[must_use]
    pub fn enchant_odds(&self) -> f32 {
        wave::enchant_odds(self.raid_omen_level())
    }

    /// Vanilla `Raid.getTotalRaidersAlive` (`Raid.java:528-530`).
    ///
    /// Like vanilla this is the size of the membership sets, not a liveness scan;
    /// dead raiders are removed by `Raider.die` → `removeFromRaid`
    /// (`Raider.java:140`) and by `updateRaiders` (`Raid.java:411-438`).
    #[must_use]
    pub fn total_raiders_alive(&self) -> i32 {
        self.with(RaidInner::total_raiders_alive)
    }

    /// Vanilla `Raid.getAllRaiders` (`Raid.java:180-186`).
    #[must_use]
    pub fn all_raiders(&self) -> Vec<Uuid> {
        self.with(|inner| inner.group_raiders.values().flatten().copied().collect())
    }

    /// Vanilla `Raid.getLeader` (`Raid.java:567-569`).
    #[must_use]
    pub fn leader(&self, wave: i32) -> Option<Uuid> {
        self.with(|inner| inner.group_leaders.get(&wave).copied())
    }

    /// Vanilla `Raid.removeLeader` (`Raid.java:623-625`).
    pub fn remove_leader(&self, wave: i32) {
        self.with(|inner| inner.group_leaders.remove(&wave));
    }

    /// Vanilla `Raid.addHeroOfTheVillage` (`Raid.java:707-709`).
    pub fn add_hero_of_the_village(&self, killer: Uuid) {
        self.with(|inner| inner.heroes_of_the_village.insert(killer));
    }

    /// Vanilla `Raid.stop` (`Raid.java:242-246`).
    ///
    /// Returns the players whose boss bar must be removed — vanilla's
    /// `raidEvent.removeAllPlayers()` — because that needs async packet sends.
    pub fn stop(&self) -> Vec<Uuid> {
        self.with(|inner| {
            inner.state.stop();
            std::mem::take(&mut inner.bossbar_players)
        })
    }

    /// Vanilla `Raid.absorbRaidOmen` (`Raid.java:228-240`).
    ///
    /// The caller has already read the player's Raid Omen amplifier (this type has
    /// no async access). Returns `true` when vanilla would award `RAID_TRIGGER`.
    pub fn absorb_raid_omen(&self, effect_amplifier: i32) -> bool {
        self.with(|inner| inner.state.absorb_raid_omen(effect_amplifier))
    }

    /// Vanilla `Raid.removeFromRaid` (`Raid.java:532-543`).
    ///
    /// `remove_from_total_health` mirrors vanilla's flag: `updateRaiders` passes
    /// `true` (`Raid.java:434`) while `Raider.die` passes `false`
    /// (`Raider.java:140`) since the corpse's health is already zero.
    pub fn remove_from_raid(&self, raider: Uuid, wave: i32, health: f32, remove_health: bool) {
        self.with(|inner| {
            let removed = inner
                .group_raiders
                .get_mut(&wave)
                .is_some_and(|raiders| raiders.remove(&raider));
            if removed && remove_health {
                inner.state.total_health -= health;
            }
        });
    }

    /// Vanilla `Raid.addWaveMob` (`Raid.java:595-615`).
    ///
    /// Membership is a `Uuid` set, so vanilla's "replace the existing copy with the
    /// same UUID" dance (`Raid.java:598-607`) is inherent.
    pub fn add_wave_mob(&self, wave: i32, raider: Uuid, health: f32, update_health: bool) {
        self.with(|inner| {
            inner.group_raiders.entry(wave).or_default().insert(raider);
            if update_health {
                inner.state.total_health += health;
            }
        });
    }

    /// Vanilla `Raid.setLeader` (`Raid.java:617-621`).
    ///
    /// Vanilla also equips the ominous banner and sets its drop chance to 2.0.
    /// Pumpkin cannot build that stack — see [`RaiderMembership::is_captain`] — so
    /// only the leader bookkeeping is performed here.
    pub fn set_leader(&self, wave: i32, raider: Uuid) {
        self.with(|inner| inner.group_leaders.insert(wave, raider));
    }

    /// Snapshot of the boss-bar audience.
    #[must_use]
    pub fn bossbar_players(&self) -> Vec<Uuid> {
        self.with(|inner| inner.bossbar_players.clone())
    }

    /// Boss bar as vanilla styles it: `RED`, `NOTCHED_10` (`Raid.java:122`).
    #[must_use]
    pub fn make_bossbar(&self, title: BarTitle, progress: f32) -> Bossbar {
        Bossbar {
            uuid: self.bossbar_uuid,
            title: title.to_component(),
            health: progress,
            color: BossbarColor::Red,
            division: BossbarDivisions::Notches10,
            flags: BossbarFlags::empty(),
        }
    }
}

impl RaidInner {
    /// Vanilla `Raid.getTotalRaidersAlive` (`Raid.java:528-530`).
    #[must_use]
    pub fn total_raiders_alive(&self) -> i32 {
        let count: usize = self.group_raiders.values().map(FxHashSet::len).sum();
        i32::try_from(count).unwrap_or(i32::MAX)
    }

    /// Vanilla `Raid.setCenter` (`Raid.java:631-633`).
    pub const fn set_center(&mut self, center: BlockPos) {
        self.center = center;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raid() -> Raid {
        Raid::new(1, BlockPos::new(0, 64, 0), Difficulty::Normal)
    }

    #[test]
    fn new_raid_mirrors_the_vanilla_constructor() {
        let raid = raid();
        assert!(raid.is_active());
        assert!(!raid.is_started());
        assert!(!raid.is_stopped());
        assert_eq!(raid.groups_spawned(), 0);
        assert_eq!(raid.raid_omen_level(), 0);
        assert_eq!(raid.max_raid_omen_level(), 5);
        assert_eq!(raid.total_raiders_alive(), 0);
        assert_eq!(raid.center(), BlockPos::new(0, 64, 0));
    }

    #[test]
    fn adding_wave_mobs_tracks_count_and_total_health() {
        let raid = raid();
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        raid.add_wave_mob(1, first, 24.0, true);
        raid.add_wave_mob(1, second, 24.0, true);
        assert_eq!(raid.total_raiders_alive(), 2);
        assert!((raid.with(|i| i.state.total_health) - 48.0).abs() < f32::EPSILON);

        // Re-adding the same UUID is idempotent (vanilla replaces the copy).
        raid.add_wave_mob(1, first, 24.0, false);
        assert_eq!(raid.total_raiders_alive(), 2);
    }

    #[test]
    fn death_removal_keeps_total_health_for_the_bar_denominator() {
        let raid = raid();
        let raider = Uuid::from_u128(1);
        raid.add_wave_mob(1, raider, 24.0, true);
        // Raider.die passes removeFromTotalHealth = false (Raider.java:140).
        raid.remove_from_raid(raider, 1, 0.0, false);
        assert_eq!(raid.total_raiders_alive(), 0);
        assert!((raid.with(|i| i.state.total_health) - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn despawn_removal_subtracts_from_total_health() {
        let raid = raid();
        let raider = Uuid::from_u128(1);
        raid.add_wave_mob(1, raider, 24.0, true);
        // updateRaiders passes true (Raid.java:434).
        raid.remove_from_raid(raider, 1, 24.0, true);
        assert!(raid.with(|i| i.state.total_health).abs() < f32::EPSILON);
    }

    #[test]
    fn removing_an_unknown_raider_changes_nothing() {
        let raid = raid();
        raid.add_wave_mob(1, Uuid::from_u128(1), 24.0, true);
        raid.remove_from_raid(Uuid::from_u128(99), 1, 24.0, true);
        assert_eq!(raid.total_raiders_alive(), 1);
        assert!((raid.with(|i| i.state.total_health) - 24.0).abs() < f32::EPSILON);
    }

    #[test]
    fn leaders_are_tracked_per_wave() {
        let raid = raid();
        let leader = Uuid::from_u128(5);
        raid.set_leader(2, leader);
        assert_eq!(raid.leader(2), Some(leader));
        assert_eq!(raid.leader(1), None);
        raid.remove_leader(2);
        assert_eq!(raid.leader(2), None);
    }

    #[test]
    fn stop_returns_the_bar_audience_once() {
        let raid = raid();
        let viewer = Uuid::from_u128(3);
        raid.with(|inner| inner.bossbar_players.push(viewer));
        assert_eq!(raid.stop(), vec![viewer]);
        assert!(raid.is_stopped());
        assert!(!raid.is_active());
        // A second stop has nobody left to clear.
        assert!(raid.stop().is_empty());
    }

    #[test]
    fn heroes_are_deduplicated() {
        let raid = raid();
        let hero = Uuid::from_u128(8);
        raid.add_hero_of_the_village(hero);
        raid.add_hero_of_the_village(hero);
        assert_eq!(raid.with(|i| i.heroes_of_the_village.len()), 1);
    }

    #[test]
    fn all_raiders_spans_every_wave() {
        let raid = raid();
        raid.add_wave_mob(1, Uuid::from_u128(1), 1.0, true);
        raid.add_wave_mob(2, Uuid::from_u128(2), 1.0, true);
        let mut all = raid.all_raiders();
        all.sort();
        assert_eq!(all, vec![Uuid::from_u128(1), Uuid::from_u128(2)]);
    }

    #[test]
    fn enchant_odds_follow_the_absorbed_omen_level() {
        let raid = raid();
        assert!(raid.enchant_odds().abs() < f32::EPSILON);
        // Raid Omen amplifier 2 -> level 3 -> 0.25 (Raid.java:696-698).
        raid.absorb_raid_omen(2);
        assert!((raid.enchant_odds() - 0.25).abs() < f32::EPSILON);
    }
}

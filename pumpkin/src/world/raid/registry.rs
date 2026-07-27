//! Per-world raid registry — vanilla `Raids`.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raids.java`.
//!
//! Vanilla `Raids` is `SavedData` holding an `Int2ObjectMap<Raid>` plus a `nextId`
//! counter and a tick counter (`Raids.java:54-56`). This port keeps the same three
//! fields; persistence is not implemented (see [`Raids`] docs).

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::effect::StatusEffect;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::world::World;

use super::member::RaiderRegistry;
use super::state::{MAX_NO_ACTION_TIME, VALID_RAID_RADIUS_SQR};
use super::{Raid, village};

/// Per-world raid registry — vanilla `Raids` (`Raids.java:49`).
///
/// # Persistence gap
///
/// Vanilla serialises the whole registry through `Raids.CODEC` into the
/// `raids` saved-data file (`Raids.java:51-53`), and raiders store a `RaidId`
/// pointing back into it (`Raider.java:203`, `215-224`). Pumpkin has no
/// saved-data framework for per-world subsystems, so raids are **in-memory only**
/// and do not survive a restart. All the fields vanilla persists are present and
/// named after the codec keys, so adding persistence later is a matter of writing
/// the serialiser, not restructuring this type.
pub struct Raids {
    /// Vanilla `Raids.raidMap` (`Raids.java:54`).
    raids: std::sync::Mutex<Vec<Arc<Raid>>>,
    /// Vanilla `Raids.nextId` (`Raids.java:55`), starting at 1.
    next_id: AtomicI32,
    /// Vanilla `Raids.tick` (`Raids.java:56`).
    tick_counter: AtomicI32,
    /// Per-raider raid/patrol state. Lives here rather than on the entities so a
    /// raid never holds a strong reference to a raider (and vice versa).
    pub raiders: RaiderRegistry,
}

impl Default for Raids {
    fn default() -> Self {
        Self::new()
    }
}

impl Raids {
    #[must_use]
    pub fn new() -> Self {
        Self {
            raids: std::sync::Mutex::new(Vec::new()),
            // Vanilla `nextId = 1` and `getUniqueId` pre-increments, so the first
            // id handed out is 2 (`Raids.java:55`, `152-154`).
            next_id: AtomicI32::new(1),
            tick_counter: AtomicI32::new(0),
            raiders: RaiderRegistry::new(),
        }
    }

    /// Vanilla `Raids.getUniqueId` (`Raids.java:152-154`): `return ++this.nextId;`.
    fn unique_id(&self) -> i32 {
        self.next_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Snapshot of the live raids.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Arc<Raid>> {
        self.raids
            .lock()
            .map_or_else(|_| Vec::new(), |raids| raids.clone())
    }

    /// Vanilla `Raids.get(int)` (`Raids.java:70-72`).
    #[must_use]
    pub fn get(&self, raid_id: i32) -> Option<Arc<Raid>> {
        self.snapshot().into_iter().find(|raid| raid.id == raid_id)
    }

    /// Number of live raids.
    #[must_use]
    pub fn len(&self) -> usize {
        self.raids.lock().map_or(0, |raids| raids.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Vanilla `Raids.getNearbyRaid` (`Raids.java:156-166`).
    ///
    /// Only **active** raids are considered, and the closest within
    /// `max_dist_sqr` wins.
    #[must_use]
    pub fn nearby_raid(&self, pos: &BlockPos, max_dist_sqr: i32) -> Option<Arc<Raid>> {
        let mut closest: Option<(Arc<Raid>, i32)> = None;
        for raid in self.snapshot() {
            if !raid.is_active() {
                continue;
            }
            let distance = raid.center().squared_distance(pos);
            let limit = closest.as_ref().map_or(max_dist_sqr, |(_, best)| *best);
            if distance < limit {
                closest = Some((raid, distance));
            }
        }
        closest.map(|(raid, _)| raid)
    }

    /// Vanilla `ServerLevel.getRaidAt` (`ServerLevel.java:1344-1346`):
    /// `raids.getNearbyRaid(pos, VALID_RAID_RADIUS_SQR)`.
    #[must_use]
    pub fn raid_at(&self, pos: &BlockPos) -> Option<Arc<Raid>> {
        self.nearby_raid(pos, VALID_RAID_RADIUS_SQR)
    }

    /// Vanilla `Raids.canJoinRaid(Raider)` (`Raids.java:102-104`).
    ///
    /// `can_join_raid` is the raider's own flag and `no_action_time` its despawn
    /// counter; Pumpkin tracks the latter as `MobEntity::no_action_time`.
    #[must_use]
    pub const fn can_join_raid(is_alive: bool, can_join_raid: bool, no_action_time: i32) -> bool {
        is_alive && can_join_raid && no_action_time <= MAX_NO_ACTION_TIME
    }

    /// Vanilla `Raids.tick(ServerLevel)` (`Raids.java:82-100`).
    ///
    /// Retires stopped raids, then ticks the rest. When the `raids` game rule is
    /// off every raid is stopped first, exactly as vanilla does inside the loop.
    pub async fn tick(&self, world: &Arc<World>) {
        self.tick_counter.fetch_add(1, Ordering::Relaxed);

        let raids_enabled = world.level_info.load().game_rules.raids;
        let mut retired = Vec::new();

        for raid in self.snapshot() {
            // Raids.java:87-89.
            if !raids_enabled {
                let players = raid.stop();
                remove_bossbar_from(world, &players, raid.bossbar_uuid).await;
            }
            // Raids.java:90-94.
            if raid.is_stopped() {
                retired.push(raid.id);
                continue;
            }
            // Raids.java:95.
            super::spawn::tick_raid(world, &raid).await;
        }

        if !retired.is_empty() {
            if let Ok(mut raids) = self.raids.lock() {
                raids.retain(|raid| !retired.contains(&raid.id));
            }
            // Drop the stale raid ids from raider records so a later raid reusing
            // an id cannot inherit orphaned members.
            for raid_id in retired {
                self.raiders.clear_raid_id(raid_id);
            }
        }
    }

    /// Vanilla `Raids.createOrExtendRaid(ServerPlayer, BlockPos)`
    /// (`Raids.java:106-141`).
    ///
    /// Returns the raid the player's omen was absorbed into, if any.
    pub async fn create_or_extend_raid(
        &self,
        world: &Arc<World>,
        player: &Arc<Player>,
        raid_position: BlockPos,
    ) -> Option<Arc<Raid>> {
        // Raids.java:108-110.
        if player.is_spectator() {
            return None;
        }
        // Raids.java:112-114.
        let (raids_enabled, difficulty) = {
            let info = world.level_info.load();
            (info.game_rules.raids, info.difficulty)
        };
        if !raids_enabled {
            return None;
        }
        // Raids.java:115-117 is the `CAN_START_RAID` environment attribute, which
        // vanilla only disables for the Nether (`DimensionTypes.java:39`). Pumpkin
        // has no environment attributes, so the dimension check stands in for it:
        // that is the sole vanilla source that turns the attribute off.
        if world.dimension.minecraft_name
            == pumpkin_data::dimension::Dimension::THE_NETHER.minecraft_name
        {
            return None;
        }
        if difficulty == Difficulty::Peaceful {
            return None;
        }

        // Raids.java:118-131 — average the occupied village POIs within 64 blocks.
        let center = village::raid_center_for(world, &raid_position);

        // Raids.java:132-135 — reuse the raid already covering the centre.
        let raid = match self.raid_at(&center) {
            Some(existing) => existing,
            None => {
                let raid = Arc::new(Raid::new(self.unique_id(), center, difficulty));
                if let Ok(mut raids) = self.raids.lock() {
                    raids.push(raid.clone());
                }
                raid
            }
        };

        // Raids.java:136-138.
        let should_absorb =
            !raid.is_started() || raid.raid_omen_level() < raid.max_raid_omen_level();
        if should_absorb {
            let amplifier = player
                .living_entity
                .get_effect(&StatusEffect::RAID_OMEN)
                .await
                .map_or(0, |effect| i32::from(effect.amplifier));
            let first_trigger = raid.absorb_raid_omen(amplifier);
            // Raid.java:235-238 — the RAID_TRIGGER stat only fires before wave one.
            if first_trigger {
                player.stats.lock().await.increment(
                    StatisticCategory::Custom,
                    CustomStatistic::RaidTrigger as i32,
                    1,
                );
            }
        }

        Some(raid)
    }

    /// Vanilla `Raider.aiStep` join path (`Raider.java:104-108`): an eligible raider
    /// standing inside an active raid's radius joins its current wave.
    pub fn try_join_nearby_raid(&self, pos: &BlockPos, raider: Uuid) -> Option<Arc<Raid>> {
        let raid = self.raid_at(pos)?;
        let wave = raid.groups_spawned();
        // Raid.joinRaid with `exists = true` skips placement and buffs
        // (`Raid.java:493-508`), only recording membership.
        raid.add_wave_mob(wave, raider, 0.0, false);
        self.raiders.update(raider, |member| {
            member.raid_id = Some(raid.id);
            member.wave = wave;
            member.can_join_raid = true;
            member.ticks_outside_raid = 0;
        });
        Some(raid)
    }

    /// Vanilla `Raider.die` (`Raider.java:127-144`).
    ///
    /// Clears the leader slot, credits the killer as a Hero of the Village, and
    /// drops the raider from its raid with `removeFromTotalHealth = false`.
    pub fn on_raider_death(&self, raider: Uuid, killer: Option<Uuid>) {
        let Some(member) = self.raiders.get(raider) else {
            return;
        };
        let Some(raid_id) = member.raid_id else {
            self.raiders.remove(raider);
            return;
        };
        if let Some(raid) = self.get(raid_id) {
            // Raider.java:134-136.
            if member.patrol_leader {
                raid.remove_leader(member.wave);
            }
            // Raider.java:137-139 — only players become heroes.
            if let Some(killer) = killer {
                raid.add_hero_of_the_village(killer);
            }
            // Raider.java:140 — `removeFromTotalHealth = false`.
            raid.remove_from_raid(raider, member.wave, 0.0, false);
        }
        self.raiders.remove(raider);
    }
}

/// Tears the raid boss bar down for the listed players.
pub(super) async fn remove_bossbar_from(world: &Arc<World>, players: &[Uuid], bossbar: Uuid) {
    for uuid in players {
        if let Some(player) = world.get_player_by_uuid(*uuid) {
            player.remove_bossbar(bossbar).await;
        }
    }
}

/// Vanilla `Raid.validPlayer` (`Raid.java:196-201`): alive, and standing in a
/// position whose nearest raid is this one.
#[must_use]
pub fn raid_bossbar_audience(world: &Arc<World>, raid: &Arc<Raid>) -> Vec<Uuid> {
    let mut audience = Vec::new();
    for player in world.players.load().iter() {
        if !player.living_entity.is_alive() {
            continue;
        }
        let pos = player.get_entity().block_pos.load();
        let is_in_this_raid = world
            .raids
            .raid_at(&pos)
            .is_some_and(|nearest| nearest.id == raid.id);
        if is_in_this_raid {
            audience.push(player.gameprofile.id);
        }
    }
    audience
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_start_at_two_like_vanilla_pre_increment() {
        let raids = Raids::new();
        assert_eq!(raids.unique_id(), 2);
        assert_eq!(raids.unique_id(), 3);
    }

    #[test]
    fn empty_registry_finds_nothing() {
        let raids = Raids::new();
        assert!(raids.is_empty());
        assert!(raids.raid_at(&BlockPos::new(0, 64, 0)).is_none());
        assert!(raids.get(2).is_none());
    }

    #[test]
    fn nearby_raid_respects_the_radius_and_active_flag() {
        let raids = Raids::new();
        let raid = Arc::new(Raid::new(2, BlockPos::new(0, 64, 0), Difficulty::Normal));
        raids.raids.lock().unwrap().push(raid.clone());

        // Inside VALID_RAID_RADIUS_SQR (96 blocks).
        assert!(raids.raid_at(&BlockPos::new(50, 64, 0)).is_some());
        // Outside it.
        assert!(raids.raid_at(&BlockPos::new(500, 64, 0)).is_none());

        // An inactive raid is invisible to the lookup (Raids.java:161).
        raid.with(|inner| inner.state.active = false);
        assert!(raids.raid_at(&BlockPos::new(0, 64, 0)).is_none());
    }

    #[test]
    fn nearest_raid_wins_when_two_overlap() {
        let raids = Raids::new();
        let near = Arc::new(Raid::new(2, BlockPos::new(10, 64, 0), Difficulty::Normal));
        let far = Arc::new(Raid::new(3, BlockPos::new(80, 64, 0), Difficulty::Normal));
        {
            let mut guard = raids.raids.lock().unwrap();
            guard.push(far);
            guard.push(near);
        }
        assert_eq!(raids.raid_at(&BlockPos::new(0, 64, 0)).unwrap().id, 2);
    }

    #[test]
    fn can_join_raid_matches_the_vanilla_conjunction() {
        assert!(Raids::can_join_raid(true, true, 0));
        assert!(Raids::can_join_raid(true, true, MAX_NO_ACTION_TIME));
        // Raids.java:103 — strictly greater than 2400 disqualifies.
        assert!(!Raids::can_join_raid(true, true, MAX_NO_ACTION_TIME + 1));
        assert!(!Raids::can_join_raid(false, true, 0));
        assert!(!Raids::can_join_raid(true, false, 0));
    }

    #[test]
    fn raider_death_clears_leader_and_credits_the_hero() {
        let raids = Raids::new();
        let raid = Arc::new(Raid::new(2, BlockPos::new(0, 64, 0), Difficulty::Normal));
        raids.raids.lock().unwrap().push(raid.clone());

        let raider = Uuid::from_u128(1);
        let killer = Uuid::from_u128(2);
        raid.add_wave_mob(1, raider, 24.0, true);
        raid.set_leader(1, raider);
        raids.raiders.update(raider, |member| {
            member.raid_id = Some(2);
            member.wave = 1;
            member.patrol_leader = true;
        });

        raids.on_raider_death(raider, Some(killer));

        assert_eq!(raid.leader(1), None);
        assert_eq!(raid.total_raiders_alive(), 0);
        assert!(raid.with(|inner| inner.heroes_of_the_village.contains(&killer)));
        // The membership record must not leak.
        assert!(raids.raiders.is_empty());
    }

    #[test]
    fn death_without_a_killer_adds_no_hero() {
        let raids = Raids::new();
        let raid = Arc::new(Raid::new(2, BlockPos::new(0, 64, 0), Difficulty::Normal));
        raids.raids.lock().unwrap().push(raid.clone());
        let raider = Uuid::from_u128(1);
        raid.add_wave_mob(1, raider, 24.0, true);
        raids.raiders.update(raider, |member| {
            member.raid_id = Some(2);
            member.wave = 1;
        });

        raids.on_raider_death(raider, None);
        assert!(raid.with(|inner| inner.heroes_of_the_village.is_empty()));
    }

    #[test]
    fn death_of_an_unregistered_raider_is_a_no_op() {
        let raids = Raids::new();
        raids.on_raider_death(Uuid::from_u128(42), Some(Uuid::from_u128(1)));
        assert!(raids.raiders.is_empty());
    }

    #[test]
    fn joining_a_nearby_raid_records_membership() {
        let raids = Raids::new();
        let raid = Arc::new(Raid::new(2, BlockPos::new(0, 64, 0), Difficulty::Normal));
        raid.with(|inner| inner.state.groups_spawned = 3);
        raids.raids.lock().unwrap().push(raid.clone());

        let raider = Uuid::from_u128(9);
        let joined = raids.try_join_nearby_raid(&BlockPos::new(20, 64, 0), raider);
        assert!(joined.is_some());
        let member = raids.raiders.get(raider).unwrap();
        assert_eq!(member.raid_id, Some(2));
        assert_eq!(member.wave, 3);
        assert!(member.can_join_raid);
        assert_eq!(raid.total_raiders_alive(), 1);
    }

    #[test]
    fn joining_far_from_any_raid_fails() {
        let raids = Raids::new();
        let raid = Arc::new(Raid::new(2, BlockPos::new(0, 64, 0), Difficulty::Normal));
        raids.raids.lock().unwrap().push(raid);
        assert!(
            raids
                .try_join_nearby_raid(&BlockPos::new(1000, 64, 0), Uuid::from_u128(9))
                .is_none()
        );
    }

    #[test]
    fn retiring_a_raid_clears_its_raider_records() {
        let raids = Raids::new();
        let raider = Uuid::from_u128(1);
        raids.raiders.update(raider, |member| {
            member.raid_id = Some(7);
        });
        raids.raiders.clear_raid_id(7);
        assert_eq!(raids.raiders.get(raider).unwrap().raid_id, None);
    }
}

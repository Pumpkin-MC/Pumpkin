//! Raider membership — the per-raider fields of vanilla `Raider`, held by id.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raider.java`.
//!
//! # Why a side table instead of fields on the mob
//!
//! Vanilla stores `raid`, `wave`, `canJoinRaid` and `ticksOutsideRaid` directly on
//! `Raider` (`Raider.java:62-65`), plus `patrolTarget` / `patrolLeader` /
//! `patrolling` on `PatrollingMonster`
//! (`/root/Vanilla/src/net/minecraft/world/entity/monster/PatrollingMonster.java:38-40`).
//! `Raider.raid` is a hard reference back to the `Raid`, and `Raid.groupRaiderMap`
//! holds hard references to every `Raider` — a cycle Java's GC collects and Rust's
//! `Arc` would not.
//!
//! Pumpkin has no `Raider` base type to hang those fields on (illagers are separate
//! structs implementing `Mob`), so this module keeps them in a per-world table keyed
//! by entity `Uuid`. That kills the cycle by construction: the raid stores ids, the
//! table stores ids, and nothing holds an `Arc` to a raider outside a tick's own
//! local scope.

use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashMap;
use uuid::Uuid;

/// The per-raider raid/patrol state vanilla keeps on the entity itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct RaiderMembership {
    /// Id of the raid this raider belongs to, mirroring vanilla `Raider.raid`
    /// (`Raider.java:62`) and the `RaidId` NBT field (`Raider.java:203`).
    pub raid_id: Option<i32>,
    /// Vanilla `Raider.wave` (`Raider.java:63`).
    pub wave: i32,
    /// Vanilla `Raider.canJoinRaid` (`Raider.java:64`).
    pub can_join_raid: bool,
    /// Vanilla `Raider.ticksOutsideRaid` (`Raider.java:65`).
    pub ticks_outside_raid: i32,
    /// Vanilla `PatrollingMonster.patrolLeader` (`PatrollingMonster.java:39`).
    pub patrol_leader: bool,
    /// Vanilla `PatrollingMonster.patrolling` (`PatrollingMonster.java:40`).
    pub patrolling: bool,
    /// Vanilla `PatrollingMonster.patrolTarget` (`PatrollingMonster.java:38`).
    pub patrol_target: Option<BlockPos>,
}

impl RaiderMembership {
    /// Vanilla `Raider.hasActiveRaid` needs both a raid and that raid being active
    /// (`Raider.java:175-177`); this half is the "has a raid" test.
    #[must_use]
    pub const fn has_raid(&self) -> bool {
        self.raid_id.is_some()
    }

    /// Vanilla `Raider.isCaptain` (`Raider.java:159-164`).
    ///
    /// Vanilla additionally requires the ominous banner in the head slot. Pumpkin
    /// has no ominous-banner item stack with the vanilla banner-pattern components
    /// (`Raid.getBannerComponentPatch`, `Raid.java:549-557`), so the banner half of
    /// the test is not represented; `patrol_leader` is set exactly where vanilla
    /// equips the banner, so the two stay in lockstep for every code path that
    /// creates a captain.
    #[must_use]
    pub const fn is_captain(&self) -> bool {
        self.patrol_leader
    }
}

/// Per-world table of raider membership records.
///
/// Vanilla's equivalent state lives on the entities; see the module docs for why
/// Pumpkin keeps it beside them instead.
#[derive(Default)]
pub struct RaiderRegistry {
    members: std::sync::Mutex<FxHashMap<Uuid, RaiderMembership>>,
}

impl RaiderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Membership record for `raider`, or `None` when it has never joined a raid
    /// or patrol.
    #[must_use]
    pub fn get(&self, raider: Uuid) -> Option<RaiderMembership> {
        self.members
            .lock()
            .map_or(None, |members| members.get(&raider).copied())
    }

    /// Read-modify-write of one record, inserting a default if absent.
    pub fn update<R>(&self, raider: Uuid, f: impl FnOnce(&mut RaiderMembership) -> R) -> Option<R> {
        self.members.lock().ok().map(|mut members| {
            let entry = members.entry(raider).or_default();
            f(entry)
        })
    }

    /// Drops a raider's record — called when it dies or leaves the world, so the
    /// table cannot grow without bound.
    pub fn remove(&self, raider: Uuid) {
        if let Ok(mut members) = self.members.lock() {
            members.remove(&raider);
        }
    }

    /// Clears the raid association but keeps patrol state, mirroring
    /// `Raider.setCurrentRaid(null)` (`Raid.removeFromRaid`, `Raid.java:539`).
    pub fn clear_raid(&self, raider: Uuid) {
        self.update(raider, |member| {
            member.raid_id = None;
        });
    }

    /// Drops every record whose raid id is `raid_id`, used when a raid is removed
    /// from the registry so stale ids cannot resolve to a later raid reusing the id.
    pub fn clear_raid_id(&self, raid_id: i32) {
        if let Ok(mut members) = self.members.lock() {
            for member in members.values_mut() {
                if member.raid_id == Some(raid_id) {
                    member.raid_id = None;
                }
            }
        }
    }

    /// Number of tracked raiders. Diagnostics and leak tests only.
    #[must_use]
    pub fn len(&self) -> usize {
        self.members.lock().map_or(0, |members| members.len())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_raider_has_no_record() {
        let registry = RaiderRegistry::new();
        assert!(registry.get(Uuid::from_u128(1)).is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn update_inserts_then_mutates() {
        let registry = RaiderRegistry::new();
        let id = Uuid::from_u128(7);
        registry.update(id, |member| {
            member.raid_id = Some(3);
            member.wave = 2;
        });
        let member = registry.get(id).expect("record was inserted");
        assert_eq!(member.raid_id, Some(3));
        assert_eq!(member.wave, 2);
        assert!(member.has_raid());

        registry.update(id, |member| member.wave = 4);
        assert_eq!(registry.get(id).map(|m| m.wave), Some(4));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn clear_raid_keeps_patrol_state() {
        let registry = RaiderRegistry::new();
        let id = Uuid::from_u128(9);
        registry.update(id, |member| {
            member.raid_id = Some(1);
            member.patrol_leader = true;
            member.patrolling = true;
        });
        registry.clear_raid(id);
        let member = registry.get(id).expect("record still present");
        assert!(!member.has_raid());
        assert!(
            member.patrol_leader,
            "captain flag survives leaving the raid"
        );
        assert!(member.is_captain());
        assert!(member.patrolling);
    }

    #[test]
    fn clear_raid_id_detaches_only_matching_raiders() {
        let registry = RaiderRegistry::new();
        let mine = Uuid::from_u128(1);
        let other = Uuid::from_u128(2);
        registry.update(mine, |m| m.raid_id = Some(5));
        registry.update(other, |m| m.raid_id = Some(6));
        registry.clear_raid_id(5);
        assert_eq!(registry.get(mine).and_then(|m| m.raid_id), None);
        assert_eq!(registry.get(other).and_then(|m| m.raid_id), Some(6));
    }

    #[test]
    fn remove_drops_the_record_entirely() {
        let registry = RaiderRegistry::new();
        let id = Uuid::from_u128(4);
        registry.update(id, |m| m.raid_id = Some(1));
        assert_eq!(registry.len(), 1);
        registry.remove(id);
        assert!(registry.is_empty(), "no record may outlive its raider");
    }
}

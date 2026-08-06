//! Village POI classification and density queries.
//!
//! Vanilla has no discrete "village" entity. It's read off POI density via
//! `PoiManager`/`PoiTypes`
//! (`net/minecraft/world/entity/ai/village/poi/{PoiManager,PoiTypes}.java`):
//! every bed and every job-site block is registered as a POI at its block
//! position, keyed by chunk section (`SectionPos.asLong`).
//!
//! - `ServerLevel.isCloseToVillage(pos, sectionDistance)`
//!   (`ServerLevel.java` ~1551) is true when
//!   `sectionsToVillage(pos) <= sectionDistance`, where `sectionsToVillage`
//!   is the distance, in chunk sections, to the nearest section containing
//!   an *occupied* POI tagged `#minecraft:village` (`home`, `meeting`, or
//!   `#acquirable_job_site` -
//!   `data/minecraft/tags/point_of_interest_type/village.json`), capped at
//!   `PoiManager.MAX_VILLAGE_DISTANCE = 6`. The underlying
//!   `SectionTracker`/`DynamicGraphMinFixedPoint` BFS
//!   (`SectionTracker.java` `checkNeighborsAfterUpdate`/`getComputedLevel`)
//!   propagates over the full 3x3x3 section neighborhood at uniform cost 1
//!   per step, so the resulting distance metric is Chebyshev distance (in
//!   sections) to the nearest source section, not Manhattan or Euclidean.
//! - `CatSpawner.spawnInVillage` (`CatSpawner.java` line 47) additionally
//!   requires `getCountInRange(HOME, pos, 48, Occupancy.IS_OCCUPIED) > 4`:
//!   more than 4 *claimed* beds within a 48-block sphere
//!   (`PoiManager.getInRange` prefilters with an axis-aligned square via
//!   `getInSquare`, then applies `distSqr(center) <= radius*radius`, which
//!   includes the Y axis).
//!
//! Villagers now claim POIs with a ticket system (`PoiRecord.freeTickets`/
//! `maxTickets`, `pumpkin_world::poi::{PoiEntry, PoiStorage}`) instead of
//! just existing at density-queryable positions - see `World::acquire_poi`/
//! `release_poi` (vanilla `PoiManager.take`/`release`, driven by
//! `AcquirePoi`/`Villager.releasePoi`). This module's own density queries
//! (`sectionsToVillage`, `getCountInRange`) therefore use the real
//! `Occupancy` filter below rather than a stand-in.
//!
//! `Occupancy::HasSpace` vs `Occupancy::IsOccupied` are **two different
//! filters for two different purposes** and must not be conflated
//! (`PoiManager.Occupancy` in vanilla): claiming a POI (e.g. a villager
//! grabbing a bed) needs `HasSpace` (`freeTickets > 0` - there's a ticket
//! left to give out); village-density checks (is this area an actual,
//! lived-in village) need `IsOccupied` (`freeTickets != maxTickets` - the POI
//! is *actually in use*, not merely present). A meeting point
//! (`maxTickets = 32`) can be both simultaneously.
//!
//! Residual gap: only bed (`HOME`) acquisition is wired up
//! (`VillagerEntity`'s rest logic calls `World::acquire_poi`/`release_poi`).
//! Job-site and meeting-point POIs are never claimed by anything in Pumpkin
//! (no profession job-site claiming, no bell-ringing meeting-point
//! claiming), so their `free_tickets` never moves off `max_tickets` and they
//! never satisfy `IS_OCCUPIED`. `sections_to_village`'s job-site/meeting
//! contribution is therefore inert until that claiming exists - only the
//! `HOME` contribution is currently meaningful. A village with claimed beds
//! still registers correctly; a freshly-generated, never-slept-in village
//! (no villager has claimed a bed yet) will not, which matches vanilla's own
//! `isVillageCenter` requiring an *occupied* POI, not merely an existing one.
//!
//! Existing worlds with POI entries persisted before this ticket system
//! landed have `free_tickets == 0` on disk (the prior code always wrote
//! `0`, regardless of type). Since there is no version marker to tell a
//! legitimately-claimed `0` apart from that legacy default, those entries
//! load as permanently unclaimable (`has_space() == false`) and permanently
//! "occupied" (`is_occupied() == true`) until the block is broken and
//! replaced (which re-adds the POI via `World::set_block_state`, resetting
//! `free_tickets` to `max_tickets`). This is a one-time migration quirk on
//! pre-existing saves, not a bug in new POI registrations.
//!
//! Registry population: entries are added/removed incrementally at
//! `World::set_block_state`, the single block-mutation chokepoint, mirroring
//! vanilla's `PoiManager` reacting to `onBlockStateChange`. Structures placed
//! directly by world generation (freshly generated villages) write blocks
//! into chunk sections in `pumpkin-world` without going through that
//! chokepoint, so they are not backfilled into the registry. Vanilla's
//! equivalent backfill runs off chunk load
//! (`PoiManager.checkConsistencyWithBlocks`/`updateFromSection`), which would
//! require hooking chunk loading in `pumpkin-world` - out of scope for this
//! pass; a freshly-generated, never-modified village will not register as
//! "close to village" until a bed or job-site block is placed or broken.
//!
//! Storage reuses `World::portal_poi`
//! (`pumpkin_world::poi::PoiStorage`), which already persists region-keyed
//! POI entries to the same `poi/` folder vanilla's `PoiManager` writes to.
//! Vanilla keeps every POI type (portals, beds, job sites, meeting,
//! beehives, ...) in one registry, so sharing the store here matches
//! vanilla's structure rather than compromising it.
//!
//! Bed head/foot: vanilla only registers the `BedPart.HEAD` half of a bed as
//! a `HOME` POI (`PoiTypes.java` ~113, `register(..., HOME, BEDS, 1, 1)`
//! filtered to `BedPart.HEAD` states). `classify_block` now takes the
//! block's state id and applies the same filter, so each bed resolves to
//! exactly one POI record - required for ticket claiming to make sense (a
//! villager must never be able to claim the foot half as if it were its own
//! separate bed).

use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BedPart, BlockProperties, WhiteBedLikeProperties};
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_util::math::position::BlockPos;

/// POI type string for beds - vanilla `PoiTypes.HOME`.
pub const POI_TYPE_HOME: &str = "minecraft:home";
/// POI type string for the generic job-site block set.
///
/// Approximates vanilla's per-profession job-site POI types
/// (`PoiTypes.ARMORER`..`PoiTypes.WEAPONSMITH`) with a single bucket driven
/// by the `c:villager_job_sites` block tag, since Pumpkin has no
/// per-profession POI registry.
pub const POI_TYPE_JOB_SITE: &str = "minecraft:job_site";
/// POI type string for the bell - vanilla `PoiTypes.MEETING`.
pub const POI_TYPE_MEETING: &str = "minecraft:meeting";

/// The village-tag POI types used by `sectionsToVillage`/`isVillageCenter`
/// (`#minecraft:village` = `home` + `meeting` + `#acquirable_job_site`).
pub const VILLAGE_TAG_POI_TYPES: [&str; 3] = [POI_TYPE_HOME, POI_TYPE_MEETING, POI_TYPE_JOB_SITE];

/// `PoiManager.MAX_VILLAGE_DISTANCE`.
pub const MAX_VILLAGE_DISTANCE: i32 = 6;

/// Vanilla `PoiManager.Occupancy` - see module docs for why `HasSpace` and
/// `IsOccupied` are different filters for different purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occupancy {
    /// Vanilla `Occupancy.HAS_SPACE` - `freeTickets > 0`. Used to *acquire*
    /// a POI.
    HasSpace,
    /// Vanilla `Occupancy.IS_OCCUPIED` - `freeTickets != maxTickets`. Used
    /// for village-density checks.
    IsOccupied,
    /// Vanilla `Occupancy.ANY` - no filter.
    Any,
}

impl Occupancy {
    /// Vanilla `Occupancy.getTest()` applied to a `(freeTickets, maxTickets)`
    /// pair (in place of a `PoiRecord` reference).
    #[must_use]
    pub const fn matches(self, free_tickets: i32, max_tickets: i32) -> bool {
        match self {
            Self::HasSpace => free_tickets > 0,
            Self::IsOccupied => free_tickets != max_tickets,
            Self::Any => true,
        }
    }
}

/// Classifies a block (at a specific state) for POI registration.
///
/// Returns `None` if it isn't a POI-bearing block. See module docs for the
/// vanilla mapping this approximates (`PoiTypes.forState`, `PoiTypes.java`
/// ~91).
#[must_use]
pub fn classify_block(block: &Block, state_id: BlockStateId) -> Option<&'static str> {
    if block.has_tag(&BlockTag::MINECRAFT_BEDS) {
        // Vanilla only registers the HEAD half as a POI - see module docs.
        let props = WhiteBedLikeProperties::from_state_id(state_id, block);
        (props.part == BedPart::Head).then_some(POI_TYPE_HOME)
    } else if *block == Block::BELL {
        Some(POI_TYPE_MEETING)
    } else if block.has_tag(&BlockTag::C_VILLAGER_JOB_SITES) {
        Some(POI_TYPE_JOB_SITE)
    } else {
        None
    }
}

/// Squared 3D distance between two block positions - vanilla
/// `BlockPos.distSqr`, used by `PoiManager.findClosest`/`getInRange`'s
/// closest-first sort.
#[must_use]
pub fn distance_sq(a: BlockPos, b: BlockPos) -> i64 {
    let dx = i64::from(a.0.x - b.0.x);
    let dy = i64::from(a.0.y - b.0.y);
    let dz = i64::from(a.0.z - b.0.z);
    dx * dx + dy * dy + dz * dz
}

/// Chebyshev distance, in chunk sections (16-block cubes), between two
/// block positions.
///
/// Pure function backing `sectionsToVillage` - see module docs on why
/// Chebyshev (not Euclidean/Manhattan) is the correct metric for
/// `SectionTracker`'s 26-neighbor BFS.
#[must_use]
pub const fn section_chebyshev_distance(a: BlockPos, b: BlockPos) -> i32 {
    let (ax, ay, az) = (a.0.x >> 4, a.0.y >> 4, a.0.z >> 4);
    let (bx, by, bz) = (b.0.x >> 4, b.0.y >> 4, b.0.z >> 4);
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    let dz = (az - bz).abs();
    let dxy = if dx > dy { dx } else { dy };
    if dxy > dz { dxy } else { dz }
}

/// Whether `candidate` is within a 3D sphere of `radius` blocks around
/// `center`.
///
/// Vanilla `PoiManager.getInRange`'s `distSqr(center) <= radius*radius`
/// filter, applied after the axis-aligned `getInSquare` prefilter
/// `PoiStorage::get_in_square` already performs.
#[must_use]
pub fn in_sphere(center: BlockPos, candidate: BlockPos, radius: i32) -> bool {
    let radius = i64::from(radius);
    distance_sq(center, candidate) <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos(Vector3::new(x, y, z))
    }

    #[test]
    fn occupancy_has_space_and_is_occupied_are_not_the_same_filter() {
        // A meeting point (max_tickets = 32) with one claimant: there is
        // still space for more claimants (HasSpace), but it's also already
        // in use (IsOccupied) - conflating the two filters here would be
        // wrong in either direction.
        let free = 31;
        let max = 32;
        assert!(Occupancy::HasSpace.matches(free, max));
        assert!(Occupancy::IsOccupied.matches(free, max));

        // A bed (max_tickets = 1), unclaimed: has space, not occupied.
        assert!(Occupancy::HasSpace.matches(1, 1));
        assert!(!Occupancy::IsOccupied.matches(1, 1));

        // A bed, claimed: no space, occupied.
        assert!(!Occupancy::HasSpace.matches(0, 1));
        assert!(Occupancy::IsOccupied.matches(0, 1));

        assert!(Occupancy::Any.matches(0, 1));
        assert!(Occupancy::Any.matches(1, 1));
    }

    #[test]
    fn chebyshev_distance_same_section_is_zero() {
        assert_eq!(
            section_chebyshev_distance(pos(0, 64, 0), pos(15, 70, 15)),
            0
        );
    }

    #[test]
    fn chebyshev_distance_uses_max_axis() {
        // 3 sections on X, 1 on Z, 0 on Y -> Chebyshev distance is 3, not
        // Manhattan (4) or Euclidean (~3.16).
        assert_eq!(
            section_chebyshev_distance(pos(0, 64, 0), pos(3 * 16, 64, 16)),
            3
        );
    }

    #[test]
    fn chebyshev_distance_is_symmetric() {
        let a = pos(5, 70, -20);
        let b = pos(-40, 64, 100);
        assert_eq!(
            section_chebyshev_distance(a, b),
            section_chebyshev_distance(b, a)
        );
    }

    #[test]
    fn in_sphere_includes_y_axis() {
        // Vanilla's getInRange filter is a true 3D sphere: a point that
        // passes the axis-aligned square prefilter on X/Z can still fail on
        // Y once distSqr is checked.
        let center = pos(0, 64, 0);
        assert!(in_sphere(center, pos(48, 64, 0), 48));
        assert!(!in_sphere(center, pos(48, 48, 0), 48));
    }

    #[test]
    fn in_sphere_boundary_is_inclusive() {
        let center = pos(0, 0, 0);
        assert!(in_sphere(center, pos(48, 0, 0), 48));
        assert!(!in_sphere(center, pos(49, 0, 0), 48));
    }

    #[test]
    fn classify_block_maps_bed_bell_and_job_site() {
        let mut head_props = WhiteBedLikeProperties::default(&Block::RED_BED);
        head_props.part = BedPart::Head;
        let head_state = head_props.to_state_id(&Block::RED_BED);
        assert_eq!(
            classify_block(&Block::RED_BED, head_state),
            Some(POI_TYPE_HOME)
        );

        let mut foot_props = WhiteBedLikeProperties::default(&Block::RED_BED);
        foot_props.part = BedPart::Foot;
        let foot_state = foot_props.to_state_id(&Block::RED_BED);
        assert_eq!(classify_block(&Block::RED_BED, foot_state), None);

        assert_eq!(
            classify_block(&Block::BELL, Block::BELL.default_state.id),
            Some(POI_TYPE_MEETING)
        );
        assert_eq!(
            classify_block(&Block::BARREL, Block::BARREL.default_state.id),
            Some(POI_TYPE_JOB_SITE)
        );
        assert_eq!(
            classify_block(&Block::STONE, Block::STONE.default_state.id),
            None
        );
    }
}

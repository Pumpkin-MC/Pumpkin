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
//! Pumpkin has no villager bed-claiming (vanilla's `AcquirePoi` behavior), so
//! every POI this registry tracks always reports "unoccupied" in vanilla's
//! terms (`PoiRecord.freeTickets` never decremented from `maxTickets`).
//! Faithfully filtering on `Occupancy.IS_OCCUPIED` would therefore make every
//! village-density query permanently return zero, a regression versus the
//! villager-count approximation this module replaces. We deliberately count
//! *any* POI of the matching type (vanilla's `Occupancy.ANY`) and defer
//! occupancy tracking until bed-claiming exists.
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
//! filtered to `BedPart.HEAD` states). This module classifies by `Block`
//! alone (no `BlockState` half tracking at the chokepoint), so both bed
//! halves are registered - a documented 2x overcount on `HOME` density that
//! does not change which side of the `> 4` threshold a real village falls
//! on, since real villages have far more beds than the threshold margin.

use pumpkin_data::Block;
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

/// Classifies a block for POI registration, or `None` if it isn't a
/// POI-bearing block. See module docs for the vanilla mapping this
/// approximates (`PoiTypes.forState`, `PoiTypes.java` ~91).
#[must_use]
pub fn classify_block(block: &Block) -> Option<&'static str> {
    if block.has_tag(&BlockTag::MINECRAFT_BEDS) {
        Some(POI_TYPE_HOME)
    } else if *block == Block::BELL {
        Some(POI_TYPE_MEETING)
    } else if block.has_tag(&BlockTag::C_VILLAGER_JOB_SITES) {
        Some(POI_TYPE_JOB_SITE)
    } else {
        None
    }
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
    let dx = i64::from(center.0.x - candidate.0.x);
    let dy = i64::from(center.0.y - candidate.0.y);
    let dz = i64::from(center.0.z - candidate.0.z);
    let radius = i64::from(radius);
    dx * dx + dy * dy + dz * dz <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::math::vector3::Vector3;

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos(Vector3::new(x, y, z))
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
        assert_eq!(classify_block(&Block::RED_BED), Some(POI_TYPE_HOME));
        assert_eq!(classify_block(&Block::BELL), Some(POI_TYPE_MEETING));
        assert_eq!(classify_block(&Block::BARREL), Some(POI_TYPE_JOB_SITE));
        assert_eq!(classify_block(&Block::STONE), None);
    }
}

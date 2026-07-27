//! Village detection for raids — Pumpkin's approximation of the vanilla POI graph.
//!
//! # The gap, stated plainly
//!
//! Vanilla answers "is this position in a village?" through `ServerLevel.isVillage`
//! (`/root/Vanilla/src/net/minecraft/server/level/ServerLevel.java:1313-1319`), which
//! calls `isCloseToVillage(pos, 1)` → `sectionsToVillage(SectionPos.of(pos))`
//! (`ServerLevel.java:1321-1330`) on the `PoiManager`. The `PoiManager` maintains a
//! per-section distance field over every occupied `#minecraft:village` POI (beds,
//! job sites, meeting points), so "village" means "within N sections of an occupied
//! village POI".
//!
//! Pumpkin has **no POI graph for village POIs**. `World::portal_poi` covers nether
//! portals only. The single piece of village-ish state that exists is
//! `VillagerEntity::home_pos` — the bed a villager has claimed — reachable through
//! `EntityBase::get_home_pos` (`pumpkin/src/entity/mob/entity_base.rs:275`).
//!
//! So this module approximates the POI distance field with **live villagers that have
//! claimed a bed**. That is a real, load-bearing deviation with these consequences:
//!
//! - A village whose villagers are all dead or unloaded stops being a village, so a
//!   raid there flips to `LOSS` (`Raid.java:265-274`) where vanilla would keep going
//!   off the still-present bed POIs.
//! - Job sites and meeting points (bells) do not count, only beds.
//! - Villagers that have not claimed a bed do not count at all.
//!
//! Every entry point below is written so that swapping in a real POI manager later
//! means changing this file and nothing else.
//!
//! The section-distance arithmetic itself is faithful: the caller-visible API mirrors
//! `isVillage` / `isCloseToVillage(pos, sectionDistance)` semantics, measured in
//! 16-block sections with Chebyshev distance, exactly like `SectionPos`-based
//! `sectionsToVillage`.

use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::world::World;

/// Vanilla `ServerLevel.isCloseToVillage` rejects any request above 6 sections
/// outright (`ServerLevel.java:1321-1327`).
pub const MAX_VILLAGE_SECTION_DISTANCE: i32 = 6;

/// Vanilla `Raids.createOrExtendRaid` gathers occupied village POIs within 64
/// blocks of the raid position (`Raids.java:118`).
pub const RAID_POI_SEARCH_RADIUS: f64 = 64.0;

/// Vanilla `Raid.VILLAGE_RADIUS_BUFFER` (`Raid.java:94`).
pub const VILLAGE_RADIUS_BUFFER: i32 = 16;

/// Section coordinate of a block coordinate (vanilla `SectionPos.blockToSectionCoord`).
const fn to_section(coord: i32) -> i32 {
    coord >> 4
}

/// Claimed-bed positions of loaded villagers within `radius` blocks of `pos`.
///
/// This is the approximation's substitute for
/// `PoiManager.getInRange(#minecraft:village, .., IS_OCCUPIED)` (`Raids.java:118`).
/// A claimed bed is the closest analogue Pumpkin has to an *occupied* village POI:
/// `home_pos` is only set once a villager actually claims the bed.
#[must_use]
pub fn occupied_village_poi_positions(
    world: &Arc<World>,
    pos: &BlockPos,
    radius: f64,
) -> Vec<BlockPos> {
    let center = pos.to_centered_f64();
    let radius_sq = radius * radius;
    let mut positions = Vec::new();
    for entity in world.entities.load().iter() {
        if entity.get_entity().entity_type != &EntityType::VILLAGER {
            continue;
        }
        let Some(home) = entity.get_home_pos() else {
            continue;
        };
        if home.to_centered_f64().squared_distance_to_vec(&center) <= radius_sq {
            positions.push(home);
        }
    }
    positions
}

/// Chebyshev section distance from `pos` to the nearest village POI, or `None`
/// when no POI is known.
///
/// Approximation of vanilla `ServerLevel.sectionsToVillage` (`ServerLevel.java:1329-1331`).
/// Vanilla precomputes a real BFS distance field in `PoiManager`; here the answer is
/// derived directly from the claimed-bed positions, which gives the same value for
/// the single-village case that matters to raids.
#[must_use]
pub fn sections_to_village(world: &Arc<World>, pos: &BlockPos) -> Option<i32> {
    // Search far enough to answer any query up to the vanilla 6-section cap.
    let search_radius = f64::from((MAX_VILLAGE_SECTION_DISTANCE + 1) * 16);
    let positions = occupied_village_poi_positions(world, pos, search_radius);
    section_distance_to_nearest(pos, &positions)
}

/// Pure section-distance reduction, split out so it can be tested without a `World`.
#[must_use]
pub fn section_distance_to_nearest(pos: &BlockPos, poi_positions: &[BlockPos]) -> Option<i32> {
    poi_positions
        .iter()
        .map(|poi| {
            let dx = (to_section(poi.0.x) - to_section(pos.0.x)).abs();
            let dy = (to_section(poi.0.y) - to_section(pos.0.y)).abs();
            let dz = (to_section(poi.0.z) - to_section(pos.0.z)).abs();
            dx.max(dy).max(dz)
        })
        .min()
}

/// Vanilla `ServerLevel.isCloseToVillage` (`ServerLevel.java:1321-1327`).
///
/// Returns `false` for `section_distance > 6`, matching vanilla's early out.
#[must_use]
pub fn is_close_to_village(world: &Arc<World>, pos: &BlockPos, section_distance: i32) -> bool {
    if section_distance > MAX_VILLAGE_SECTION_DISTANCE {
        return false;
    }
    sections_to_village(world, pos).is_some_and(|distance| distance <= section_distance)
}

/// Vanilla `ServerLevel.isVillage` (`ServerLevel.java:1313-1315`):
/// `isCloseToVillage(pos, 1)`.
#[must_use]
pub fn is_village(world: &Arc<World>, pos: &BlockPos) -> bool {
    is_close_to_village(world, pos, 1)
}

/// Vanilla `Raids.createOrExtendRaid` centre calculation (`Raids.java:118-131`).
///
/// Averages the occupied village POIs within 64 blocks of `raid_position` and floors
/// the result; falls back to `raid_position` when there are none, exactly as vanilla
/// does when `count == 0`.
#[must_use]
pub fn raid_center_for(world: &Arc<World>, raid_position: &BlockPos) -> BlockPos {
    let positions = occupied_village_poi_positions(world, raid_position, RAID_POI_SEARCH_RADIUS);
    average_center(raid_position, &positions)
}

/// Pure half of [`raid_center_for`], testable without a `World`.
///
/// Vanilla sums the POI positions into a `Vec3`, scales by `1/count`, then calls
/// `BlockPos.containing` (which floors each component) — `Raids.java:120-131`.
#[must_use]
pub fn average_center(fallback: &BlockPos, poi_positions: &[BlockPos]) -> BlockPos {
    let count = poi_positions.len();
    if count == 0 {
        return *fallback;
    }
    let mut total = Vector3::new(0.0f64, 0.0f64, 0.0f64);
    for poi in poi_positions {
        total = total.add_raw(f64::from(poi.0.x), f64::from(poi.0.y), f64::from(poi.0.z));
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "POI counts are far below f64's exact integer range"
    )]
    let scale = 1.0 / count as f64;
    BlockPos::floored(total.x * scale, total.y * scale, total.z * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_poi_means_no_village() {
        let pos = BlockPos::new(0, 64, 0);
        assert_eq!(section_distance_to_nearest(&pos, &[]), None);
    }

    #[test]
    fn same_section_is_distance_zero() {
        let pos = BlockPos::new(4, 64, 7);
        let poi = BlockPos::new(9, 68, 2);
        assert_eq!(section_distance_to_nearest(&pos, &[poi]), Some(0));
    }

    #[test]
    fn adjacent_section_is_distance_one() {
        // x = 20 lands in section 1, x = 4 in section 0.
        let pos = BlockPos::new(4, 64, 4);
        let poi = BlockPos::new(20, 64, 4);
        assert_eq!(section_distance_to_nearest(&pos, &[poi]), Some(1));
    }

    #[test]
    fn distance_is_chebyshev_not_manhattan() {
        // Three sections along x and three along z is still distance 3.
        let pos = BlockPos::new(0, 64, 0);
        let poi = BlockPos::new(48, 64, 48);
        assert_eq!(section_distance_to_nearest(&pos, &[poi]), Some(3));
    }

    #[test]
    fn vertical_sections_count_too() {
        let pos = BlockPos::new(0, 64, 0);
        let poi = BlockPos::new(0, 96, 0);
        assert_eq!(section_distance_to_nearest(&pos, &[poi]), Some(2));
    }

    #[test]
    fn nearest_poi_wins() {
        let pos = BlockPos::new(0, 64, 0);
        let far = BlockPos::new(200, 64, 0);
        let near = BlockPos::new(20, 64, 0);
        assert_eq!(section_distance_to_nearest(&pos, &[far, near]), Some(1));
    }

    #[test]
    fn negative_coordinates_floor_toward_negative_infinity() {
        // -1 >> 4 == -1, so a block at -1 sits in section -1, not 0.
        let pos = BlockPos::new(-1, 64, -1);
        let poi = BlockPos::new(0, 64, 0);
        assert_eq!(section_distance_to_nearest(&pos, &[poi]), Some(1));
    }

    #[test]
    fn center_falls_back_to_the_raid_position() {
        let fallback = BlockPos::new(11, 65, -7);
        assert_eq!(average_center(&fallback, &[]), fallback);
    }

    #[test]
    fn center_is_the_floored_mean_of_the_pois() {
        let fallback = BlockPos::new(0, 0, 0);
        let pois = [
            BlockPos::new(0, 64, 0),
            BlockPos::new(10, 64, 10),
            BlockPos::new(20, 70, 20),
        ];
        // Mean = (10, 66, 10) exactly.
        assert_eq!(average_center(&fallback, &pois), BlockPos::new(10, 66, 10));
    }

    #[test]
    fn center_floors_rather_than_rounds() {
        let fallback = BlockPos::new(0, 0, 0);
        // Mean x = 1/2 = 0.5 -> floors to 0; mean z = 3/2 = 1.5 -> floors to 1.
        let pois = [BlockPos::new(0, 64, 1), BlockPos::new(1, 64, 2)];
        assert_eq!(average_center(&fallback, &pois), BlockPos::new(0, 64, 1));
    }

    #[test]
    fn center_floors_negative_means_downward() {
        let fallback = BlockPos::new(0, 0, 0);
        // Mean x = -1/2 = -0.5 -> floors to -1, not 0.
        let pois = [BlockPos::new(0, 64, 0), BlockPos::new(-1, 64, 0)];
        assert_eq!(average_center(&fallback, &pois), BlockPos::new(-1, 64, 0));
    }
}

use pumpkin_data::structures::{
    ConcentricRingsStructurePlacement, RandomSpreadStructurePlacement, StructurePlacement,
    StructurePlacementType,
};
use pumpkin_util::math::{floor_div, position::BlockPos};

use crate::generation::structure::placement::{
    GlobalStructureCache, get_structure_chunk_in_region,
};

/// Block-level position of a found structure plus squared distance from the
/// search origin, used internally to track the running nearest candidate.
#[derive(Debug, Clone)]
pub struct FoundStructure {
    pub pos: BlockPos,
    pub distance_sq: f64,
}

/// Finds the block position of the nearest structure whose placement is listed
/// in `placements`, within `max_search_radius` chunk-region rings.
///
/// Mirrors the two-pass logic in vanilla's
/// `ChunkGenerator.findNearestMapStructure`:
///
/// 1. **Concentric-rings** placements (strongholds) are resolved in one pass
///    from the pre-computed [`GlobalStructureCache`].
/// 2. **Random-spread** placements are searched ring-by-ring outward, stopping
///    at the first radius that produces any result.
///
/// The best candidate from both passes is returned.
pub fn find_nearest_structure(
    origin: BlockPos,
    placements: &[&StructurePlacement],
    max_search_radius: i32,
    world_seed: i64,
    global_cache: &GlobalStructureCache,
    mut is_valid_pos: impl FnMut(BlockPos, &StructurePlacement) -> bool,
) -> Option<BlockPos> {
    if placements.is_empty() {
        return None;
    }

    let mut nearest: Option<FoundStructure> = None;

    // ── Pass 1: Concentric-rings (strongholds) ──────────────────────────────
    for p in placements {
        if let StructurePlacementType::ConcentricRings(rings) = &p.placement_type
            && let Some(found) = find_nearest_concentric(origin, rings, global_cache)
            && nearest
                .as_ref()
                .is_none_or(|n| found.distance_sq < n.distance_sq)
        {
            nearest = Some(found);
        }
    }

    let random_spread: Vec<(&StructurePlacement, &RandomSpreadStructurePlacement, u32)> =
        placements
            .iter()
            .filter_map(|p| {
                if let StructurePlacementType::RandomSpread(r) = &p.placement_type {
                    Some((*p, r, p.salt))
                } else {
                    None
                }
            })
            .collect();

    if !random_spread.is_empty() {
        let chunk_origin_x = origin.0.x >> 4;
        let chunk_origin_z = origin.0.z >> 4;

        'radius: for radius in 0..=max_search_radius {
            for (p, placement, salt) in &random_spread {
                if let Some(found) = find_nearest_random_spread_at_radius(
                    origin,
                    chunk_origin_x,
                    chunk_origin_z,
                    radius,
                    world_seed,
                    placement,
                    p,
                    *salt,
                    &mut is_valid_pos,
                ) {
                    if nearest
                        .as_ref()
                        .is_none_or(|n| found.distance_sq < n.distance_sq)
                    {
                        nearest = Some(found);
                    }
                    break 'radius;
                }
            }
        }
    }

    nearest.map(|f| f.pos)
}

fn find_nearest_concentric(
    origin: BlockPos,
    // Kept for potential future bounds / distance validation.
    _rings: &ConcentricRingsStructurePlacement,
    global_cache: &GlobalStructureCache,
) -> Option<FoundStructure> {
    let strongholds = global_cache.get_stronghold_chunks();
    if strongholds.is_empty() {
        return None;
    }

    let ox = origin.0.x as f64;
    let oz = origin.0.z as f64;

    strongholds
        .iter()
        .map(|(cx, cz)| {
            // Centre of the chunk in block coords.
            let bx = (cx << 4) + 8;
            let bz = (cz << 4) + 8;
            let dx = bx as f64 - ox;
            let dz = bz as f64 - oz;
            FoundStructure {
                pos: BlockPos::new(bx, 0, bz),
                distance_sq: dx * dx + dz * dz,
            }
        })
        .min_by(|a, b| a.distance_sq.partial_cmp(&b.distance_sq).unwrap())
}

#[allow(clippy::too_many_arguments)]
fn find_nearest_random_spread_at_radius(
    origin: BlockPos,
    chunk_origin_x: i32,
    chunk_origin_z: i32,
    radius: i32,
    world_seed: i64,
    placement: &RandomSpreadStructurePlacement,
    parent_placement: &StructurePlacement,
    salt: u32,
    is_valid_pos: &mut dyn FnMut(BlockPos, &StructurePlacement) -> bool,
) -> Option<FoundStructure> {
    let spacing = placement.spacing;
    let ox = origin.0.x as f64;
    let oz = origin.0.z as f64;

    let mut best: Option<FoundStructure> = None;

    for rx_off in -radius..=radius {
        for rz_off in -radius..=radius {
            if rx_off.abs() != radius && rz_off.abs() != radius {
                continue;
            }

            let rx = floor_div(chunk_origin_x, spacing) + rx_off;
            let rz = floor_div(chunk_origin_z, spacing) + rz_off;

            let (struct_cx, struct_cz) =
                get_structure_chunk_in_region(placement, world_seed, rx, rz, salt);

            let bx = (struct_cx << 4) + 8;
            let bz = (struct_cz << 4) + 8;
            let pos = BlockPos::new(bx, 0, bz);

            if is_valid_pos(pos, parent_placement) {
                let dx = bx as f64 - ox;
                let dz = bz as f64 - oz;
                let dist_sq = dx * dx + dz * dz;

                if best.as_ref().is_none_or(|b| dist_sq < b.distance_sq) {
                    best = Some(FoundStructure {
                        pos,
                        distance_sq: dist_sq,
                    });
                }
            }
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::placement::GlobalStructureCache;
    use pumpkin_data::structures::{
        RandomSpreadStructurePlacement, StructurePlacement, StructurePlacementType,
    };
    use pumpkin_util::math::position::BlockPos;

    #[test]
    fn test_find_nearest_structure_empty() {
        let origin = BlockPos::new(0, 0, 0);
        let global_cache = GlobalStructureCache::new();
        let result = find_nearest_structure(origin, &[], 100, 12345, &global_cache, |_, _| true);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_nearest_structure_validation_filter() {
        let origin = BlockPos::new(0, 0, 0);
        let global_cache = GlobalStructureCache::new();

        let placement = StructurePlacement {
            frequency_reduction_method: None,
            frequency: None,
            salt: 10387312,
            exclusion_zone: None,
            placement_type: StructurePlacementType::RandomSpread(RandomSpreadStructurePlacement {
                spacing: 32,
                separation: 8,
                spread_type: None,
            }),
        };

        // If validation closure always returns false, no structure should be found
        let result_false =
            find_nearest_structure(origin, &[&placement], 2, 12345, &global_cache, |_, _| false);
        assert!(result_false.is_none());

        // If validation closure always returns true, a structure should be found
        let result_true =
            find_nearest_structure(origin, &[&placement], 2, 12345, &global_cache, |_, _| true);
        assert!(result_true.is_some());
    }
}

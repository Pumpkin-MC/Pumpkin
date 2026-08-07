use pumpkin_data::structures::{
    ConcentricRingsStructurePlacement, RandomSpreadStructurePlacement, StructurePlacement,
    StructurePlacementType, StructureSet,
};
use pumpkin_util::math::{floor_div, position::BlockPos};

use crate::generation::structure::placement::{
    GlobalStructureCache, get_structure_chunk_in_region,
};

use super::WorldGenerator;

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

    let random_spread: Vec<(&RandomSpreadStructurePlacement, u32)> = placements
        .iter()
        .filter_map(|p| {
            if let StructurePlacementType::RandomSpread(r) = &p.placement_type {
                Some((r, p.salt))
            } else {
                None
            }
        })
        .collect();

    if !random_spread.is_empty() {
        let chunk_origin_x = origin.0.x >> 4;
        let chunk_origin_z = origin.0.z >> 4;

        'radius: for radius in 0..=max_search_radius {
            for (placement, salt) in &random_spread {
                if let Some(found) = find_nearest_random_spread_at_radius(
                    origin,
                    chunk_origin_x,
                    chunk_origin_z,
                    radius,
                    world_seed,
                    placement,
                    *salt,
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

/// Finds the nearest candidate that actually produces one of `target_structures`.
/// Explorer maps use this instead of pointing at a placement-only candidate whose
/// biome may reject the requested structure.
#[must_use]
#[expect(clippy::too_many_lines)]
pub fn find_nearest_structure_start(
    origin: BlockPos,
    structure_set: &StructureSet,
    target_structures: &[pumpkin_data::structures::StructureKeys],
    max_search_radius: i32,
    generator: &WorldGenerator,
) -> Option<BlockPos> {
    use crate::{
        ProtoChunk,
        biome::{BiomeSupplier, MultiNoiseBiomeSupplier},
        generation::{
            biome_coords,
            noise::router::{
                multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
                surface_height_sampler::{
                    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
                },
            },
            positions::chunk_pos::{start_block_x, start_block_z},
            structure::{
                lazily_generate_structure,
                placement::should_generate_structure,
                structures::{StructureGeneratorContext, create_chunk_random},
            },
        },
    };
    use pumpkin_data::structures::Structure;

    let WorldGenerator::Noise(noise_generator) = generator else {
        return None;
    };
    let StructurePlacementType::RandomSpread(placement) = &structure_set.placement.placement_type
    else {
        return None;
    };

    let chunk_origin_x = origin.0.x >> 4;
    let chunk_origin_z = origin.0.z >> 4;
    let region_origin_x = floor_div(chunk_origin_x, placement.spacing);
    let region_origin_z = floor_div(chunk_origin_z, placement.spacing);
    let world_seed = noise_generator.random_config.seed as i64;
    let global_cache = &noise_generator.global_structure_cache;

    for radius in 0..=max_search_radius {
        let mut nearest: Option<FoundStructure> = None;
        for region_x_offset in -radius..=radius {
            for region_z_offset in -radius..=radius {
                if region_x_offset.abs() != radius && region_z_offset.abs() != radius {
                    continue;
                }
                let (chunk_x, chunk_z) = get_structure_chunk_in_region(
                    placement,
                    world_seed,
                    region_origin_x + region_x_offset,
                    region_origin_z + region_z_offset,
                    structure_set.placement.salt,
                );
                let placement_chunk = ProtoChunk::new(chunk_x, chunk_z, generator);
                if !should_generate_structure(
                    &structure_set.placement,
                    &noise_generator.structure_calculator,
                    chunk_x,
                    chunk_z,
                    global_cache,
                    &placement_chunk,
                    &[],
                ) {
                    continue;
                }

                for &key in target_structures {
                    let start =
                        global_cache.get_or_compute_structure_start(key, chunk_x, chunk_z, || {
                            let start_x = start_block_x(chunk_x);
                            let start_z = start_block_z(chunk_z);
                            let settings = noise_generator.settings;
                            let mut height_sampler = SurfaceHeightEstimateSampler::generate(
                                &noise_generator.base_router.surface_estimator,
                                &SurfaceHeightSamplerBuilderOptions::new(
                                    biome_coords::from_block(start_x),
                                    biome_coords::from_block(start_z),
                                    4,
                                    settings.shape.min_y as i32,
                                    settings.shape.height as i32,
                                    (settings.shape.height
                                        / settings.shape.vertical_cell_block_count() as u16)
                                        as usize,
                                ),
                            );
                            let mut biome_sampler = MultiNoiseSampler::generate(
                                &noise_generator.base_router.multi_noise,
                                &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
                            );
                            let biome_supplier: &dyn BiomeSupplier =
                                &MultiNoiseBiomeSupplier::OVERWORLD;
                            let context = StructureGeneratorContext {
                                seed: world_seed,
                                chunk_x,
                                chunk_z,
                                random: create_chunk_random(world_seed, chunk_x, chunk_z),
                                sea_level: settings.sea_level,
                                min_y: noise_generator.dimension.min_y,
                                height_sampler: Some(&mut height_sampler),
                                structure_key: Some(key),
                            };
                            lazily_generate_structure(
                                &key,
                                Structure::get(&key),
                                context,
                                biome_supplier,
                                &mut biome_sampler,
                            )
                        });
                    let Some(start) = start else {
                        continue;
                    };
                    let position = start.start_pos;
                    let dx = f64::from(position.0.x - origin.0.x);
                    let dz = f64::from(position.0.z - origin.0.z);
                    let found = FoundStructure {
                        pos: position,
                        distance_sq: dx * dx + dz * dz,
                    };
                    if nearest
                        .as_ref()
                        .is_none_or(|current| found.distance_sq < current.distance_sq)
                    {
                        nearest = Some(found);
                    }
                }
            }
        }
        if let Some(found) = nearest {
            return Some(found.pos);
        }
    }
    None
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

fn find_nearest_random_spread_at_radius(
    origin: BlockPos,
    chunk_origin_x: i32,
    chunk_origin_z: i32,
    radius: i32,
    world_seed: i64,
    placement: &RandomSpreadStructurePlacement,
    salt: u32,
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
            let dx = bx as f64 - ox;
            let dz = bz as f64 - oz;
            let dist_sq = dx * dx + dz * dz;

            if best.as_ref().is_none_or(|b| dist_sq < b.distance_sq) {
                best = Some(FoundStructure {
                    pos: BlockPos::new(bx, 0, bz),
                    distance_sq: dist_sq,
                });
            }
        }
    }

    best
}

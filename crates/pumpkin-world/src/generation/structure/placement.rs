use pumpkin_data::structures::{
    ConcentricRingsStructurePlacement, FrequencyReductionMethod, RandomSpreadStructurePlacement,
    SpreadType, StructurePlacement, StructurePlacementCalculator, StructurePlacementType,
    StructureSet,
};
use pumpkin_util::{
    math::floor_div,
    random::{
        RandomGenerator, RandomImpl, get_large_feature_seed, get_region_seed,
        legacy_rand::LegacyRand,
    },
};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::sync::OnceLock;

use crate::biome::{BiomeSupplier, MultiNoiseBiomeSupplier};
use crate::generation::noise::router::{
    multi_noise_sampler::MultiNoiseSampler, proto_noise_router::ProtoMultiNoiseRouter,
};
use dashmap::DashMap;
use pumpkin_data::structures::StructureKeys;

use super::structures::StructurePosition;
/// A thread-safe global cache for structures that require world-wide placement calculations
/// rather than localized chunk-based math (e.g., Strongholds using Concentric Rings).
///
/// This prevents chunk generation deadlocks by allowing chunks to query a pre-calculated
/// mathematical layout in `O(1)` time instead of triggering cascading chunk loads.
pub struct GlobalStructureCache {
    /// A cached list of mathematically predicted (`chunk_x`, `chunk_z`) coordinates.
    stronghold_chunks: OnceLock<Vec<(i32, i32)>>,
    /// Memoized structure starts, keyed by (structure, start chunk x, start chunk z).
    ///
    /// A jigsaw structure's placement is fully determined by its start chunk and the
    /// world seed, so it is computed once here instead of being recomputed for every
    /// surrounding chunk whose structure references overlap it.
    structure_starts: OnceLock<DashMap<(StructureKeys, i32, i32), Option<StructurePosition>>>,
}

struct RingTask {
    initial_x: i32,
    initial_z: i32,
    search_rng: LegacyRand,
}

impl GlobalStructureCache {
    /// Creates a new, empty global structure cache.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stronghold_chunks: OnceLock::new(),
            structure_starts: OnceLock::new(),
        }
    }

    pub fn get_stronghold_chunks(&self) -> &[(i32, i32)] {
        self.stronghold_chunks
            .get()
            .map_or(&[], std::vec::Vec::as_slice)
    }

    pub fn init_strongholds(&self, chunks: Vec<(i32, i32)>) {
        let _ = self.stronghold_chunks.set(chunks);
    }

    /// Returns the memoized structure start for the given structure and start chunk,
    /// computing it via `compute` on the first request and caching the result.
    ///
    /// Because a structure's placement depends only on its start chunk and the world
    /// seed, every chunk whose references overlap that structure can reuse the cached
    /// result instead of re-running the expensive jigsaw expansion.
    pub fn get_or_compute_structure_start(
        &self,
        key: StructureKeys,
        chunk_x: i32,
        chunk_z: i32,
        compute: impl FnOnce() -> Option<StructurePosition>,
    ) -> Option<StructurePosition> {
        let cache = self.structure_starts.get_or_init(DashMap::new);
        if let Some(cached) = cache.get(&(key, chunk_x, chunk_z)) {
            return cached.value().clone();
        }
        let computed = compute();
        cache.insert((key, chunk_x, chunk_z), computed.clone());
        computed
    }

    /// Calculates the 128 ring positions matching vanilla Minecraft's
    /// `ChunkGeneratorStructureState.generateRingPositions` exactly.
    #[must_use]
    pub fn calculate_strongholds(
        seed: i64,
        placement: &ConcentricRingsStructurePlacement,
        multi_noise: &ProtoMultiNoiseRouter,
    ) -> Vec<(i32, i32)> {
        let distance_param = f64::from(placement.distance);
        let mut spread = placement.spread;
        let count = placement.count;

        let preferred_biomes = pumpkin_data::tag::get_tag_ids(
            pumpkin_data::tag::RegistryKey::WorldgenBiome,
            placement
                .preferred_biomes
                .strip_prefix('#')
                .unwrap_or(placement.preferred_biomes),
        )
        .unwrap_or(&[]);

        let mut random = LegacyRand::from_seed(seed as u64);
        let mut angle = random.next_f64() * PI * 2.0;
        let mut position_in_circle = 0;
        let mut circle = 0;

        let mut tasks = Vec::with_capacity(count as usize);

        for i in 0..count {
            let dist = 4.0 * distance_param
                + distance_param * f64::from(circle) * 6.0
                + (random.next_f64() - 0.5) * (distance_param * 2.5);

            let initial_x = (angle.cos() * dist + 0.5).floor() as i32;
            let initial_z = (angle.sin() * dist + 0.5).floor() as i32;

            let fork_seed = random.next_i64();
            let search_rng = LegacyRand::from_seed(fork_seed as u64);

            tasks.push(RingTask {
                initial_x,
                initial_z,
                search_rng,
            });

            angle += (PI * 2.0) / f64::from(spread);
            position_in_circle += 1;

            if position_in_circle == spread {
                circle += 1;
                position_in_circle = 0;

                spread += 2 * spread / (circle + 1);
                spread = spread.min(count - i);
                angle += random.next_f64() * PI * 2.0;
            }
        }

        tasks
            .into_par_iter()
            .map(|mut task| {
                let mut sampler = MultiNoiseSampler::generate(multi_noise);
                let noise_center_x = (task.initial_x << 2) + 2;
                let noise_center_z = (task.initial_z << 2) + 2;

                let mut result = None;
                let mut found = 0;

                for z in -28..=28 {
                    for x in -28..=28 {
                        let noise_x = noise_center_x + x;
                        let noise_z = noise_center_z + z;
                        let biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(
                            noise_x,
                            0,
                            noise_z,
                            &mut sampler,
                        );
                        if preferred_biomes.contains(&(biome.id as u16)) {
                            if result.is_none() || task.search_rng.next_bounded_i32(found + 1) == 0
                            {
                                result = Some((noise_x >> 2, noise_z >> 2));
                            }
                            found += 1;
                        }
                    }
                }

                result.unwrap_or((task.initial_x, task.initial_z))
            })
            .collect()
    }

    #[must_use]
    pub fn calculate_strongholds_without_biomes(
        seed: i64,
        placement: &ConcentricRingsStructurePlacement,
    ) -> Vec<(i32, i32)> {
        let distance_param = f64::from(placement.distance);
        let mut spread = placement.spread;
        let count = placement.count;

        let mut random = LegacyRand::from_seed(seed as u64);
        let mut angle = random.next_f64() * PI * 2.0;
        let mut position_in_circle = 0;
        let mut circle = 0;

        let mut chunks = Vec::with_capacity(count as usize);

        for i in 0..count {
            let dist = 4.0 * distance_param
                + distance_param * f64::from(circle) * 6.0
                + (random.next_f64() - 0.5) * (distance_param * 2.5);

            let initial_x = (angle.cos() * dist + 0.5).floor() as i32;
            let initial_z = (angle.sin() * dist + 0.5).floor() as i32;

            chunks.push((initial_x, initial_z));

            angle += (PI * 2.0) / f64::from(spread);
            position_in_circle += 1;

            if position_in_circle == spread {
                circle += 1;
                position_in_circle = 0;

                spread += 2 * spread / (circle + 1);
                spread = spread.min(count - i);
                angle += random.next_f64() * PI * 2.0;
            }
        }

        chunks
    }

    /// Retrieves the list of chunk coordinates for Concentric Ring structures.
    /// If the cache is empty, it calculates the 128 ring positions.
    pub fn get_or_calculate_strongholds(
        &self,
        seed: i64,
        placement: &ConcentricRingsStructurePlacement,
        multi_noise: Option<&ProtoMultiNoiseRouter>,
    ) -> &[(i32, i32)] {
        self.stronghold_chunks.get_or_init(|| {
            multi_noise.map_or_else(
                || Self::calculate_strongholds_without_biomes(seed, placement),
                |multi_noise| Self::calculate_strongholds(seed, placement, multi_noise),
            )
        })
    }
}

impl Default for GlobalStructureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn should_generate_structure(
    placement: &StructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    global_cache: &GlobalStructureCache,
) -> bool {
    is_start_chunk(
        &placement.placement_type,
        calculator,
        chunk_x,
        chunk_z,
        placement.salt,
        global_cache,
    ) && apply_frequency_reduction(
        placement.frequency_reduction_method,
        calculator.seed,
        chunk_x,
        chunk_z,
        placement.salt,
        placement.frequency.unwrap_or(1.0),
    ) && !placement.exclusion_zone.as_ref().is_some_and(|zone| {
        let set_name = zone
            .other_set
            .strip_prefix("minecraft:")
            .unwrap_or(zone.other_set);
        StructureSet::get(set_name).is_some_and(|set| {
            (chunk_x - zone.chunk_count..=chunk_x + zone.chunk_count).any(|x| {
                (chunk_z - zone.chunk_count..=chunk_z + zone.chunk_count).any(|z| {
                    should_generate_structure(&set.placement, calculator, x, z, global_cache)
                })
            })
        })
    })
}

fn apply_frequency_reduction(
    method: Option<FrequencyReductionMethod>,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    frequency: f32,
) -> bool {
    if frequency >= 1.0 {
        return true;
    }

    let method = method.unwrap_or(FrequencyReductionMethod::Default);
    should_generate_frequency(method, seed, chunk_x, chunk_z, salt, frequency)
}

fn should_generate_frequency(
    method: FrequencyReductionMethod,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    frequency: f32,
) -> bool {
    match method {
        FrequencyReductionMethod::Default => {
            let region_seed = get_region_seed(seed as u64, salt as i32, chunk_x, chunk_z as u32);
            let mut random = LegacyRand::from_seed(region_seed);
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType1 => {
            let x = chunk_x >> 4;
            let z = chunk_z >> 4;
            let mut random = LegacyRand::from_seed((i64::from(x ^ (z << 4)) ^ seed) as u64);
            random.next_i32();
            random.next_bounded_i32((1.0 / frequency) as i32) == 0
        }
        FrequencyReductionMethod::LegacyType2 => {
            let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, 10387320);
            let mut random = LegacyRand::from_seed(region_seed);
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType3 => {
            let feature_seed = get_large_feature_seed(seed as u64, chunk_x, chunk_z);
            let mut random = LegacyRand::from_seed(feature_seed);
            random.next_f64() < f64::from(frequency)
        }
    }
}

fn is_start_chunk(
    placement_type: &StructurePlacementType,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    global_cache: &GlobalStructureCache,
) -> bool {
    match placement_type {
        StructurePlacementType::RandomSpread(placement) => {
            is_start_chunk_random_spread(placement, calculator, chunk_x, chunk_z, salt)
        }
        StructurePlacementType::ConcentricRings(placement) => {
            let strongholds =
                global_cache.get_or_calculate_strongholds(calculator.seed, placement, None);
            strongholds.contains(&(chunk_x, chunk_z))
        }
    }
}

/// Predicts the exact chunk (X, Z) where a structure will attempt to spawn in a given Region (rx, rz).
#[must_use]
pub fn get_structure_chunk_in_region(
    placement: &RandomSpreadStructurePlacement,
    seed: i64,
    rx: i32,
    rz: i32,
    salt: u32,
) -> (i32, i32) {
    let region_seed = get_region_seed(seed as u64, rx, rz, salt);
    let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));

    let bound = placement.spacing - placement.separation;
    let spread_type = placement.spread_type.unwrap_or(SpreadType::Linear);

    let rand_x = spread_type.get(&mut random, bound);
    let rand_z = spread_type.get(&mut random, bound);

    (
        rx * placement.spacing + rand_x,
        rz * placement.spacing + rand_z,
    )
}

fn get_start_chunk_random_spread(
    placement: &RandomSpreadStructurePlacement,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> (i32, i32) {
    // 1. Find the region
    let rx = floor_div(chunk_x, placement.spacing);
    let rz = floor_div(chunk_z, placement.spacing);

    // 2. Get the structure chunk for that region
    get_structure_chunk_in_region(placement, seed, rx, rz, salt)
}

fn is_start_chunk_random_spread(
    placement: &RandomSpreadStructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> bool {
    let pos = get_start_chunk_random_spread(placement, calculator.seed, chunk_x, chunk_z, salt);
    (chunk_x == pos.0) && (chunk_z == pos.1)
}
#[cfg(test)]
mod tests {
    use pumpkin_data::structures::{
        RandomSpreadStructurePlacement, StructurePlacementCalculator, StructureSet,
    };
    use pumpkin_util::random::{
        RandomGenerator, RandomImpl, get_region_seed, legacy_rand::LegacyRand,
    };

    use crate::generation::structure::placement::{
        GlobalStructureCache, apply_frequency_reduction, get_start_chunk_random_spread,
        is_start_chunk, should_generate_structure,
    };

    #[test]
    fn get_start_chunk_random() {
        let region_seed = get_region_seed(123, 1, 1, 14357620);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));
        assert_eq!(random.next_bounded_i32(32 - 8), 8);
    }

    #[test]
    fn get_start_chunk() {
        let random = RandomSpreadStructurePlacement {
            spacing: 32,
            separation: 8,
            spread_type: None,
        };
        let (x, z) = get_start_chunk_random_spread(&random, 123, 1, 1, 14357620);
        assert_eq!(x, 5);
        assert_eq!(z, 4);
    }

    #[test]
    fn pillager_outposts_respect_the_village_exclusion_zone() {
        let seed = 0;
        let calculator = StructurePlacementCalculator::new(seed);
        let cache = GlobalStructureCache::new();
        let outposts = &StructureSet::PILLAGER_OUTPOSTS;
        let villages = &StructureSet::VILLAGES;
        let excluded = (-1002, -595);
        assert!(is_start_chunk(
            &outposts.placement.placement_type,
            &calculator,
            excluded.0,
            excluded.1,
            outposts.placement.salt,
            &cache,
        ));
        assert!(apply_frequency_reduction(
            outposts.placement.frequency_reduction_method,
            seed,
            excluded.0,
            excluded.1,
            outposts.placement.salt,
            outposts.placement.frequency.unwrap(),
        ));
        assert!((-10..=10).any(|dx| {
            (-10..=10).any(|dz| {
                is_start_chunk(
                    &villages.placement.placement_type,
                    &calculator,
                    excluded.0 + dx,
                    excluded.1 + dz,
                    villages.placement.salt,
                    &cache,
                )
            })
        }));

        assert!(!should_generate_structure(
            &outposts.placement,
            &calculator,
            excluded.0,
            excluded.1,
            &cache,
        ));
    }
}

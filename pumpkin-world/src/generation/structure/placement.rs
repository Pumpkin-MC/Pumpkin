use pumpkin_data::structures::{
    ConcentricRingsStructurePlacement, FrequencyReductionMethod, RandomSpreadStructurePlacement,
    SpreadType, StructurePlacement, StructurePlacementCalculator, StructurePlacementType,
};
use pumpkin_util::{
    math::floor_div,
    random::{
        RandomGenerator, RandomImpl, get_carver_seed, get_region_seed, legacy_rand::LegacyRand,
        xoroshiro128::Xoroshiro,
    },
};
use std::f64::consts::PI;
use std::sync::OnceLock;

use crate::biome::BiomeSupplier;
use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
/// A thread-safe global cache for structures that require world-wide placement calculations
/// rather than localized chunk-based math (e.g., Strongholds using Concentric Rings).
///
/// This prevents chunk generation deadlocks by allowing chunks to query a pre-calculated
/// mathematical layout in `O(1)` time instead of triggering cascading chunk loads.
pub struct GlobalStructureCache {
    /// A cached list of mathematically predicted (chunk_x, chunk_z) coordinates.
    stronghold_chunks: OnceLock<Vec<(i32, i32)>>,
}
impl GlobalStructureCache {
    /// Creates a new, empty global structure cache.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stronghold_chunks: OnceLock::new(),
        }
    }

    /// Retrieves the list of chunk coordinates for Concentric Ring structures.
    /// If the cache is empty, it calculates the 128 ring positions mathematically.
    #[allow(clippy::cast_precision_loss)]
    pub fn get_or_calculate_strongholds(
        &self,
        seed: i64,
        placement: &ConcentricRingsStructurePlacement,
        biome_supplier: &dyn BiomeSupplier,
        multi_noise_sampler: &mut MultiNoiseSampler,
        allowed_biomes: &[u16],
    ) -> &[(i32, i32)] {
        self.stronghold_chunks.get_or_init(|| {
            let mut chunks = Vec::with_capacity(placement.count as usize);

            let distance_param = f64::from(placement.distance); // Usually 32
            let mut spread = placement.spread; // Usually 3
            let count = placement.count; // Usually 128

            // The random generator for stronghold placement is based on the world seed and a fixed salt.
            // This ensures that the stronghold layout is consistent across all worlds with the same seed.
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed as u64));

            // The initial angle includes a random rotation jitter for the whole world
            let mut angle = random.next_f64() * PI * 2.0;
            let mut position_in_circle = 0;
            let mut circle = 0;

            for i in 0..count {
                // 1. Distance Formula
                // dist = (4 * spacing) + (spacing * ring_index * 6) + (random_jitter)
                // The jitter is +/- (spacing * 1.25)
                let dist = 4.0 * distance_param
                    + distance_param * f64::from(circle) * 6.0
                    + (random.next_f64() - 0.5) * (distance_param * 2.5);

                let initial_x = (angle.cos() * dist + 0.5).floor() as i32;
                let initial_z = (angle.sin() * dist + 0.5).floor() as i32;

                // 2. RNG Forking
                // We must fork/split the random generator for the biome search.
                // This keeps the main angle/distance sequence identical across all worlds.
                let fork_seed = random.next_i64();

                let mut biome_search_generator =
                    RandomGenerator::Legacy(LegacyRand::from_seed(fork_seed as u64));

                // 3. Reservoir Sampling Biome Search
                // Strongholds search the entire 112-block square
                // and pick one valid location at random (Reservoir Sampling).
                let mut found_pos = None;
                let mut found_count = 0;

                let center_block_x = (initial_x << 4) + 8;
                let center_block_z = (initial_z << 4) + 8;
                let step = 4;
                let search_radius = 112;

                for dz in (-search_radius..=search_radius).step_by(step as usize) {
                    for dx in (-search_radius..=search_radius).step_by(step as usize) {
                        let test_x = center_block_x + dx;
                        let test_z = center_block_z + dz;

                        let biome_x = crate::generation::biome_coords::from_block(test_x);
                        let biome_y = crate::generation::biome_coords::from_block(0);
                        let biome_z = crate::generation::biome_coords::from_block(test_z);

                        let biome =
                            biome_supplier.biome(biome_x, biome_y, biome_z, multi_noise_sampler);

                        if allowed_biomes.contains(&(biome.id as u16)) {
                            found_count += 1;
                            // Reservoir sampling: Pick the Nth valid biome with 1/N probability
                            if found_pos.is_none()
                                || biome_search_generator.next_bounded_i32(found_count) == 0
                            {
                                found_pos = Some((test_x >> 4, test_z >> 4));
                            }
                        }
                    }
                }

                let (final_chunk_x, final_chunk_z) = found_pos.unwrap_or((initial_x, initial_z));
                chunks.push((final_chunk_x, final_chunk_z));

                // 4. Dynamic Ring Progression
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
        })
    }
}

impl Default for GlobalStructureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
#[expect(clippy::too_many_arguments)]
pub fn should_generate_structure(
    placement: &StructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    global_cache: &GlobalStructureCache,
    biome_supplier: &dyn BiomeSupplier,
    multi_noise_sampler: &mut MultiNoiseSampler,
    allowed_biomes: &[u16],
) -> bool {
    is_start_chunk(
        &placement.placement_type,
        calculator,
        chunk_x,
        chunk_z,
        placement.salt,
        global_cache,
        biome_supplier,
        multi_noise_sampler,
        allowed_biomes,
    ) && apply_frequency_reduction(
        placement.frequency_reduction_method,
        calculator.seed,
        chunk_x,
        chunk_z,
        placement.salt,
        placement.frequency.unwrap_or(1.0),
    )
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
            let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, salt);
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType1 => {
            let x = chunk_x >> 4;
            let z = chunk_z >> 4;
            let mut random =
                RandomGenerator::Xoroshiro(Xoroshiro::from_seed((x ^ z << 4) as u64 ^ seed as u64));
            random.next_i32();
            random.next_bounded_i32((1.0 / frequency) as i32) == 0
        }
        FrequencyReductionMethod::LegacyType2 => {
            let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, 10387320);
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType3 => {
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed as u64));
            let carver_seed = get_carver_seed(&mut random, seed as u64, chunk_x, chunk_z);
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed));
            random.next_f64() < f64::from(frequency)
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn is_start_chunk(
    placement_type: &StructurePlacementType,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    global_cache: &GlobalStructureCache,
    biome_supplier: &dyn BiomeSupplier,
    multi_noise_sampler: &mut MultiNoiseSampler,
    allowed_biomes: &[u16],
) -> bool {
    match placement_type {
        StructurePlacementType::RandomSpread(placement) => {
            is_start_chunk_random_spread(placement, calculator, chunk_x, chunk_z, salt)
        }
        StructurePlacementType::ConcentricRings(placement) => {
            let strongholds = global_cache.get_or_calculate_strongholds(
                calculator.seed,
                placement,
                biome_supplier,
                multi_noise_sampler,
                allowed_biomes,
            );
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
    use pumpkin_data::structures::RandomSpreadStructurePlacement;
    use pumpkin_util::random::{
        RandomGenerator, RandomImpl, get_region_seed, legacy_rand::LegacyRand,
    };

    use crate::generation::structure::placement::get_start_chunk_random_spread;

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
}

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
    ) -> &[(i32, i32)] {
        self.stronghold_chunks.get_or_init(|| {
            let mut chunks = Vec::with_capacity(placement.count as usize);

            let mut distance = f64::from(placement.distance);
            let mut current_ring_count = placement.spread;
            let mut current_ring_index = 0;

            // Derive an initial angle from the world seed
            let mut angle = (seed as f64).sin() * PI * 2.0;

            for _ in 0..placement.count {
                let chunk_x = (angle.cos() * distance).round() as i32;
                let chunk_z = (angle.sin() * distance).round() as i32;

                chunks.push((chunk_x, chunk_z));

                angle += (PI * 2.0) / f64::from(current_ring_count);
                current_ring_index += 1;

                if current_ring_index == current_ring_count {
                    current_ring_index = 0;
                    current_ring_count += current_ring_count * 2 / 5;
                    distance += f64::from(placement.distance) * 1.25;
                    angle += (PI * 2.0) / f64::from(current_ring_count);
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
            is_start_chunk_concentric_rings(placement, calculator, chunk_x, chunk_z, global_cache)
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

fn is_start_chunk_concentric_rings(
    placement: &ConcentricRingsStructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    global_cache: &GlobalStructureCache,
) -> bool {
    let strongholds = global_cache.get_or_calculate_strongholds(calculator.seed, placement);
    strongholds.contains(&(chunk_x, chunk_z))
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

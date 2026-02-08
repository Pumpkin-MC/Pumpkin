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
#[must_use]
pub fn should_generate_structure(
    placement: &StructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
) -> bool {
    is_start_chunk(
        &placement.placement_type,
        calculator,
        chunk_x,
        chunk_z,
        placement.salt,
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
            random.next_f64() < frequency as f64
        }
    }
}

fn is_start_chunk(
    placement_type: &StructurePlacementType,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> bool {
    match placement_type {
        StructurePlacementType::RandomSpread(placement) => {
            is_start_chunk_random_spread(placement, calculator, chunk_x, chunk_z, salt)
        }
        StructurePlacementType::ConcentricRings(placement) => {
            is_start_chunk_concentric_rings(placement, calculator, chunk_x, chunk_z, salt)
        }
    }
}

fn get_start_chunk_random_spread(
    placement: &RandomSpreadStructurePlacement,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> (i32, i32) {
    let x = floor_div(chunk_x, placement.spacing);
    let z = floor_div(chunk_z, placement.spacing);
    let region_seed = get_region_seed(seed as u64, x, z, salt);
    let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));
    let bound = placement.spacing - placement.separation;
    let spread_type = placement.spread_type.unwrap_or(SpreadType::Linear);
    let rand_x = spread_type.get(&mut random, bound);
    let rand_z = spread_type.get(&mut random, bound);
    (
        x * placement.spacing + rand_x,
        z * placement.spacing + rand_z,
    )
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
    _placement: &ConcentricRingsStructurePlacement,
    _calculator: &StructurePlacementCalculator,
    _chunk_x: i32,
    _chunk_z: i32,
    _salt: u32,
) -> bool {
    // TODO: Implement proper concentric rings logic
    rand::random_bool(1.0 / 1000.0)
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

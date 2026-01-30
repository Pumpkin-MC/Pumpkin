use pumpkin_util::{
    math::floor_div,
    random::{
        RandomGenerator, RandomImpl, get_carver_seed, get_region_seed, legacy_rand::LegacyRand,
        xoroshiro128::Xoroshiro,
    },
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StructurePlacement {
    frequency_reduction_method: Option<FrequencyReductionMethod>,
    frequency: Option<f32>,
    salt: u32,
    #[serde(flatten)]
    r#type: StructurePlacementType,
}

impl StructurePlacement {
    #[must_use]
    pub fn should_generate(
        &self,
        calculator: &StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
    ) -> bool {
        self.r#type
            .is_start_chunk(calculator, chunk_x, chunk_z, self.salt)
            && self.apply_frequency_reduction(calculator.seed, chunk_x, chunk_z)
        // TODO: add exclusion_zone, only used for pillager_outposts
    }

    fn apply_frequency_reduction(&self, seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
        let frequency = self.frequency.unwrap_or(1.0);
        frequency >= 1.0
            || self
                .frequency_reduction_method
                .as_ref()
                .unwrap_or(&FrequencyReductionMethod::Default)
                .should_generate(seed, chunk_x, chunk_z, self.salt, frequency)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrequencyReductionMethod {
    Default,
    #[serde(rename = "legacy_type_1")]
    LegacyType1,
    #[serde(rename = "legacy_type_2")]
    LegacyType2,
    #[serde(rename = "legacy_type_3")]
    LegacyType3,
}

impl FrequencyReductionMethod {
    #[must_use]
    pub fn should_generate(
        &self,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        salt: u32,
        frequency: f32,
    ) -> bool {
        match self {
            Self::Default => {
                let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, salt);
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
                random.next_f32() < frequency
            }
            Self::LegacyType1 => {
                let x = chunk_x >> 4;
                let z = chunk_z >> 4;
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(
                    (x ^ z << 4) as u64 ^ seed as u64,
                ));
                random.next_i32(); // yeah mojang just does that and does not use the value
                random.next_bounded_i32((1.0 / frequency) as i32) == 0
            }
            Self::LegacyType2 => {
                let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, 10387320);
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
                random.next_f32() < frequency
            }
            Self::LegacyType3 => {
                let mut random: RandomGenerator =
                    RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed as u64));
                let carver_seed = get_carver_seed(&mut random, seed as u64, chunk_x, chunk_z);
                let mut random: RandomGenerator =
                    RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed));

                random.next_f64() < frequency as f64
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum StructurePlacementType {
    #[serde(rename = "minecraft:random_spread")]
    RandomSpread(RandomSpreadStructurePlacement),
    #[serde(rename = "minecraft:concentric_rings")]
    ConcentricRings(ConcentricRingsStructurePlacement),
}

impl StructurePlacementType {
    #[must_use]
    pub fn is_start_chunk(
        &self,
        calculator: &StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
        salt: u32,
    ) -> bool {
        match self {
            Self::RandomSpread(placement) => {
                placement.is_start_chunk(calculator, chunk_x, chunk_z, salt)
            }
            // This is needed for Stronghold, since it is placed in rings
            Self::ConcentricRings(placement) => {
                placement.is_start_chunk(calculator, chunk_x, chunk_z, salt)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct RandomSpreadStructurePlacement {
    spacing: i32,
    separation: i32,
    spread_type: Option<SpreadType>,
}

#[derive(Deserialize)]
pub struct ConcentricRingsStructurePlacement {
    spread: i32,
    distance: i32,
    count: i32,
    preferred_biomes: String,
}

impl ConcentricRingsStructurePlacement {
    #[must_use]
    pub fn is_start_chunk(
        &self,
        _calculator: &StructurePlacementCalculator,
        _chunk_x: i32,
        _chunk_z: i32,
        _salt: u32,
    ) -> bool {
        // TODO: this is entirely wrong, but the original logic is too complex for now
        // let x = (chunk_x << 4) as f64;
        // let z = (chunk_z << 4) as f64;
        // let distance_sq = x * x + z * z;
        // let distance = distance_sq.sqrt();

        // let ring_width = 1000.0;
        // let ring_gap = 2000.0;

        // let in_ring = (distance % (ring_width + ring_gap)) < ring_width;

        // if !in_ring {
        //     return false;
        // }

        // let seed = (chunk_x as i64).wrapping_mul(341873128712)
        //     ^ (chunk_z as i64).wrapping_mul(132897987541)
        //     ^ (salt as i64);

        // (seed.abs() % 400) == 0
        rand::random_bool(1.0 / 1000.0)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpreadType {
    Linear,
    Triangular,
}

impl SpreadType {
    pub fn get(&self, random: &mut RandomGenerator, bound: i32) -> i32 {
        match self {
            Self::Linear => random.next_bounded_i32(bound),
            Self::Triangular => i32::midpoint(
                random.next_bounded_i32(bound),
                random.next_bounded_i32(bound),
            ),
        }
    }
}

impl RandomSpreadStructurePlacement {
    fn get_start_chunk(&self, seed: i64, chunk_x: i32, chunk_z: i32, salt: u32) -> (i32, i32) {
        let x = floor_div(chunk_x, self.spacing);
        let z = floor_div(chunk_z, self.spacing);
        let region_seed = get_region_seed(seed as u64, x, z, salt);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));
        let bound = self.spacing - self.separation;
        let spread_type = self.spread_type.as_ref().unwrap_or(&SpreadType::Linear);
        let rand_x = spread_type.get(&mut random, bound);
        let rand_z = spread_type.get(&mut random, bound);
        (x * self.spacing + rand_x, z * self.spacing + rand_z)
    }

    #[must_use]
    pub fn is_start_chunk(
        &self,
        calculator: &StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
        salt: u32,
    ) -> bool {
        let pos = self.get_start_chunk(calculator.seed, chunk_x, chunk_z, salt);
        (chunk_x == pos.0) && (chunk_z == pos.1)
    }
}

pub struct StructurePlacementCalculator {
    pub seed: i64,
}

impl StructurePlacementCalculator {
    #[must_use]
    pub const fn new(seed: i64) -> Self {
        Self { seed }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::{
        RandomGenerator, RandomImpl, get_region_seed, legacy_rand::LegacyRand,
    };

    use crate::generation::structure::placement::RandomSpreadStructurePlacement;

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
        let (x, z) = random.get_start_chunk(123, 1, 1, 14357620);
        assert_eq!(x, 5);
        assert_eq!(z, 4);
    }
}

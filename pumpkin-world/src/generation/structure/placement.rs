use pumpkin_util::{
    math::floor_div,
    random::{
        RandomGenerator, RandomImpl, get_carver_seed, get_region_seed, xoroshiro128::Xoroshiro,
    },
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StructurePlacement {
    frequency_reduction_method: Option<FrequencyReductionMethod>,
    frequency: Option<f32>,
    salt: i32,
    #[serde(flatten)]
    r#type: StructurePlacementType,
}

impl StructurePlacement {
    pub fn should_generate(
        &self,
        calculator: StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
    ) -> bool {
        self.r#type
            .is_start_chunk(&calculator, chunk_x, chunk_z, self.salt)
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
    LegacyType1,
    LegacyType2,
    LegacyType3,
}

impl FrequencyReductionMethod {
    pub fn should_generate(
        &self,
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        salt: i32,
        frequency: f32,
    ) -> bool {
        match self {
            FrequencyReductionMethod::Default => {
                let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, salt);
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
                random.next_f32() < frequency
            }
            FrequencyReductionMethod::LegacyType1 => {
                let x = chunk_x >> 4;
                let z = chunk_z >> 4;
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(
                    (x ^ z << 4) as u64 ^ seed as u64,
                ));
                random.next_i32(); // yeah mojang just does that and does not use the value
                random.next_bounded_i32((1.0 / frequency) as i32) == 0
            }
            FrequencyReductionMethod::LegacyType2 => {
                let region_seed = get_region_seed(seed as u64, chunk_x, chunk_z, 10387320);
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
                random.next_f32() < frequency
            }
            FrequencyReductionMethod::LegacyType3 => {
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
    ConcentricRings,
}

impl StructurePlacementType {
    pub fn is_start_chunk(
        &self,
        calculator: &StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
        salt: i32,
    ) -> bool {
        match self {
            StructurePlacementType::RandomSpread(placement) => {
                placement.is_start_chunk(calculator, chunk_x, chunk_z, salt)
            }
            StructurePlacementType::ConcentricRings => false, // TODO, This is needed for Stronghold, since it is placed in rings
        }
    }
}

#[derive(Deserialize)]
pub struct RandomSpreadStructurePlacement {
    spacing: i32,
    separation: i32,
    spread_type: SpreadType,
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
            SpreadType::Linear => random.next_bounded_i32(bound),
            SpreadType::Triangular => {
                (random.next_bounded_i32(bound) + random.next_bounded_i32(bound)) / 2
            }
        }
    }
}

impl RandomSpreadStructurePlacement {
    fn get_start_chunk(&self, seed: i64, chunk_x: i32, chunk_z: i32, salt: i32) -> (i32, i32) {
        let x = floor_div(chunk_x, self.spacing);
        let z = floor_div(chunk_z, self.spacing);
        let region_seed = get_region_seed(seed as u64, x, z, salt);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(region_seed));
        let bound = self.spacing - self.separation;
        let rand_x = self.spread_type.get(&mut random, bound);
        let rand_z = self.spread_type.get(&mut random, bound);
        (x * self.spacing + rand_x, z * self.spacing + rand_z)
    }

    pub fn is_start_chunk(
        &self,
        calculator: &StructurePlacementCalculator,
        chunk_x: i32,
        chunk_z: i32,
        salt: i32,
    ) -> bool {
        let pos = self.get_start_chunk(calculator.seed, chunk_x, chunk_z, salt);
        (chunk_x == pos.0) && (chunk_z == pos.1)
    }
}

pub struct StructurePlacementCalculator {
    pub seed: i64,
}

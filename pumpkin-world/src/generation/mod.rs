#![allow(dead_code)]

pub mod aquifer_sampler;
mod biome;
mod blender;
mod block_predicate;
mod block_state_provider;
pub mod carver;
pub mod chunk_noise;
mod feature;
pub mod generator;
pub mod height_limit;
pub mod height_provider;
pub mod noise;
pub mod ore_sampler;
pub mod positions;
pub mod proto_chunk;
pub mod rule;
mod rule_test;
pub mod settings;
pub mod structure;
mod surface;
pub mod y_offset;

use generator::{GeneratorInit, VanillaGenerator};
use pumpkin_util::{
    random::{
        RandomDeriver, RandomDeriverImpl, RandomImpl, legacy_rand::LegacyRand,
        xoroshiro128::Xoroshiro,
    },
    world_seed::Seed,
};

use crate::dimension::Dimension;

pub fn get_world_gen(seed: Seed, dimension: Dimension) -> Box<VanillaGenerator> {
    // TODO decide which WorldGenerator to pick based on config.
    Box::new(VanillaGenerator::new(seed, dimension))
}

pub struct GlobalRandomConfig {
    pub seed: u64,
    base_random_deriver: RandomDeriver,
    aquifer_random_deriver: RandomDeriver,
    ore_random_deriver: RandomDeriver,
}

impl GlobalRandomConfig {
    pub fn new(seed: u64, legacy: bool) -> Self {
        let random_deriver = if legacy {
            LegacyRand::from_seed(seed).next_splitter()
        } else {
            Xoroshiro::from_seed(seed).next_splitter()
        };

        let aquifer_deriver = random_deriver
            .split_string("minecraft:aquifer")
            .next_splitter();
        let ore_deriver = random_deriver.split_string("minecraft:ore").next_splitter();
        Self {
            seed,
            base_random_deriver: random_deriver,
            aquifer_random_deriver: aquifer_deriver,
            ore_random_deriver: ore_deriver,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }
}

pub mod section_coords {
    use num_traits::PrimInt;

    #[inline]
    pub fn block_to_section<T: PrimInt>(coord: T) -> T {
        coord >> 4
    }

    pub fn get_offset_pos(chunk_coord: i32, offset: i32) -> i32 {
        section_to_block(chunk_coord) + offset
    }

    #[inline]
    pub fn section_to_block<T: PrimInt>(coord: T) -> T {
        coord << 4
    }
}

pub mod biome_coords {
    use num_traits::PrimInt;

    #[inline]
    pub fn from_block<T: PrimInt>(coord: T) -> T {
        coord >> 2
    }

    #[inline]
    pub fn to_block<T: PrimInt>(coord: T) -> T {
        coord << 2
    }

    #[inline]
    pub fn from_chunk<T: PrimInt>(coord: T) -> T {
        coord << 2
    }

    #[inline]
    pub fn to_chunk<T: PrimInt>(coord: T) -> T {
        coord >> 2
    }
}

#[derive(PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

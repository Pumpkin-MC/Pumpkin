#![allow(dead_code)]

mod biome;
pub mod blender;
mod block_predicate;
mod block_state_provider;
pub mod carver;
mod feature;
pub mod generator;
pub mod height_limit;
pub mod height_provider;
pub mod noise;
pub mod positions;
pub mod proto_chunk;
pub mod proto_chunk_test;
pub mod rule;
pub mod structure;
mod surface;

use generator::flat::FlatGenerator;
use generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::{
    random::xoroshiro128::{Xoroshiro, XoroshiroSplitter},
    world_seed::Seed,
};

/// Builds the configured world generator for `dimension`.
///
/// `level_type` mirrors the vanilla `level-type` from `server.properties`
/// (the `minecraft:` prefix is optional); `minecraft:flat` selects the flat
/// generator and any other value falls back to vanilla noise generation.
/// `generator_settings` is the matching `generator-settings` value, consulted
/// only by the flat generator.
#[must_use]
pub fn get_world_gen(
    seed: Seed,
    dimension: Dimension,
    level_type: &str,
    generator_settings: &str,
) -> WorldGenerator {
    let level_type = level_type
        .trim()
        .strip_prefix("minecraft:")
        .unwrap_or_else(|| level_type.trim());

    match level_type {
        "flat" => WorldGenerator::Flat(Box::new(FlatGenerator::new(
            seed,
            dimension,
            generator_settings,
        ))),
        "normal" | "large_biomes" | "amplified" => {
            WorldGenerator::Vanilla(Box::new(VanillaGenerator::new(seed, dimension)))
        }
        other => {
            tracing::warn!("Unknown level-type `{other}`, falling back to normal generation");
            WorldGenerator::Vanilla(Box::new(VanillaGenerator::new(seed, dimension)))
        }
    }
}

pub struct GlobalRandomConfig {
    pub seed: u64,
    pub legacy_random_source: bool,
    base_random_deriver: XoroshiroSplitter,
    aquifer_random_deriver: XoroshiroSplitter,
    pub ore_random_deriver: XoroshiroSplitter,
}

impl GlobalRandomConfig {
    #[must_use]
    pub fn new(seed: u64, legacy_random_source: bool) -> Self {
        let random_deriver = Xoroshiro::from_seed(seed).next_splitter();

        let aquifer_deriver = random_deriver
            .split_string("minecraft:aquifer")
            .next_splitter();
        let ore_deriver = random_deriver.split_string("minecraft:ore").next_splitter();
        Self {
            seed,
            legacy_random_source,
            base_random_deriver: random_deriver,
            aquifer_random_deriver: aquifer_deriver,
            ore_random_deriver: ore_deriver,
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }
}

pub mod section_coords {
    #[inline]
    #[must_use]
    pub const fn block_to_section(coord: i32) -> i32 {
        coord >> 4
    }

    #[must_use]
    pub const fn get_offset_pos(chunk_coord: i32, offset: i32) -> i32 {
        section_to_block(chunk_coord) + offset
    }

    #[inline]
    #[must_use]
    pub const fn section_to_block(coord: i32) -> i32 {
        coord << 4
    }
}

pub mod biome_coords {
    #[inline]
    #[must_use]
    pub const fn from_block(coord: i32) -> i32 {
        coord >> 2
    }

    #[inline]
    #[must_use]
    pub const fn to_block(coord: i32) -> i32 {
        coord << 2
    }

    #[inline]
    #[must_use]
    pub const fn from_chunk(coord: i32) -> i32 {
        coord << 2
    }

    #[inline]
    #[must_use]
    pub const fn to_chunk(coord: i32) -> i32 {
        coord >> 2
    }
}

#[derive(PartialEq, Eq)]
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

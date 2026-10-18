#![allow(dead_code)]

mod biome;
pub mod blender;
pub mod block_predicate;
pub mod block_state_provider;
pub mod carver;
pub mod feature;
mod feature_order;
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

use generator::VanillaGenerator;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::{
    random::xoroshiro128::{Xoroshiro, XoroshiroSplitter},
    world_seed::Seed,
};

#[must_use]
pub fn get_world_gen(
    seed: Seed,
    dimension: Dimension,
    is_flat: bool,
    flat_layers: Vec<generator::FlatLayer>,
    flat_biome: String,
) -> Box<generator::WorldGenerator> {
    get_world_gen_with_settings(seed, dimension, is_flat, flat_layers, flat_biome, None)
}

#[must_use]
pub fn get_world_gen_with_settings(
    seed: Seed,
    dimension: Dimension,
    is_flat: bool,
    flat_layers: Vec<generator::FlatLayer>,
    flat_biome: String,
    generator_settings: Option<&str>,
) -> Box<generator::WorldGenerator> {
    get_world_gen_with_all_settings(
        seed,
        dimension,
        is_flat,
        flat_layers,
        flat_biome,
        generator_settings,
        None,
        None,
        generator::FlatDecoration::default(),
    )
}

#[expect(clippy::too_many_arguments)]
#[must_use]
pub fn get_world_gen_with_all_settings(
    seed: Seed,
    dimension: Dimension,
    is_flat: bool,
    flat_layers: Vec<generator::FlatLayer>,
    flat_biome: String,
    generator_settings: Option<&str>,
    biome_source: Option<&crate::world_info::BiomeSource>,
    structure_overrides: Option<&[String]>,
    flat_decoration: generator::FlatDecoration,
) -> Box<generator::WorldGenerator> {
    if is_flat {
        Box::new(generator::WorldGenerator::Flat(Box::new(
            generator::flat::FlatGenerator::new(
                seed,
                dimension,
                flat_layers,
                flat_biome,
                flat_decoration,
            ),
        )))
    } else {
        Box::new(generator::WorldGenerator::Noise(Box::new(
            VanillaGenerator::new_with_all_settings(
                seed,
                dimension,
                generator_settings,
                biome_source,
                structure_overrides,
            ),
        )))
    }
}

/// Builds the generator a world's `world_gen_settings` give `dimension`, or the vanilla one
/// when they have none for it.
#[must_use]
pub fn world_generator_for(
    settings: Option<&crate::world_info::WorldGenSettings>,
    dimension: Dimension,
    seed: Seed,
) -> Box<generator::WorldGenerator> {
    let mut is_flat = false;
    let mut flat_layers = Vec::new();
    let mut flat_biome = "minecraft:plains".to_string();
    let mut generator_settings_name: Option<String> = None;
    let mut biome_source: Option<crate::world_info::BiomeSource> = None;
    let mut structure_overrides: Option<Vec<String>> = None;
    let mut flat_decoration = generator::FlatDecoration::default();

    if let Some(wgs) = settings
        && let Some(dim_settings) = wgs.dimensions.get(dimension.minecraft_name)
    {
        biome_source.clone_from(&dim_settings.generator.biome_source);

        if dim_settings.generator.generator_type == "minecraft:flat" {
            is_flat = true;
            let flat_settings = dim_settings
                .generator
                .settings
                .as_ref()
                .and_then(crate::world_info::GeneratorSettings::as_flat_settings)
                .or_else(|| {
                    crate::world_info::FlatLevelGeneratorPreset::from_name("classic_flat")
                        .map(|p| p.settings)
                });
            if let Some(flat_settings) = flat_settings {
                flat_layers = flat_settings.to_flat_layers();
                structure_overrides = flat_settings.structure_overrides_vec();
                flat_decoration = generator::FlatDecoration {
                    features: flat_settings.features,
                    lakes: flat_settings.lakes,
                };
                flat_biome = flat_settings.biome;
            }
        } else if let Some(crate::world_info::GeneratorSettings::Reference(s)) =
            &dim_settings.generator.settings
        {
            generator_settings_name = Some(s.clone());
        }
        if !wgs.generate_structures {
            structure_overrides = Some(Vec::new());
        }
    }

    get_world_gen_with_all_settings(
        seed,
        dimension,
        is_flat,
        flat_layers,
        flat_biome,
        generator_settings_name.as_deref(),
        biome_source.as_ref(),
        structure_overrides.as_deref(),
        flat_decoration,
    )
}

pub struct GlobalRandomConfig {
    pub seed: u64,
    pub legacy_random_source: bool,
    pub base_random_deriver: XoroshiroSplitter,
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

use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_data::{BlockState, chunk::Biome};

use super::noise::router::{
    multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
    proto_noise_router::ProtoNoiseRouters,
};
use crate::biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::generation::noise::CHUNK_DIM;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed, biome_coords};

pub mod structure_finder;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use rustc_hash::FxHashMap;

pub mod flat;

#[derive(Clone, Debug)]
pub struct FlatLayer {
    pub block: String,
    pub height: i32,
}

pub enum WorldGenerator {
    Noise(Box<VanillaGenerator>),
    Flat(flat::FlatGenerator),
}

impl WorldGenerator {
    #[must_use]
    pub const fn dimension(&self) -> &Dimension {
        match self {
            Self::Noise(noise_gen) => &noise_gen.dimension,
            Self::Flat(flat_gen) => &flat_gen.dimension,
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        match self {
            Self::Noise(noise_gen) => noise_gen.random_config.seed,
            Self::Flat(flat_gen) => flat_gen.seed,
        }
    }

    #[must_use]
    pub const fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        match self {
            Self::Noise(noise_gen) => Some(&noise_gen.global_structure_cache),
            Self::Flat(_) => None,
        }
    }
}

pub struct VanillaGenerator {
    pub random_config: GlobalRandomConfig,
    pub base_router: ProtoNoiseRouters,
    pub dimension: Dimension,
    pub settings: &'static GenerationSettings,
    pub biome_mixer_seed: i64,

    pub terrain_cache: TerrainCache,

    pub default_block: &'static BlockState,

    pub global_structure_cache: crate::generation::structure::placement::GlobalStructureCache,
    pub structure_calculator: StructurePlacementCalculator,
    pub structure_allowed_biomes: FxHashMap<usize, Vec<u16>>,
}

impl VanillaGenerator {
    /// Resolves the fuzzy quart chosen by `BiomeManager#getBiome` from the
    /// uncached generator source. This deliberately does not use a proto
    /// chunk's local palette: a fuzzy edge lookup may select a neighbor quart.
    ///
    /// Vanilla references: BiomeManager.java:38-69 and
    /// SurfaceSystem.java:110,119,156-157.
    #[must_use]
    pub fn terrain_gen_biome_at_block(
        &self,
        x: i32,
        y: i32,
        z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> &'static Biome {
        let quart = crate::generation::biome::get_biome_blend(
            self.dimension.min_y as i8,
            self.dimension.height as u16,
            self.biome_mixer_seed,
            x,
            y,
            z,
        );

        if self.dimension == Dimension::THE_END {
            TheEndBiomeSupplier.biome(quart.x, quart.y, quart.z, sampler)
        } else if self.dimension == Dimension::THE_NETHER {
            MultiNoiseBiomeSupplier::NETHER.biome(quart.x, quart.y, quart.z, sampler)
        } else {
            MultiNoiseBiomeSupplier::OVERWORLD.biome(quart.x, quart.y, quart.z, sampler)
        }
    }

    /// Builds a sampler whose FlatCache covers the one-quart fuzzy halo around
    /// the surface/carver chunk. Positions beyond that halo safely take
    /// MultiNoiseSampler's uncached path; no scheduler or write radius changes
    /// are implied by this phase-one resolver.
    #[must_use]
    pub fn terrain_gen_biome_sampler(&self, chunk_x: i32, chunk_z: i32) -> MultiNoiseSampler<'_> {
        const FUZZY_QUART_HALO: i32 = 1;
        let start_quart_x = biome_coords::from_chunk(chunk_x) - FUZZY_QUART_HALO;
        let start_quart_z = biome_coords::from_chunk(chunk_z) - FUZZY_QUART_HALO;
        // `horizontal_biome_end` is inclusive. With a start of -1 quart,
        // `CHUNK_QUARTS + 1` caches exactly [-1, 4] for a 16-block chunk.
        let cached_quart_end = biome_coords::from_block(CHUNK_DIM as i32) + FUZZY_QUART_HALO;
        let options = MultiNoiseSamplerBuilderOptions::new(
            start_quart_x,
            start_quart_z,
            cached_quart_end as usize,
        );

        MultiNoiseSampler::generate(&self.base_router.multi_noise, &options)
    }
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed, dimension: Dimension) -> Self {
        let settings = GenerationSettings::from_dimension(&dimension);
        let random_config = GlobalRandomConfig::new(seed.0, settings.legacy_random_source);

        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base = if dimension == Dimension::OVERWORLD {
            OVERWORLD_BASE_NOISE_ROUTER
        } else if dimension == Dimension::THE_NETHER {
            NETHER_BASE_NOISE_ROUTER
        } else if dimension == Dimension::THE_END {
            END_BASE_NOISE_ROUTER
        } else {
            tracing::error!("Unsupported dimension for noise router: {:?}", dimension);
            OVERWORLD_BASE_NOISE_ROUTER
        };
        let terrain_cache = TerrainCache::from_random(&random_config);

        let default_block = settings.default_block;
        let base_router = ProtoNoiseRouters::generate(&base, &random_config);
        let biome_mixer_seed = crate::biome::hash_seed(seed.0);

        let mut structure_allowed_biomes = FxHashMap::default();
        for (i, set) in StructureSet::ALL.iter().enumerate() {
            structure_allowed_biomes.insert(
                i,
                crate::generation::proto_chunk::ProtoChunk::get_allowed_biomes(set),
            );
        }

        Self {
            random_config,
            base_router,
            dimension,
            settings,
            biome_mixer_seed,
            terrain_cache,
            default_block,
            global_structure_cache:
                crate::generation::structure::placement::GlobalStructureCache::new(),
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            structure_allowed_biomes,
        }
    }
}

use pumpkin_data::BlockState;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::vector2::Vector2;

use super::noise::router::multi_noise_sampler::{
    MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
};
use super::noise::router::proto_noise_router::ProtoNoiseRouters;
use crate::biome::position_finder::{FittestPositionFinder, OVERWORLD_SPAWN_TARGET};
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed, biome_coords};

pub mod structure_finder;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use rustc_hash::FxHashMap;

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

/// Finds the horizontal world-spawn position for a freshly created Overworld by climate.
///
/// Mirrors vanilla's spawn search (`Climate.Sampler`'s `findSpawnPosition`)
/// instead of blindly defaulting to `(0, 0)` regardless of what's actually
/// there, e.g. open ocean (Pumpkin-MC/Pumpkin#1303).
///
/// This reproduces vanilla's noise-based search only: it looks for a
/// position whose climate (temperature/humidity/continentalness/erosion/
/// weirdness) resembles ordinary dry land and is closest to the origin
/// among those. It does not perform vanilla's follow-up per-chunk scan for
/// an actual non-waterlogged block, so a small lake or river can still very
/// rarely be chosen. The Y coordinate is intentionally not resolved here:
/// callers already re-derive it from the generated terrain at the chosen
/// (x, z) once chunks are loaded (see `World::get_top_block`).
#[must_use]
pub fn find_overworld_spawn_position(seed: Seed) -> Vector2<i32> {
    let generator = VanillaGenerator::new(seed, Dimension::OVERWORLD);

    let sampler = |x: i32, z: i32| -> [i64; 7] {
        let biome_x = biome_coords::from_block(x);
        let biome_z = biome_coords::from_block(z);
        // A single-point sampler: build a throwaway `MultiNoiseSampler` scoped
        // to just this one biome-coord column instead of a whole chunk, since
        // the search below queries scattered points up to 2048 blocks apart.
        let options = MultiNoiseSamplerBuilderOptions::new(biome_x, biome_z, 0);
        let mut noise_sampler =
            MultiNoiseSampler::generate(&generator.base_router.multi_noise, &options);
        noise_sampler.sample(biome_x, 0, biome_z).convert_to_list()
    };

    FittestPositionFinder::find_best_spawn_position(&OVERWORLD_SPAWN_TARGET, &sampler)
}

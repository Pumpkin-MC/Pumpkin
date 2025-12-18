use dimension::Dimension;
use generation::settings::GenerationSettings;
use pumpkin_data::BlockState;
use pumpkin_util::math::vector2::Vector2;

pub mod biome;
pub mod block;
pub mod chunk;
pub mod chunk_system;
pub mod cylindrical_chunk_iterator;
pub mod data;
pub mod dimension;
pub mod entity;
pub mod generation;
pub mod inventory;
pub mod item;
pub mod level;
pub mod lock;
pub mod tick;
pub mod world;
pub mod world_info;

pub type BlockId = u16;
pub type BlockStateId = u16;

pub const CURRENT_MC_VERSION: &str = "1.21.10";
pub const CURRENT_BEDROCK_MC_VERSION: &str = "1.21.111";

#[macro_export]
macro_rules! global_path {
    ($path:expr) => {{
        use std::path::Path;
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($path)
    }};
}

// TODO: is there a way to do in-file benches?
pub use generation::{
    GlobalRandomConfig, noise::router::proto_noise_router::ProtoNoiseRouters,
    proto_chunk::ProtoChunk, settings::GENERATION_SETTINGS, settings::GeneratorSetting,
};

use crate::generation::{chunk_noise::CHUNK_DIM, proto_chunk::TerrainCache};
pub fn bench_create_and_populate_noise(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    default_block: &'static BlockState,
) {
    use crate::biome::hash_seed;
    use crate::generation::chunk_noise::ChunkNoiseGenerator;
    use crate::generation::noise::router::surface_height_sampler::{
        SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use crate::generation::{
        aquifer_sampler::{FluidLevel, FluidLevelSampler},
        biome_coords,
        positions::chunk_pos,
    };

    let biome_mixer_seed = hash_seed(random_config.seed);
    let mut chunk = ProtoChunk::new(0, 0, settings, default_block, biome_mixer_seed);

    // Create noise sampler and other required components
    let generation_shape = &settings.shape;
    let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
    let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
        FluidLevel::new(settings.sea_level, settings.default_fluid.name),
        FluidLevel::new(-54, &pumpkin_data::Block::LAVA),
    ));

    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);

    let mut noise_sampler = ChunkNoiseGenerator::new(
        &base_router.noise,
        random_config,
        horizontal_cell_count as usize,
        start_x,
        start_z,
        generation_shape,
        sampler,
        settings.aquifers_enabled,
        settings.ore_veins_enabled,
    );

    // Surface height estimator
    let biome_pos = Vector2::new(
        biome_coords::from_block(start_x),
        biome_coords::from_block(start_z),
    );
    let horizontal_biome_end = biome_coords::from_block(
        horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
    );
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
        generation_shape.min_y as i32,
        generation_shape.max_y() as i32,
        generation_shape.vertical_cell_block_count() as usize,
    );
    let mut surface_height_estimate_sampler =
        SurfaceHeightEstimateSampler::generate(&base_router.surface_estimator, &surface_config);

    chunk.populate_noise(&mut noise_sampler, &mut surface_height_estimate_sampler);
}

pub fn bench_create_and_populate_biome(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    default_block: &'static BlockState,
) {
    use crate::biome::hash_seed;
    use crate::generation::noise::router::multi_noise_sampler::{
        MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
    };
    use crate::generation::{biome_coords, positions::chunk_pos};

    let biome_mixer_seed = hash_seed(random_config.seed);
    let mut chunk = ProtoChunk::new(0, 0, settings, default_block, biome_mixer_seed);

    // Create multi-noise sampler
    let generation_shape = &settings.shape;
    let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);
    let biome_pos = Vector2::new(
        biome_coords::from_block(start_x),
        biome_coords::from_block(start_z),
    );
    let horizontal_biome_end = biome_coords::from_block(
        horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
    );
    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
    );
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&base_router.multi_noise, &multi_noise_config);

    chunk.populate_biomes(Dimension::Overworld, &mut multi_noise_sampler);
}

pub fn bench_create_and_populate_noise_with_surface(
    base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    settings: &GenerationSettings,
    terrain_cache: &TerrainCache,
    default_block: &'static BlockState,
) {
    use crate::biome::hash_seed;
    use crate::generation::chunk_noise::ChunkNoiseGenerator;
    use crate::generation::noise::router::{
        multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        surface_height_sampler::{
            SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
        },
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use crate::generation::{
        aquifer_sampler::{FluidLevel, FluidLevelSampler},
        biome_coords,
        positions::chunk_pos,
    };

    let biome_mixer_seed = hash_seed(random_config.seed);
    let mut chunk = ProtoChunk::new(0, 0, settings, default_block, biome_mixer_seed);

    // Create all required components
    let generation_shape = &settings.shape;
    let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);

    // Multi-noise sampler for biomes
    let biome_pos = Vector2::new(
        biome_coords::from_block(start_x),
        biome_coords::from_block(start_z),
    );
    let horizontal_biome_end = biome_coords::from_block(
        horizontal_cell_count * generation_shape.horizontal_cell_block_count(),
    );
    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
    );
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&base_router.multi_noise, &multi_noise_config);

    // Noise sampler
    let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
        FluidLevel::new(settings.sea_level, settings.default_fluid.name),
        FluidLevel::new(-54, &pumpkin_data::Block::LAVA),
    ));

    let mut noise_sampler = ChunkNoiseGenerator::new(
        &base_router.noise,
        random_config,
        horizontal_cell_count as usize,
        start_x,
        start_z,
        generation_shape,
        sampler,
        settings.aquifers_enabled,
        settings.ore_veins_enabled,
    );

    // Surface height estimator
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
        generation_shape.min_y as i32,
        generation_shape.max_y() as i32,
        generation_shape.vertical_cell_block_count() as usize,
    );
    let mut surface_height_estimate_sampler =
        SurfaceHeightEstimateSampler::generate(&base_router.surface_estimator, &surface_config);

    chunk.populate_biomes(Dimension::Overworld, &mut multi_noise_sampler);
    chunk.populate_noise(&mut noise_sampler, &mut surface_height_estimate_sampler);
    chunk.build_surface(
        settings,
        random_config,
        terrain_cache,
        &mut surface_height_estimate_sampler,
    );
}

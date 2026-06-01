use pumpkin_data::{
    Block, BlockState, chunk_gen_settings::GenerationSettings, dimension::Dimension,
};
use pumpkin_util::math::vector2::Vector2;

pub mod biome;
pub mod block;
pub mod chunk;
pub mod chunk_system;
pub mod cylindrical_chunk_iterator;
pub mod data;
pub mod dimension;
pub mod generation;
pub mod inventory;
pub mod level;
pub mod lighting;
pub mod poi;
pub mod tick;
pub mod world;
pub mod world_info;

pub type BlockId = u16;
pub type BlockStateId = u16;

pub const CURRENT_MC_VERSION: &str = "26.1";

pub const CURRENT_BEDROCK_MC_VERSION: &str = "1.26.20";
pub const CURRENT_BEDROCK_MC_PROTOCOL: u32 = 975;

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
    proto_chunk::ProtoChunk,
};

use crate::generation::{
    biome_coords,
    noise::{CHUNK_DIM, ChunkNoiseGenerator, aquifer_sampler::FluidLevel},
    positions::chunk_pos,
    proto_chunk::TerrainCache,
};

pub fn bench_create_and_populate_noise(
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::{GeneratorInit, VanillaGenerator};
    use crate::generation::noise::router::surface_height_sampler::{
        SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), Dimension::OVERWORLD);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    // Create noise sampler and other required components
    let settings = generator.settings;
    let generation_shape = &settings.shape;
    let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
    let sampler = StandardChunkFluidLevelSampler::new(
        FluidLevel::new(
            settings.sea_level,
            Block::from_state_id(settings.default_fluid.id),
        ),
        FluidLevel::new(-54, &pumpkin_data::Block::LAVA),
    );

    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);

    let mut noise_sampler = ChunkNoiseGenerator::new(
        &generator.base_router.noise,
        &generator.random_config,
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
        horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
    );
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
        generation_shape.min_y as i32,
        generation_shape.max_y() as i32,
        generation_shape.vertical_cell_block_count() as usize,
    );
    let mut surface_height_estimate_sampler = SurfaceHeightEstimateSampler::generate(
        &generator.base_router.surface_estimator,
        &surface_config,
    );

    chunk.populate_noise(
        &generator,
        &mut noise_sampler,
        &generator.random_config.ore_random_deriver,
        &mut surface_height_estimate_sampler,
    );
}

pub fn bench_create_and_populate_biome(
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::{GeneratorInit, VanillaGenerator};
    use crate::generation::noise::router::multi_noise_sampler::{
        MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
    };
    use crate::generation::{biome_coords, positions::chunk_pos};
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), Dimension::OVERWORLD);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    // Create multi-noise sampler
    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);
    let biome_pos = Vector2::new(
        biome_coords::from_block(start_x),
        biome_coords::from_block(start_z),
    );
    let horizontal_biome_end = biome_coords::from_block(16);
    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
    );
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&generator.base_router.multi_noise, &multi_noise_config);

    chunk.populate_biomes(&generator, &mut multi_noise_sampler);
}

pub fn bench_create_and_populate_noise_with_surface(
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::{GeneratorInit, VanillaGenerator};
    use crate::generation::noise::router::{
        multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        surface_height_sampler::{
            SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
        },
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), Dimension::OVERWORLD);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    // Create all required components
    let settings = generator.settings;
    let generation_shape = &settings.shape;
    let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
    let start_x = chunk_pos::start_block_x(0);
    let start_z = chunk_pos::start_block_z(0);

    // Multi-noise sampler for biomes
    let biome_pos = Vector2::new(
        biome_coords::from_block(start_x),
        biome_coords::from_block(start_z),
    );
    let horizontal_biome_end = biome_coords::from_block(16);
    let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
        biome_pos.x,
        biome_pos.y,
        horizontal_biome_end as usize,
    );
    let mut multi_noise_sampler =
        MultiNoiseSampler::generate(&generator.base_router.multi_noise, &multi_noise_config);

    // Noise sampler
    let sampler = StandardChunkFluidLevelSampler::new(
        FluidLevel::new(
            settings.sea_level,
            Block::from_state_id(settings.default_fluid.id),
        ),
        FluidLevel::new(-54, &Block::LAVA),
    );

    let mut noise_sampler = ChunkNoiseGenerator::new(
        &generator.base_router.noise,
        &generator.random_config,
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
    let mut surface_height_estimate_sampler = SurfaceHeightEstimateSampler::generate(
        &generator.base_router.surface_estimator,
        &surface_config,
    );

    chunk.populate_biomes(&generator, &mut multi_noise_sampler);
    chunk.populate_noise(
        &generator,
        &mut noise_sampler,
        &generator.random_config.ore_random_deriver,
        &mut surface_height_estimate_sampler,
    );
    chunk.build_surface(&generator, &mut surface_height_estimate_sampler);
}

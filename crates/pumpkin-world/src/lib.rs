#![allow(clippy::unreachable)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::dimension_type::DimensionType as Dimension;
use pumpkin_data::{Block, BlockState, chunk_gen_settings::GenerationSettings};
use pumpkin_util::math::vector2::Vector2;

pub mod attributes;
pub mod biome;
pub mod block;
pub mod cardinal_lighting;
pub mod chunk;
pub mod chunk_system;
pub mod cylindrical_chunk_iterator;
pub mod data;
pub mod dimension;
pub mod dimension_type;
pub mod generation;
pub mod inventory;
pub mod level;
pub mod lighting;
pub mod poi;
pub mod tick;
pub mod time;
pub mod world;
pub mod world_info;

pub const CURRENT_MC_VERSION: &str = "26.2";
pub const CURRENT_BEDROCK_MC_VERSION: &str = "1.26.40";
pub const CURRENT_BEDROCK_MC_PROTOCOL: u32 = 2168;

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
pub(crate) mod test_support {
    use pumpkin_codecs::{Decode, json_ops::JsonOps};
    use serde_json::json;

    use crate::dimension_type::DimensionType;

    pub fn dimension(name: &str) -> DimensionType {
        let mut value = json!({
            "ambient_light": 0.0,
            "attributes": {},
            "coordinate_scale": 1.0,
            "has_ceiling": false,
            "has_ender_dragon_fight": false,
            "has_skylight": true,
            "height": 384,
            "infiniburn": "#minecraft:infiniburn_overworld",
            "logical_height": 384,
            "min_y": -64,
            "monster_spawn_block_light_limit": 0,
            "monster_spawn_light_level": 7
        });

        match name {
            "the_nether" => {
                value["has_ceiling"] = true.into();
                value["has_skylight"] = false.into();
                value["height"] = 256.into();
                value["logical_height"] = 128.into();
                value["min_y"] = 0.into();
                value["infiniburn"] = "#minecraft:infiniburn_nether".into();
                value["cardinal_light"] = "nether".into();
            }
            "the_end" => {
                value["height"] = 256.into();
                value["logical_height"] = 256.into();
                value["min_y"] = 0.into();
                value["infiniburn"] = "#minecraft:infiniburn_end".into();
                value["skybox"] = "end".into();
            }
            "overworld" => {}
            _ => panic!("unknown test dimension {name}"),
        }

        DimensionType::parse(value, &JsonOps).into_result().unwrap()
    }
}

#[cfg(test)]
pub(crate) fn init_test_registries() {
    use pumpkin_registry::{
        BOOTSTRAP, ROOT, Registry, RegistryBuilder, bootstrap::BootstrapManager,
    };
    use pumpkin_util::identifier::Identifier;
    use std::sync::Arc;

    BOOTSTRAP.get_or_init(BootstrapManager::new);
    ROOT.get_or_init(|| {
        RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root"))
            .expect("test root registry must initialize")
    });
}

#[macro_export]
macro_rules! global_path {
    ($path:expr) => {{
        use std::path::Path;
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
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
    dimension: Dimension,
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::VanillaGenerator;
    use crate::generation::noise::router::surface_height_sampler::{
        SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), dimension);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    // Create noise sampler and other required components
    let settings = generator.settings();
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
        Vec::new(),
        Vec::new(),
        None,
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

    generator.populate_noise(
        &mut chunk,
        &mut noise_sampler,
        &generator.random_config.ore_random_deriver,
        &mut surface_height_estimate_sampler,
    );
}

pub fn bench_create_and_populate_biome(
    dimension: Dimension,
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::VanillaGenerator;
    use crate::generation::noise::router::multi_noise_sampler::{
        MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
    };
    use crate::generation::{biome_coords, positions::chunk_pos};
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), dimension);
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

    generator.populate_biomes(&mut chunk, &mut multi_noise_sampler);
}

pub fn bench_create_and_populate_noise_with_surface(
    dimension: Dimension,
    _base_router: &ProtoNoiseRouters,
    random_config: &GlobalRandomConfig,
    _settings: &GenerationSettings,
    _terrain_cache: &TerrainCache,
    _default_block: &'static BlockState,
) {
    use crate::generation::generator::VanillaGenerator;
    use crate::generation::noise::router::{
        multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        surface_height_sampler::{
            SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
        },
    };
    use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(random_config.seed), dimension);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    // Create all required components
    let settings = generator.settings();
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
        Vec::new(),
        Vec::new(),
        None,
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

    generator.populate_biomes(&mut chunk, &mut multi_noise_sampler);
    generator.populate_noise(
        &mut chunk,
        &mut noise_sampler,
        &generator.random_config.ore_random_deriver,
        &mut surface_height_estimate_sampler,
    );
    generator.build_surface(&mut chunk, &mut surface_height_estimate_sampler);
}

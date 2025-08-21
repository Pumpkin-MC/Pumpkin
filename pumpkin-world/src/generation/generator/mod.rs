use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::BlockState;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{
    noise::router::proto_noise_router::ProtoNoiseRouters,
    settings::gen_settings_from_dimension,
};
use crate::chunk::format::LightContainer;
use crate::generation::proto_chunk::TerrainCache;
use crate::level::Level;
use crate::world::BlockRegistryExt;
use crate::{chunk::ChunkLight, dimension::Dimension};
use crate::{
    chunk::{
        ChunkData, ChunkSections, SubChunk,
        palette::{BiomePalette, BlockPalette},
    },
    generation::{GlobalRandomConfig, Seed, proto_chunk::ProtoChunk},
};

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

#[async_trait]
pub trait WorldGenerator: Sync + Send {
    fn generate_chunk(
        &self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        at: &Vector2<i32>,
    ) -> ChunkData;
}

pub struct VanillaGenerator {
    random_config: GlobalRandomConfig,
    base_router: ProtoNoiseRouters,
    dimension: Dimension,

    terrain_cache: TerrainCache,

    default_block: &'static BlockState,
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed, dimension: Dimension) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0, false);

        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base = match dimension {
            Dimension::Overworld => OVERWORLD_BASE_NOISE_ROUTER,
            Dimension::Nether => NETHER_BASE_NOISE_ROUTER,
            Dimension::End => END_BASE_NOISE_ROUTER,
        };
        let terrain_cache = TerrainCache::from_random(&random_config);
        let generation_settings = gen_settings_from_dimension(&dimension);

        let default_block = generation_settings.default_block.get_state();
        let base_router = ProtoNoiseRouters::generate(&base, &random_config);
        Self {
            random_config,
            base_router,
            dimension,
            terrain_cache,
            default_block,
        }
    }
}

impl WorldGenerator for VanillaGenerator {
    fn generate_chunk(
        &self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        at: &Vector2<i32>,
    ) -> ChunkData {
        let generation_settings = gen_settings_from_dimension(&self.dimension);

        let height: usize = match self.dimension {
            Dimension::Overworld => 384,
            Dimension::Nether | Dimension::End => 256,
        };
        let sub_chunks = height / BlockPalette::SIZE;
        let sections = (0..sub_chunks).map(|_| SubChunk::default()).collect();
        let mut sections = ChunkSections::new(sections, generation_settings.shape.min_y as i32);

        // Calculate biome mixer seed
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(self.random_config.seed);
        
        let mut proto_chunk = ProtoChunk::new(
            *at,
            generation_settings,
            self.default_block,
            biome_mixer_seed,
        );

        // Create the required components for generation
        use crate::generation::chunk_noise::{ChunkNoiseGenerator, CHUNK_DIM};
        use crate::generation::noise::router::{
            multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
            surface_height_sampler::{SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions},
        };
        use crate::generation::{biome_coords, positions::chunk_pos, aquifer_sampler::{FluidLevel, FluidLevelSampler}};
        use crate::generation::proto_chunk::StandardChunkFluidLevelSampler;

        let generation_shape = &generation_settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
        let start_x = chunk_pos::start_block_x(at);
        let start_z = chunk_pos::start_block_z(at);

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
            MultiNoiseSampler::generate(&self.base_router.multi_noise, &multi_noise_config);

        // Noise sampler
        let sampler = FluidLevelSampler::Chunk(Box::new(StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                generation_settings.sea_level,
                generation_settings.default_fluid.name,
            ),
            FluidLevel::new(-54, &pumpkin_data::Block::LAVA), 
        )));

        let mut noise_sampler = ChunkNoiseGenerator::new(
            &self.base_router.noise,
            &self.random_config,
            horizontal_cell_count as usize,
            start_x,
            start_z,
            generation_shape,
            sampler,
            generation_settings.aquifers_enabled,
            generation_settings.ore_veins_enabled,
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
            SurfaceHeightEstimateSampler::generate(&self.base_router.surface_estimator, &surface_config);

        proto_chunk.populate_biomes(self.dimension, &mut multi_noise_sampler);
        proto_chunk.populate_noise(&mut noise_sampler, &mut surface_height_estimate_sampler);
        proto_chunk.build_surface(generation_settings, &self.random_config, &self.terrain_cache, &mut surface_height_estimate_sampler);
        proto_chunk.generate_features_and_structure(level, block_registry, &self.random_config);

        for y in 0..biome_coords::from_block(generation_settings.shape.height) {
            let relative_y = y as usize;
            let section_index = relative_y / BiomePalette::SIZE;
            let relative_y = relative_y % BiomePalette::SIZE;
            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..BiomePalette::SIZE {
                    for x in 0..BiomePalette::SIZE {
                        let absolute_y =
                            biome_coords::from_block(generation_settings.shape.min_y as i32)
                                + y as i32;
                        let biome =
                            proto_chunk.get_biome(&Vector3::new(x as i32, absolute_y, z as i32));
                        section.biomes.set(x, relative_y, z, biome.id);
                    }
                }
            }
        }
        for y in 0..generation_settings.shape.height {
            let relative_y = (y as i32 - sections.min_y) as usize;
            let section_index = relative_y / BlockPalette::SIZE;
            let relative_y = relative_y % BlockPalette::SIZE;
            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..BlockPalette::SIZE {
                    for x in 0..BlockPalette::SIZE {
                        let absolute_y = generation_settings.shape.min_y as i32 + y as i32;
                        let block = proto_chunk
                            .get_block_state(&Vector3::new(x as i32, absolute_y, z as i32));
                        section.block_states.set(x, relative_y, z, block.0);
                    }
                }
            }
        }
        ChunkData {
            light_engine: ChunkLight {
                sky_light: (0..sections.sections.len() + 2)
                    .map(|_| LightContainer::new_filled(15))
                    .collect(),
                block_light: (0..sections.sections.len() + 2)
                    .map(|_| LightContainer::new_empty(15))
                    .collect(),
            },
            section: sections,
            heightmap: Default::default(),
            position: *at,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            block_entities: Default::default(),
        }
    }
}

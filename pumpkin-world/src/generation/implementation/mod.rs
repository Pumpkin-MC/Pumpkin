use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    chunk::{
        ChunkData, ChunkSections, SubChunk,
        palette::{BiomePalette, BlockPalette},
    },
    generation::{
        GlobalRandomConfig, Seed, WorldGenerator, generator::GeneratorInit, proto_chunk::ProtoChunk,
    },
};

use super::{
    biome_coords,
    noise_router::proto_noise_router::ProtoNoiseRouters,
    settings::{GENERATION_SETTINGS, GeneratorSetting},
};

pub struct VanillaGenerator {
    random_config: GlobalRandomConfig,
    base_router: ProtoNoiseRouters,
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0, false);
        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base_router = ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
        Self {
            random_config,
            base_router,
        }
    }
}

impl WorldGenerator for VanillaGenerator {
    fn generate_chunk(&self, at: &Vector2<i32>) -> ChunkData {
        // TODO: Dont hardcode this
        let generation_settings = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();

        let sub_chunks = generation_settings.shape.height as usize / BlockPalette::SIZE;
        let sections = (0..sub_chunks).map(|_| SubChunk::max_light()).collect();
        let mut sections = ChunkSections::new(sections, generation_settings.shape.min_y as i32);

        let mut proto_chunk = ProtoChunk::new(
            *at,
            &self.base_router,
            &self.random_config,
            generation_settings,
        );
        proto_chunk.populate_biomes();
        proto_chunk.populate_noise();
        proto_chunk.build_surface();

        for y in 0..biome_coords::from_block(generation_settings.shape.height) {
            for z in 0..BiomePalette::SIZE {
                for x in 0..BiomePalette::SIZE {
                    let absolute_y =
                        biome_coords::from_block(generation_settings.shape.min_y as i32) + y as i32;
                    let biome =
                        proto_chunk.get_biome(&Vector3::new(x as i32, absolute_y, z as i32));
                    sections.set_relative_biome(x, y as usize, z, biome.id);
                }
            }
        }

        for y in 0..generation_settings.shape.height {
            for z in 0..BlockPalette::SIZE {
                for x in 0..BlockPalette::SIZE {
                    let absolute_y = generation_settings.shape.min_y as i32 + y as i32;
                    let block =
                        proto_chunk.get_block_state(&Vector3::new(x as i32, absolute_y, z as i32));
                    sections.set_relative_block(x, y as usize, z, block.state_id);
                }
            }
        }

        ChunkData {
            section: sections,
            heightmap: Default::default(),
            position: *at,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
        }
    }
}

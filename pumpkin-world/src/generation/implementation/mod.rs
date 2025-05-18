use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{
    biome_coords, noise_router::proto_noise_router::ProtoNoiseRouters,
    settings::gen_settings_from_dimension,
};
use crate::chunk::ChunkHeightmaps;
use crate::chunk::format::LightContainer;
use crate::{chunk::ChunkLight, dimension::Dimension};
use crate::{
    chunk::{
        ChunkData, ChunkSections, SubChunk,
        palette::{BiomePalette, BlockPalette},
    },
    generation::{
        GlobalRandomConfig, Seed, WorldGenerator, generator::GeneratorInit, proto_chunk::ProtoChunk,
    },
};

pub struct VanillaGenerator {
    random_config: GlobalRandomConfig,
    base_router: ProtoNoiseRouters,
    dimension: Dimension,
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
        let base_router = ProtoNoiseRouters::generate(&base, &random_config);
        Self {
            random_config,
            base_router,
            dimension,
        }
    }
}

impl WorldGenerator for VanillaGenerator {
    fn generate_chunk(&self, at: &Vector2<i32>) -> ChunkData {
        let generation_settings = gen_settings_from_dimension(&self.dimension);

        let sub_chunks = generation_settings.shape.height as usize / BlockPalette::SIZE;
        let sections = (0..sub_chunks).map(|_| SubChunk::default()).collect();
        let mut sections = ChunkSections::new(sections, generation_settings.shape.min_y as i32);

        let mut proto_chunk = ProtoChunk::new(
            *at,
            &self.base_router,
            &self.random_config,
            generation_settings,
        );
        proto_chunk.populate_biomes(self.dimension);
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
        let heightmap = ChunkHeightmaps {
            world_surface: proto_chunk.flat_surface_height_map,
            motion_blocking: proto_chunk.flat_motion_blocking_height_map,
            motion_blocking_no_leaves: proto_chunk.flat_motion_blocking_no_leaves_height_map,
        };
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
            heightmap,
            position: *at,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            block_entities: Default::default(),
        }
    }
}

use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::BlockState;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{
    biome_coords, noise::router::proto_noise_router::ProtoNoiseRouters,
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

        let mut proto_chunk = ProtoChunk::new(
            *at,
            &self.base_router,
            &self.random_config,
            generation_settings,
            &self.terrain_cache,
            self.default_block,
        );
        proto_chunk.populate_biomes(self.dimension);
        proto_chunk.populate_noise();
        proto_chunk.build_surface();
        proto_chunk.generate_features_and_structure(level, block_registry);

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

        let mut sky_light_sections: Box<[LightContainer]> = (0..sections.sections.len() + 2)
            .map(|_| LightContainer::new_empty(0))
            .collect();

        let mut current_section = 0;
        for index in (0..sections.sections.len() + 2).rev() {
            // The first and last sections are always empty
            if index == 0 {
                sky_light_sections[index] = LightContainer::new_empty(0);
                continue;
            } else if index == sections.sections.len() + 1 {
                sky_light_sections[index] = LightContainer::new_filled(15);
            } else if let Some(section) = sections.sections.get(index - 1) {
                if section.block_states.is_empty() && section.block_states.get(0, 0, 0) == 0 {
                    sky_light_sections[index] = LightContainer::new_filled(15);
                    current_section = index;
                } else {
                    break;
                }
            }
        }

        let start_height =
            sections.min_y + (current_section as i32 - 1) * BlockPalette::SIZE as i32;

        for x in 0..16 {
            for z in 0..16 {
                for y in (sections.min_y..=start_height).rev() {
                    let state_id = sections.get_block_absolute_y(x, y, z).unwrap_or(0);
                    let mut block_state = BlockState::from_id(state_id);
                    if block_state.is_air() {
                        let section_index = (y - sections.min_y) as usize / BlockPalette::SIZE;
                        let relative_y = (y - sections.min_y) as usize % BlockPalette::SIZE;
                        sky_light_sections[section_index + 1].set(x, relative_y, z, 15);
                    } else {
                        // If we hit a non-air block, need to propagate the light down if possible
                        // This is a bit of a hack, but it works for now
                        let mut light = 15 - block_state.opacity;
                        let mut section_index = (y - sections.min_y) as usize / BlockPalette::SIZE;
                        let mut relative_y = (y - sections.min_y) as usize % BlockPalette::SIZE;
                        while light > 0 {
                            sky_light_sections[section_index + 1].set(x, relative_y, z, light);
                            if relative_y == 0 {
                                if section_index == 0 {
                                    // If we are at the bottom section, we can't propagate light down
                                    break;
                                }
                                section_index -= 1;
                                relative_y = BlockPalette::SIZE - 1;
                            } else {
                                relative_y -= 1;
                            }
                            block_state = BlockState::from_id(
                                sections.get_block_absolute_y(x, y - 1, z).unwrap_or(0),
                            );
                            if light <= block_state.opacity {
                                break;
                            }
                            light -= block_state.opacity;
                        }
                        break;
                    }
                }
            }
        }

        ChunkData {
            light_engine: ChunkLight {
                sky_light: sky_light_sections,
                block_light: (0..sections.sections.len() + 2)
                    .map(|_| LightContainer::new_empty(0))
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

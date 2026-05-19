use pumpkin_world::chunk::{ChunkData, ChunkLight, ChunkSections};
use pumpkin_world::generation::biome_coords;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::dimension::Dimension;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use pumpkin_world::ProtoChunk;
use crate::level::SyncChunk;

use pumpkin_data::chunk::ChunkStatus;
use std::sync::Mutex;

pub use pumpkin_world::chunk_system_data::StagedChunkEnum;

pub enum Chunk {
    Level(SyncChunk),
    Proto(Box<ProtoChunk>),
}

impl Chunk {
    #[must_use]
    pub fn get_stage_id(&self) -> u8 {
        match self {
            Self::Proto(data) => data.stage_id(),
            Self::Level(_) => StagedChunkEnum::Full as u8,
        }
    }
    pub fn get_proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Self::Proto(chunk) => chunk,
        }
    }
    #[must_use]
    pub fn get_proto_chunk(&self) -> &ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Self::Proto(chunk) => chunk,
        }
    }
    pub fn upgrade_to_level_chunk(
        &mut self,
        dimension: &Dimension,
        lighting_config: &LightingEngineConfig,
    ) {
        // Take ownership of the ProtoChunk by temporarily replacing with a dummy value
        // This allows us to move the light data instead of cloning it
        let proto_chunk_box = match std::mem::replace(
            self,
            Self::Level(Arc::new(ChunkData {
                section: ChunkSections::new(0, 0),
                heightmap: Default::default(),
                x: 0,
                z: 0,
                block_ticks: Default::default(),
                fluid_ticks: Default::default(),
                pending_block_entities: Default::default(),
                light_engine: Mutex::new(ChunkLight::default()),
                light_populated: AtomicBool::new(false),
                status: ChunkStatus::Empty,
                blending_data: None,
                dirty: AtomicBool::new(false),
            })),
        ) {
            Self::Proto(proto) => proto,
            Self::Level(_) => panic!("Cannot upgrade a Level chunk"),
        };

        let proto_chunk = *proto_chunk_box;

        let total_sections = dimension.height as usize / 16;
        let sections = ChunkSections::new(total_sections, dimension.min_y);

        let proto_biome_height = biome_coords::from_block(proto_chunk.height() as i32);
        let biome_min_y = biome_coords::from_block(dimension.min_y);

        for y_offset in 0..proto_biome_height {
            let section_index = y_offset as usize / 4;
            let relative_y = y_offset as usize % 4;

            if let Some(section) = sections
                .biome_sections
                .write()
                .unwrap()
                .get_mut(section_index)
            {
                let absolute_biome_y = biome_min_y + y_offset;

                for z in 0..4 {
                    for x in 0..4 {
                        let biome = proto_chunk.get_biome_id(x as i32, absolute_biome_y, z as i32);
                        section.set(x, relative_y, z, biome);
                    }
                }
            }
        }

        let proto_block_height = proto_chunk.height();

        for y_offset in 0..proto_block_height {
            let section_index = (y_offset as usize) / 16;
            let relative_y = (y_offset as usize) % 16;

            if let Some(section) = sections
                .block_sections
                .write()
                .unwrap()
                .get_mut(section_index)
            {
                for z in 0..16 {
                    for x in 0..16 {
                        let block =
                            proto_chunk.get_block_state_raw(x as i32, y_offset as i32, z as i32);
                        section.set(x, relative_y, z, block);
                    }
                }
            }
        }

        // Move the light data instead of cloning it
        // By taking ownership of proto_chunk, we can move the light data directly
        // This prevents keeping duplicate lighting data in memory
        let light_data = proto_chunk.light;

        // Only mark lit if past the lighting stage, and the lighting config is "default" ("full" and "dark" modes skip proper lighting)
        let is_lit = proto_chunk.stage >= StagedChunkEnum::Lighting
            && *lighting_config == LightingEngineConfig::Default;

        // Convert pending block entities from structure generation to actual block entities
        let mut pending_block_entities = FxHashMap::default();
        for nbt in proto_chunk.pending_block_entities {
            if let Some(x) = nbt.get_int("x")
                && let Some(y) = nbt.get_int("y")
                && let Some(z) = nbt.get_int("z")
            {
                pending_block_entities
                    .insert(pumpkin_util::math::position::BlockPos::new(x, y, z), nbt);
            }
        }

        let mut chunk = ChunkData {
            light_engine: Mutex::new(light_data),
            light_populated: AtomicBool::new(is_lit),
            section: sections,
            heightmap: Default::default(),
            x: proto_chunk.x,
            z: proto_chunk.z,
            dirty: AtomicBool::new(true),
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            pending_block_entities: Mutex::new(pending_block_entities),
            status: proto_chunk.stage.into(),
            blending_data: proto_chunk.blending_data,
        };

        chunk.heightmap = Mutex::new(chunk.calculate_heightmap());
        *self = Self::Level(Arc::new(chunk));
    }
}

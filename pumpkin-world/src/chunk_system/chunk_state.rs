use crate::block::entities::block_entity_from_nbt;
use crate::chunk::{ChunkData, ChunkLight, ChunkSections};
use crate::generation::biome_coords;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::dimension::Dimension;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::ProtoChunk;
use crate::level::SyncChunk;

use pumpkin_data::chunk::ChunkStatus;
use std::sync::Mutex;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum StagedChunkEnum {
    None,
    /// Initial empty chunk, ready for biome population
    Empty = 1, // EMPTY STRUCTURE_STARTS STRUCTURE_REFERENCES
    /// Chunk with biomes populated, ready for noise generation
    Biomes,
    StructureStart,
    StructureReferences,
    /// Chunk with terrain noise generated, ready for surface building
    Noise,
    /// Chunk with surface built, ready for features and structures
    Surface, // SURFACE CARVERS
    /// Chunk with features and structures, ready for lighting
    Features, // FEATURES SPAWN
    /// Chunk with lighting calculated, ready for finalization
    Lighting, // INITIALIZE LIGHT
    /// Fully generated chunk
    Full,
}

impl From<u8> for StagedChunkEnum {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Empty,
            2 => Self::Biomes,
            3 => Self::StructureStart,
            4 => Self::StructureReferences,
            5 => Self::Noise,
            6 => Self::Surface,
            7 => Self::Features,
            8 => Self::Lighting,
            9 => Self::Full,
            _ => panic!(),
        }
    }
}

impl From<ChunkStatus> for StagedChunkEnum {
    fn from(status: ChunkStatus) -> Self {
        match status {
            ChunkStatus::Empty => Self::Empty,
            ChunkStatus::StructureStarts => Self::StructureStart,
            ChunkStatus::StructureReferences => Self::StructureReferences,
            ChunkStatus::Biomes => Self::Biomes,
            ChunkStatus::Noise => Self::Noise,
            ChunkStatus::Surface => Self::Surface,
            ChunkStatus::Carvers => Self::Surface,
            ChunkStatus::Features => Self::Features,
            ChunkStatus::Spawn => Self::Features,
            ChunkStatus::InitializeLight => Self::Lighting,
            ChunkStatus::Light => Self::Lighting,
            ChunkStatus::Full => Self::Full,
        }
    }
}

impl From<StagedChunkEnum> for ChunkStatus {
    fn from(status: StagedChunkEnum) -> Self {
        match status {
            StagedChunkEnum::Empty => Self::Empty,
            StagedChunkEnum::StructureStart => Self::StructureStarts,
            StagedChunkEnum::StructureReferences => Self::StructureReferences,
            StagedChunkEnum::Biomes => Self::Biomes,
            StagedChunkEnum::Noise => Self::Noise,
            StagedChunkEnum::Surface => Self::Surface,
            StagedChunkEnum::Features => Self::Features,
            StagedChunkEnum::Lighting => Self::Light,
            StagedChunkEnum::Full => Self::Full,
            _ => panic!(),
        }
    }
}

impl StagedChunkEnum {
    pub const fn level_to_stage(level: i8) -> Self {
        if level <= 43 {
            Self::Full
        } else if level <= 44 {
            Self::Lighting
        } else if level <= 45 {
            Self::Features
        } else if level <= 46 {
            Self::Surface
        } else {
            Self::None
        }
    }

    /// Total number of state values (0 = None … 9 = Full).
    pub const COUNT: usize = Self::Full as usize + 1;
    pub const FULL_DEPENDENCIES: &'static [Self] =
        &[Self::Full, Self::Lighting, Self::Features, Self::Surface];
    pub const FULL_RADIUS: i32 = 3;
    pub const fn get_direct_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Features => 1,
            Self::Lighting => 1,
            Self::Full => 1,
            _ => panic!(),
        }
    }
    pub const fn get_write_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Features => 1,
            Self::Lighting => 1,
            Self::Full => 0,
            _ => panic!(),
        }
    }
    pub const fn get_direct_dependencies(self) -> &'static [Self] {
        match self {
            // In vanilla StructureStart is first, but since it needs the biome in Vanilla it gets computed in StructureStart and
            // the Biome Step, this should be more efficient
            Self::Biomes => &[Self::Empty],
            Self::StructureStart => &[Self::Biomes],
            Self::StructureReferences => &[Self::StructureStart],
            Self::Noise => &[Self::StructureReferences],
            Self::Surface => &[Self::Noise],
            Self::Features => &[Self::Surface, Self::Surface],
            Self::Lighting => &[Self::Features, Self::Features],
            Self::Full => &[Self::Lighting, Self::Lighting],
            _ => panic!(),
        }
    }
}

pub enum Chunk {
    Level(SyncChunk),
    Proto(Box<ProtoChunk>),
}

impl Chunk {
    pub fn get_stage_id(&self) -> u8 {
        match self {
            Self::Proto(data) => data.stage_id(),
            Self::Level(_) => 9,
        }
    }
    pub fn get_proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Chunk::Proto(chunk) => chunk,
        }
    }
    pub fn get_proto_chunk(&self) -> &ProtoChunk {
        match self {
            Self::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Chunk::Proto(chunk) => chunk,
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
            Chunk::Level(Arc::new(ChunkData {
                section: ChunkSections::new(0, 0),
                heightmap: Default::default(),
                x: 0,
                z: 0,
                block_ticks: Default::default(),
                fluid_ticks: Default::default(),
                block_entities: Default::default(),
                light_engine: Mutex::new(ChunkLight::default()),
                light_populated: AtomicBool::new(false),
                status: ChunkStatus::Empty,
                dirty: AtomicBool::new(false),
            })),
        ) {
            Chunk::Proto(proto) => proto,
            Chunk::Level(_) => panic!("Cannot upgrade a Level chunk"),
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
        let mut block_entities = FxHashMap::default();
        for nbt in proto_chunk.pending_block_entities {
            if let Some(block_entity) = block_entity_from_nbt(&nbt) {
                let pos = block_entity.get_position();
                block_entities.insert(pos, block_entity);
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
            block_entities: Mutex::new(block_entities),
            status: proto_chunk.stage.into(),
        };

        chunk.heightmap = Mutex::new(chunk.calculate_heightmap());
        *self = Self::Level(Arc::new(chunk));
    }
}

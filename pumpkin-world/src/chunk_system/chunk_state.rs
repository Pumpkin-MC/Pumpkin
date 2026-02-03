use crate::chunk::{ChunkData, ChunkSections, SubChunk};
use crate::generation::biome_coords;
use pumpkin_data::dimension::Dimension;
use std::sync::Arc;

use crate::ProtoChunk;
use crate::level::SyncChunk;
use tokio::sync::RwLock;

use pumpkin_data::chunk::ChunkStatus;

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
    /// Chunk with features and structures, ready for finalization
    Features, // FEATURES INITIALIZE_LIGHT LIGHT SPAWN
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
            8 => Self::Full,
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
            ChunkStatus::InitializeLight => Self::Features,
            ChunkStatus::Light => Self::Features,
            ChunkStatus::Spawn => Self::Features,
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
            Self::Features
        } else if level <= 45 {
            Self::Surface
        } else {
            Self::None
        }
    }
    pub const FULL_DEPENDENCIES: &'static [Self] = &[Self::Full, Self::Features, Self::Surface];
    pub const FULL_RADIUS: i32 = 2;
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
            Self::Full => &[Self::Features, Self::Features],
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
            Self::Level(_) => 8,
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
    pub fn upgrade_to_level_chunk(&mut self, dimension: &Dimension) {
        let proto_chunk = self.get_proto_chunk();

        let total_sections = dimension.height as usize / 16;
        let mut sections = ChunkSections::new(
            vec![SubChunk::default(); total_sections].into_boxed_slice(),
            dimension.min_y,
        );

        let proto_biome_height = biome_coords::from_block(proto_chunk.height());
        let biome_min_y = biome_coords::from_block(dimension.min_y);

        for y_offset in 0..proto_biome_height {
            let section_index = y_offset as usize / 4;
            let relative_y = y_offset as usize % 4;

            if let Some(section) = sections.sections.get_mut(section_index) {
                let absolute_biome_y = biome_min_y + y_offset as i32;

                for z in 0..4 {
                    for x in 0..4 {
                        let biome = proto_chunk.get_biome_id(x as i32, absolute_biome_y, z as i32);
                        section.biomes.set(x, relative_y, z, biome);
                    }
                }
            }
        }

        let proto_block_height = proto_chunk.height();

        for y_offset in 0..proto_block_height {
            let section_index = (y_offset as usize) / 16;
            let relative_y = (y_offset as usize) % 16;

            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..16 {
                    for x in 0..16 {
                        let block =
                            proto_chunk.get_block_state_raw(x as i32, y_offset as i32, z as i32);
                        section.block_states.set(x, relative_y, z, block);
                    }
                }
            }
        }

        let mut chunk = ChunkData {
            light_engine: proto_chunk.light.clone(),
            section: sections,
            heightmap: Default::default(),
            x: proto_chunk.x,
            z: proto_chunk.z,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            block_entities: Default::default(),
            status: proto_chunk.stage.into(),
        };

        chunk.heightmap = chunk.calculate_heightmap();
        *self = Self::Level(Arc::new(RwLock::new(chunk)));
    }
}

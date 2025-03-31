use pumpkin_data::chunk::Biome;
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coordinates::ChunkRelativeBlockCoordinates;

pub mod format;
pub mod io;
mod palette;

// TODO
const WORLD_HEIGHT: usize = 384;
pub const CHUNK_AREA: usize = 16 * 16;
pub const BIOME_VOLUME: usize = 64;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;
pub const SUBCHUNKS_COUNT: usize = WORLD_HEIGHT / 16;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

#[derive(Error, Debug)]
pub enum ChunkReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Tried to read chunk which does not exist")]
    ChunkNotExist,
    #[error("Failed to parse chunk from bytes: {0}")]
    ParsingError(ChunkParsingError),
}

#[derive(Error, Debug)]
pub enum ChunkWritingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Chunk serializing error: {0}")]
    ChunkSerializingError(String),
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression scheme not recognised")]
    UnknownCompression,
    #[error("Error while working with zlib compression: {0}")]
    ZlibError(std::io::Error),
    #[error("Error while working with Gzip compression: {0}")]
    GZipError(std::io::Error),
    #[error("Error while working with LZ4 compression: {0}")]
    LZ4Error(std::io::Error),
    #[error("Error while working with zstd compression: {0}")]
    ZstdError(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(i32)]
pub enum TickPriority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

impl TickPriority {
    pub fn values() -> [TickPriority; 7] {
        [
            TickPriority::ExtremelyHigh,
            TickPriority::VeryHigh,
            TickPriority::High,
            TickPriority::Normal,
            TickPriority::Low,
            TickPriority::VeryLow,
            TickPriority::ExtremelyLow,
        ]
    }
}

impl From<i32> for TickPriority {
    fn from(value: i32) -> Self {
        match value {
            -3 => TickPriority::ExtremelyHigh,
            -2 => TickPriority::VeryHigh,
            -1 => TickPriority::High,
            0 => TickPriority::Normal,
            1 => TickPriority::Low,
            2 => TickPriority::VeryLow,
            3 => TickPriority::ExtremelyLow,
            _ => panic!("Invalid tick priority: {}", value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledTick {
    pub block_pos: BlockPos,
    pub delay: u16,
    pub priority: TickPriority,
    pub target_block_id: u16,
}

pub struct ChunkData {
    pub section: ChunkSection,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
    pub block_ticks: Vec<ScheduledTick>,
    pub fluid_ticks: Vec<ScheduledTick>,

    pub dirty: bool,
}

/// Represents pure block data for a chunk.
/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
/// There are currently 24 subchunks per chunk.
///
/// A chunk can be:
/// - Subchunks: 24 separate subchunks are stored.
#[derive(PartialEq, Debug, Clone)]
pub struct ChunkSection {
    pub sections: Box<[SubChunk; SUBCHUNKS_COUNT]>,
}

/// The whole chunk is filled with one block type and biome.
#[derive(PartialEq, Debug, Clone)]
pub struct HomogeneousChunk {
    block_state: u16,
    biome: Biome,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SubChunk {
    pub block_states: SubchunkBlocks,
    pub biomes: SubchunkBiomes,
}

impl SubChunk {
    pub fn new_single(block_state: u16, biome: &'static Biome) -> Self {
        Self {
            block_states: SubchunkBlocks::Homogeneous(block_state),
            biomes: SubchunkBiomes::Homogeneous(biome),
        }
    }
    pub fn empty() -> Self {
        Self {
            block_states: SubchunkBlocks::Homogeneous(0),
            biomes: SubchunkBiomes::Homogeneous(&Biome::PLAINS),
        }
    }
}

/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
///
/// A subchunk can be:
/// - Homogeneous: the whole subchunk is filled with one block type, like air or water.
/// - Heterogeneous: 16^3 = 4096 individual blocks are stored.
#[derive(Clone, PartialEq, Debug)]
pub enum SubchunkBlocks {
    Homogeneous(u16),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    Heterogeneous(Box<[u16; SUBCHUNK_VOLUME]>),
}

/// A subchunk can be:
/// - Homogeneous: the whole subchunk is filled with one biome, like plains or badlans.
/// - Heterogeneous: 64 individual biomes are stored.
#[derive(Clone, PartialEq, Debug)]
pub enum SubchunkBiomes {
    Homogeneous(&'static Biome),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    Heterogeneous(Box<[&'static Biome; BIOME_VOLUME]>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    #[serde(serialize_with = "nbt_long_array")]
    pub world_surface: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    pub motion_blocking: Box<[i64]>,
}

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            // 0 packed into an i64 7 times.
            motion_blocking: vec![0; 37].into_boxed_slice(),
            world_surface: vec![0; 37].into_boxed_slice(),
        }
    }
}

impl SubchunkBlocks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Homogeneous(block) => Some(*block),
            Self::Heterogeneous(blocks) => blocks.get(Self::convert_index(position)).copied(),
        }
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_block: u16,
    ) {
        match self {
            Self::Homogeneous(block) => {
                if *block != new_block {
                    let mut blocks = Box::new([*block; SUBCHUNK_VOLUME]);
                    blocks[Self::convert_index(position)] = new_block;

                    *self = Self::Heterogeneous(blocks)
                }
            }
            Self::Heterogeneous(blocks) => {
                blocks[Self::convert_index(position)] = new_block;

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Homogeneous(new_block)
                }
            }
        }
    }

    pub fn clone_as_array(&self) -> Box<[u16; SUBCHUNK_VOLUME]> {
        match &self {
            Self::Homogeneous(block) => Box::new([*block; SUBCHUNK_VOLUME]),
            Self::Heterogeneous(blocks) => blocks.clone(),
        }
    }

    fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
        // % works for negative numbers as intended.
        (index.y.get_absolute() % 16) as usize * CHUNK_AREA
            + *index.z as usize * 16
            + *index.x as usize
    }
}

impl SubchunkBiomes {
    /// Gets the given block in the chunk
    pub fn get_biome(&self, position: ChunkRelativeBlockCoordinates) -> Option<&Biome> {
        match self {
            Self::Homogeneous(biome) => Some(biome),
            Self::Heterogeneous(biomes) => biomes.get(Self::convert_index(position)).copied(),
        }
    }

    pub fn set_biome(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_biome: &'static Biome,
    ) {
        match self {
            Self::Homogeneous(biome) => {
                if *biome != new_biome {
                    let mut biome = Box::new([*biome; BIOME_VOLUME]);
                    biome[Self::convert_index(position)] = new_biome;

                    *self = Self::Heterogeneous(biome)
                }
            }
            Self::Heterogeneous(bioes) => {
                bioes[Self::convert_index(position)] = new_biome;

                if bioes.iter().all(|b| *b == new_biome) {
                    *self = Self::Homogeneous(new_biome)
                }
            }
        }
    }

    pub fn clone_as_array(&self) -> Box<[&Biome; BIOME_VOLUME]> {
        match self {
            Self::Homogeneous(biome) => Box::new([biome; BIOME_VOLUME]),
            Self::Heterogeneous(biomes) => biomes.clone(),
        }
    }

    fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
        // % works for negative numbers as intended.
        (index.y.get_absolute() % 4) as usize * 4 as usize
            + *index.z as usize * 4
            + *index.x as usize
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        let sections: [SubChunk; SUBCHUNKS_COUNT] = core::array::from_fn(|_| SubChunk::empty());
        Self {
            sections: Box::new(sections),
        }
    }
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.sections
            .get((position.y.get_absolute() / 16) as usize)
            .and_then(|subchunk| subchunk.block_states.get_block(position))
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_biome(&mut self, position: ChunkRelativeBlockCoordinates, biome: &'static Biome) {
        self.sections[(position.y.get_absolute() / 16) as usize]
            .biomes
            .set_biome(position, biome);
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_block: u16,
    ) {
        self.sections[(position.y.get_absolute() / 16) as usize]
            .block_states
            .set_block(position, new_block);
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.section.get_block(position)
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.section.set_block(position, block_id);
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block: u16,
    ) {
        self.section.set_block_no_heightmap_update(position, block);
    }

    #[expect(dead_code)]
    fn calculate_heightmap(&self) -> ChunkHeightmaps {
        // figure out how LongArray is formatted
        // figure out how to find out if block is motion blocking
        todo!()
    }
}
#[derive(Error, Debug)]
pub enum ChunkParsingError {
    #[error("Failed reading chunk status {0}")]
    FailedReadStatus(pumpkin_nbt::Error),
    #[error("The chunk isn't generated yet")]
    ChunkNotGenerated,
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
}

#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(pumpkin_nbt::Error),
}

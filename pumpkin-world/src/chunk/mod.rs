use palette::{BiomePalette, BlockPalette};
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod format;
pub mod io;
pub mod palette;

// TODO
const WORLD_HEIGHT: usize = 384;
pub const CHUNK_AREA: usize = BlockPalette::SIZE * BlockPalette::SIZE;
pub const BIOME_VOLUME: usize = 64;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * BlockPalette::SIZE;
pub const SUBCHUNKS_COUNT: usize = WORLD_HEIGHT / BlockPalette::SIZE;
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
    pub section: ChunkSections,
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
#[derive(Debug)]
pub struct ChunkSections {
    pub sections: Box<[SubChunk; SUBCHUNKS_COUNT]>,
    min_y: i32,
}

impl ChunkSections {
    #[cfg(test)]
    pub fn dump_blocks(&self) -> Vec<u16> {
        let mut dump = Vec::new();
        for section in self.sections.iter() {
            section.block_states.iter_yzx(|raw_id| {
                dump.push(raw_id);
            });
        }
        dump
    }

    #[cfg(test)]
    pub fn dump_biomes(&self) -> Vec<u16> {
        let mut dump = Vec::new();
        for section in self.sections.iter() {
            section.biomes.iter_yzx(|raw_id| {
                dump.push(raw_id);
            });
        }
        dump
    }
}

#[derive(Debug, Default)]
pub struct SubChunk {
    pub block_states: BlockPalette,
    pub biomes: BiomePalette,
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
            // 9 bits per entry
            // 0 packed into an i64 7 times.
            motion_blocking: vec![0; 37].into_boxed_slice(),
            world_surface: vec![0; 37].into_boxed_slice(),
        }
    }
}

impl ChunkSections {
    pub fn new(min_y: i32) -> Self {
        let sections: [SubChunk; SUBCHUNKS_COUNT] = core::array::from_fn(|_| SubChunk::default());
        Self {
            sections: Box::new(sections),
            min_y,
        }
    }
    /// Gets the given block in the chunk
    pub fn get_block(&self, relative_x: usize, relative_y: usize, relative_z: usize) -> u16 {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;
        self.sections[section_index]
            .block_states
            .get_id(relative_x, relative_y, relative_z)
    }

    /// Sets the given block in the chunk, returning the old block
    #[inline]
    pub fn set_block(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: u16,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(relative_x, relative_y, relative_z, block_state_id);
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: u16,
    ) {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;
        self.sections[section_index].block_states.set_id(
            relative_x,
            relative_y,
            relative_z,
            block_state_id,
        );
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_biome(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        biome_id: u16,
    ) {
        debug_assert!(relative_x < BiomePalette::SIZE);
        debug_assert!(relative_z < BiomePalette::SIZE);

        let section_index = relative_y / BiomePalette::SIZE;
        let relative_y = relative_y % BiomePalette::SIZE;
        self.sections[section_index]
            .biomes
            .set_id(relative_x, relative_y, relative_z, biome_id);
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    #[inline]
    pub fn get_block(&self, relative_x: usize, relative_y: usize, relative_z: usize) -> u16 {
        self.section.get_block(relative_x, relative_y, relative_z)
    }

    /// Sets the given block in the chunk
    #[inline]
    pub fn set_block(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: u16,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.section
            .set_block(relative_x, relative_y, relative_z, block_state_id);
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    #[inline]
    pub fn set_block_no_heightmap_update(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: u16,
    ) {
        self.section
            .set_block(relative_x, relative_y, relative_z, block_state_id);
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

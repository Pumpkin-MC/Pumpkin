use pumpkin_data::block::Block;
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::{position::{chunk_section_from_pos, BlockPos}, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::repeat_with, sync::Arc};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};

use crate::{WORLD_HEIGHT, coordinates::ChunkRelativeBlockCoordinates};

pub mod format;
pub mod io;

pub const CHUNK_AREA: usize = 16 * 16;
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
    #[error("Failed to parse Chunk from bytes: {0}")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub delay: u16,
    pub priority: TickPriority,
    pub target_block_id: u16,
}

#[derive(Debug, Clone)]
pub struct FluidTick {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub delay: u16,
    pub priority: TickPriority,
    pub target_block: Block,
}

pub struct ChunkData {
    /// See description in `Subchunks`
    pub subchunks: Subchunks,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
    pub block_ticks: Arc<RwLock<Vec<ScheduledTick>>>,
    pub fluid_ticks: Arc<RwLock<Vec<FluidTick>>>,
    pub block_state_updates: Mutex<HashMap<BlockPos, u16>>,
}

/// # Subchunks
/// Subchunks - its an areas in chunk, what are 16 blocks in height.
/// Current amount is 24.
///
/// Subchunks can be single and multi.
///
/// Single means a single block in all chunk, like
/// chunk, what filled only air or only water.
///
/// Multi means a normal chunk, what contains 24 subchunks.
#[derive(PartialEq, Debug, Clone)]
pub enum Subchunks {
    Single(u16),
    Multi(Box<[Subchunk; SUBCHUNKS_COUNT]>),
}

/// # Subchunk
/// Subchunk - its an area in chunk, what are 16 blocks in height
///
/// Subchunk can be single and multi.
///
/// Single means a single block in all subchunk, like
/// subchunk, what filled only air or only water.
///
/// Multi means a normal subchunk, what contains 4096 blocks.
#[derive(Clone, PartialEq, Debug)]
pub enum Subchunk {
    Single(u16),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    Multi(Box<[u16; SUBCHUNK_VOLUME]>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    #[serde(serialize_with = "nbt_long_array")]
    motion_blocking: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    world_surface: Box<[i64]>,
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

impl Subchunk {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Single(block) => Some(*block),
            Self::Multi(blocks) => blocks.get(convert_index(position)).copied(),
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
            Self::Single(block) => {
                if *block != new_block {
                    let mut blocks = Box::new([*block; SUBCHUNK_VOLUME]);
                    blocks[convert_index(position)] = new_block;

                    *self = Self::Multi(blocks)
                }
            }
            Self::Multi(blocks) => {
                blocks[convert_index(position)] = new_block;

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Single(new_block)
                }
            }
        }
    }

    pub fn clone_as_array(&self) -> Box<[u16; SUBCHUNK_VOLUME]> {
        match &self {
            Self::Single(block) => Box::new([*block; SUBCHUNK_VOLUME]),
            Self::Multi(blocks) => blocks.clone(),
        }
    }
}

impl Subchunks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Single(block) => Some(*block),
            Self::Multi(subchunks) => subchunks
                .get((position.y.get_absolute() / 16) as usize)
                .and_then(|subchunk| subchunk.get_block(position)),
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
            Self::Single(block) => {
                if *block != new_block {
                    let mut subchunks = vec![Subchunk::Single(0); SUBCHUNKS_COUNT];

                    subchunks[(position.y.get_absolute() / 16) as usize]
                        .set_block(position, new_block);

                    *self = Self::Multi(subchunks.try_into().unwrap());
                }
            }
            Self::Multi(subchunks) => {
                subchunks[(position.y.get_absolute() / 16) as usize].set_block(position, new_block);

                if subchunks
                    .iter()
                    .all(|subchunk| *subchunk == Subchunk::Single(new_block))
                {
                    *self = Self::Single(new_block)
                }
            }
        }
    }

    //TODO: Needs optimizations
    pub fn array_iter(&self) -> Box<dyn Iterator<Item = Box<[u16; SUBCHUNK_VOLUME]>> + '_> {
        match self {
            Self::Single(block) => {
                Box::new(repeat_with(|| Box::new([*block; SUBCHUNK_VOLUME])).take(SUBCHUNKS_COUNT))
            }
            Self::Multi(blocks) => {
                Box::new(blocks.iter().map(|subchunk| subchunk.clone_as_array()))
            }
        }
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.subchunks.get_block(position)
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.subchunks.set_block(position, block_id);
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
        self.subchunks
            .set_block_no_heightmap_update(position, block);
    }

    #[expect(dead_code)]
    fn calculate_heightmap(&self) -> ChunkHeightmaps {
        // figure out how LongArray is formatted
        // figure out how to find out if block is motion blocking
        todo!()
    }

    pub async fn get_blocks_to_tick(&self) -> Vec<ScheduledTick> {
        let mut blocks_to_tick = Vec::new();
        let mut block_ticks = self.block_ticks.write().await;
        for priority in TickPriority::values() {
            for tick in block_ticks.iter_mut() {
                if tick.priority == priority {
                    tick.delay -= 1;
                    if tick.delay == 0 {
                        blocks_to_tick.push(tick.clone());
                    }
                }
            }
        }
        block_ticks.retain(|tick| tick.delay > 0);
        blocks_to_tick
    }

    pub async fn get_block_state_updates(&self) -> Vec<Vec<(BlockPos, u16)>> {
        let mut block_state_updates = self.block_state_updates.lock().await;
        // Needs to group by chunk section
        let mut block_state_updates_by_chunk_section = HashMap::new();
        for (position, block_state_id) in block_state_updates.drain() {
            let chunk_section = chunk_section_from_pos(&position);
            block_state_updates_by_chunk_section
                .entry(chunk_section)
                .or_insert(Vec::new())
                .push((position, block_state_id));
        }
        block_state_updates_by_chunk_section
            .values()
            .cloned()
            .collect()
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

fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
    // % works for negative numbers as intended.
    (index.y.get_absolute() % 16) as usize * CHUNK_AREA + *index.z as usize * 16 + *index.x as usize
}
#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(pumpkin_nbt::Error),
}

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_nbt::{deserializer::from_bytes, nbt_long_array};
use pumpkin_util::math::{ceil_log2, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    iter::repeat_with,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};
use thiserror::Error;

use crate::{
    block::BlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    level::LevelFolder,
    WORLD_HEIGHT,
};

pub mod anvil;
pub mod linear;

pub const CHUNK_AREA: usize = 16 * 16;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;
pub const SUBCHUNKS_COUNT: usize = WORLD_HEIGHT / 16;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

/// File locks manager to prevent multiple threads from writing to the same file at the same time
/// but allowing multiple threads to read from the same file at the same time.
static FILE_LOCK_MANAGER: LazyLock<Arc<FileLocksManager>> = LazyLock::new(Arc::default);
pub trait ChunkReader: Sync + Send {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError>;
}

pub trait ChunkWriter: Send + Sync {
    fn write_chunk(
        &self,
        chunk: &ChunkData,
        level_folder: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<(), ChunkWritingError>;
}

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

/// A guard that allows reading from a file while preventing writing to it
/// This is used to prevent writes while a read is in progress.
/// (dont suffer for "write starvation" problem)
///
/// When the guard is dropped, the file is unlocked.
pub struct FileReadGuard<'a> {
    _guard: Ref<'a, PathBuf, ()>,
}

/// A guard that allows writing to a file while preventing reading from it
/// This is used to prevent multiple threads from writing to the same file at the same time.
/// (dont suffer for "write starvation" problem)
///
/// When the guard is dropped, the file is unlocked.
pub struct FileWriteGuard<'a> {
    _guard: RefMut<'a, PathBuf, ()>,
}

/// Central File Lock Manager for chunk files
/// This is used to prevent multiple threads from writing to the same file at the same time
#[derive(Clone, Default)]
pub struct FileLocksManager {
    locks: DashMap<PathBuf, ()>,
}

#[derive(Clone)]
pub struct ChunkData {
    /// See description in `Subchunks`
    pub subchunks: Subchunks,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
}

/// # Subchunks
/// Subchunks - its an areas in chunk, what are 16 blocks in height.
/// Current amouth is 24.
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    // block name
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    #[serde(serialize_with = "nbt_long_array")]
    motion_blocking: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    world_surface: Box<[i64]>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    data: Option<Box<[i64]>>,
    palette: Vec<PaletteEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    data_version: i32,
    #[serde(rename = "xPos")]
    x_pos: i32,
    // #[serde(rename = "yPos")]
    //y_pos: i32,
    #[serde(rename = "zPos")]
    z_pos: i32,
    status: ChunkStatus,
    #[serde(rename = "sections")]
    sections: Vec<ChunkSection>,
    heightmaps: ChunkHeightmaps,
}

impl FileLocksManager {
    pub fn get_read_guard(&self, path: &Path) -> FileReadGuard {
        if let Some(lock) = self.locks.get(path) {
            FileReadGuard { _guard: lock }
        } else {
            FileReadGuard {
                _guard: self
                    .locks
                    .entry(path.to_path_buf())
                    .or_insert(())
                    .downgrade(),
            }
        }
    }

    pub fn get_write_guard(&self, path: &Path) -> FileWriteGuard {
        FileWriteGuard {
            _guard: self.locks.entry(path.to_path_buf()).or_insert(()),
        }
    }

    pub fn remove_file_lock(path: &Path) {
        FILE_LOCK_MANAGER.locks.remove(path);
    }
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
}

// I can't use an tag because it will break ChunkNBT, but status need to have a big S, so "Status"
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ChunkStatusWrapper {
    status: ChunkStatus,
}

impl ChunkData {
    pub fn from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        if from_bytes::<ChunkStatusWrapper>(chunk_data)
            .map_err(ChunkParsingError::FailedReadStatus)?
            .status
            != ChunkStatus::Full
        {
            return Err(ChunkParsingError::ChunkNotGenerated);
        }

        let chunk_data = from_bytes::<ChunkNbt>(chunk_data)
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        if chunk_data.x_pos != position.x || chunk_data.z_pos != position.z {
            log::error!(
                "Expected chunk at {}:{}, but got {}:{}",
                position.x,
                position.z,
                chunk_data.x_pos,
                chunk_data.z_pos
            );
            // lets still continue
        }

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut subchunks = Subchunks::Single(0);
        let mut block_index = 0; // which block we're currently at

        for section in chunk_data.sections.into_iter() {
            let block_states = match section.block_states {
                Some(states) => states,
                None => continue, // TODO @lukas0008 this should instead fill all blocks with the only element of the palette
            };

            let palette = block_states
                .palette
                .iter()
                .map(|entry| match BlockState::new(&entry.name) {
                    // Block not found, Often the case when World has an newer or older version then block registry
                    None => BlockState::AIR,
                    Some(state) => state,
                })
                .collect::<Vec<_>>();

            let block_data = match block_states.data {
                None => {
                    // We skipped placing an empty subchunk.
                    // We need to increase the y coordinate of the next subchunk being placed.
                    block_index += SUBCHUNK_VOLUME;
                    continue;
                }
                Some(d) => d,
            };

            // How many bits each block has in one of the palette u64s
            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            };
            // How many blocks there are in one of the palettes u64s
            let blocks_in_palette = 64 / block_bit_size;

            let mask = (1 << block_bit_size) - 1;
            'block_loop: for block in block_data.iter() {
                for i in 0..blocks_in_palette {
                    let index = (block >> (i * block_bit_size)) & mask;
                    let block = &palette[index as usize];

                    // TODO allow indexing blocks directly so we can just use block_index and save some time?
                    // this is fine because we initialized the heightmap of `blocks`
                    // from the cached value in the world file
                    subchunks.set_block_no_heightmap_update(
                        ChunkRelativeBlockCoordinates {
                            z: ((block_index % CHUNK_AREA) / 16).into(),
                            y: Height::from_absolute((block_index / CHUNK_AREA) as u16),
                            x: (block_index % 16).into(),
                        },
                        block.get_id(),
                    );

                    block_index += 1;

                    // if `SUBCHUNK_VOLUME `is not divisible by `blocks_in_palette` the block_data
                    // can sometimes spill into other subchunks. We avoid that by aborting early
                    if (block_index % SUBCHUNK_VOLUME) == 0 {
                        break 'block_loop;
                    }
                }
            }
        }

        Ok(ChunkData {
            subchunks,
            heightmap: chunk_data.heightmaps,
            position,
        })
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

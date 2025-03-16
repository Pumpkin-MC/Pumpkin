use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::vector2::Vector2;
use serde::{Deserialize, Serialize};
use std::iter::repeat_with;
use thiserror::Error;

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

#[derive(Clone)]
pub struct ChunkData {
    /// See description in [`ChunkBlocks`]
    pub blocks: ChunkBlocks,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
    pub dirty: bool,
}

/// Represents pure block data for a chunk.
/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
/// There are currently 24 subchunks per chunk.
///
/// A chunk can be:
/// - Homogeneous: the whole chunk is filled with one block type, like air or water.
/// - Subchunks: 24 separate subchunks are stored.
#[derive(PartialEq, Debug, Clone)]
pub enum ChunkBlocks {
    Homogeneous(u16),
    Subchunks(Box<[SubchunkBlocks; SUBCHUNKS_COUNT]>),
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

impl SubchunkBlocks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Homogeneous(block) => Some(*block),
            Self::Heterogeneous(blocks) => blocks.get(convert_index(position)).copied(),
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
                    blocks[convert_index(position)] = new_block;

                    *self = Self::Heterogeneous(blocks)
                }
            }
            Self::Heterogeneous(blocks) => {
                blocks[convert_index(position)] = new_block;

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
}

impl ChunkBlocks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Homogeneous(block) => Some(*block),
            Self::Subchunks(subchunks) => subchunks
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
            Self::Homogeneous(block) => {
                if *block != new_block {
                    let mut subchunks = vec![SubchunkBlocks::Homogeneous(0); SUBCHUNKS_COUNT];

                    subchunks[(position.y.get_absolute() / 16) as usize]
                        .set_block(position, new_block);

                    *self = Self::Subchunks(subchunks.try_into().unwrap());
                }
            }
            Self::Subchunks(subchunks) => {
                subchunks[(position.y.get_absolute() / 16) as usize].set_block(position, new_block);

                if subchunks
                    .iter()
                    .all(|subchunk| *subchunk == SubchunkBlocks::Homogeneous(new_block))
                {
                    *self = Self::Homogeneous(new_block)
                }
            }
        }
    }

    //TODO: Needs optimizations
    pub fn array_iter_subchunks(
        &self,
    ) -> Box<dyn Iterator<Item = Box<[u16; SUBCHUNK_VOLUME]>> + '_> {
        match self {
            Self::Homogeneous(block) => {
                Box::new(repeat_with(|| Box::new([*block; SUBCHUNK_VOLUME])).take(SUBCHUNKS_COUNT))
            }
            Self::Subchunks(subchunks) => {
                Box::new(subchunks.iter().map(|subchunk| subchunk.clone_as_array()))
            }
        }
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.blocks.get_block(position)
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.blocks.set_block(position, block_id);
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
        self.blocks.set_block_no_heightmap_update(position, block);
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

fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
    // % works for negative numbers as intended.
    (index.y.get_absolute() % 16) as usize * CHUNK_AREA + *index.z as usize * 16 + *index.x as usize
}
#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(pumpkin_nbt::Error),
}

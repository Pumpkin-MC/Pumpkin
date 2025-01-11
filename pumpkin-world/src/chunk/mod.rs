use fastnbt::LongArray;
use pumpkin_data::chunk_status::ChunkStatus;
use pumpkin_util::math::{ceil_log2, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::repeat_with};
use thiserror::Error;

use crate::{
    block::BlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    level::LevelFolder,
    WORLD_HEIGHT,
};

pub mod anvil;

pub const CHUNK_AREA: usize = 16 * 16;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;
pub const SUBCHUNKS_COUNT: usize = WORLD_HEIGHT / 16;
pub const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

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
}

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
/// Subchunks can be block, blockstate and multi.
///
/// Block means a single block in all chunk, like
/// chunk, what filled only air or only water.
///
/// Blockstate means like a block, but its includes a state.
///
/// Multi means a normal chunk, what contains 24 subchunks.
#[derive(PartialEq, Debug)]
pub enum Subchunks {
    Block(u16),
    BlockState(BlockState),
    Multi(Box<[Subchunk; SUBCHUNKS_COUNT]>),
}

/// # Subchunk
/// Subchunk - its an area in chunk, what are 16 blocks in height
///
/// Subchunk can be block, blockstate and multi.
///
/// Block means a single block in all subchunk, like
/// subchunk, what filled only air or only water.
///
/// Blockstate means like a block, but its includes a state.
///
/// MultiBlock means a normal subchunk, what contains 4096 blocks.
#[derive(Clone, PartialEq, Debug)]
pub enum Subchunk {
    Block(u16),
    BlockState(BlockState),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    MultiBlock(Box<[u16; SUBCHUNK_VOLUME]>),

    // ----------------- WARNING! ------------------
    // | In future we need to use bitvec crate,    |
    // | or something like this.                   |
    // | We need optimizations for multiblockstate,|
    // | because its a very big memory consume.    |
    // ---------------------------------------------
    /// Ordering: yzx (y being the most significant)
    MultiBlockState(Box<[BlockState; SUBCHUNK_VOLUME]>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    // block name
    name: String,
    properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    // #[serde(with = "LongArray")]
    motion_blocking: LongArray,
    // #[serde(with = "LongArray")]
    world_surface: LongArray,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i8,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    //  #[serde(with = "LongArray")]
    data: Option<LongArray>,
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

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            // 0 packed into an i64 7 times.
            motion_blocking: LongArray::new(vec![0; 37]),
            world_surface: LongArray::new(vec![0; 37]),
        }
    }
}

impl Subchunk {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Block(block) => Some(*block),
            Self::BlockState(blockstate) => Some(blockstate.block_id),
            Self::MultiBlock(blocks) => blocks.get(convert_index(position)).copied(),
            Self::MultiBlockState(blocks) => blocks
                .get(convert_index(position))
                .map(|s| &s.block_id)
                .copied(),
        }
    }

    /// Gets the given blockstate in the chunk
    pub fn get_blockstate(&self, position: ChunkRelativeBlockCoordinates) -> Option<BlockState> {
        match &self {
            Self::Block(block) => BlockState::from_block(*block),
            Self::BlockState(blockstate) => Some(*blockstate),
            Self::MultiBlock(blocks) => blocks
                .get(convert_index(position))
                .and_then(|b| BlockState::from_block(*b)),
            Self::MultiBlockState(blockstates) => blockstates.get(convert_index(position)).copied(),
        }
    }

    /// Sets the given block in the chunk
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given blockstate in the chunk
    pub fn set_blockstate(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        blockstate: BlockState,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.set_blockstate_no_heightmap_update(position, blockstate);
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
            Self::Block(block) => {
                if *block != new_block {
                    let mut blocks = Box::new([*block; SUBCHUNK_VOLUME]);
                    blocks[convert_index(position)] = new_block;

                    *self = Self::MultiBlock(blocks)
                }
            }
            Self::BlockState(blockstate) => {
                if blockstate.block_id != new_block {
                    if let Some(new_blockstate) = BlockState::from_block(new_block) {
                        let mut blockstates = Box::new([*blockstate; SUBCHUNK_VOLUME]);
                        blockstates[convert_index(position)] = new_blockstate;

                        *self = Self::MultiBlockState(blockstates)
                    }
                }
            }
            Self::MultiBlock(blocks) => {
                blocks[convert_index(position)] = new_block;

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Block(new_block)
                }
            }
            Self::MultiBlockState(blockstates) => {
                if let Some(new_blockstate) = BlockState::from_block(new_block) {
                    blockstates[convert_index(position)] = new_blockstate;

                    if blockstates.iter().all(|b| b.block_id == new_block) {
                        *self = Self::Block(new_block)
                    } else if blockstates.iter().all(|b| *b == new_blockstate) {
                        *self = Self::BlockState(new_blockstate)
                    }
                }
            }
        }
    }

    /// Sets the given blockstate in the chunk
    /// Contrary to `set_blockstate` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_blockstate_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_blockstate: BlockState,
    ) {
        match self {
            Self::Block(block) => {
                if let Some(blockstate) = BlockState::from_block(*block) {
                    let mut blockstates = Box::new([blockstate; SUBCHUNK_VOLUME]);
                    blockstates[convert_index(position)] = new_blockstate;

                    *self = Self::MultiBlockState(blockstates)
                }
            }
            Self::BlockState(blockstate) => {
                if *blockstate != new_blockstate {
                    let mut blockstates = Box::new([*blockstate; SUBCHUNK_VOLUME]);
                    blockstates[convert_index(position)] = new_blockstate;

                    *self = Self::MultiBlockState(blockstates)
                }
            }
            Self::MultiBlock(blocks) => {
                let mut blockstates =
                    Box::new(blocks.map(|block| BlockState::from_block(block).unwrap()));
                blockstates[convert_index(position)] = new_blockstate;

                *self = Self::MultiBlockState(blockstates)
            }
            Self::MultiBlockState(blockstates) => {
                blockstates[convert_index(position)] = new_blockstate;

                if blockstates.iter().all(|b| *b == new_blockstate) {
                    *self = Self::BlockState(new_blockstate)
                }
            }
        }
    }

    pub fn clone_as_blockstates(&self) -> Box<[i32; SUBCHUNK_VOLUME]> {
        match &self {
            Self::Block(block) => Box::new([*block as i32; SUBCHUNK_VOLUME]),
            Self::BlockState(blockstate) => Box::new([blockstate.as_i32(); SUBCHUNK_VOLUME]),
            Self::MultiBlock(blocks) => Box::new(blocks.map(|b| b as i32)),
            Self::MultiBlockState(blockstates) => Box::new(blockstates.map(|s| s.as_i32())),
        }
    }
}

impl Subchunks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Block(block) => Some(*block),
            Self::BlockState(blockstate) => Some(blockstate.block_id),
            Self::Multi(subchunks) => subchunks
                .get((position.y.get_absolute() / 16) as usize)
                .and_then(|subchunk| subchunk.get_block(position)),
        }
    }

    /// Gets the given block in the chunk
    pub fn get_blockstate(&self, position: ChunkRelativeBlockCoordinates) -> Option<BlockState> {
        match &self {
            Self::Block(block) => BlockState::from_block(*block),
            Self::BlockState(blockstate) => Some(*blockstate),
            Self::Multi(subchunks) => subchunks
                .get((position.y.get_absolute() / 16) as usize)
                .and_then(|subchunk| subchunk.get_blockstate(position)),
        }
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_blockstate(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        blockstate: BlockState,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.set_blockstate_no_heightmap_update(position, blockstate)
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
            Self::Block(block) => {
                if *block != new_block {
                    let mut subchunks = vec![Subchunk::Block(*block); SUBCHUNKS_COUNT];

                    subchunks[(position.y.get_absolute() / 16) as usize]
                        .set_block(position, new_block);

                    *self = Self::Multi(subchunks.try_into().unwrap());
                }
            }
            Self::BlockState(blockstate) => {
                let mut subchunks = vec![Subchunk::BlockState(*blockstate); SUBCHUNKS_COUNT];

                subchunks[(position.y.get_absolute() / 16) as usize].set_block(position, new_block);

                *self = Self::Multi(subchunks.try_into().unwrap());
            }
            Self::Multi(subchunks) => {
                subchunks[(position.y.get_absolute() / 16) as usize].set_block(position, new_block);

                if subchunks
                    .iter()
                    .all(|subchunk| *subchunk == Subchunk::Block(new_block))
                {
                    *self = Self::Block(new_block)
                }
            }
        }
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_blockstate_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        new_blockstate: BlockState,
    ) {
        match self {
            Self::Block(block) => {
                let mut subchunks = vec![Subchunk::Block(*block); SUBCHUNKS_COUNT];

                subchunks[(position.y.get_absolute() / 16) as usize]
                    .set_block(position, new_blockstate.block_id);

                *self = Self::Multi(subchunks.try_into().unwrap());
            }
            Self::BlockState(blockstate) => {
                let mut subchunks = vec![Subchunk::BlockState(*blockstate); SUBCHUNKS_COUNT];

                subchunks[(position.y.get_absolute() / 16) as usize]
                    .set_blockstate(position, new_blockstate);

                *self = Self::Multi(subchunks.try_into().unwrap());
            }
            Self::Multi(subchunks) => {
                subchunks[(position.y.get_absolute() / 16) as usize]
                    .set_blockstate(position, new_blockstate);

                if subchunks
                    .iter()
                    .all(|subchunk| *subchunk == Subchunk::BlockState(new_blockstate))
                {
                    *self = Self::BlockState(new_blockstate)
                }
            }
        }
    }

    //TODO: Needs optimizations
    pub fn array_iter(&self) -> Box<dyn Iterator<Item = Box<[i32; SUBCHUNK_VOLUME]>> + '_> {
        match self {
            Self::Block(block) => Box::new(
                repeat_with(|| Box::new([*block as i32; SUBCHUNK_VOLUME])).take(SUBCHUNKS_COUNT),
            ),
            Self::BlockState(blockstate) => Box::new(
                repeat_with(|| Box::new([blockstate.as_i32(); SUBCHUNK_VOLUME]))
                    .take(SUBCHUNKS_COUNT),
            ),
            Self::Multi(subchunks) => Box::new(
                subchunks
                    .iter()
                    .map(|subchunk| subchunk.clone_as_blockstates()),
            ),
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
        if fastnbt::from_bytes::<ChunkStatusWrapper>(chunk_data)
            .map_err(|_| ChunkParsingError::FailedReadStatus)?
            .status
            != ChunkStatus::Full
        {
            return Err(ChunkParsingError::ChunkNotGenerated);
        }

        let chunk_data = fastnbt::from_bytes::<ChunkNbt>(chunk_data)
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
        let mut subchunks = Subchunks::Block(0);
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
    #[error("Failed reading chunk status")]
    FailedReadStatus,
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
    ErrorSerializingChunk(fastnbt::error::Error),
}

use fastnbt::LongArray;
use pumpkin_core::{math::vector2::Vector2, rle_vec::RleVec};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
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

#[derive(Error, Debug)]
pub enum ChunkReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
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
/// Subchunks can be single and multi.
///
/// Single means a single block in all chunk, like
/// chunk, what filled only air or only water.
///
/// Multi means a normal chunk, what contains 24 subchunks.
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
    Rle(RleVec<u16>),
    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    Multi(Box<[u16; SUBCHUNK_VOLUME]>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    name: String,
    _properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    //  #[serde(with = "LongArray")]
    data: Option<LongArray>,
    palette: Vec<PaletteEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    // #[serde(with = "LongArray")]
    motion_blocking: LongArray,
    // #[serde(with = "LongArray")]
    world_surface: LongArray,
}

#[derive(Deserialize, Debug)]
#[expect(dead_code)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i32,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    #[expect(dead_code)]
    data_version: usize,

    #[serde(rename = "sections")]
    sections: Vec<ChunkSection>,

    heightmaps: ChunkHeightmaps,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "Status")]
enum ChunkStatus {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:structure_starts")]
    StructureStarts,
    #[serde(rename = "minecraft:structure_references")]
    StructureReferences,
    #[serde(rename = "minecraft:biomes")]
    Biomes,
    #[serde(rename = "minecraft:noise")]
    Noise,
    #[serde(rename = "minecraft:surface")]
    Surface,
    #[serde(rename = "minecraft:carvers")]
    Carvers,
    #[serde(rename = "minecraft:features")]
    Features,
    #[serde(rename = "minecraft:initialize_light")]
    InitLight,
    #[serde(rename = "minecraft:light")]
    Light,
    #[serde(rename = "minecraft:spawn")]
    Spawn,
    #[serde(rename = "minecraft:full")]
    Full,
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
    /// Creates subchunk with rle compression
    pub fn new_rle(block_id: u16) -> Self {
        let mut rle = RleVec::new();
        rle.push_n(SUBCHUNK_VOLUME, block_id);
        Self::Rle(rle)
    }

    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        match &self {
            Self::Single(block) => Some(*block),
            Self::Rle(blocks) => blocks.get(convert_index(position)).copied(),
            Self::Multi(blocks) => blocks.get(convert_index(position)).copied(),
        }
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block_id: u16,
        compressed: bool,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id, compressed)
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
        compressed: bool,
    ) {
        match self {
            Self::Single(block) => {
                if *block != new_block {
                    if compressed {
                        let mut blocks = RleVec::new();
                        blocks.push_n(SUBCHUNK_VOLUME, *block);
                        blocks.set(convert_index(position), new_block);
                        *self = Self::Rle(blocks)
                    } else {
                        let mut blocks = Box::new([*block; SUBCHUNK_VOLUME]);
                        blocks[convert_index(position)] = new_block;
                        *self = Self::Multi(blocks)
                    }
                }
            }
            Self::Rle(blocks) => {
                blocks.set(convert_index(position), new_block);

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Single(new_block)
                } else if !compressed {
                    *self = Self::Multi(blocks.to_vec().try_into().unwrap())
                }
            }
            Self::Multi(blocks) => {
                blocks[convert_index(position)] = new_block;

                if blocks.iter().all(|b| *b == new_block) {
                    *self = Self::Single(new_block)
                } else if compressed {
                    *self = Self::Rle(RleVec::from_iter(blocks.into_iter()))
                }
            }
        }
    }

    pub fn optimize(&mut self) {
        match self {
            Self::Multi(blocks) => {
                if blocks.iter().all(|b| b == blocks.first().unwrap()) {
                    *self = Self::Single(*blocks.first().unwrap())
                } else {
                    *self = Self::Rle(RleVec::from_iter(blocks.into_iter()))
                }
            }
            Self::Rle(blocks) => {
                let mut runs = blocks.runs();
                let first_run = runs.next().unwrap();

                if runs.all(|r| r == first_run) {
                    *self = Self::Single(*first_run.value)
                }
            }
            _ => {}
        }
    }

    pub fn clone_as_array(&self) -> Box<[u16; SUBCHUNK_VOLUME]> {
        match &self {
            Self::Single(block) => Box::new([*block; SUBCHUNK_VOLUME]),
            Self::Rle(blocks) => blocks.to_vec().try_into().unwrap(),
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
    pub fn set_block(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block_id: u16,
        compressed: bool,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id, compressed)
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
        compressed: bool,
    ) {
        match self {
            Self::Single(block) => {
                if *block != new_block {
                    let mut subchunks = vec![Subchunk::Single(0); SUBCHUNKS_COUNT];

                    subchunks[(position.y.get_absolute() / 16) as usize]
                        .set_block(position, new_block, compressed);

                    *self = Self::Multi(subchunks.try_into().unwrap());
                }
            }
            Self::Multi(subchunks) => {
                subchunks[(position.y.get_absolute() / 16) as usize]
                    .set_block(position, new_block, compressed);

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
                Box::new(vec![Box::new([*block; SUBCHUNK_VOLUME]); SUBCHUNKS_COUNT].into_iter())
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
    pub fn set_block(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block_id: u16,
        compressed: bool,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.subchunks.set_block(position, block_id, compressed);
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
        compressed: bool,
    ) {
        self.subchunks
            .set_block_no_heightmap_update(position, block, compressed);
    }

    #[expect(dead_code)]
    fn calculate_heightmap(&self) -> ChunkHeightmaps {
        // figure out how LongArray is formatted
        // figure out how to find out if block is motion blocking
        todo!()
    }
}

impl ChunkData {
    pub fn from_bytes(chunk_data: &[u8], at: Vector2<i32>) -> Result<Self, ChunkParsingError> {
        if fastnbt::from_bytes::<ChunkStatus>(chunk_data)
            .map_err(|_| ChunkParsingError::FailedReadStatus)?
            != ChunkStatus::Full
        {
            return Err(ChunkParsingError::ChunkNotGenerated);
        }

        let chunk_data = fastnbt::from_bytes::<ChunkNbt>(chunk_data)
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

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
            let block_bit_size = {
                let size = 64 - (palette.len() as i64 - 1).leading_zeros();
                max(4, size)
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
                    //
                    // TODO add option to load compressed or not
                    subchunks.set_block_no_heightmap_update(
                        ChunkRelativeBlockCoordinates {
                            z: ((block_index % CHUNK_AREA) / 16).into(),
                            y: Height::from_absolute((block_index / CHUNK_AREA) as u16),
                            x: (block_index % 16).into(),
                        },
                        block.get_id(),
                        false,
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
            position: at,
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

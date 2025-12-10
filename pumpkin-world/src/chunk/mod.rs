use crate::BlockStateId;
use crate::block::entities::BlockEntity;
use crate::chunk::format::LightContainer;
use crate::tick::scheduler::ChunkTickScheduler;
use palette::{BiomePalette, BlockPalette};
use pumpkin_data::block_properties::blocks_movement;
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Block::MINECRAFT_LEAVES;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::position::BlockPos;
use serde::{Deserialize, Serialize};
use std::ops::{BitAnd, BitOr};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

pub mod format;
pub mod io;
pub mod palette;

// TODO
pub const CHUNK_WIDTH: usize = BlockPalette::SIZE;
pub const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub const BIOME_VOLUME: usize = BiomePalette::VOLUME;
pub const SUBCHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_WIDTH;

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

// Clone here cause we want to clone a snapshot of the chunk so we don't block writing for too long
pub struct ChunkData {
    pub section: ChunkSections,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub x: i32,
    pub z: i32,
    pub block_ticks: ChunkTickScheduler<&'static Block>,
    pub fluid_ticks: ChunkTickScheduler<&'static Fluid>,
    pub block_entities: HashMap<BlockPos, Arc<dyn BlockEntity>>,
    pub light_engine: ChunkLight,
    pub status: ChunkStatus,
    pub dirty: bool,
}

#[derive(Clone)]
pub struct ChunkEntityData {
    /// Chunk X
    pub x: i32,
    /// Chunk Z
    pub z: i32,
    pub data: HashMap<uuid::Uuid, NbtCompound>,

    pub dirty: bool,
}

/// Represents pure block data for a chunk.
/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
/// There are currently 24 subchunks per chunk.
///
/// A chunk can be:
/// - Subchunks: 24 separate subchunks are stored.
#[derive(Clone)]
pub struct ChunkSections {
    pub sections: Box<[SubChunk]>,
    pub min_y: i32,
}

impl ChunkSections {
    #[cfg(test)]
    pub fn dump_blocks(&self) -> Vec<u16> {
        // TODO: this is not optimal, we could use rust iters
        let mut dump = Vec::new();
        for section in self.sections.iter() {
            section.block_states.for_each(|raw_id| {
                dump.push(raw_id);
            });
        }
        dump
    }

    #[cfg(test)]
    pub fn dump_biomes(&self) -> Vec<u8> {
        // TODO: this is not optimal, we could use rust iters
        let mut dump = Vec::new();
        for section in self.sections.iter() {
            section.biomes.for_each(|raw_id| {
                dump.push(raw_id);
            });
        }
        dump
    }
}

#[derive(Default, Clone)]
pub struct SubChunk {
    pub block_states: BlockPalette,
    pub biomes: BiomePalette,
}

#[derive(Default, Clone)]
pub struct ChunkLight {
    pub sky_light: Box<[LightContainer]>,
    pub block_light: Box<[LightContainer]>,
}

#[derive(Debug, Clone, Copy)]
pub enum ChunkHeightmapType {
    WorldSurface = 0,
    MotionBlocking = 1,
    MotionBlockingNoLeaves = 2,
}
impl TryFrom<usize> for ChunkHeightmapType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ChunkHeightmapType::WorldSurface),
            1 => Ok(ChunkHeightmapType::MotionBlocking),
            2 => Ok(ChunkHeightmapType::MotionBlockingNoLeaves),
            _ => Err("Invalid usize value for ChunkHeightmapType. The value should be 0~2."),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    #[serde(serialize_with = "nbt_long_array")]
    pub world_surface: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    pub motion_blocking: Box<[i64]>,
    #[serde(serialize_with = "nbt_long_array")]
    pub motion_blocking_no_leaves: Box<[i64]>,
}

impl ChunkHeightmaps {
    pub fn set(&mut self, _type: ChunkHeightmapType, pos: BlockPos, min_y: i32) {
        let data = match _type {
            ChunkHeightmapType::WorldSurface => &mut self.world_surface,
            ChunkHeightmapType::MotionBlocking => &mut self.motion_blocking,
            ChunkHeightmapType::MotionBlockingNoLeaves => &mut self.motion_blocking_no_leaves,
        };

        let local_x = (pos.0.x & 15) as usize;
        let local_z = (pos.0.z & 15) as usize;

        let adjust_height = (pos.0.y + min_y.abs()) as usize;

        assert!(adjust_height <= 2 << 9);

        //chunk column index in 16*16 chunk.
        let column_idx = local_z * 16 + local_x;

        // Each height value uses 9 bits, calculate starting bit position
        let bit_start_idx = column_idx * 9;

        // Find where these 9 bits start within a 64-bit packed array element
        // We use bit_start_index % 63, which means the last bit of i64 won't be used,
        // but this avoids the hassle of bit concatenation.
        let packed_array_bit_start_idx = bit_start_idx as u32 % 63;

        let mask = {
            if packed_array_bit_start_idx == 0 {
                //0b0000_0000_0111_1111_...
                !(0x1FF << (64 - 9))
            } else {
                !(0x1FF << (64 - packed_array_bit_start_idx - 9))
            }
        };

        let height_bit_bytes = adjust_height
            .wrapping_shl(64 - 9 - packed_array_bit_start_idx)
            .to_ne_bytes();
        let height = i64::from_ne_bytes(height_bit_bytes);

        let packed_array_idx = column_idx / 7;

        data[packed_array_idx] = data[packed_array_idx].bitand(mask).bitor(height);
    }

    pub fn get_height(&self, _type: ChunkHeightmapType, x: i32, z: i32, min_y: i32) -> i32 {
        let data = match _type {
            ChunkHeightmapType::WorldSurface => &self.world_surface,
            ChunkHeightmapType::MotionBlocking => &self.motion_blocking,
            ChunkHeightmapType::MotionBlockingNoLeaves => &self.motion_blocking_no_leaves,
        };

        let local_x = (x & 15) as usize;
        let local_z = (z & 15) as usize;

        let column_idx = local_z * 16 + local_x;
        let bit_start_idx = column_idx * 9;

        let packed_array_bit_start_idx = bit_start_idx as u32 % 63;

        let mask = {
            if packed_array_bit_start_idx == 0 {
                //0b1111_1111_1000_0000_...
                0x1ff << (64 - 9)
            } else {
                0x1ff << (64 - packed_array_bit_start_idx - 9)
            }
        };

        let packed_array_idx = column_idx / 7;

        let height_bit_bytes_i64 = data[packed_array_idx].bitand(mask).to_ne_bytes();

        (u64::from_ne_bytes(height_bit_bytes_i64)
            .wrapping_shr(64 - (packed_array_bit_start_idx + 9)) as i32)
            - min_y.abs()
    }

    pub fn log_heightmap(&self, _type: ChunkHeightmapType, min_y: i32) {
        let mut header = "Z/X".to_string();
        for x in 0..16 {
            header.push_str(&format!("{x:4}"));
        }

        let grid: String = (0..16)
            .map(|z| {
                let mut row = format!("{z:3}");
                row.push_str(
                    &(0..16)
                        .map(|x| format!("{:4}", self.get_height(_type, x, z, min_y)))
                        .collect::<String>(),
                );
                row
            })
            .collect::<Vec<_>>()
            .join("\n");

        log::info!("\nHeightMap:\n{header}\n{grid}");
    }
}

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            // 9 bits per entry
            // 0 packed into an i64 7 times.
            motion_blocking: vec![0; 37].into_boxed_slice(),
            motion_blocking_no_leaves: vec![0; 37].into_boxed_slice(),
            world_surface: vec![0; 37].into_boxed_slice(),
        }
    }
}

impl ChunkSections {
    pub fn new(sections: Box<[SubChunk]>, min_y: i32) -> Self {
        Self { sections, min_y }
    }

    pub fn get_block_absolute_y(
        &self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
    ) -> Option<BlockStateId> {
        let y = y - self.min_y;
        if y < 0 {
            None
        } else {
            let relative_y = y as usize;
            self.get_relative_block(relative_x, relative_y, relative_z)
        }
    }

    pub fn get_rough_biome_absolute_y(
        &self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
    ) -> Option<u8> {
        let y = y - self.min_y;
        if y < 0 {
            None
        } else {
            let relative_y = y as usize;
            self.get_noise_biome(
                relative_y / BlockPalette::SIZE,
                relative_x >> 2 & 3,
                relative_y >> 2 & 3,
                relative_z >> 2 & 3,
            )
        }
    }

    /// Returns the replaced block state ID
    pub fn set_block_absolute_y(
        &mut self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let y = y - self.min_y;
        debug_assert!(y >= 0);
        let relative_y = y as usize;

        self.set_relative_block(relative_x, relative_y, relative_z, block_state_id)
    }

    /// Gets the given block in the chunk
    fn get_relative_block(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
    ) -> Option<BlockStateId> {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;
        self.sections
            .get(section_index)
            .map(|section| section.block_states.get(relative_x, relative_y, relative_z))
    }

    /// Sets the given block in the chunk, returning the old block state ID
    #[inline]
    pub fn set_relative_block(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(relative_x, relative_y, relative_z, block_state_id)
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
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;
        if let Some(section) = self.sections.get_mut(section_index) {
            return section
                .block_states
                .set(relative_x, relative_y, relative_z, block_state_id);
        }
        0
    }

    pub fn set_relative_biome(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        biome_id: u8,
    ) {
        debug_assert!(relative_x < BiomePalette::SIZE);
        debug_assert!(relative_z < BiomePalette::SIZE);

        let section_index = relative_y / BiomePalette::SIZE;
        let relative_y = relative_y % BiomePalette::SIZE;
        if let Some(section) = self.sections.get_mut(section_index) {
            section
                .biomes
                .set(relative_x, relative_y, relative_z, biome_id);
        }
    }

    pub fn get_noise_biome(
        &self,
        index: usize,
        scale_x: usize,
        scale_y: usize,
        scale_z: usize,
    ) -> Option<u8> {
        debug_assert!(scale_x < BiomePalette::SIZE);
        debug_assert!(scale_z < BiomePalette::SIZE);
        self.sections
            .get(index)
            .map(|section| section.biomes.get(scale_x, scale_y, scale_z))
    }

    pub fn get_top_y(&self, relative_x: usize, relative_z: usize, first_y: i32) -> Option<i32> {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let mut y = first_y;
        while y >= self.min_y {
            if let Some(block_state_id) = self.get_block_absolute_y(relative_x, y, relative_z)
                && !BlockState::from_id(block_state_id).is_air()
            {
                return Some(y);
            }
            y -= 1;
        }
        None
    }
}

impl ChunkData {
    /// Gets the given block in the chunk
    #[inline]
    pub fn get_relative_block(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
    ) -> Option<BlockStateId> {
        self.section
            .get_relative_block(relative_x, relative_y, relative_z)
    }

    /// Sets the given block in the chunk
    #[inline]
    pub fn set_relative_block(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) {
        // TODO @LUK_ESC? update the heightmap
        self.section
            .set_relative_block(relative_x, relative_y, relative_z, block_state_id);
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
        block_state_id: BlockStateId,
    ) {
        self.section
            .set_relative_block(relative_x, relative_y, relative_z, block_state_id);
    }

    //TODO: Tracking heightmaps update.
    pub fn calculate_heightmap(&mut self) -> ChunkHeightmaps {
        let highest_non_empty_subchunk = self.get_highest_non_empty_subchunk();
        let mut heightmaps = ChunkHeightmaps::default();

        for x in 0..16 {
            for z in 0..16 {
                self.populate_heightmaps(&mut heightmaps, highest_non_empty_subchunk, x, z);
            }
        }

        // log::info!("WorldSurface:");
        // heightmaps.log_heightmap(ChunkHeightmapType::WorldSurface, self.section.min_y);
        // log::info!("MotionBlocking:");
        // heightmaps.log_heightmap(ChunkHeightmapType::MotionBlocking, self.section.min_y);
        // log::info!("min_y: {}", self.section.min_y);
        heightmaps
    }

    #[inline]
    fn populate_heightmaps(
        &self,
        heightmaps: &mut ChunkHeightmaps,
        start_sub_chunk: usize,
        x: usize,
        z: usize,
    ) {
        let start_height = (start_sub_chunk as i32) * 16 - self.section.min_y.abs() + 15;
        let mut has_found = [false, false, false];

        for y in (self.section.min_y..=start_height).rev() {
            let pos = BlockPos::new(x as i32, y, z as i32);
            let state_id = self.section.get_block_absolute_y(x, y, z).unwrap();
            let block_state = BlockState::from_id(state_id);
            let block = Block::from_state_id(state_id);

            if !block_state.is_air() && !has_found[ChunkHeightmapType::WorldSurface as usize] {
                heightmaps.set(ChunkHeightmapType::WorldSurface, pos, self.section.min_y);
                has_found[ChunkHeightmapType::WorldSurface as usize] = true;
            }

            let is_motion_blocking = blocks_movement(block_state, block)
                || Fluid::from_registry_key(block.registry_key())
                    .is_some_and(|fluid| !fluid.states.is_empty());

            if !has_found[ChunkHeightmapType::MotionBlocking as usize] && is_motion_blocking {
                heightmaps.set(ChunkHeightmapType::MotionBlocking, pos, self.section.min_y);
                has_found[ChunkHeightmapType::MotionBlocking as usize] = true;
            }

            if !has_found[ChunkHeightmapType::MotionBlockingNoLeaves as usize]
                && is_motion_blocking
                && !block.has_tag(&MINECRAFT_LEAVES)
            {
                heightmaps.set(
                    ChunkHeightmapType::MotionBlockingNoLeaves,
                    pos,
                    self.section.min_y,
                );
                has_found[ChunkHeightmapType::MotionBlockingNoLeaves as usize] = true;
            }

            if !has_found.contains(&false) {
                return;
            }
        }

        let pos = BlockPos::new(x as i32, self.section.min_y, z as i32);
        for (idx, is_set) in has_found.iter().enumerate() {
            if !(*is_set) {
                heightmaps.set(idx.try_into().unwrap(), pos, self.section.min_y);
            }
        }
    }

    pub fn get_highest_non_empty_subchunk(&self) -> usize {
        for (i, sub_chunk) in self.section.sections.iter().enumerate().rev() {
            if sub_chunk.block_states.non_air_block_count() != 0 {
                return i;
            }
        }
        0
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

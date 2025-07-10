use crate::block::entities::BlockEntity;
use palette::{BiomePalette, BlockPalette};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_long_array;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

use crate::BlockStateId;
use crate::chunk::format::LightContainer;

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
            _ => panic!("Invalid tick priority: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduledTick {
    pub block_pos: BlockPos,
    pub delay: u16,
    pub priority: TickPriority,
    pub target_block_id: u16,
}

// Clone here cause we want to clone a snapshot of the chunk so we don't block writing for too long
pub struct ChunkData {
    pub section: ChunkSections,
    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
    pub block_ticks: Vec<ScheduledTick>,
    pub fluid_ticks: Vec<ScheduledTick>,
    pub block_entities: HashMap<BlockPos, Arc<dyn BlockEntity>>,
    pub light_engine: ChunkLight,

    pub dirty: bool,
}

impl ChunkData {
    pub fn get_and_tick_block_ticks(&mut self) -> VecDeque<ScheduledTick> {
        let mut ticks = VecDeque::new();
        let mut remaining_ticks = Vec::new();
        for mut tick in self.block_ticks.drain(..) {
            tick.delay = tick.delay.saturating_sub(1);
            if tick.delay == 0 {
                ticks.push_back(tick);
            } else {
                remaining_ticks.push(tick);
            }
        }

        self.block_ticks = remaining_ticks;
        ticks
    }

    pub fn get_and_tick_fluid_ticks(&mut self) -> Vec<ScheduledTick> {
        let mut ticks = Vec::new();
        self.fluid_ticks.retain_mut(|tick| {
            tick.delay = tick.delay.saturating_sub(1);
            if tick.delay == 0 {
                ticks.push(*tick);
                false
            } else {
                true
            }
        });
        ticks
    }

    pub fn is_block_tick_scheduled(&self, block_pos: &BlockPos, block_id: u16) -> bool {
        self.block_ticks
            .iter()
            .any(|tick| tick.block_pos == *block_pos && tick.target_block_id == block_id)
    }

    pub fn is_fluid_tick_scheduled(&self, block_pos: &BlockPos) -> bool {
        self.fluid_ticks
            .iter()
            .any(|tick| tick.block_pos == *block_pos)
    }

    pub fn schedule_block_tick(
        &mut self,
        block_id: u16,
        block_pos: BlockPos,
        delay: u16,
        priority: TickPriority,
    ) {
        self.block_ticks.push(ScheduledTick {
            block_pos,
            delay,
            priority,
            target_block_id: block_id,
        });
    }

    pub fn schedule_fluid_tick(&mut self, block_id: u16, block_pos: &BlockPos, delay: u16) {
        if self
            .fluid_ticks
            .iter()
            .any(|tick| tick.block_pos == *block_pos && tick.target_block_id == block_id)
        {
            // If a fluid tick is already scheduled for this block, we don't need to schedule it again
            return;
        }
        self.fluid_ticks.push(ScheduledTick {
            block_pos: *block_pos,
            delay,
            priority: TickPriority::Normal,
            target_block_id: block_id,
        });
    }
}

#[derive(Clone)]
pub struct ChunkEntityData {
    pub chunk_position: Vector2<i32>,
    pub data: HashMap<uuid::Uuid, NbtCompound>,

    pub dirty: bool,
}

/// Represents pure block data for a chunk.
/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
/// There are currently 24 subchunks per chunk.
///
/// A chunk can be:
/// - Subchunks: 24 separate subchunks are stored.
#[derive(Debug, Clone)]
pub struct ChunkSections {
    pub sections: Box<[SubChunk]>,
    min_y: i32,
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

#[derive(Debug, Default, Clone)]
pub struct SubChunk {
    pub block_states: BlockPalette,
    pub biomes: BiomePalette,
}

#[derive(Debug, Default, Clone)]
pub struct ChunkLight {
    pub sky_light: Box<[LightContainer]>,
    pub block_light: Box<[LightContainer]>,
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

    pub fn set_block_absolute_y(
        &mut self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
        block_state: BlockStateId,
    ) {
        let y = y - self.min_y;
        debug_assert!(y >= 0);
        let relative_y = y as usize;

        self.set_relative_block(relative_x, relative_y, relative_z, block_state);
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

    /// Sets the given block in the chunk, returning the old block
    #[inline]
    pub fn set_relative_block(
        &mut self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: BlockStateId,
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
        block_state_id: BlockStateId,
    ) {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;
        if let Some(section) = self.sections.get_mut(section_index) {
            section
                .block_states
                .set(relative_x, relative_y, relative_z, block_state_id);
        }
    }

    /// Sets the given block in the chunk, returning the old block
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
        self.sections[section_index]
            .biomes
            .set(relative_x, relative_y, relative_z, biome_id);
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

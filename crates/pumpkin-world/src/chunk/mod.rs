use crate::chunk::format::LightContainer;
use crate::tick::scheduler::ChunkTickScheduler;
use palette::{BiomePalette, BlockPalette, has_random_ticking_fluid};
use pumpkin_data::block_properties::{blocks_movement, has_random_ticks, is_air};
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Block::MINECRAFT_LEAVES;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashMap;

use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use thiserror::Error;
use tokio::sync::Mutex;

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
    IoError(std::io::Error),
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
    IoError(std::io::Error),
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
    pub heightmap: std::sync::Mutex<ChunkHeightmaps>,
    pub x: i32,
    pub z: i32,
    pub block_ticks: ChunkTickScheduler<&'static Block>,
    pub fluid_ticks: ChunkTickScheduler<&'static Fluid>,
    pub pending_block_entities: std::sync::Mutex<FxHashMap<BlockPos, NbtCompound>>,
    pub light_engine: std::sync::Mutex<ChunkLight>,
    pub light_populated: AtomicBool,
    pub status: ChunkStatus,
    pub blending_data: Option<crate::generation::blender::blending_data::BlendingData>,
    pub unknown_nbt: NbtCompound,
    pub dirty: AtomicBool,
    pub inhabited_time: AtomicU64,
}

pub struct ChunkEntityData {
    /// Chunk X
    pub x: i32,
    /// Chunk Z
    pub z: i32,
    pub data: Mutex<Vec<NbtCompound>>,

    pub dirty: AtomicBool,
}

/// Represents pure block data for a chunk.
/// Subchunks are vertical portions of a chunk. They are 16 blocks tall.
/// There are currently 24 subchunks per chunk.
///
/// A chunk can be:
/// - Subchunks: 24 separate subchunks are stored.
pub struct ChunkSections {
    pub block_sections: RwLock<Box<[BlockPalette]>>,
    pub random_tick_sections: RwLock<Option<Box<[RandomTickSectionCache]>>>,
    pub randomly_ticking_mask: std::sync::atomic::AtomicU32,
    pub biome_sections: RwLock<Box<[BiomePalette]>>,
    /// Section-level data that Pumpkin does not currently model. This includes
    /// future fields nested in the block-state and biome compounds.
    pub unknown_nbt: RwLock<Box<[NbtCompound]>>,
    pub min_y: i32,
}

#[derive(Default, Clone, Copy)]
pub struct RandomTickSectionCache {
    pub random_ticking_block_count: u16,
    pub random_ticking_fluid_count: u16,
}

impl RandomTickSectionCache {
    #[must_use]
    pub const fn is_randomly_ticking(&self) -> bool {
        self.random_ticking_block_count > 0 || self.random_ticking_fluid_count > 0
    }
}

impl ChunkSections {
    /// Number of 16-block sections this chunk currently holds.
    ///
    /// Derived from the live section array rather than stored, so it cannot go stale after
    /// [`ChunkData::pad_sections_to`] grows the chunk.
    #[must_use]
    pub fn section_count(&self) -> usize {
        self.block_sections.read().unwrap().len()
    }

    #[cfg(test)]
    #[must_use]
    pub fn dump_blocks(&self) -> Vec<BlockStateId> {
        self.block_sections
            .read()
            .unwrap()
            .iter()
            .flat_map(|section| section.iter())
            .collect()
    }

    #[cfg(test)]
    #[must_use]
    pub fn dump_biomes(&self) -> Vec<u8> {
        self.biome_sections
            .read()
            .unwrap()
            .iter()
            .flat_map(|section| section.iter())
            .collect()
    }
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
    OceanFloor = 3,
}
impl TryFrom<usize> for ChunkHeightmapType {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::WorldSurface),
            1 => Ok(Self::MotionBlocking),
            2 => Ok(Self::MotionBlockingNoLeaves),
            3 => Ok(Self::OceanFloor),
            _ => Err("Invalid usize value for ChunkHeightmapType. The value should be 0~3."),
        }
    }
}

impl ChunkHeightmapType {
    /// All variants, in the order used for per-column iteration during
    /// heightmap computation/updates.
    pub const ALL: [Self; 4] = [
        Self::WorldSurface,
        Self::MotionBlocking,
        Self::MotionBlockingNoLeaves,
        Self::OceanFloor,
    ];

    #[must_use]
    pub fn is_opaque(&self, block_state: &BlockState) -> bool {
        let block = block_state.id.to_block_id();
        match self {
            Self::WorldSurface => !block_state.is_air(),
            Self::MotionBlocking => blocks_movement(block_state, block) || block_state.is_liquid(),
            Self::MotionBlockingNoLeaves => {
                (blocks_movement(block_state, block) || block_state.is_liquid())
                    && !block.has_tag(MINECRAFT_LEAVES)
            }
            // Vanilla `Heightmap.Types.OCEAN_FLOOR` uses `MATERIAL_MOTION_BLOCKING`
            // (`BlockStateBase::blocksMotion`), unlike `MOTION_BLOCKING` it does NOT
            // additionally count fluids as blocking.
            Self::OceanFloor => blocks_movement(block_state, block),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkHeightmaps {
    pub world_surface: Option<Box<[i64]>>,
    pub motion_blocking: Option<Box<[i64]>>,
    pub motion_blocking_no_leaves: Option<Box<[i64]>>,
    pub ocean_floor: Option<Box<[i64]>>,
}

impl ChunkHeightmaps {
    pub fn set(&mut self, heightmap: ChunkHeightmapType, x: i32, z: i32, height: i32, min_y: i32) {
        let data = match heightmap {
            ChunkHeightmapType::WorldSurface => &mut self.world_surface,
            ChunkHeightmapType::MotionBlocking => &mut self.motion_blocking,
            ChunkHeightmapType::MotionBlockingNoLeaves => &mut self.motion_blocking_no_leaves,
            ChunkHeightmapType::OceanFloor => &mut self.ocean_floor,
        };

        let data = data.get_or_insert_with(|| vec![0; 37].into_boxed_slice());

        let local_x = (x & 15) as usize;
        let local_z = (z & 15) as usize;
        let column_idx = local_z * 16 + local_x;

        // In Minecraft 1.16+, height is stored as (y - min_y + 1). 0 means below min_y.
        // It uses 9 bits per value, packed such that they do not cross u64 boundaries.
        // 64 / 9 = 7 values per u64.
        let val = (height - min_y + 1).max(0) as u64;

        let array_idx = column_idx / 7;
        let shift = (column_idx % 7) * 9;

        let mask = 0x1FFu64 << shift;

        let mut current = data[array_idx] as u64;
        current = (current & !mask) | ((val & 0x1FF) << shift);
        data[array_idx] = current as i64;
    }

    #[must_use]
    pub fn get(&self, heightmap: ChunkHeightmapType, x: i32, z: i32, min_y: i32) -> i32 {
        let data = match heightmap {
            ChunkHeightmapType::WorldSurface => &self.world_surface,
            ChunkHeightmapType::MotionBlocking => &self.motion_blocking,
            ChunkHeightmapType::MotionBlockingNoLeaves => &self.motion_blocking_no_leaves,
            ChunkHeightmapType::OceanFloor => &self.ocean_floor,
        };

        let Some(data) = data else {
            return min_y - 1;
        };

        let local_x = (x & 15) as usize;
        let local_z = (z & 15) as usize;
        let column_idx = local_z * 16 + local_x;

        let array_idx = column_idx / 7;
        let shift = (column_idx % 7) * 9;

        let current = data[array_idx] as u64;
        let val = (current >> shift) & 0x1FF;

        (val as i32) + min_y - 1
    }

    #[expect(clippy::too_many_arguments)]
    pub fn update<F>(
        &mut self,
        heightmap_type: ChunkHeightmapType,
        local_x: i32,
        local_y: i32,
        local_z: i32,
        block_state: &BlockState,
        min_y: i32,
        get_block: F,
    ) -> bool
    where
        F: Fn(i32) -> &'static BlockState,
    {
        let first_available = self.get(heightmap_type, local_x, local_z, min_y) + 1;
        if local_y <= first_available - 2 {
            return false;
        }

        if heightmap_type.is_opaque(block_state) {
            if local_y >= first_available {
                self.set(heightmap_type, local_x, local_z, local_y, min_y);
                return true;
            }
        } else if first_available - 1 == local_y {
            for y in (min_y..local_y).rev() {
                let state = get_block(y);
                if heightmap_type.is_opaque(state) {
                    self.set(heightmap_type, local_x, local_z, y, min_y);
                    return true;
                }
            }
            self.set(heightmap_type, local_x, local_z, min_y - 1, min_y);
            return true;
        }

        false
    }
}

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            motion_blocking: None,
            motion_blocking_no_leaves: None,
            world_surface: None,
            ocean_floor: None,
        }
    }
}

impl ChunkSections {
    #[must_use]
    pub fn build_random_tick_sections_cache(
        block_sections: &[BlockPalette],
    ) -> (Option<Box<[RandomTickSectionCache]>>, u32) {
        let mut mask = 0;
        let mut has_ticks = false;
        let cache = block_sections
            .iter()
            .enumerate()
            .map(|(i, section)| {
                let (random_ticking_block_count, random_ticking_fluid_count) =
                    section.random_ticking_counts();
                if random_ticking_block_count > 0 || random_ticking_fluid_count > 0 {
                    mask |= 1 << i;
                    has_ticks = true;
                }
                RandomTickSectionCache {
                    random_ticking_block_count,
                    random_ticking_fluid_count,
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        if has_ticks {
            (Some(cache), mask)
        } else {
            (None, 0)
        }
    }

    #[must_use]
    pub fn new(num_sections: usize, min_y: i32) -> Self {
        let block_sections = vec![BlockPalette::default(); num_sections].into_boxed_slice();
        let (random_tick_sections, randomly_ticking_mask) =
            Self::build_random_tick_sections_cache(&block_sections);
        let biome_sections = vec![BiomePalette::default(); num_sections].into_boxed_slice();
        let unknown_nbt = vec![NbtCompound::new(); num_sections].into_boxed_slice();

        Self {
            block_sections: RwLock::new(block_sections),
            random_tick_sections: RwLock::new(random_tick_sections),
            randomly_ticking_mask: std::sync::atomic::AtomicU32::new(randomly_ticking_mask),
            biome_sections: RwLock::new(biome_sections),
            unknown_nbt: RwLock::new(unknown_nbt),
            min_y,
        }
    }

    #[must_use]
    pub(crate) fn from_palettes(
        block_sections: Box<[BlockPalette]>,
        biome_sections: Box<[BiomePalette]>,
        min_y: i32,
    ) -> Self {
        assert_eq!(
            block_sections.len(),
            biome_sections.len(),
            "block and biome section counts must match"
        );
        let (random_tick_sections, randomly_ticking_mask) =
            Self::build_random_tick_sections_cache(&block_sections);
        let unknown_nbt = vec![NbtCompound::new(); block_sections.len()].into_boxed_slice();

        Self {
            block_sections: RwLock::new(block_sections),
            random_tick_sections: RwLock::new(random_tick_sections),
            randomly_ticking_mask: std::sync::atomic::AtomicU32::new(randomly_ticking_mask),
            biome_sections: RwLock::new(biome_sections),
            unknown_nbt: RwLock::new(unknown_nbt),
            min_y,
        }
    }

    #[must_use]
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
        &self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let y = y - self.min_y;
        if y < 0 {
            return Block::AIR.default_state.id;
        }
        let relative_y = y as usize;
        self.set_block_no_heightmap_update(relative_x, relative_y, relative_z, block_state_id)
    }

    #[must_use]
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
        self.block_sections
            .read()
            .unwrap()
            .get(section_index)
            .map(|section| section.get(relative_x, relative_y, relative_z))
    }

    /// Sets the given block in the chunk, returning the old block state ID
    #[inline]
    pub fn set_relative_block(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        self.set_block_no_heightmap_update(relative_x, relative_y, relative_z, block_state_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let section_index = relative_y / BlockPalette::SIZE;
        let relative_y = relative_y % BlockPalette::SIZE;

        // Keep lock order consistent to avoid deadlocks: block sections first, then random-tick cache.
        let mut sections = self.block_sections.write().unwrap();
        let mut random_tick_sections_guard = self.random_tick_sections.write().unwrap();

        if let Some(section) = sections.get_mut(section_index) {
            let replaced_block_state_id =
                section.set(relative_x, relative_y, relative_z, block_state_id);
            if replaced_block_state_id == block_state_id {
                return replaced_block_state_id;
            }

            if (has_random_ticks(block_state_id) || has_random_ticking_fluid(block_state_id))
                && random_tick_sections_guard.is_none()
            {
                // `sections` is already held here; use its length rather than
                // `section_count()`, which would re-lock the same RwLock and deadlock.
                let new_cache =
                    vec![RandomTickSectionCache::default(); sections.len()].into_boxed_slice();
                *random_tick_sections_guard = Some(new_cache);
            }

            if let Some(random_tick_sections) = random_tick_sections_guard.as_mut() {
                let random_tick_cache = &mut random_tick_sections[section_index];
                if has_random_ticks(replaced_block_state_id) {
                    random_tick_cache.random_ticking_block_count = random_tick_cache
                        .random_ticking_block_count
                        .saturating_sub(1);
                }
                if has_random_ticking_fluid(replaced_block_state_id) {
                    random_tick_cache.random_ticking_fluid_count = random_tick_cache
                        .random_ticking_fluid_count
                        .saturating_sub(1);
                }

                if has_random_ticks(block_state_id) {
                    random_tick_cache.random_ticking_block_count = random_tick_cache
                        .random_ticking_block_count
                        .saturating_add(1);
                }
                if has_random_ticking_fluid(block_state_id) {
                    random_tick_cache.random_ticking_fluid_count = random_tick_cache
                        .random_ticking_fluid_count
                        .saturating_add(1);
                }

                // Update the bitmask
                let mut mask = self
                    .randomly_ticking_mask
                    .load(std::sync::atomic::Ordering::Relaxed);
                if random_tick_cache.is_randomly_ticking() {
                    mask |= 1 << section_index;
                } else {
                    mask &= !(1 << section_index);
                }
                self.randomly_ticking_mask
                    .store(mask, std::sync::atomic::Ordering::Relaxed);
            }

            return replaced_block_state_id;
        }
        BlockStateId::AIR
    }

    pub fn set_relative_biome(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        biome_id: u8,
    ) {
        debug_assert!(relative_x < BiomePalette::SIZE);
        debug_assert!(relative_z < BiomePalette::SIZE);

        let section_index = relative_y / BiomePalette::SIZE;
        let relative_y = relative_y % BiomePalette::SIZE;
        if let Some(section) = self.biome_sections.write().unwrap().get_mut(section_index) {
            section.set(relative_x, relative_y, relative_z, biome_id);
        }
    }

    #[must_use]
    pub fn get_noise_biome(
        &self,
        index: usize,
        scale_x: usize,
        scale_y: usize,
        scale_z: usize,
    ) -> Option<u8> {
        debug_assert!(scale_x < BiomePalette::SIZE);
        debug_assert!(scale_z < BiomePalette::SIZE);
        self.biome_sections
            .read()
            .unwrap()
            .get(index)
            .map(|section| section.get(scale_x, scale_y, scale_z))
    }

    #[must_use]
    pub fn get_top_y(&self, relative_x: usize, relative_z: usize, first_y: i32) -> Option<i32> {
        debug_assert!(relative_x < BlockPalette::SIZE);
        debug_assert!(relative_z < BlockPalette::SIZE);

        let mut y = first_y;
        while y >= self.min_y {
            if let Some(block_state_id) = self.get_block_absolute_y(relative_x, y, relative_z)
                && !is_air(block_state_id)
            {
                return Some(y);
            }
            y -= 1;
        }
        None
    }
}

impl ChunkData {
    /// Grow this chunk to `total_sections` 16-block sections, padding with air / empty light.
    ///
    /// The chunk-data packet carries no explicit section count: the client reads exactly
    /// `dimension.height / 16` sections, taken from the dimension registry it received at
    /// login. A chunk holding fewer sections than that serializes short, so the client runs
    /// past the end of the buffer and drops the connection with a protocol error.
    ///
    /// Chunks can end up short from two directions: worldgen sizing sections off the noise
    /// generator's `shape.height` (128 for Nether/End) instead of the dimension height (256),
    /// and loading a saved chunk whose file simply does not contain the upper sections - the
    /// on-disk reader derives its section count from the highest `Y` tag present, so any file
    /// written while the first bug was live reloads just as short as it was saved. Padding
    /// here fixes both, and is a no-op when the chunk is already the right size.
    ///
    /// Padding sections are air. Their sky light is `sky_light` - which must be 15 in a
    /// dimension that has sky light, and 0 in one that does not.
    ///
    /// Getting that wrong is not cosmetic. Padded sections sit above every section the chunk
    /// actually stored, and the on-disk reader documents the same invariant: sections above
    /// the highest one carrying a `SkyLight` tag see open sky and are 15. Padding them at 0
    /// instead caps the column with a lid of artificial darkness, and sky light stops
    /// propagating down - which shows up as black caves that snap to full brightness the
    /// moment a block update forces a relight.
    pub fn pad_sections_to(&self, total_sections: usize, sky_light: u8) {
        let mut block_sections = self.section.block_sections.write().unwrap();
        if block_sections.len() >= total_sections {
            return;
        }

        let mut biome_sections = self.section.biome_sections.write().unwrap();
        let mut unknown_nbt = self.section.unknown_nbt.write().unwrap();

        let mut blocks = std::mem::take(&mut *block_sections).into_vec();
        let mut biomes = std::mem::take(&mut *biome_sections).into_vec();
        let mut unknown = std::mem::take(&mut *unknown_nbt).into_vec();

        blocks.resize_with(total_sections, BlockPalette::default);
        // Biomes have no meaning above the generated range since nothing samples them there;
        // repeat the topmost real section rather than inventing a value.
        let pad_biome = biomes.last().cloned().unwrap_or_default();
        biomes.resize(total_sections, pad_biome);
        unknown.resize_with(total_sections, NbtCompound::new);

        *block_sections = blocks.into_boxed_slice();
        *biome_sections = biomes.into_boxed_slice();
        *unknown_nbt = unknown.into_boxed_slice();
        drop(block_sections);
        drop(biome_sections);
        drop(unknown_nbt);

        // The light arrays derive their own section count independently during serialization,
        // so they must grow in lockstep or the light block desyncs from the block block.
        let mut light = self.light_engine.lock().unwrap();
        if light.sky_light.len() < total_sections {
            let mut sky = std::mem::take(&mut light.sky_light).into_vec();
            let mut block = std::mem::take(&mut light.block_light).into_vec();
            sky.resize_with(total_sections, || LightContainer::new_empty(sky_light));
            // Block light genuinely is 0 up there: nothing emits light in never-generated space.
            block.resize_with(total_sections, || LightContainer::new_empty(0));
            light.sky_light = sky.into_boxed_slice();
            light.block_light = block.into_boxed_slice();
        }
    }

    /// Returns the replaced block state ID
    pub fn set_block_absolute_y(
        &self,
        relative_x: usize,
        y: i32,
        relative_z: usize,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let min_y = self.section.min_y;
        let y_rel = y - min_y;
        if y_rel < 0 {
            return Block::AIR.default_state.id;
        }
        let relative_y = y_rel as usize;

        let old = self.section.set_block_no_heightmap_update(
            relative_x,
            relative_y,
            relative_z,
            block_state_id,
        );
        if old != block_state_id {
            let state = BlockState::from_id(block_state_id);
            self.update_heightmap(relative_x, relative_y, relative_z, state);
        }
        old
    }

    fn update_heightmap(
        &self,
        relative_x: usize,
        relative_y: usize,
        relative_z: usize,
        block_state: &BlockState,
    ) {
        let mut heightmap = self.heightmap.lock().unwrap();
        let min_y = self.section.min_y;
        let x = relative_x as i32;
        let y = relative_y as i32 + min_y;
        let z = relative_z as i32;

        for hm_type in ChunkHeightmapType::ALL {
            heightmap.update(hm_type, x, z, y, block_state, min_y, |y_at| {
                let id = self
                    .section
                    .get_block_absolute_y(relative_x, y_at, relative_z)
                    .unwrap_or(BlockStateId::AIR);
                BlockState::from_id(id)
            });
        }
    }

    /// Gets the given block in the chunk
    #[inline]
    #[must_use]
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
        let state = BlockState::from_id(block_state_id);
        self.update_heightmap(relative_x, relative_y, relative_z, state);
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
    pub fn calculate_heightmap(&self) -> ChunkHeightmaps {
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
        let mut has_found = [false; ChunkHeightmapType::ALL.len()];

        for y in (self.section.min_y..=start_height).rev() {
            let state_id = self.section.get_block_absolute_y(x, y, z).unwrap();
            let block_state = BlockState::from_id(state_id);

            for hm_type in ChunkHeightmapType::ALL {
                let idx = hm_type as usize;
                if !has_found[idx] && hm_type.is_opaque(block_state) {
                    heightmaps.set(hm_type, x as i32, z as i32, y, self.section.min_y);
                    has_found[idx] = true;
                }
            }

            if has_found.iter().all(|&found| found) {
                return;
            }
        }

        for (idx, is_set) in has_found.iter().enumerate() {
            if !(*is_set) {
                heightmaps.set(
                    idx.try_into().unwrap(),
                    x as i32,
                    z as i32,
                    self.section.min_y - 1,
                    self.section.min_y,
                );
            }
        }
    }

    /// Recompute any heightmap that was absent from this chunk's on-disk NBT.
    ///
    /// `ChunkHeightmaps::get` answers a missing heightmap with `min_y - 1`, the
    /// same sentinel it returns for a column that genuinely contains no opaque
    /// block. That conflation is only safe if "absent" never reaches a reader,
    /// so absence is resolved here, at the load boundary, where it is still
    /// distinguishable from emptiness. After this runs, `min_y - 1` means
    /// exactly one thing: the column really is empty.
    ///
    /// Vanilla worlds hit this constantly: only the heightmap types listed for
    /// a chunk's status are serialized, so chunks saved below `minecraft:full`
    /// carry `WORLD_SURFACE_WG`/`OCEAN_FLOOR_WG` (or no `Heightmaps` entries at
    /// all) rather than `WORLD_SURFACE`, and some of those already hold terrain.
    ///
    /// A recomputed heightmap can still be `None` when the chunk holds no
    /// opaque block anywhere; that is the empty-column case and is correct.
    pub fn prime_missing_heightmaps(&self) {
        let missing = {
            let heightmap = self
                .heightmap
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            heightmap.world_surface.is_none()
                || heightmap.motion_blocking.is_none()
                || heightmap.motion_blocking_no_leaves.is_none()
                || heightmap.ocean_floor.is_none()
        };
        if !missing {
            return;
        }

        // Only the absent maps are replaced; a heightmap that really was written
        // stays exactly as written.
        let computed = self.calculate_heightmap();
        let mut heightmap = self
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if heightmap.world_surface.is_none() {
            heightmap.world_surface = computed.world_surface;
        }
        if heightmap.motion_blocking.is_none() {
            heightmap.motion_blocking = computed.motion_blocking;
        }
        if heightmap.motion_blocking_no_leaves.is_none() {
            heightmap.motion_blocking_no_leaves = computed.motion_blocking_no_leaves;
        }
        if heightmap.ocean_floor.is_none() {
            heightmap.ocean_floor = computed.ocean_floor;
        }
    }

    #[must_use]
    pub fn get_highest_non_empty_subchunk(&self) -> usize {
        self.section
            .block_sections
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .rev()
            .find(|(_, sub)| !sub.has_only_air())
            .map_or(0, |(idx, _)| idx)
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

#[cfg(test)]
mod tests {
    use super::{ChunkData, ChunkLight, ChunkSections, LightContainer};
    use crate::chunk::palette::BlockPalette;
    use crate::tick::scheduler::ChunkTickScheduler;
    use pumpkin_data::chunk::ChunkStatus;
    use pumpkin_data::{Block, block_properties::has_random_ticks};
    use pumpkin_nbt::compound::NbtCompound;
    use std::sync::atomic::{AtomicBool, AtomicU64};

    #[test]
    fn random_tick_cache_initializes_from_palette_contents() {
        let mut sections = vec![BlockPalette::default(), BlockPalette::default()];
        sections[1].set(0, 0, 0, Block::LAVA.default_state.id);

        let (cache, _mask) = ChunkSections::build_random_tick_sections_cache(&sections);
        let cache = cache.unwrap();
        assert!(!cache[0].is_randomly_ticking());
        assert!(cache[1].random_ticking_fluid_count > 0);
        assert!(cache[1].is_randomly_ticking());
    }

    #[test]
    fn random_tick_cache_updates_on_block_mutation() {
        let min_y = -64;
        let sections = ChunkSections::new(1, min_y);

        assert!(
            sections
                .random_tick_sections
                .read()
                .unwrap()
                .as_ref()
                .is_none_or(|c| !c[0].is_randomly_ticking()),
            "fresh sections should not be randomly ticking"
        );

        let random_block_state = Block::WHEAT.default_state.id;
        assert!(
            has_random_ticks(random_block_state),
            "test requires a known randomly ticking block state"
        );

        sections.set_block_absolute_y(0, min_y, 0, random_block_state);
        {
            let cache = sections.random_tick_sections.read().unwrap();
            let cache = cache.as_ref().unwrap();
            assert_eq!(cache[0].random_ticking_block_count, 1);
            assert_eq!(cache[0].random_ticking_fluid_count, 0);
            assert!(cache[0].is_randomly_ticking());
        };

        sections.set_block_absolute_y(0, min_y, 0, Block::STONE.default_state.id);
        {
            let cache = sections.random_tick_sections.read().unwrap();
            let cache = cache.as_ref().unwrap();
            assert_eq!(cache[0].random_ticking_block_count, 0);
            assert_eq!(cache[0].random_ticking_fluid_count, 0);
            assert!(!cache[0].is_randomly_ticking());
        };

        sections.set_block_absolute_y(0, min_y, 0, Block::LAVA.default_state.id);
        {
            let cache = sections.random_tick_sections.read().unwrap();
            let cache = cache.as_ref().unwrap();
            assert!(cache[0].random_ticking_fluid_count > 0);
            assert!(cache[0].is_randomly_ticking());
        }
    }

    #[test]
    fn heightmap_is_opaque() {
        use crate::chunk::ChunkHeightmapType;

        let air = Block::AIR.default_state;
        let stone = Block::STONE.default_state;
        let leaves = Block::OAK_LEAVES.default_state;
        let water = Block::WATER.default_state;

        // WORLD_SURFACE: Everything except air
        assert!(!ChunkHeightmapType::WorldSurface.is_opaque(air));
        assert!(ChunkHeightmapType::WorldSurface.is_opaque(stone));
        assert!(ChunkHeightmapType::WorldSurface.is_opaque(leaves));
        assert!(ChunkHeightmapType::WorldSurface.is_opaque(water));

        // MOTION_BLOCKING: Blocks movement OR is liquid
        assert!(!ChunkHeightmapType::MotionBlocking.is_opaque(air));
        assert!(ChunkHeightmapType::MotionBlocking.is_opaque(stone));
        assert!(ChunkHeightmapType::MotionBlocking.is_opaque(leaves)); // Leaves block movement
        assert!(ChunkHeightmapType::MotionBlocking.is_opaque(water)); // Water is liquid

        // MOTION_BLOCKING_NO_LEAVES: Blocks movement OR is liquid, but NOT leaves
        assert!(!ChunkHeightmapType::MotionBlockingNoLeaves.is_opaque(air));
        assert!(ChunkHeightmapType::MotionBlockingNoLeaves.is_opaque(stone));
        assert!(!ChunkHeightmapType::MotionBlockingNoLeaves.is_opaque(leaves)); // Excludes leaves
        assert!(ChunkHeightmapType::MotionBlockingNoLeaves.is_opaque(water)); // Water is liquid

        // OCEAN_FLOOR: blocksMotion only, unlike MOTION_BLOCKING water is NOT counted
        assert!(!ChunkHeightmapType::OceanFloor.is_opaque(air));
        assert!(ChunkHeightmapType::OceanFloor.is_opaque(stone));
        assert!(ChunkHeightmapType::OceanFloor.is_opaque(leaves)); // Leaves block movement
        assert!(!ChunkHeightmapType::OceanFloor.is_opaque(water)); // Water does not
    }

    #[test]
    fn heightmap_ocean_floor_round_trip() {
        use crate::chunk::{ChunkHeightmapType, ChunkHeightmaps};

        let mut heightmaps = ChunkHeightmaps::default();
        let min_y = -64;
        heightmaps.set(ChunkHeightmapType::OceanFloor, 3, 5, 40, min_y);
        assert_eq!(
            heightmaps.get(ChunkHeightmapType::OceanFloor, 3, 5, min_y),
            40
        );
        // Absent heightmap data (e.g. a chunk saved before this variant existed)
        // must be handled gracefully rather than panicking.
        assert_eq!(
            heightmaps.get(ChunkHeightmapType::OceanFloor, 0, 0, min_y),
            min_y - 1
        );
    }

    /// Regression test for the live "Network Protocol Error" disconnect on Nether entry.
    ///
    /// The chunk-data packet carries no section count - the client reads exactly
    /// `dimension.height / 16` sections from the dimension registry. A chunk holding fewer
    /// than that serializes short and the client reads off the end of the buffer, which is
    /// what players actually hit (`IndexOutOfBoundsException` in `LevelChunkSection.read`).
    ///
    /// Worldgen was fixed to build full-height chunks, but that alone was not enough: chunks
    /// already written to disk while the bug was live still reload short, because the on-disk
    /// reader derives its section count from the highest `Y` tag present in the file. Every
    /// one of those chunks re-crashed the client on every visit until padded here.
    #[test]
    fn padding_grows_short_chunks_to_the_full_dimension_height() {
        // A Nether chunk as saved by the buggy code: 8 sections (the noise generator's
        // 128-block `shape.height`) instead of the dimension's 16 (256 blocks).
        let chunk = ChunkData {
            section: ChunkSections::new(8, 0),
            heightmap: std::sync::Mutex::default(),
            x: 0,
            z: 0,
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks: ChunkTickScheduler::default(),
            pending_block_entities: std::sync::Mutex::default(),
            light_engine: std::sync::Mutex::new(ChunkLight {
                sky_light: vec![LightContainer::new_empty(0); 8].into_boxed_slice(),
                block_light: vec![LightContainer::new_empty(0); 8].into_boxed_slice(),
            }),
            light_populated: AtomicBool::new(false),
            status: ChunkStatus::Full,
            blending_data: None,
            unknown_nbt: NbtCompound::new(),
            dirty: AtomicBool::new(false),
            inhabited_time: AtomicU64::new(0),
        };
        assert_eq!(chunk.section.section_count(), 8);

        // Overworld-like: has sky light, so padded sections must be 15, not 0.
        chunk.pad_sections_to(16, 15);

        assert_eq!(chunk.section.section_count(), 16);
        assert_eq!(chunk.section.biome_sections.read().unwrap().len(), 16);
        // Light must grow in lockstep: it derives its own count during serialization, so a
        // padded block array with a short light array just moves the desync, not fixes it.
        let light = chunk.light_engine.lock().unwrap();
        assert_eq!(light.sky_light.len(), 16);
        assert_eq!(light.block_light.len(), 16);
        // Padded sections sit above everything the chunk stored, so they see open sky. Padding
        // them dark caps the column and stops sky light propagating down into caves - which is
        // exactly the black-caves-until-a-block-update regression this guards against. Block
        // light is genuinely 0 up there since nothing emits light in never-generated space.
        for i in 8..16 {
            assert!(
                matches!(light.sky_light[i], LightContainer::Empty(15)),
                "padded sky light section {i} must be full, got {:?}",
                light.sky_light[i]
            );
            assert!(matches!(light.block_light[i], LightContainer::Empty(0)));
        }
        drop(light);

        // Padding sections are air.
        for section in chunk.section.block_sections.read().unwrap().iter().skip(8) {
            for id in section {
                assert_eq!(id, Block::AIR.default_state.id);
            }
        }

        // Idempotent: an already-correct chunk is untouched.
        chunk.pad_sections_to(16, 15);
        assert_eq!(chunk.section.section_count(), 16);
    }
}

use std::{
    io::Cursor,
    path::PathBuf,
    pin::Pin,
    sync::{
        RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use bytes::Bytes;
use futures::future::join_all;
use pumpkin_data::{Block, chunk::ChunkStatus, fluid::Fluid};
use pumpkin_nbt::{compound::NbtCompound, from_bytes, nbt_long_array};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    block::entities::block_entity_from_nbt,
    chunk::{
        ChunkEntityData, ChunkReadingError, ChunkSerializingError,
        format::anvil::{SingleChunkDataSerializer, WORLD_DATA_VERSION},
        io::{Dirtiable, file_manager::PathFromLevelFolder},
    },
    generation::section_coords,
    level::LevelFolder,
    tick::{ScheduledTick, scheduler::ChunkTickScheduler},
};
use pumpkin_util::math::vector2::Vector2;
use serde::{Deserialize, Serialize};

use super::{
    ChunkData, ChunkHeightmaps, ChunkLight, ChunkParsingError, ChunkSections,
    palette::{BiomePalette, BlockPalette},
};
use crate::block::BlockStateCodec;

pub mod anvil;
pub mod linear;

impl SingleChunkDataSerializer for ChunkData {
    #[inline]
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        Self::internal_from_bytes(bytes, pos).map_err(ChunkReadingError::ParsingError)
    }

    #[inline]
    fn to_bytes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>> {
        Box::pin(async move { self.internal_to_bytes().await })
    }

    #[inline]
    fn position(&self) -> (i32, i32) {
        (self.x, self.z)
    }
}

impl PathFromLevelFolder for ChunkData {
    #[inline]
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.region_folder.join(file_name)
    }
}

impl Dirtiable for ChunkData {
    #[inline]
    fn mark_dirty(&self, flag: bool) {
        self.dirty.store(flag, Ordering::Relaxed);
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

impl ChunkData {
    pub fn internal_from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        let chunk_data = from_bytes::<ChunkNbt>(Cursor::new(chunk_data))
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        if chunk_data.x_pos != position.x || chunk_data.z_pos != position.y {
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for chunk {},{} but got it for {},{}!",
                position.x, position.y, chunk_data.x_pos, chunk_data.z_pos,
            )));
        }
        let (block_lights, sky_lights, block_palettes, biome_palettes) = chunk_data
            .sections
            .into_iter()
            .map(|section| {
                // When loading light data, missing data should default to 0 (no light)
                let block_light = section
                    .block_light
                    .map(LightContainer::Full)
                    .unwrap_or(LightContainer::Empty(0));
                let sky_light = section
                    .sky_light
                    .map(LightContainer::Full)
                    .unwrap_or(LightContainer::Empty(0));

                // Convert NBT to Palettes
                let block_palette = section
                    .block_states
                    .map(BlockPalette::from_disk_nbt)
                    .unwrap_or_default();
                let biome_palette = section
                    .biomes
                    .map(BiomePalette::from_disk_nbt)
                    .unwrap_or_default();

                (block_light, sky_light, block_palette, biome_palette)
            })
            .fold(
                (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(mut bl, mut sl, mut bp, mut bip), (b_l, s_l, b_p, bi_p)| {
                    bl.push(b_l);
                    sl.push(s_l);
                    bp.push(b_p);
                    bip.push(bi_p);
                    (bl, sl, bp, bip)
                },
            );

        // Assemble the LightEngine
        let light_engine = ChunkLight {
            block_light: block_lights.into_boxed_slice(),
            sky_light: sky_lights.into_boxed_slice(),
        };

        // Assemble the ChunkSections
        let min_y = section_coords::section_to_block(chunk_data.min_y_section);
        let section = ChunkSections {
            count: block_palettes.len(),
            block_sections: RwLock::new(block_palettes.into_boxed_slice()),
            biome_sections: RwLock::new(biome_palettes.into_boxed_slice()),
            min_y,
        };
        Ok(Self {
            section,
            heightmap: std::sync::Mutex::new(chunk_data.heightmaps),
            x: position.x,
            z: position.y,
            // This chunk is read from disk, so it has not been modified
            dirty: AtomicBool::new(false),
            block_ticks: ChunkTickScheduler::from_iter(chunk_data.block_ticks),
            fluid_ticks: ChunkTickScheduler::from_iter(chunk_data.fluid_ticks),
            block_entities: {
                let mut block_entities = FxHashMap::default();
                for nbt in chunk_data.block_entities {
                    let block_entity = block_entity_from_nbt(&nbt);
                    if let Some(block_entity) = block_entity {
                        block_entities.insert(block_entity.get_position(), block_entity);
                    }
                }
                std::sync::Mutex::new(block_entities)
            },
            light_engine: std::sync::Mutex::new(light_engine),
            light_populated: AtomicBool::new(chunk_data.light_correct),
            status: chunk_data.status,
        })
    }

    async fn internal_to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        let is_light_correct = self
            .light_populated
            .load(std::sync::atomic::Ordering::Relaxed);

        let sections = {
            let light_lock = self.light_engine.lock().unwrap();
            let block_lock = self.section.block_sections.read().unwrap();
            let biome_lock = self.section.biome_sections.read().unwrap();
            let min_section_y = (self.section.min_y >> 4) as i8;

            (0..self.section.count)
                .map(|i| ChunkSectionNBT {
                    y: i as i8 + min_section_y,
                    block_states: Some(block_lock[i].to_disk_nbt()),
                    biomes: Some(biome_lock[i].to_disk_nbt()),

                    block_light: match light_lock.block_light.get(i) {
                        Some(LightContainer::Empty(default)) if *default == 0 => None,
                        Some(LightContainer::Empty(default)) => {
                            let value = (*default << 4) | *default;
                            Some(vec![value; LightContainer::ARRAY_SIZE].into_boxed_slice())
                        }
                        Some(LightContainer::Full(data)) => Some(data.clone()),
                        None => None,
                    },
                    sky_light: match light_lock.sky_light.get(i) {
                        Some(LightContainer::Empty(default)) if *default == 0 => None,
                        Some(LightContainer::Empty(default)) => {
                            let value = (*default << 4) | *default;
                            Some(vec![value; LightContainer::ARRAY_SIZE].into_boxed_slice())
                        }
                        Some(LightContainer::Full(data)) => Some(data.clone()),
                        None => None,
                    },
                })
                .collect::<Vec<_>>()
        };

        // Lock and clone heightmaps
        let heightmaps = self.heightmap.lock().unwrap().clone();

        // Lock and clone entities
        let entities_to_serialize = {
            let entities_guard = self.block_entities.lock().unwrap();
            entities_guard.values().cloned().collect::<Vec<_>>()
        };

        let block_entities_nbt = join_all(entities_to_serialize.into_iter().map(
            |block_entity| async move {
                let mut nbt = NbtCompound::new();
                block_entity.write_internal(&mut nbt).await;
                nbt
            },
        ))
        .await;

        // Build the final NBT
        let nbt = ChunkNbt {
            data_version: WORLD_DATA_VERSION,
            x_pos: self.x,
            z_pos: self.z,
            min_y_section: section_coords::block_to_section(self.section.min_y),
            status: self.status,
            heightmaps,
            sections,
            block_ticks: self.block_ticks.to_vec(),
            fluid_ticks: self.fluid_ticks.to_vec(),
            block_entities: block_entities_nbt,
            light_correct: is_light_correct,
        };

        let mut result = Vec::new();
        pumpkin_nbt::to_bytes(&nbt, &mut result)
            .map_err(ChunkSerializingError::ErrorSerializingChunk)?;
        Ok(result.into())
    }
}

impl PathFromLevelFolder for ChunkEntityData {
    #[inline]
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.entities_folder.join(file_name)
    }
}

impl Dirtiable for ChunkEntityData {
    #[inline]
    fn mark_dirty(&self, flag: bool) {
        self.dirty.store(flag, Ordering::Relaxed);
    }

    #[inline]
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

impl SingleChunkDataSerializer for ChunkEntityData {
    #[inline]
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        Self::internal_from_bytes(bytes, pos).map_err(ChunkReadingError::ParsingError)
    }

    #[inline]
    fn to_bytes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>> {
        Box::pin(async move { self.internal_to_bytes().await })
    }

    #[inline]
    fn position(&self) -> (i32, i32) {
        (self.x, self.z)
    }
}

impl ChunkEntityData {
    fn internal_from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        let chunk_entity_data = pumpkin_nbt::from_bytes::<EntityNbt>(Cursor::new(chunk_data))
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        if chunk_entity_data.position[0] != position.x
            || chunk_entity_data.position[1] != position.y
        {
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for entity chunk {},{} but got it for {},{}!",
                position.x,
                position.y,
                chunk_entity_data.position[0],
                chunk_entity_data.position[1],
            )));
        }
        let mut map = FxHashMap::default();
        for entity_nbt in chunk_entity_data.entities {
            let uuid = if let Some(uuid) = entity_nbt.get_int_array("UUID") {
                Uuid::from_u128(
                    (uuid[0] as u128) << 96
                        | (uuid[1] as u128) << 64
                        | (uuid[2] as u128) << 32
                        | (uuid[3] as u128),
                )
            } else {
                log::debug!(
                    "Entity in chunk {},{} is missing UUID: {:?}",
                    position.x,
                    position.y,
                    entity_nbt
                );
                continue;
            };

            map.insert(uuid, entity_nbt);
        }

        Ok(Self {
            x: position.x,
            z: position.y,
            data: Mutex::new(map),
            dirty: AtomicBool::new(false),
        })
    }

    async fn internal_to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        let nbt = EntityNbt {
            data_version: WORLD_DATA_VERSION,
            position: [self.x, self.z],
            entities: self.data.lock().await.values().cloned().collect(),
        };

        let mut result = Vec::new();
        pumpkin_nbt::to_bytes(&nbt, &mut result)
            .map_err(ChunkSerializingError::ErrorSerializingChunk)?;
        Ok(result.into())
    }
}

#[derive(Serialize, Deserialize)]
struct ChunkSectionNBT {
    #[serde(skip_serializing_if = "Option::is_none")]
    block_states: Option<ChunkSectionBlockStates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    biomes: Option<ChunkSectionBiomes>,
    #[serde(rename = "BlockLight", skip_serializing_if = "Option::is_none")]
    block_light: Option<Box<[u8]>>,
    #[serde(rename = "SkyLight", skip_serializing_if = "Option::is_none")]
    sky_light: Option<Box<[u8]>>,
    #[serde(rename = "Y")]
    y: i8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkSectionBiomes {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) data: Option<Box<[i64]>>,
    pub(crate) palette: Vec<PaletteBiomeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// NOTE: Change not documented in the wiki; biome palettes are directly just the name now
#[serde(rename_all = "PascalCase", transparent)]
pub struct PaletteBiomeEntry {
    /// Biome name
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChunkSectionBlockStates {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) data: Option<Box<[i64]>>,
    pub(crate) palette: Vec<BlockStateCodec>,
}

#[derive(Debug, Clone)]
pub enum LightContainer {
    Empty(u8),
    Full(Box<[u8]>),
}

impl LightContainer {
    pub const DIM: usize = 16;
    pub const ARRAY_SIZE: usize = Self::DIM * Self::DIM * Self::DIM / 2;

    #[must_use]
    pub fn new_empty(default: u8) -> Self {
        assert!(default <= 15, "Default value must be between 0 and 15");
        Self::Empty(default)
    }

    #[must_use]
    pub fn new(data: Box<[u8]>) -> Self {
        assert!(
            data.len() == Self::ARRAY_SIZE,
            "Data length must be {}",
            Self::ARRAY_SIZE
        );
        Self::Full(data)
    }

    #[must_use]
    pub fn new_filled(default: u8) -> Self {
        assert!(default <= 15, "Default value must be between 0 and 15");
        let value = default << 4 | default;
        Self::Full([value; Self::ARRAY_SIZE].into())
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty(_))
    }

    const fn index(x: usize, y: usize, z: usize) -> usize {
        y * 16 * 16 + z * 16 + x
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        match self {
            Self::Full(data) => {
                let index = Self::index(x, y, z);
                data[index >> 1] >> (4 * (index & 1)) & 0x0F
            }
            Self::Empty(default) => *default,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: u8) {
        match self {
            Self::Full(data) => {
                let index = Self::index(x, y, z);
                let mask = 0x0F << (4 * (index & 1));
                data[index >> 1] &= !mask;
                data[index >> 1] |= value << (4 * (index & 1));
            }
            Self::Empty(default) => {
                if value != *default {
                    *self = Self::new_filled(*default);
                    self.set(x, y, z, value);
                }
            }
        }
    }

    pub fn fill(&mut self, value: u8) {
        *self = Self::new_filled(value);
    }
}

impl Default for LightContainer {
    fn default() -> Self {
        Self::new_empty(15)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    data_version: i32,
    #[serde(rename = "xPos")]
    x_pos: i32,
    #[serde(rename = "zPos")]
    z_pos: i32,
    #[serde(rename = "yPos")]
    min_y_section: i32,
    status: ChunkStatus,
    #[serde(rename = "sections")]
    sections: Vec<ChunkSectionNBT>,
    heightmaps: ChunkHeightmaps,
    #[serde(rename = "block_ticks")]
    block_ticks: Vec<ScheduledTick<&'static Block>>,
    #[serde(rename = "fluid_ticks")]
    fluid_ticks: Vec<ScheduledTick<&'static Fluid>>,
    #[serde(rename = "block_entities")]
    block_entities: Vec<NbtCompound>,
    #[serde(rename = "isLightOn", default)]
    light_correct: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct EntityNbt {
    data_version: i32,
    position: [i32; 2],
    entities: Vec<NbtCompound>,
}

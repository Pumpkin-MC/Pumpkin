use std::collections::HashMap;

use anvil::REGION_SIZE;
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_nbt::{compound::NbtCompound, from_bytes, nbt_long_array};

use pumpkin_util::math::{ceil_log2, vector2::Vector2};
use serde::{Deserialize, Serialize};

use crate::{
    block::ChunkBlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
};

use super::{
    CHUNK_AREA, ChunkBlocks, ChunkData, ChunkHeightmaps, ChunkParsingError, ChunkSerializingError,
    SUBCHUNK_VOLUME,
};

pub mod anvil;
pub mod linear;

// I can't use an tag because it will break ChunkNBT, but status need to have a big S, so "Status"
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ChunkStatusWrapper {
    status: ChunkStatus,
}

/// The number of bits that identify two chunks in the same region
pub const SUBREGION_BITS: u8 = pumpkin_util::math::ceil_log2(REGION_SIZE as u32);

pub const SUBREGION_AND: i32 = i32::pow(2, SUBREGION_BITS as u32) - 1;

pub const fn get_chunk_index(pos: &Vector2<i32>) -> usize {
    let local_x = pos.x & SUBREGION_AND;
    let local_z = pos.z & SUBREGION_AND;
    let index = (local_z << SUBREGION_BITS) + local_x;
    index as usize
}

pub const fn get_region_coords(at: &Vector2<i32>) -> (i32, i32) {
    // Divide by 32 for the region coordinates
    (at.x >> SUBREGION_BITS, at.z >> SUBREGION_BITS)
}

/// Used for Saving
pub trait DataToBytes
where
    Self: Send + Sync,
{
    type Data: Send + Sync + Sized;

    fn data_to_bytes(data: &Self::Data) -> Result<Vec<u8>, ChunkSerializingError>;
}

/// Used for Reading
pub trait BytesToData
where
    Self: Send + Sync,
{
    type Data: Send + Sync + Sized;

    fn bytes_to_data(data: &[u8], position: Vector2<i32>) -> Result<Self::Data, ChunkParsingError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PaletteEntry {
    // block name
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
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

// #[serde(rename_all = "PascalCase")]
struct EntityNbt {
    data_version: i32,
    /// The Chunk position
    position: Vector2<i32>,
    entities: Vec<NbtCompound>,
}

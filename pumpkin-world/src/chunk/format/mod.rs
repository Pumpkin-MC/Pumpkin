use std::collections::HashMap;

use pumpkin_data::{block::Block, chunk::ChunkStatus};
use pumpkin_nbt::{from_bytes, nbt_long_array};

use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use serde::{Deserialize, Serialize};

use crate::block::ChunkBlockState;
use crate::generation::section_coords;

use super::{
    CHUNK_AREA, ChunkData, ChunkHeightmaps, ChunkParsingError, ChunkSections, SUBCHUNK_VOLUME,
    ScheduledTick, TickPriority,
    palette::{BlockPalette, encompassing_bits},
};

pub mod anvil;
pub mod linear;

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
        // TODO: Implement chunk stages?
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
            return Err(ChunkParsingError::ErrorDeserializingChunk(format!(
                "Expected data for chunk {},{} but got it for {},{}!",
                position.x, position.z, chunk_data.x_pos, chunk_data.z_pos,
            )));
        }

        // TODO: Biomes

        let min_y = section_coords::section_to_block(chunk_data.min_y_section);
        let mut final_section = ChunkSections::new(min_y);
        let mut block_index = 0; // which block we're currently at

        for section in chunk_data.sections.into_iter() {
            let block_states = match section.block_states {
                Some(states) => states,
                None => continue, // TODO @lukas0008 this should instead fill all blocks with the only element of the palette
            };

            let palette = block_states
                .palette
                .iter()
                .map(ChunkBlockState::from_palette)
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
            let bits_per_block_id = encompassing_bits(palette.len()).max(4);
            // How many blocks there are in one of the palettes u64s
            let blocks_in_palette = 64 / bits_per_block_id;

            let mask = (1 << bits_per_block_id) - 1;
            'block_loop: for block in block_data.iter() {
                for i in 0..blocks_in_palette {
                    let index = (block >> (i * bits_per_block_id)) & mask;
                    let block = &palette[index as usize];

                    let relative_x = block_index % BlockPalette::SIZE;
                    let relative_y = block_index / CHUNK_AREA;
                    let relative_z = (block_index % CHUNK_AREA) / BlockPalette::SIZE;

                    final_section.set_block_no_heightmap_update(
                        relative_x,
                        relative_y,
                        relative_z,
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
            section: final_section,
            heightmap: chunk_data.heightmaps,
            position,
            // This chunk is read from disk, so it has not been modified
            dirty: false,
            block_ticks: chunk_data
                .block_ticks
                .iter()
                .map(|tick| ScheduledTick {
                    block_pos: BlockPos::new(tick.x, tick.y, tick.z),
                    delay: tick.delay as u16,
                    priority: TickPriority::from(tick.priority),
                    target_block_id: Block::from_registry_key(
                        &tick.target_block.replace("minecraft:", ""),
                    )
                    .unwrap_or(Block::AIR)
                    .id,
                })
                .collect(),
            fluid_ticks: chunk_data
                .fluid_ticks
                .iter()
                .map(|tick| ScheduledTick {
                    block_pos: BlockPos::new(tick.x, tick.y, tick.z),
                    delay: tick.delay as u16,
                    priority: TickPriority::from(tick.priority),
                    target_block_id: Block::from_registry_key(
                        &tick.target_block.replace("minecraft:", ""),
                    )
                    .unwrap_or(Block::AIR)
                    .id,
                })
                .collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ChunkSectionNBT {
    #[serde(skip_serializing_if = "Option::is_none")]
    block_states: Option<ChunkSectionBlockStates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    biomes: Option<ChunkSectionBiomes>,
    // TODO
    // #[serde(rename = "BlockLight", skip_serializing_if = "Option::is_none")]
    // block_light: Option<Box<[u8]>>,
    // #[serde(rename = "SkyLight", skip_serializing_if = "Option::is_none")]
    // sky_light: Option<Box<[u8]>>,
    #[serde(rename = "Y")]
    y: i8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChunkSectionBiomes {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    data: Option<Box<[i64]>>,
    palette: Vec<PaletteBiomeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PaletteBiomeEntry {
    /// Biome name
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    #[serde(
        serialize_with = "nbt_long_array",
        skip_serializing_if = "Option::is_none"
    )]
    data: Option<Box<[i64]>>,
    palette: Vec<PaletteBlockEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PaletteBlockEntry {
    /// Block name
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializedScheduledTick {
    #[serde(rename = "x")]
    x: i32,
    #[serde(rename = "y")]
    y: i32,
    #[serde(rename = "z")]
    z: i32,
    #[serde(rename = "t")]
    delay: i32,
    #[serde(rename = "p")]
    priority: i32,
    #[serde(rename = "i")]
    target_block: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    data_version: i32,
    #[serde(rename = "xPos")]
    x_pos: i32,
    #[serde(rename = "yPos")]
    min_y_section: i32,
    #[serde(rename = "zPos")]
    z_pos: i32,
    status: ChunkStatus,
    #[serde(rename = "sections")]
    sections: Vec<ChunkSectionNBT>,
    heightmaps: ChunkHeightmaps,
    #[serde(rename = "block_ticks")]
    block_ticks: Vec<SerializedScheduledTick>,
    #[serde(rename = "fluid_ticks")]
    fluid_ticks: Vec<SerializedScheduledTick>,
}

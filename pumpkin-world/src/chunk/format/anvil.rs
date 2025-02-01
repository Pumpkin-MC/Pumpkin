use fastnbt::LongArray;
use indexmap::IndexMap;
use pumpkin_data::chunk::ChunkStatus;
use pumpkin_util::math::{ceil_log2, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::block::block_registry::BLOCK_ID_TO_REGISTRY_ID;
use crate::block::BlockState;
use crate::chunk::{
    ChunkData, ChunkHeightmaps, ChunkParsingError, Subchunks, CHUNK_AREA, SUBCHUNK_VOLUME,
};
use crate::coordinates::{ChunkRelativeBlockCoordinates, Height};

use super::{ChunkFormat, ChunkReadingError, ChunkWritingError};

// 1.21.4
const WORLD_DATA_VERSION: i32 = 4189;

#[derive(Clone, Default)]
pub struct AnvilChunkFormat;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    // block name
    name: String,
    properties: Option<HashMap<String, String>>,
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

// I can't use an tag because it will break ChunkNBT, but status need to have a big S, so "Status"
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ChunkStatusWrapper {
    status: ChunkStatus,
}

#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(fastnbt::error::Error),
}

impl ChunkFormat for AnvilChunkFormat {
    fn read_chunk(
        &self,
        chunk_bytes: Vec<u8>,
        at: &Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError> {
        if fastnbt::from_bytes::<ChunkStatusWrapper>(&chunk_bytes)
            .map_err(|_| ChunkReadingError::InvalidHeader)?
            .status
            != ChunkStatus::Full
        {
            return Err(ChunkReadingError::ChunkNotExist);
        }

        let chunk_data = fastnbt::from_bytes::<ChunkNbt>(&chunk_bytes).map_err(|e| {
            ChunkReadingError::ParsingError(ChunkParsingError::ErrorDeserializingChunk(
                e.to_string(),
            ))
        })?;

        if chunk_data.x_pos != at.x || chunk_data.z_pos != at.z {
            log::error!(
                "Expected chunk at {}:{}, but got {}:{}",
                at.x,
                at.z,
                chunk_data.x_pos,
                chunk_data.z_pos
            );
            // lets still continue
        }

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
            position: *at,
        })
    }

    fn save_chunk(
        &self,
        chunk_data: &ChunkData,
        _at: &Vector2<i32>,
    ) -> Result<Vec<u8>, super::ChunkWritingError> {
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.subchunks.array_iter().enumerate() {
            // get unique blocks
            let unique_blocks: HashSet<_> = blocks.iter().collect();

            let palette: IndexMap<_, _> = unique_blocks
                .into_iter()
                .enumerate()
                .map(|(i, block)| {
                    let name = BLOCK_ID_TO_REGISTRY_ID.get(block).unwrap().as_str();
                    (block, (name, i))
                })
                .collect();

            // Determine the number of bits needed to represent the largest index in the palette
            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            };

            let mut section_longs = Vec::new();
            let mut current_pack_long: i64 = 0;
            let mut bits_used_in_pack: u32 = 0;

            // Empty data if the palette only contains one index https://minecraft.fandom.com/wiki/Chunk_format
            // if palette.len() > 1 {}
            // TODO: Update to write empty data. Rn or read does not handle this elegantly
            for block in blocks.iter() {
                // Push if next bit does not fit
                if bits_used_in_pack + block_bit_size as u32 > 64 {
                    section_longs.push(current_pack_long);
                    current_pack_long = 0;
                    bits_used_in_pack = 0;
                }
                let index = palette.get(block).expect("Just added all unique").1;
                current_pack_long |= (index as i64) << bits_used_in_pack;
                bits_used_in_pack += block_bit_size as u32;

                assert!(bits_used_in_pack <= 64);

                // If the current 64-bit integer is full, push it to the section_longs and start a new one
                if bits_used_in_pack >= 64 {
                    section_longs.push(current_pack_long);
                    current_pack_long = 0;
                    bits_used_in_pack = 0;
                }
            }

            // Push the last 64-bit integer if it contains any data
            if bits_used_in_pack > 0 {
                section_longs.push(current_pack_long);
            }

            sections.push(ChunkSection {
                y: i as i8 - 4,
                block_states: Some(ChunkSectionBlockStates {
                    data: Some(LongArray::new(section_longs)),
                    palette: palette
                        .into_iter()
                        .map(|entry| PaletteEntry {
                            name: entry.1 .0.to_owned(),
                            properties: None,
                        })
                        .collect(),
                }),
            });
        }

        let nbt = ChunkNbt {
            data_version: WORLD_DATA_VERSION,
            x_pos: chunk_data.position.x,
            z_pos: chunk_data.position.z,
            status: ChunkStatus::Full,
            heightmaps: chunk_data.heightmap.clone(),
            sections,
        };

        fastnbt::to_bytes(&nbt).map_err(|e| ChunkWritingError::ChunkSerializingError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;

    use crate::chunk::db::anvil_saver::AnvilSaver;
    use crate::chunk::db::{ChunkStorage, ChunkStorageReadingError};

    use crate::generation::{get_world_gen, Seed};
    use crate::{
        chunk::format::{anvil::AnvilChunkFormat, ChunkFormat},
        level::LevelFolder,
    };

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let result = AnvilSaver.read_raw_chunk(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &Vector2::new(0, 0),
        );
        assert!(matches!(
            result,
            Err(ChunkStorageReadingError::ChunkNotExist)
        ));
    }

    #[test]
    fn test_writing() {
        let generator = get_world_gen(Seed(0));
        let level_folder = LevelFolder {
            root_folder: PathBuf::from("./tmp"),
            region_folder: PathBuf::from("./tmp/region"),
        };
        if fs::exists(&level_folder.root_folder).unwrap() {
            fs::remove_dir_all(&level_folder.root_folder).expect("Could not delete directory");
        }

        fs::create_dir_all(&level_folder.region_folder).expect("Could not create directory");

        // Generate chunks
        let mut chunks = vec![];
        for x in -5..5 {
            for y in -5..5 {
                let position = Vector2::new(x, y);
                chunks.push((position, generator.generate_chunk(position)));
            }
        }

        for i in 0..5 {
            println!("Iteration {}", i + 1);
            for (at, chunk) in &chunks {
                AnvilSaver
                    .write_raw_chunk(
                        AnvilChunkFormat
                            .save_chunk(chunk, at)
                            .expect("Failed to write raw chunk"),
                        &level_folder,
                        at,
                    )
                    .expect("Failed to write chunk");
            }

            let mut read_chunks = vec![];
            for (at, _chunk) in &chunks {
                read_chunks.push(
                    AnvilChunkFormat
                        .read_chunk(
                            AnvilSaver
                                .read_raw_chunk(&level_folder, at)
                                .expect("Failed to read raw chunk"),
                            at,
                        )
                        .expect("Failed to read chunk"),
                );
            }

            for (at, chunk) in &chunks {
                let read_chunk = read_chunks
                    .iter()
                    .find(|chunk| chunk.position == *at)
                    .expect("Missing chunk");
                assert_eq!(chunk.subchunks, read_chunk.subchunks, "Chunks don't match");
            }
        }

        fs::remove_dir_all(&level_folder.root_folder).expect("Could not delete directory");

        println!("Checked chunks successfully");
    }
}

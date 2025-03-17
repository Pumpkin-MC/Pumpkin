use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use super::{
    AnvilFile, ChunkNbt, ChunkSection, ChunkSectionBlockStates, PaletteEntry, WORLD_DATA_VERSION,
};
use async_trait::async_trait;
use bytes::Bytes;
use indexmap::IndexMap;
use pumpkin_data::{block::Block, chunk::ChunkStatus};
use pumpkin_nbt::{from_bytes, to_bytes};
use pumpkin_util::math::{ceil_log2, vector2::Vector2};

use crate::{
    block::ChunkBlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    storage::{
        CHUNK_AREA, ChunkBlocks, ChunkData, ChunkParsingError, ChunkReadingError,
        ChunkSerializingError, ChunkWritingError, SUBCHUNK_VOLUME,
        format::{BytesToData, ChunkStatusWrapper, DataToBytes, get_chunk_index},
        io::{ChunkSerializer, LoadedData},
    },
};
#[derive(Default)]
pub struct AnvilChunkFormat {
    anvil: AnvilFile,
}

#[async_trait]
impl ChunkSerializer for AnvilChunkFormat {
    type Data = ChunkData;
    type WriteBackend = PathBuf;

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        AnvilFile::get_chunk_key(chunk)
    }

    fn should_write(&self, is_watched: bool) -> bool {
        self.anvil.should_write(is_watched)
    }

    async fn write(&self, path: PathBuf) -> Result<(), std::io::Error> {
        self.anvil.write(path);
    }

    fn read(bytes: Bytes) -> Result<Self, ChunkReadingError> {
        let anvil = AnvilFile::read(bytes)?;
        Ok(Self { anvil })
    }

    async fn update_chunk(&mut self, chunk: &ChunkData) -> Result<(), ChunkWritingError> {
        self.anvil.update_chunk::<Self>(chunk.position, chunk).await
    }

    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<ChunkData, ChunkReadingError>>,
    ) {
        // Create an unbounded buffer so we don't block the rayon thread pool
        let (bridge_send, mut bridge_recv) = tokio::sync::mpsc::unbounded_channel();

        // Don't par iter here so we can prevent backpressure with the await in the async
        // runtime
        for chunk in chunks.iter().cloned() {
            let index = get_chunk_index(&chunk);
            match &self.anvil.chunks_data[index] {
                None => stream
                    .send(LoadedData::Missing(chunk))
                    .await
                    .expect("Failed to send chunk"),
                Some(chunk_metadata) => {
                    let send = bridge_send.clone();
                    let chunk_data = chunk_metadata.serialized_data.clone();
                    rayon::spawn(move || {
                        let result = match chunk_data.to_chunk::<Self>(chunk) {
                            Ok(chunk) => LoadedData::Loaded(chunk),
                            Err(err) => LoadedData::Error((chunk, err)),
                        };

                        send.send(result)
                            .expect("Failed to send anvil chunks from rayon thread");
                    });
                }
            }
        }
        // Drop the original so streams clean-up
        drop(bridge_send);

        // We don't want to waste work, so recv unbounded from the rayon thread pool, then re-send
        // to the channel

        while let Some(data) = bridge_recv.recv().await {
            stream
                .send(data)
                .await
                .expect("Failed to send anvil chunks from bridge");
        }
    }
}

impl DataToBytes for AnvilChunkFormat {
    type Data = ChunkData;

    fn data_to_bytes(chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.blocks.array_iter_subchunks().enumerate() {
            // get unique blocks
            let unique_blocks: HashSet<_> = blocks.iter().collect();

            let palette: IndexMap<_, _> = unique_blocks
                .into_iter()
                .enumerate()
                .map(|(i, block)| {
                    let name = Block::from_state_id(*block).unwrap().name;
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
                    data: Some(section_longs.into_boxed_slice()),
                    palette: palette
                        .into_iter()
                        .map(|entry| PaletteEntry {
                            name: entry.1.0.to_string(),
                            properties: {
                                let block = Block::from_state_id(*entry.0).unwrap();
                                if let Some(properties) = block.properties(*entry.0) {
                                    let props = properties.to_props();
                                    let mut props_map = HashMap::new();
                                    for prop in props {
                                        props_map.insert(prop.0.clone(), prop.1.clone());
                                    }
                                    Some(props_map)
                                } else {
                                    None
                                }
                            },
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

        let mut result = Vec::new();
        to_bytes(&nbt, &mut result).map_err(ChunkSerializingError::ErrorSerializingChunk)?;
        Ok(result)
    }
}

impl BytesToData for AnvilChunkFormat {
    type Data = ChunkData;
    fn bytes_to_data(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<ChunkData, ChunkParsingError> {
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

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut blocks = ChunkBlocks::Homogeneous(0);
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
                    blocks.set_block_no_heightmap_update(
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
            blocks,
            heightmap: chunk_data.heightmaps,
            position,
            entities: vec![],
            // This chunk is read from disk, so it has not been modified
            dirty: false,
        })
    }
}

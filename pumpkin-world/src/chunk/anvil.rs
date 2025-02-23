use async_trait::async_trait;
use bytes::*;
use flate2::bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use indexmap::IndexMap;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_nbt::serializer::to_bytes;
use pumpkin_util::math::ceil_log2;
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::sync::Notify;

use std::{
    collections::HashSet,
    io::{Read, Write},
    sync::{Arc, atomic::AtomicBool},
};

use crate::block::registry::STATE_ID_TO_REGISTRY_ID;
use crate::chunks_io::{ChunkSerializer, LoadedData};

use super::{
    ChunkData, ChunkNbt, ChunkReadingError, ChunkSection, ChunkSectionBlockStates,
    ChunkSerializingError, ChunkWritingError, CompressionError, PaletteEntry,
};

/// The side size of a region in chunks (one region is 32x32 chunks)
pub const REGION_SIZE: usize = 32;

/// The number of bits that identify two chunks in the same region
pub const SUBREGION_BITS: u8 = pumpkin_util::math::ceil_log2(REGION_SIZE as u32);

/// The number of chunks in a region
pub const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// The number of bytes in a sector (4 KiB)
const SECTOR_BYTES: usize = 4096;

// 1.21.4
const WORLD_DATA_VERSION: i32 = 4189;

#[derive(Clone, Default)]
pub struct AnvilChunkFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    /// GZip Compression
    GZip = 1,
    /// ZLib Compression
    ZLib = 2,
    /// LZ4 Compression (since 24w04a)
    LZ4 = 4,
    /// Custom compression algorithm (since 24w05a)
    Custom = 127,
}

#[derive(Default)]
pub struct AnvilChunkData {
    length: u32,
    compression: Option<Compression>,
    compressed_data: Vec<u8>,
}

pub struct AnvilChunkFile {
    timestamp_table: [u32; CHUNK_COUNT],
    chunks_data: [Option<Arc<AnvilChunkData>>; CHUNK_COUNT],
}

impl Compression {
    const NO_COMPRESSION: u8 = 3;

    fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut decoder = GzDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut decoder = ZlibDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }
            Compression::LZ4 => {
                let mut decoder =
                    lz4::Decoder::new(compressed_data).map_err(CompressionError::LZ4Error)?;
                let mut decompressed_data = Vec::new();
                decoder
                    .read_to_end(&mut decompressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                Ok(decompressed_data)
            }
            Compression::Custom => todo!(),
        }
    }

    fn compress_data(
        &self,
        uncompressed_data: &[u8],
        compression_level: u32,
    ) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut encoder = GzEncoder::new(
                    uncompressed_data,
                    flate2::Compression::new(compression_level),
                );
                let mut chunk_data = Vec::new();
                encoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut encoder = ZlibEncoder::new(
                    uncompressed_data,
                    flate2::Compression::new(compression_level),
                );
                let mut chunk_data = Vec::new();
                encoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }

            Compression::LZ4 => {
                let mut compressed_data = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(compression_level)
                    .build(&mut compressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                if let Err(err) = encoder.write_all(uncompressed_data) {
                    return Err(CompressionError::LZ4Error(err));
                }
                if let (_output, Err(err)) = encoder.finish() {
                    return Err(CompressionError::LZ4Error(err));
                }
                Ok(compressed_data)
            }
            Compression::Custom => todo!(),
        }
    }

    /// Returns Ok when a compression is found otherwise an Err
    #[allow(clippy::result_unit_err)]
    pub fn from_byte(byte: u8) -> Result<Option<Self>, ()> {
        match byte {
            1 => Ok(Some(Self::GZip)),
            2 => Ok(Some(Self::ZLib)),
            // Uncompressed (since a version before 1.15.1)
            3 => Ok(None),
            4 => Ok(Some(Self::LZ4)),
            127 => Ok(Some(Self::Custom)),
            // Unknown format
            _ => Err(()),
        }
    }
}

impl From<pumpkin_config::chunk::Compression> for Compression {
    fn from(value: pumpkin_config::chunk::Compression) -> Self {
        // :c
        match value {
            pumpkin_config::chunk::Compression::GZip => Self::GZip,
            pumpkin_config::chunk::Compression::ZLib => Self::ZLib,
            pumpkin_config::chunk::Compression::LZ4 => Self::LZ4,
            pumpkin_config::chunk::Compression::Custom => Self::Custom,
        }
    }
}

impl AnvilChunkData {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError> {
        let mut buffer = bytes;

        let length = buffer.get_u32();

        let compression_method = buffer.get_u8();
        let compression = Compression::from_byte(compression_method)
            .map_err(|_| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        Ok(AnvilChunkData {
            length,
            compression,
            compressed_data: buffer.to_vec(),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        // 4 bytes for the *length* and 1 byte for the *compression* method
        let total_size = self.compressed_data.len() + 4 + 1;
        let sector_count = total_size.div_ceil(SECTOR_BYTES);
        let padded_size = sector_count * SECTOR_BYTES;

        let mut bytes = Vec::with_capacity(padded_size);

        bytes.put_u32(self.length);
        bytes.put_u8(
            self.compression
                .map_or(Compression::NO_COMPRESSION, |c| c as u8),
        );
        bytes.extend_from_slice(&self.compressed_data);

        bytes.resize(padded_size, 0);
        bytes
    }

    fn to_chunk(&self, pos: Vector2<i32>) -> Result<ChunkData, ChunkReadingError> {
        // -1 for the padding/align of the *compression* byte
        let bytes = &self.compressed_data[..self.length as usize - 1];

        if let Some(compression) = self.compression {
            let decompressed_data = compression
                .decompress_data(bytes)
                .expect("Failed to decompress chunk data");

            ChunkData::from_bytes(&decompressed_data, pos)
        } else {
            ChunkData::from_bytes(bytes, pos)
        }
        .map_err(ChunkReadingError::ParsingError)
    }

    fn from_chunk(chunk: &ChunkData) -> Result<Self, ChunkWritingError> {
        let raw_bytes = chunk_to_bytes(chunk)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        let compression: Compression = ADVANCED_CONFIG.chunk.compression.algorithm.clone().into();
        let compressed_data = compression
            .compress_data(&raw_bytes, ADVANCED_CONFIG.chunk.compression.level)
            .map_err(ChunkWritingError::Compression)?;

        Ok(AnvilChunkData {
            length: compressed_data.len() as u32 + 1,
            compression: Some(compression),
            compressed_data,
        })
    }
}

impl AnvilChunkFile {
    pub const fn get_region_coords(at: Vector2<i32>) -> (i32, i32) {
        // Divide by 32 for the region coordinates
        (at.x >> SUBREGION_BITS, at.z >> SUBREGION_BITS)
    }

    const fn get_chunk_index(pos: &Vector2<i32>) -> usize {
        let local_x = (pos.x & 31) as usize;
        let local_z = (pos.z & 31) as usize;
        (local_z << 5) + local_x
    }
}

impl Default for AnvilChunkFile {
    fn default() -> Self {
        Self {
            timestamp_table: [0; CHUNK_COUNT],
            chunks_data: [const { None }; CHUNK_COUNT],
        }
    }
}

#[async_trait]
impl ChunkSerializer for AnvilChunkFile {
    type Data = ChunkData;

    fn get_chunk_key(chunk: Vector2<i32>) -> String {
        let (region_x, region_z) = Self::get_region_coords(chunk);
        format!("./r.{}.{}.mca", region_x, region_z)
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut chunk_data: Vec<u8> = Vec::new();

        let mut location_bytes = [0; SECTOR_BYTES];
        let mut timestamp_bytes = [0; SECTOR_BYTES];

        let mut location_buf = location_bytes.as_mut_slice();
        let mut timestamp_buf = timestamp_bytes.as_mut_slice();

        // The first two sectors are reserved for the location table
        let mut current_sector: u32 = 2;
        for i in 0..CHUNK_COUNT {
            let chunk = if let Some(chunk_data) = &self.chunks_data[i] {
                chunk_data
            } else {
                // If the chunk is not present, we write 0 to the location and timestamp tables
                location_buf.put_u32(0);
                timestamp_buf.put_u32(0);
                continue;
            };

            let chunk_bytes = chunk.to_bytes();
            // The number of sectors the chunk data occupies
            let sector_count = (chunk_bytes.len() / SECTOR_BYTES) as u32;

            // The location is stored as a 32-bit integer where the first 24 bits are the sector offset and the last 8 bits are the sector count
            location_buf.put_u32((current_sector << 8) | sector_count);
            timestamp_buf.put_u32(self.timestamp_table[i]);

            chunk_data.extend_from_slice(&chunk_bytes);

            current_sector += sector_count;
        }

        [
            location_bytes.as_slice(),
            timestamp_bytes.as_slice(),
            chunk_data.as_slice(),
        ]
        .concat()
        .into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError> {
        let (headers, chunks) = bytes.split_at(SECTOR_BYTES * 2);
        let (mut location_bytes, mut timestamp_bytes) = headers.split_at(SECTOR_BYTES);

        let mut chunk_file = AnvilChunkFile::default();

        for i in 0..CHUNK_COUNT {
            chunk_file.timestamp_table[i] = timestamp_bytes.get_u32();
            let location = location_bytes.get_u32();

            let sector_count = (location & 0xFF) as usize;
            let sector_offset = (location >> 8) as usize;

            // If the sector offset and count is 0, the chunk is not present
            if sector_offset == 0 && sector_count == 0 {
                continue;
            }

            //we correct the sectors values, -2 for the first two sectors (location and timestamp tables)
            //and * SECTOR_BYTES for the byte offset at byte level
            let bytes_offset = (sector_offset - 2) * SECTOR_BYTES;
            let bytes_count = sector_count * SECTOR_BYTES;

            chunk_file.chunks_data[i] = Some(Arc::new(AnvilChunkData::from_bytes(
                &chunks[bytes_offset..bytes_offset + bytes_count],
            )?));
        }

        Ok(chunk_file)
    }

    fn add_chunk_data(&mut self, chunks_data: &[Self::Data]) -> Result<(), ChunkWritingError> {
        for chunk in chunks_data {
            let index = AnvilChunkFile::get_chunk_index(&chunk.position);
            self.chunks_data[index] = Some(Arc::new(AnvilChunkData::from_chunk(chunk)?));
        }

        Ok(())
    }

    async fn stream_chunk_data(
        &self,
        chunks: &[Vector2<i32>],
        channel: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        let notify = Arc::new(Notify::new());
        let mut chunk_data = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let index = AnvilChunkFile::get_chunk_index(chunk);
            if let Some(data) = &self.chunks_data[index] {
                chunk_data.push((*chunk, data.clone(), channel.clone()))
            } else {
                channel
                    .send(LoadedData::Missing(*chunk))
                    .await
                    .expect("Failed to stream missing message from anvil chunk serializer!");
            }
        }

        if chunk_data.is_empty() {
            return;
        }

        let internal_notify = notify.clone();
        let finished_before_await_notify = Arc::new(AtomicBool::new(false));
        let finished_fast = finished_before_await_notify.clone();
        rayon::spawn(move || {
            chunk_data
                .into_par_iter()
                .for_each(|(at, chunk_data, channel)| {
                    let result = match chunk_data.to_chunk(at) {
                        Ok(chunk) => LoadedData::Loaded(chunk),
                        Err(err) => LoadedData::Error((at, err)),
                    };

                    channel
                        .blocking_send(result)
                        .expect("Failed to stream data from anvil chunk serializer!");
                });
            finished_fast.store(true, std::sync::atomic::Ordering::Relaxed);
            internal_notify.notify_waiters();
        });

        if finished_before_await_notify.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        notify.notified().await;
    }
}

pub fn chunk_to_bytes(chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
    let mut sections = Vec::new();

    for (i, blocks) in chunk_data.subchunks.array_iter().enumerate() {
        // get unique blocks
        let unique_blocks: HashSet<_> = blocks.iter().collect();

        let palette: IndexMap<_, _> = unique_blocks
            .into_iter()
            .enumerate()
            .map(|(i, block)| {
                let name = STATE_ID_TO_REGISTRY_ID.get(block).unwrap();
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
                            /*
                            let properties = &get_block(entry.1 .0).unwrap().properties;
                            let mut map = HashMap::new();
                            for property in properties {
                                map.insert(property.name.to_string(), property.values.clone());
                            }
                            Some(map)
                            */
                            None
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
        status: super::ChunkStatus::Full,
        heightmaps: chunk_data.heightmap.clone(),
        sections,
    };

    let mut result = Vec::new();
    to_bytes(&nbt, &mut result).map_err(ChunkSerializingError::ErrorSerializingChunk)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use temp_dir::TempDir;
    use tokio::sync::{RwLock, mpsc};

    use crate::generation::{Seed, get_world_gen};
    use crate::{
        chunk::anvil::AnvilChunkFile,
        chunks_io::{ChunkFileManager, ChunkIO, LoadedData},
        level::LevelFolder,
    };
    #[tokio::test(flavor = "multi_thread")]
    async fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile>::default();
        let (send, mut recv) = mpsc::channel(1);

        chunk_saver
            .stream_chunks(
                &LevelFolder {
                    root_folder: PathBuf::from(""),
                    region_folder: region_path,
                },
                &[Vector2::new(0, 0)],
                send,
            )
            .await;

        let mut chunks = Vec::new();
        while let Some(chunk) = recv.recv().await {
            chunks.push(chunk);
        }

        assert!(chunks.len() == 1 && matches!(chunks[0], LoadedData::Missing(_)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_writing() {
        let generator = get_world_gen(Seed(0));

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
        };
        fs::create_dir(&level_folder.region_folder).expect("couldn't create region folder");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile>::default();

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
            chunk_saver
                .save_chunks(
                    &level_folder,
                    chunks
                        .clone()
                        .into_iter()
                        .map(|(at, chunk)| (at, Arc::new(RwLock::new(chunk))))
                        .collect::<Vec<_>>(),
                )
                .await
                .expect("Failed to write chunk");

            let (send, mut recv) = mpsc::channel(chunks.len());
            chunk_saver
                .stream_chunks(
                    &level_folder,
                    &chunks.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
                    send,
                )
                .await;

            let mut read_chunks = Vec::new();
            while let Some(chunk) = recv.recv().await {
                match chunk {
                    LoadedData::Loaded(chunk) => {
                        read_chunks.push(chunk);
                    }
                    LoadedData::Missing(_) => {}
                    LoadedData::Error((position, error)) => {
                        panic!("Error reading chunk at {:?} | Error: {:?}", position, error)
                    }
                };
            }

            for (at, chunk) in &chunks {
                let read_chunk = read_chunks
                    .iter()
                    .find(|chunk| chunk.position == *at)
                    .expect("Missing chunk");
                assert_eq!(chunk.subchunks, read_chunk.subchunks, "Chunks don't match");
            }
        }

        println!("Checked chunks successfully");
    }

    // TODO
    /*
    #[test]
    fn test_load_java_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
        };

        fs::create_dir(&level_folder.region_folder).unwrap();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join(file!())
                .parent()
                .unwrap()
                .join("../../assets/r.0.0.mca"),
            level_folder.region_folder.join("r.0.0.mca"),
        )
        .unwrap();

        let mut actually_tested = false;
        for x in 0..(1 << 5) {
            for z in 0..(1 << 5) {
                let result = AnvilChunkFormat {}.read_chunk(&level_folder, &Vector2 { x, z });

                match result {
                    Ok(_) => actually_tested = true,
                    Err(ChunkReadingError::ParsingError(ChunkParsingError::ChunkNotGenerated)) => {}
                    Err(ChunkReadingError::ChunkNotExist) => {}
                    Err(e) => panic!("{:?}", e),
                }

                println!("=========== OK ===========");
            }
        }

        assert!(actually_tested);
    }
    */
}

use bytes::*;
use fastnbt::LongArray;
use flate2::bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use indexmap::IndexMap;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_util::math::ceil_log2;
use pumpkin_util::math::vector2::Vector2;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use std::{
    collections::HashSet,
    io::{Read, Write},
};

use crate::block::registry::BLOCK_ID_TO_REGISTRY_ID;
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
    chunks_data: [Option<AnvilChunkData>; CHUNK_COUNT],
}

impl Compression {
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
        let total_size = self.compressed_data.len() + 5;
        let sector_count = total_size.div_ceil(SECTOR_BYTES);
        let padded_size = sector_count * SECTOR_BYTES;

        let mut bytes = Vec::with_capacity(padded_size);

        bytes.put_u32(self.length);
        bytes.put_u8(self.compression.map_or(3, |c| c as u8));
        bytes.extend_from_slice(&self.compressed_data);

        bytes.resize(padded_size, 0);
        bytes
    }

    fn to_chunk(&self, pos: Vector2<i32>) -> Result<ChunkData, ChunkReadingError> {
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
        (at.x >> SUBREGION_BITS, at.z >> SUBREGION_BITS) // Divide by 32 for the region coordinates
    }

    const fn get_chunk_index(pos: Vector2<i32>) -> usize {
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

impl ChunkSerializer for AnvilChunkFile {
    type Data = ChunkData;

    fn get_chunk_key(chunk: Vector2<i32>) -> String {
        let (region_x, region_z) = Self::get_region_coords(chunk);
        format!("./r.{}.{}.mca", region_x, region_z)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut chunk_data: Vec<u8> = Vec::new();

        let mut location_bytes: Vec<u8> = Vec::with_capacity(SECTOR_BYTES);
        let mut timestamp_bytes: Vec<u8> = Vec::with_capacity(SECTOR_BYTES);

        // The first two sectors are reserved for the location table
        let mut current_sector: u32 = 2;
        for i in 0..CHUNK_COUNT {
            let chunk = if let Some(chunk_data) = &self.chunks_data[i] {
                chunk_data
            } else {
                location_bytes.put_u32(0);
                timestamp_bytes.put_u32(0);
                continue;
            };

            let chunk_bytes = chunk.to_bytes();
            let sector_count = (chunk_bytes.len() / SECTOR_BYTES) as u32;

            location_bytes.put_u32((current_sector << 8) | sector_count);
            timestamp_bytes.put_u32(self.timestamp_table[i]);

            chunk_data.extend(chunk_bytes);

            current_sector += sector_count;
        }

        [
            location_bytes.as_slice(),
            timestamp_bytes.as_slice(),
            chunk_data.as_slice(),
        ]
        .concat()
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

            //we correct the sectors values
            let bytes_offset = (sector_offset - 2) * SECTOR_BYTES;
            let bytes_count = sector_count * SECTOR_BYTES;

            chunk_file.chunks_data[i] = Some(AnvilChunkData::from_bytes(
                &chunks[bytes_offset..bytes_offset + bytes_count],
            )?);
        }

        Ok(chunk_file)
    }

    fn add_chunks_data(&mut self, chunks_data: &[&Self::Data]) -> Result<(), ChunkWritingError> {
        let mut chunks = chunks_data
            .par_iter()
            .map(|chunk| {
                let chunk_index = Self::get_chunk_index(chunk.position);
                (chunk_index, chunk)
            })
            .collect::<Vec<_>>();

        chunks.par_sort_unstable_by_key(|(index, _)| *index);

        for (chunk_index, chunk_data) in chunks {
            self.chunks_data[chunk_index] = Some(AnvilChunkData::from_chunk(chunk_data)?);
        }

        Ok(())
    }

    fn get_chunks_data(
        &self,
        chunks: &[Vector2<i32>],
    ) -> Vec<LoadedData<Self::Data, ChunkReadingError>> {
        let mut chunks = chunks
            .par_iter()
            .map(|chunk| (Self::get_chunk_index(*chunk), *chunk))
            .collect::<Vec<_>>();

        chunks.par_sort_unstable_by_key(|(index, _)| *index);

        let mut fetched_chunks = Vec::with_capacity(chunks.len());
        for (chunk_index, at) in chunks {
            let chunk = self.chunks_data[chunk_index].as_ref().map_or_else(
                || LoadedData::Missing(at),
                |chunk_data| match chunk_data.to_chunk(at) {
                    Ok(chunk) => LoadedData::Loaded(chunk),
                    Err(err) => LoadedData::Error((at, err)),
                },
            );

            fetched_chunks.push(chunk);
        }

        fetched_chunks
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
        status: super::ChunkStatus::Full,
        heightmaps: chunk_data.heightmap.clone(),
        sections,
    };

    fastnbt::to_bytes(&nbt).map_err(ChunkSerializingError::ErrorSerializingChunk)
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;

    use crate::chunk::anvil::AnvilChunkFile;
    use crate::chunks_io::{ChunkFileManager, ChunkIO, LoadedData};
    use crate::generation::{get_world_gen, Seed};
    use crate::level::LevelFolder;

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile>::default();
        let chunks = chunk_saver.load_chunks(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &[Vector2::new(0, 0)],
        );
        assert!(chunks.len() == 1 && matches!(chunks[0], LoadedData::Missing(_)));
    }

    #[test]
    fn test_writing() {
        let generator = get_world_gen(Seed(0));
        let level_folder = LevelFolder {
            root_folder: PathBuf::from("./tmp_Anvil"),
            region_folder: PathBuf::from("./tmp_Anvil/region"),
        };
        if fs::exists(&level_folder.root_folder).unwrap() {
            fs::remove_dir_all(&level_folder.root_folder).expect("Could not delete directory");
        }

        fs::create_dir_all(&level_folder.region_folder).expect("Could not create directory");
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
                    &chunks
                        .iter()
                        .map(|(at, chunk)| (*at, chunk.clone()))
                        .collect::<Vec<_>>(),
                )
                .expect("Failed to write chunk");

            let read_chunks = chunk_saver
                .load_chunks(
                    &level_folder,
                    &chunks.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
                )
                .into_iter()
                .filter_map(|chunk| match chunk {
                    LoadedData::Loaded(chunk) => Some(chunk),
                    LoadedData::Missing(_) => None,
                    LoadedData::Error((position, error)) => {
                        panic!("Error reading chunk at {:?} | Error: {:?}", position, error)
                    }
                })
                .collect::<Vec<_>>();

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

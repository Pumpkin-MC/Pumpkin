use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
};

use bytes::*;
use fastnbt::LongArray;
use flate2::bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use fs2::FileExt;
use indexmap::IndexMap;
use pumpkin_core::math::ceil_log2;

use crate::{
    block::block_registry::BLOCK_ID_TO_REGISTRY_ID, chunk::ChunkWritingError, level::LevelFolder,
};

use super::{
    ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSection, ChunkSectionBlockStates,
    ChunkSerializingError, ChunkWriter, CompressionError, PaletteEntry,
};

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

impl Compression {
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
}

impl ChunkReader for AnvilChunkFormat {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<super::ChunkData, ChunkReadingError> {
        let region = (at.x >> 5, at.z >> 5);

        let mut region_file = OpenOptions::new()
            .read(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            })?;

        region_file.lock_exclusive().unwrap();

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        // fill the location and timestamp tables
        region_file
            .read_exact(&mut location_table)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        region_file
            .read_exact(&mut timestamp_table)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;
        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = BytesMut::new();
        offset.put_u8(0);
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let offset_at = offset.get_u32() as u64 * 4096;
        let size_at = location_table[table_entry as usize + 3] as usize * 4096;

        if offset_at == 0 && size_at == 0 {
            return Err(ChunkReadingError::ChunkNotExist);
        }

        // Read the file using the offset and size
        let mut file_buf = {
            region_file
                .seek(std::io::SeekFrom::Start(offset_at))
                .map_err(|_| ChunkReadingError::RegionIsInvalid)?;
            let mut out = vec![0; size_at];
            region_file
                .read_exact(&mut out)
                .map_err(|_| ChunkReadingError::RegionIsInvalid)?;
            out
        };

        let mut header: Bytes = file_buf.drain(0..5).collect();
        if header.remaining() != 5 {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let size = header.get_u32();
        let compression = header.get_u8();

        let compression = Compression::from_byte(compression)
            .map_err(|_| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        // size includes the compression scheme byte, so we need to subtract 1
        let chunk_data: Vec<u8> = file_buf.drain(0..size as usize - 1).collect();

        let decompressed_chunk = if let Some(compression) = compression {
            compression
                .decompress_data(&chunk_data)
                .map_err(ChunkReadingError::Compression)?
        } else {
            chunk_data
        };

        ChunkData::from_bytes(&decompressed_chunk, *at).map_err(ChunkReadingError::ParsingError)
    }
}

impl ChunkWriter for AnvilChunkFormat {
    fn write_chunk(
        &self,
        chunk_data: &ChunkData,
        level_folder: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
        // return Ok(()); // REMOVE
        // TODO: update timestamp
        let region = (at.x >> 5, at.z >> 5);

        let mut region_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(
                level_folder
                    .region_folder
                    .join(format!("./r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        region_file.lock_exclusive().unwrap();

        // Serialize chunk data
        let raw_bytes = self
            .to_bytes(chunk_data)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        let compression = Compression::ZLib;
        let compressed_data = compression
            .compress_data(&raw_bytes, 6)
            .map_err(ChunkWritingError::Compression)?;

        // comppressed data + compression type
        let length = compressed_data.len() as u32 + 1;

        // | 0 1 2 3 |        4         |        5..      |
        // | length  | compression type | compressed data |
        let mut chunk_payload = BytesMut::with_capacity(5);
        // Header
        chunk_payload.put_u32(length);
        chunk_payload.put_u8(compression as u8);
        // Payload
        chunk_payload.put_slice(&compressed_data);

        // Calculate sector size
        let sector_size = chunk_payload.len().div_ceil(4096);

        // Region file header tables
        let mut location_table = [0u8; 4096];
        let mut timestamp_table = [0u8; 4096];

        let file_meta = region_file
            .metadata()
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        // The header consists of 8 KiB of data
        // fill the location and timestamp tables if they exist
        if file_meta.len() >= 8192 {
            region_file
                .read_exact(&mut location_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
            region_file
                .read_exact(&mut timestamp_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
        }



        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;

        let table_index = (chunk_x as usize + chunk_z as usize * 32) * 4;

        let mut chunk_data_location: u64;

        // | 0 1 2  |      3       |
        // | offset | sector count |
        let chunk_location = &location_table[table_index..table_index + 4];
        if chunk_location[3] >= sector_size as u8 {
            chunk_data_location = u32::from_be_bytes([0, chunk_location[0], chunk_location[1], chunk_location[2]]) as u64;
        } else {
            chunk_data_location = self.find_free_sector(&location_table, sector_size) as u64;
        }

        assert!(chunk_data_location < 10000 * 4096, "There are way to many sections wtf. Do you wanna blow up your disc?");
        assert!(chunk_data_location > 1, "Nah won't let your overwrite my header. Not cool");

        // Construct location header
        location_table[table_index] = (chunk_data_location >> 16) as u8;
        location_table[table_index + 1] = (chunk_data_location >> 8) as u8;
        location_table[table_index + 2] = (chunk_data_location >> 0) as u8;
        location_table[table_index + 3] = sector_size as u8;

        // Write new location and timestamp table
        region_file.seek(SeekFrom::Start(0)).unwrap();
        region_file
            .write_all(&[location_table, timestamp_table].concat())
            .map_err(|e| ChunkWritingError::IoError(e.kind()))?;

        // Seek to where the chunk is located
        region_file.seek(SeekFrom::Start(chunk_data_location * 4096)).unwrap();

        // Write header and payload
        region_file
            .write_all(&chunk_payload)
            .expect("Failed to write header");

        // length of compression type + payload and 4 for length
        let padding = ((sector_size * 4096) as u32 - ((length + 4) & 0xFFF)) & 0xFFF;

        region_file
            .write_all(&vec![0u8; padding as usize])
            .expect("Failed to add padding");

        region_file.flush().unwrap();
        region_file.unlock().unwrap();

        Ok(())
    }
}

impl AnvilChunkFormat {
    pub fn to_bytes(&self, chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.blocks.iter_subchunks().enumerate() {
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
            // Calculate how many blocks can be packed into a single 64-bit integer
            let _blocks_in_pack = 64 / block_bit_size;

            let mut section_longs = Vec::new();
            let mut current_pack_long: i64 = 0;
            let mut bits_used_in_pack: u32 = 0;

            for block in blocks {
                let index = palette.get(block).expect("Just added all unique").1;
                current_pack_long |= (index as i64) << bits_used_in_pack;
                bits_used_in_pack += block_bit_size as u32;

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
            heightmaps: chunk_data.blocks.heightmap.clone(),
            sections,
        };

        fastnbt::to_bytes(&nbt).map_err(ChunkSerializingError::ErrorSerializingChunk)
    }

    /// Returns the next free writable sector
    /// The sector is absolute which means it always has a spacing of 2 sectors
    fn find_free_sector(&self, location_table: &[u8; 4096], sector_size: usize) -> usize {
        let mut used_sectors: Vec<u16> = Vec::new();
        for i in 0..1024 {
            let entry_offset = i * 4;
            let location_offset =
                u32::from_be_bytes([0, location_table[entry_offset], location_table[entry_offset + 1], location_table[entry_offset + 2]]) as u64;
            let length = location_table[entry_offset + 3] as u64;
            let sector_count = location_offset;
            for used_sector in sector_count..sector_count + length {
                used_sectors.push(used_sector as u16);
            }
        }

        if used_sectors.is_empty() {
            return 3;
        }

        used_sectors.sort();

        let mut prev_sector = &used_sectors[0];
        for sector in used_sectors[1..].iter() { // Iterate over consecutive pairs
            if sector - prev_sector > sector_size as u16 {
                return (prev_sector + 1) as usize;
            }
            prev_sector = sector;
        }

        (used_sectors.last().unwrap().clone() + 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pumpkin_core::math::vector2::Vector2;

    use crate::{
        chunk::{anvil::AnvilChunkFormat, ChunkReader, ChunkReadingError},
        level::LevelFolder,
    };

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let result = AnvilChunkFormat.read_chunk(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &Vector2::new(0, 0),
        );
        assert!(matches!(result, Err(ChunkReadingError::ChunkNotExist)));
    }
}

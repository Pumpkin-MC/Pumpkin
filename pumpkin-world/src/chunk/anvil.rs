use bytes::*;
use flate2::bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use indexmap::IndexMap;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_nbt::serializer::to_bytes;
use pumpkin_util::math::ceil_log2;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
};

use crate::block::registry::STATE_ID_TO_REGISTRY_ID;
use crate::{chunk::ChunkWritingError, level::LevelFolder};

use super::{
    ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSection, ChunkSectionBlockStates,
    ChunkSerializingError, ChunkWriter, CompressionError, PaletteEntry,
};

// 1.21.4
const WORLD_DATA_VERSION: i32 = 4189;

#[derive(Clone, Default)]
pub struct AnvilChunkFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        at: &pumpkin_util::math::vector2::Vector2<i32>,
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
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
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

        // Serialize chunk data
        let raw_bytes = Self::to_bytes(chunk_data)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        // Compress chunk data
        let compression: Compression = ADVANCED_CONFIG.chunk.compression.algorithm.clone().into();
        let compressed_data = compression
            .compress_data(&raw_bytes, ADVANCED_CONFIG.chunk.compression.level)
            .map_err(ChunkWritingError::Compression)?;

        // Length of compressed data + compression type
        let length = compressed_data.len() as u32 + 1;

        // | 0 1 2 3 |        4         |        5..      |
        // | length  | compression type | compressed data |
        let mut chunk_payload = BytesMut::with_capacity(5);
        // Payload Header + Body
        chunk_payload.put_u32(length);
        chunk_payload.put_u8(compression as u8);
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
        // Try to fill the location and timestamp tables if they already exist
        if file_meta.len() >= 8192 {
            region_file
                .read_exact(&mut location_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
            region_file
                .read_exact(&mut timestamp_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
        }

        // Get location table index
        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;
        let table_index = (chunk_x as usize + chunk_z as usize * 32) * 4;

        // | 0 1 2  |      3       |
        // | offset | sector count |
        // Get the entry from the current location table and check
        // if the new chunk fits in the space of the old chunk
        let chunk_location = &location_table[table_index..table_index + 4];
        let chunk_data_location: u64 = if chunk_location[3] >= sector_size as u8 {
            // Return old chunk location
            u32::from_be_bytes([0, chunk_location[0], chunk_location[1], chunk_location[2]]) as u64
        } else {
            // Retrieve next writable sector
            self.find_free_sector(&location_table, sector_size) as u64
        };

        assert!(
            chunk_data_location > 1,
            "This should never happen. The header would be corrupted"
        );

        // Construct location header
        location_table[table_index] = (chunk_data_location >> 16) as u8;
        location_table[table_index + 1] = (chunk_data_location >> 8) as u8;
        location_table[table_index + 2] = chunk_data_location as u8;
        location_table[table_index + 3] = sector_size as u8;

        // Get epoch may result in errors if after the year 2106 :(
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // Construct timestamp header
        timestamp_table[table_index] = (epoch >> 24) as u8;
        timestamp_table[table_index + 1] = (epoch >> 16) as u8;
        timestamp_table[table_index + 2] = (epoch >> 8) as u8;
        timestamp_table[table_index + 3] = epoch as u8;

        // Write new location and timestamp table
        region_file.seek(SeekFrom::Start(0)).unwrap();
        region_file
            .write_all(&[location_table, timestamp_table].concat())
            .map_err(|e| ChunkWritingError::IoError(e.kind()))?;

        // Seek to where the chunk is located
        region_file
            .seek(SeekFrom::Start(chunk_data_location * 4096))
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        // Write header and payload
        region_file
            .write_all(&chunk_payload)
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        // Calculate padding to fill the sectors
        // (length + 4) 3 bits for length and 1 for compression type + payload length
        let padding = ((sector_size * 4096) as u32 - ((length + 4) & 0xFFF)) & 0xFFF;

        // Write padding
        region_file
            .write_all(&vec![0u8; padding as usize])
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        region_file
            .flush()
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        Ok(())
    }
}

impl AnvilChunkFormat {
    pub fn to_bytes(chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
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
                            name: entry.1 .0.to_string(),
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

    /// Returns the next free writable sector
    /// The sector is absolute which means it always has a spacing of 2 sectors
    fn find_free_sector(&self, location_table: &[u8; 4096], sector_size: usize) -> usize {
        let mut used_sectors: Vec<u16> = Vec::new();
        for i in 0..1024 {
            let entry_offset = i * 4;
            let location_offset = u32::from_be_bytes([
                0,
                location_table[entry_offset],
                location_table[entry_offset + 1],
                location_table[entry_offset + 2],
            ]) as u64;
            let length = location_table[entry_offset + 3] as u64;
            let sector_count = location_offset;
            for used_sector in sector_count..sector_count + length {
                used_sectors.push(used_sector as u16);
            }
        }

        if used_sectors.is_empty() {
            return 2;
        }

        used_sectors.sort();

        let mut prev_sector = &used_sectors[0];
        for sector in used_sectors[1..].iter() {
            // Iterate over consecutive pairs
            if sector - prev_sector > sector_size as u16 {
                return (prev_sector + 1) as usize;
            }
            prev_sector = sector;
        }

        (*used_sectors.last().unwrap() + 1) as usize
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use temp_dir::TempDir;

    use crate::chunk::ChunkWriter;
    use crate::generation::{get_world_gen, Seed};
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

    #[test]
    fn test_writing() {
        let generator = get_world_gen(Seed(0));

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
        };
        fs::create_dir(&level_folder.region_folder).expect("couldn't create region folder");

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
                AnvilChunkFormat
                    .write_chunk(chunk, &level_folder, at)
                    .expect("Failed to write chunk");
            }

            let mut read_chunks = vec![];
            for (at, _chunk) in &chunks {
                read_chunks.push(
                    AnvilChunkFormat
                        .read_chunk(&level_folder, at)
                        .expect("Could not read chunk"),
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

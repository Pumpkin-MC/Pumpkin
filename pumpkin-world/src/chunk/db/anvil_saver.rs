use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::chunk::{compression::Compression, db::ChunkStorageWritingError};

use super::{ChunkStorage, ChunkStorageReadingError, CompressionError};

pub struct AnvilSaver;

impl ChunkStorage for AnvilSaver {
    fn read_raw_chunk(
        &self,
        save_file: &crate::level::LevelFolder,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<Vec<u8>, ChunkStorageReadingError> {
        let region = (at.x >> 5, at.z >> 5);

        let mut region_file = OpenOptions::new()
            .read(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkStorageReadingError::ChunkNotExist,
                kind => ChunkStorageReadingError::IoError(kind),
            })?;

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        // fill the location and timestamp tables
        region_file
            .read_exact(&mut location_table)
            .map_err(|err| ChunkStorageReadingError::IoError(err.kind()))?;
        region_file
            .read_exact(&mut timestamp_table)
            .map_err(|err| ChunkStorageReadingError::IoError(err.kind()))?;

        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;
        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = BytesMut::new();
        offset.put_u8(0);
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let offset_at = offset.get_u32() as u64 * 4096;
        let size_at = location_table[table_entry as usize + 3] as usize * 4096;

        if offset_at == 0 && size_at == 0 {
            return Err(ChunkStorageReadingError::ChunkNotExist);
        }

        // Read the file using the offset and size
        let mut file_buf = {
            region_file
                .seek(std::io::SeekFrom::Start(offset_at))
                .map_err(|_| ChunkStorageReadingError::RegionIsInvalid)?;
            let mut out = vec![0; size_at];
            region_file
                .read_exact(&mut out)
                .map_err(|_| ChunkStorageReadingError::RegionIsInvalid)?;
            out
        };

        let mut header: Bytes = file_buf.drain(0..5).collect();
        if header.remaining() != 5 {
            return Err(ChunkStorageReadingError::InvalidHeader);
        }

        let size = header.get_u32();
        let compression = header.get_u8();

        let compression = Compression::from_byte(compression).map_err(|_| {
            ChunkStorageReadingError::Compression(CompressionError::UnknownCompression)
        })?;

        // size includes the compression scheme byte, so we need to subtract 1
        let chunk_data: Vec<u8> = file_buf.drain(0..size as usize - 1).collect();

        let decompressed_chunk = if let Some(compression) = compression {
            compression
                .decompress_data(&chunk_data)
                .map_err(ChunkStorageReadingError::Compression)?
        } else {
            chunk_data
        };

        Ok(decompressed_chunk)
    }

    fn write_raw_chunk(
        &self,
        chunk: Vec<u8>,
        level_folder: &crate::level::LevelFolder,
        at: &pumpkin_util::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkStorageWritingError> {
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
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        // Compress chunk data
        let compression = Compression::ZLib;
        let compressed_data = compression
            .compress_data(&chunk, 6)
            .map_err(ChunkStorageWritingError::Compression)?;

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
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        // The header consists of 8 KiB of data
        // Try to fill the location and timestamp tables if they already exist
        if file_meta.len() >= 8192 {
            region_file
                .read_exact(&mut location_table)
                .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;
            region_file
                .read_exact(&mut timestamp_table)
                .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;
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
            .map_err(|e| ChunkStorageWritingError::IoError(e.kind()))?;

        // Seek to where the chunk is located
        region_file
            .seek(SeekFrom::Start(chunk_data_location * 4096))
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        // Write header and payload
        region_file
            .write_all(&chunk_payload)
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        // Calculate padding to fill the sectors
        // (length + 4) 3 bits for length and 1 for compression type + payload length
        let padding = ((sector_size * 4096) as u32 - ((length + 4) & 0xFFF)) & 0xFFF;

        // Write padding
        region_file
            .write_all(&vec![0u8; padding as usize])
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        region_file
            .flush()
            .map_err(|err| ChunkStorageWritingError::IoError(err.kind()))?;

        Ok(())
    }
}

impl AnvilSaver {
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

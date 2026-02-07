//! Anvil region file format support.
//!
//! This module provides low-level reading and writing of Minecraft's Anvil
//! region file format (`.mca` files). A region file stores up to 1024 chunks
//! (32x32) using a sector-based allocation scheme.
//!
//! # Region File Layout
//!
//! ```text
//! Offset   Size   Description
//! 0        4096   Location table (1024 entries × 4 bytes)
//! 4096     4096   Timestamp table (1024 entries × 4 bytes)
//! 8192+    ...    Chunk data sectors (4096 bytes each)
//! ```
//!
//! Each location entry encodes the sector offset (upper 24 bits) and
//! sector count (lower 8 bits). Chunk data is preceded by a 5-byte header:
//! 4 bytes for the data length (including compression byte) and 1 byte for
//! the compression method.
//!
//! # Compression Methods
//!
//! | ID | Method      |
//! |----|-------------|
//! | 1  | `GZip`      |
//! | 2  | `ZLib`      |
//! | 3  | Uncompressed|
//!
//! # Example
//!
//! ```no_run
//! use pumpkin_nbt::anvil::RegionFile;
//!
//! // Open a region file
//! let data = std::fs::read("r.0.0.mca").unwrap();
//! let region = RegionFile::from_bytes(&data).unwrap();
//!
//! // Read chunk at local coordinates (0, 0)
//! if let Some(chunk_data) = region.read_chunk(0, 0).unwrap() {
//!     // chunk_data is decompressed NBT bytes
//!     println!("Chunk has {} bytes", chunk_data.len());
//! }
//! ```

use std::io::{self, Read, Write};

use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};

/// Number of chunks per region side (32×32 = 1024 chunks per region).
pub const REGION_SIZE: usize = 32;

/// Total number of chunk slots in a region file.
pub const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// Size of a single sector in bytes (4 KiB).
pub const SECTOR_BYTES: usize = 4096;

/// Number of header sectors (location table + timestamp table).
pub const HEADER_SECTORS: usize = 2;

/// Byte offset where chunk data sectors begin.
pub const DATA_OFFSET: usize = HEADER_SECTORS * SECTOR_BYTES;

/// Compression methods used in Anvil region files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMethod {
    /// `GZip` compression (ID 1).
    GZip = 1,
    /// ZLib/Deflate compression (ID 2). This is the default used by vanilla.
    ZLib = 2,
    /// No compression (ID 3).
    None = 3,
}

impl CompressionMethod {
    /// Parse a compression method from its byte ID.
    pub const fn from_id(id: u8) -> Result<Self, AnvilError> {
        match id {
            1 => Ok(Self::GZip),
            2 => Ok(Self::ZLib),
            3 => Ok(Self::None),
            _ => Err(AnvilError::UnknownCompression(id)),
        }
    }
}

/// Errors that can occur during Anvil region file operations.
#[derive(Debug)]
pub enum AnvilError {
    /// The region file data is too small to contain the header tables.
    FileTooSmall(usize),
    /// Chunk coordinates are out of bounds (must be 0..31).
    ChunkOutOfBounds(u8, u8),
    /// The sector offset points outside the file.
    SectorOutOfBounds {
        chunk_x: u8,
        chunk_z: u8,
        offset: u32,
        file_len: usize,
    },
    /// The chunk data header specifies a length larger than the allocated sectors.
    ChunkDataTooLarge {
        chunk_x: u8,
        chunk_z: u8,
        data_len: u32,
        available: usize,
    },
    /// An unknown compression method ID was encountered.
    UnknownCompression(u8),
    /// An I/O error occurred during decompression or other operations.
    Io(io::Error),
    /// The chunk data length in the header is zero or invalid.
    InvalidDataLength(u32),
}

impl std::fmt::Display for AnvilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileTooSmall(size) => {
                write!(
                    f,
                    "Region file too small: {size} bytes (minimum {DATA_OFFSET})"
                )
            }
            Self::ChunkOutOfBounds(x, z) => {
                write!(
                    f,
                    "Chunk coordinates out of bounds: ({x}, {z}), must be 0..31"
                )
            }
            Self::SectorOutOfBounds {
                chunk_x,
                chunk_z,
                offset,
                file_len,
            } => write!(
                f,
                "Sector offset {offset} for chunk ({chunk_x}, {chunk_z}) points past file end ({file_len} bytes)"
            ),
            Self::ChunkDataTooLarge {
                chunk_x,
                chunk_z,
                data_len,
                available,
            } => write!(
                f,
                "Chunk ({chunk_x}, {chunk_z}) data length {data_len} exceeds available sector space {available}"
            ),
            Self::UnknownCompression(id) => {
                write!(f, "Unknown compression method: {id}")
            }
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::InvalidDataLength(len) => {
                write!(f, "Invalid chunk data length: {len}")
            }
        }
    }
}

impl std::error::Error for AnvilError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for AnvilError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Location table entry for a single chunk within a region file.
///
/// Encodes the starting sector (offset from file start, in 4 KiB sectors)
/// and the number of sectors allocated for the chunk data.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkLocation {
    /// Sector offset from the beginning of the file. A value of 0 means the
    /// chunk is not present.
    pub offset: u32,
    /// Number of 4 KiB sectors allocated for the chunk.
    pub sector_count: u8,
}

impl ChunkLocation {
    /// Returns `true` if this chunk slot is empty (offset and sector count are both 0).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.offset == 0 && self.sector_count == 0
    }

    /// Decode a location entry from 4 big-endian bytes.
    const fn from_bytes(bytes: [u8; 4]) -> Self {
        let offset = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let sector_count = bytes[3];
        Self {
            offset,
            sector_count,
        }
    }

    /// Encode this location entry as 4 big-endian bytes.
    const fn to_bytes(self) -> [u8; 4] {
        [
            ((self.offset >> 16) & 0xFF) as u8,
            ((self.offset >> 8) & 0xFF) as u8,
            (self.offset & 0xFF) as u8,
            self.sector_count,
        ]
    }
}

/// A parsed Anvil region file.
///
/// Provides random access to individual chunk data by local coordinates
/// (0..31, 0..31). Chunk data is stored compressed and is decompressed
/// on read.
pub struct RegionFile {
    /// Location table: 1024 entries mapping chunk positions to sector offsets.
    pub locations: [ChunkLocation; CHUNK_COUNT],
    /// Timestamp table: 1024 Unix timestamps (seconds since epoch).
    pub timestamps: [u32; CHUNK_COUNT],
    /// Raw file data (including headers). Chunk sectors are read from this.
    data: Vec<u8>,
}

impl RegionFile {
    /// Parse a region file from a byte slice.
    ///
    /// The input must contain at least the 8 KiB header (location + timestamp
    /// tables). Chunk data sectors follow the header.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AnvilError> {
        if bytes.len() < DATA_OFFSET {
            return Err(AnvilError::FileTooSmall(bytes.len()));
        }

        let mut locations = [ChunkLocation::default(); CHUNK_COUNT];
        let mut timestamps = [0u32; CHUNK_COUNT];

        // Parse location table (first 4096 bytes)
        for (i, location) in locations.iter_mut().enumerate() {
            let base = i * 4;
            let entry_bytes = [
                bytes[base],
                bytes[base + 1],
                bytes[base + 2],
                bytes[base + 3],
            ];
            *location = ChunkLocation::from_bytes(entry_bytes);
        }

        // Parse timestamp table (second 4096 bytes)
        for (i, timestamp) in timestamps.iter_mut().enumerate() {
            let base = SECTOR_BYTES + i * 4;
            *timestamp = u32::from_be_bytes([
                bytes[base],
                bytes[base + 1],
                bytes[base + 2],
                bytes[base + 3],
            ]);
        }

        Ok(Self {
            locations,
            timestamps,
            data: bytes.to_vec(),
        })
    }

    /// Create a new empty region file with no chunks.
    #[must_use]
    pub fn new() -> Self {
        let mut data = vec![0u8; DATA_OFFSET];
        // The header is already zeroed, meaning all chunks are absent.
        // Ensure data is exactly 8192 bytes (2 sectors for headers).
        data.resize(DATA_OFFSET, 0);

        Self {
            locations: [ChunkLocation::default(); CHUNK_COUNT],
            timestamps: [0u32; CHUNK_COUNT],
            data,
        }
    }

    /// Compute the index into the location/timestamp tables for the given
    /// local chunk coordinates.
    const fn chunk_index(x: u8, z: u8) -> Result<usize, AnvilError> {
        if x >= REGION_SIZE as u8 || z >= REGION_SIZE as u8 {
            return Err(AnvilError::ChunkOutOfBounds(x, z));
        }
        Ok((x as usize) + (z as usize) * REGION_SIZE)
    }

    /// Convert world-space chunk coordinates to region-local coordinates.
    ///
    /// World chunk (cx, cz) maps to region (cx >> 5, cz >> 5), and local
    /// coordinates are (cx & 31, cz & 31).
    #[must_use]
    pub const fn world_to_local(chunk_x: i32, chunk_z: i32) -> (u8, u8) {
        ((chunk_x & 31) as u8, (chunk_z & 31) as u8)
    }

    /// Convert world-space chunk coordinates to the region file coordinates.
    ///
    /// Returns (`region_x`, `region_z`) suitable for constructing the filename
    /// `r.{region_x}.{region_z}.mca`.
    #[must_use]
    pub const fn chunk_to_region(chunk_x: i32, chunk_z: i32) -> (i32, i32) {
        (chunk_x >> 5, chunk_z >> 5)
    }

    /// Check whether a chunk exists at the given local coordinates.
    pub fn has_chunk(&self, x: u8, z: u8) -> Result<bool, AnvilError> {
        let idx = Self::chunk_index(x, z)?;
        Ok(!self.locations[idx].is_empty())
    }

    /// Get the timestamp for a chunk at the given local coordinates.
    ///
    /// Returns 0 if the chunk has never been saved.
    pub fn get_timestamp(&self, x: u8, z: u8) -> Result<u32, AnvilError> {
        let idx = Self::chunk_index(x, z)?;
        Ok(self.timestamps[idx])
    }

    /// Read and decompress chunk data at the given local coordinates.
    ///
    /// Returns `None` if the chunk slot is empty. The returned bytes are
    /// the decompressed NBT data (ready to be parsed with `Nbt::read` or
    /// the serde deserializer).
    pub fn read_chunk(&self, x: u8, z: u8) -> Result<Option<Vec<u8>>, AnvilError> {
        let idx = Self::chunk_index(x, z)?;
        let loc = self.locations[idx];

        if loc.is_empty() {
            return Ok(None);
        }

        let byte_offset = loc.offset as usize * SECTOR_BYTES;
        let max_bytes = loc.sector_count as usize * SECTOR_BYTES;

        // Validate the offset is within the file
        if byte_offset + max_bytes > self.data.len() {
            return Err(AnvilError::SectorOutOfBounds {
                chunk_x: x,
                chunk_z: z,
                offset: loc.offset,
                file_len: self.data.len(),
            });
        }

        let sector_data = &self.data[byte_offset..byte_offset + max_bytes];

        // Read chunk header: 4 bytes length + 1 byte compression
        if sector_data.len() < 5 {
            return Err(AnvilError::InvalidDataLength(0));
        }

        let data_len = u32::from_be_bytes([
            sector_data[0],
            sector_data[1],
            sector_data[2],
            sector_data[3],
        ]);

        if data_len < 1 {
            return Err(AnvilError::InvalidDataLength(data_len));
        }

        // data_len includes the compression byte
        let total_data = data_len as usize;
        if total_data + 4 > sector_data.len() {
            return Err(AnvilError::ChunkDataTooLarge {
                chunk_x: x,
                chunk_z: z,
                data_len,
                available: sector_data.len() - 4,
            });
        }

        let compression = CompressionMethod::from_id(sector_data[4])?;
        let compressed = &sector_data[5..4 + total_data];

        decompress(compressed, compression)
            .map(Some)
            .map_err(AnvilError::Io)
    }

    /// Write (compressed) chunk data at the given local coordinates.
    ///
    /// The `nbt_data` should be the raw NBT bytes (not compressed). This method
    /// compresses them with the specified method and rebuilds the region file
    /// with updated sector allocation.
    ///
    /// The `timestamp` is the Unix timestamp (seconds since epoch) for the chunk.
    pub fn write_chunk(
        &mut self,
        x: u8,
        z: u8,
        nbt_data: &[u8],
        compression: CompressionMethod,
        timestamp: u32,
    ) -> Result<(), AnvilError> {
        let idx = Self::chunk_index(x, z)?;

        let compressed = compress(nbt_data, compression)?;

        // Chunk header: 4 bytes length + 1 byte compression + compressed data
        // length field includes the compression byte
        let data_len = compressed.len() as u32 + 1;
        let total_with_header = 4 + data_len as usize;

        // Calculate sectors needed (round up to SECTOR_BYTES)
        let sectors_needed = total_with_header.div_ceil(SECTOR_BYTES);
        if sectors_needed > 255 {
            return Err(AnvilError::ChunkDataTooLarge {
                chunk_x: x,
                chunk_z: z,
                data_len,
                available: 255 * SECTOR_BYTES,
            });
        }

        // Build the chunk sector data (padded to sector boundary)
        let padded_len = sectors_needed * SECTOR_BYTES;
        let mut chunk_sector = vec![0u8; padded_len];

        // Write header
        let len_bytes = data_len.to_be_bytes();
        chunk_sector[0..4].copy_from_slice(&len_bytes);
        chunk_sector[4] = compression as u8;
        chunk_sector[5..5 + compressed.len()].copy_from_slice(&compressed);

        // Rebuild the file: collect all existing chunk sectors, replace/add the target chunk
        let mut chunks: Vec<(usize, Vec<u8>)> = Vec::new(); // (index, sector_data)
        for i in 0..CHUNK_COUNT {
            if i == idx {
                // Will be replaced
                continue;
            }
            let loc = self.locations[i];
            if loc.is_empty() {
                continue;
            }
            let start = loc.offset as usize * SECTOR_BYTES;
            let end = start + loc.sector_count as usize * SECTOR_BYTES;
            if end <= self.data.len() {
                chunks.push((i, self.data[start..end].to_vec()));
            }
        }

        // Add the new/updated chunk
        chunks.push((idx, chunk_sector));

        // Sort by index for deterministic output
        chunks.sort_by_key(|(i, _)| *i);

        // Rebuild the file
        let mut new_data = vec![0u8; DATA_OFFSET]; // Start with empty headers
        let mut new_locations = [ChunkLocation::default(); CHUNK_COUNT];
        let mut new_timestamps = self.timestamps;

        let mut current_sector = HEADER_SECTORS as u32;

        for (i, sector_data) in &chunks {
            let sector_count = (sector_data.len() / SECTOR_BYTES) as u8;
            new_locations[*i] = ChunkLocation {
                offset: current_sector,
                sector_count,
            };
            new_data.extend_from_slice(sector_data);
            current_sector += sector_count as u32;
        }

        // Update timestamp for the written chunk
        new_timestamps[idx] = timestamp;

        // Write location table into header
        for (i, loc) in new_locations.iter().enumerate() {
            let bytes = loc.to_bytes();
            let base = i * 4;
            new_data[base..base + 4].copy_from_slice(&bytes);
        }

        // Write timestamp table into header
        for (i, ts) in new_timestamps.iter().enumerate() {
            let bytes = ts.to_be_bytes();
            let base = SECTOR_BYTES + i * 4;
            new_data[base..base + 4].copy_from_slice(&bytes);
        }

        self.data = new_data;
        self.locations = new_locations;
        self.timestamps = new_timestamps;

        Ok(())
    }

    /// Remove a chunk from the region file.
    ///
    /// Clears the location and timestamp entries. The sector data is not
    /// immediately reclaimed but will be omitted on the next `write_chunk`
    /// or `to_bytes` call if the file is rebuilt.
    pub fn remove_chunk(&mut self, x: u8, z: u8) -> Result<(), AnvilError> {
        let idx = Self::chunk_index(x, z)?;
        self.locations[idx] = ChunkLocation::default();
        self.timestamps[idx] = 0;
        Ok(())
    }

    /// Serialize the region file to bytes.
    ///
    /// Returns the complete file contents suitable for writing to disk.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Write the region file to a writer.
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.data)
    }

    /// Count the number of chunks present in this region file.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.locations.iter().filter(|loc| !loc.is_empty()).count()
    }

    /// Iterate over all present chunks, yielding (`local_x`, `local_z`) pairs.
    pub fn present_chunks(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.locations.iter().enumerate().filter_map(|(i, loc)| {
            if loc.is_empty() {
                None
            } else {
                let x = (i % REGION_SIZE) as u8;
                let z = (i / REGION_SIZE) as u8;
                Some((x, z))
            }
        })
    }
}

impl Default for RegionFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Decompress data using the specified compression method.
fn decompress(data: &[u8], method: CompressionMethod) -> Result<Vec<u8>, io::Error> {
    match method {
        CompressionMethod::GZip => {
            let mut decoder = GzDecoder::new(data);
            let mut output = Vec::new();
            decoder.read_to_end(&mut output)?;
            Ok(output)
        }
        CompressionMethod::ZLib => {
            let mut decoder = ZlibDecoder::new(data);
            let mut output = Vec::new();
            decoder.read_to_end(&mut output)?;
            Ok(output)
        }
        CompressionMethod::None => Ok(data.to_vec()),
    }
}

/// Compress data using the specified compression method.
fn compress(data: &[u8], method: CompressionMethod) -> Result<Vec<u8>, AnvilError> {
    match method {
        CompressionMethod::GZip => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        }
        CompressionMethod::ZLib => {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        }
        CompressionMethod::None => Ok(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_region_file_is_empty() {
        let region = RegionFile::new();
        assert_eq!(region.chunk_count(), 0);
        assert_eq!(region.to_bytes().len(), DATA_OFFSET);
    }

    #[test]
    fn chunk_index_bounds() {
        assert!(RegionFile::chunk_index(0, 0).is_ok());
        assert!(RegionFile::chunk_index(31, 31).is_ok());
        assert!(RegionFile::chunk_index(32, 0).is_err());
        assert!(RegionFile::chunk_index(0, 32).is_err());
    }

    #[test]
    fn world_to_local_conversion() {
        assert_eq!(RegionFile::world_to_local(0, 0), (0, 0));
        assert_eq!(RegionFile::world_to_local(31, 31), (31, 31));
        assert_eq!(RegionFile::world_to_local(32, 32), (0, 0));
        assert_eq!(RegionFile::world_to_local(-1, -1), (31, 31));
        assert_eq!(RegionFile::world_to_local(33, 65), (1, 1));
    }

    #[test]
    fn chunk_to_region_conversion() {
        assert_eq!(RegionFile::chunk_to_region(0, 0), (0, 0));
        assert_eq!(RegionFile::chunk_to_region(31, 31), (0, 0));
        assert_eq!(RegionFile::chunk_to_region(32, 32), (1, 1));
        assert_eq!(RegionFile::chunk_to_region(-1, -1), (-1, -1));
        assert_eq!(RegionFile::chunk_to_region(-32, -32), (-1, -1));
        assert_eq!(RegionFile::chunk_to_region(-33, -33), (-2, -2));
    }

    #[test]
    fn location_entry_roundtrip() {
        let loc = ChunkLocation {
            offset: 12345,
            sector_count: 3,
        };
        let bytes = loc.to_bytes();
        let decoded = ChunkLocation::from_bytes(bytes);
        assert_eq!(decoded.offset, loc.offset);
        assert_eq!(decoded.sector_count, loc.sector_count);
    }

    #[test]
    fn location_entry_empty() {
        let loc = ChunkLocation::default();
        assert!(loc.is_empty());

        let loc = ChunkLocation {
            offset: 1,
            sector_count: 0,
        };
        assert!(!loc.is_empty());
    }

    #[test]
    fn write_and_read_chunk_zlib() {
        let mut region = RegionFile::new();
        let nbt_data = b"Hello, this is some test NBT data for chunk storage!";

        region
            .write_chunk(0, 0, nbt_data, CompressionMethod::ZLib, 1000)
            .unwrap();

        assert!(region.has_chunk(0, 0).unwrap());
        assert!(!region.has_chunk(1, 0).unwrap());
        assert_eq!(region.get_timestamp(0, 0).unwrap(), 1000);
        assert_eq!(region.chunk_count(), 1);

        let read_data = region.read_chunk(0, 0).unwrap().unwrap();
        assert_eq!(read_data, nbt_data);
    }

    #[test]
    fn write_and_read_chunk_gzip() {
        let mut region = RegionFile::new();
        let nbt_data = b"GZip compressed chunk test data";

        region
            .write_chunk(5, 10, nbt_data, CompressionMethod::GZip, 2000)
            .unwrap();

        let read_data = region.read_chunk(5, 10).unwrap().unwrap();
        assert_eq!(read_data, nbt_data);
    }

    #[test]
    fn write_and_read_chunk_uncompressed() {
        let mut region = RegionFile::new();
        let nbt_data = b"Uncompressed chunk data";

        region
            .write_chunk(31, 31, nbt_data, CompressionMethod::None, 3000)
            .unwrap();

        let read_data = region.read_chunk(31, 31).unwrap().unwrap();
        assert_eq!(read_data, nbt_data);
    }

    #[test]
    fn read_empty_chunk_returns_none() {
        let region = RegionFile::new();
        assert_eq!(region.read_chunk(0, 0).unwrap(), None);
    }

    #[test]
    fn multiple_chunks() {
        let mut region = RegionFile::new();

        for i in 0..10u8 {
            let data = format!("chunk data {i}");
            region
                .write_chunk(
                    i,
                    0,
                    data.as_bytes(),
                    CompressionMethod::ZLib,
                    i as u32 * 100,
                )
                .unwrap();
        }

        assert_eq!(region.chunk_count(), 10);

        for i in 0..10u8 {
            let expected = format!("chunk data {i}");
            let read_data = region.read_chunk(i, 0).unwrap().unwrap();
            assert_eq!(read_data, expected.as_bytes());
            assert_eq!(region.get_timestamp(i, 0).unwrap(), i as u32 * 100);
        }
    }

    #[test]
    fn overwrite_chunk() {
        let mut region = RegionFile::new();

        region
            .write_chunk(5, 5, b"first", CompressionMethod::ZLib, 100)
            .unwrap();
        region
            .write_chunk(5, 5, b"second version", CompressionMethod::ZLib, 200)
            .unwrap();

        let data = region.read_chunk(5, 5).unwrap().unwrap();
        assert_eq!(data, b"second version");
        assert_eq!(region.get_timestamp(5, 5).unwrap(), 200);
        assert_eq!(region.chunk_count(), 1);
    }

    #[test]
    fn remove_chunk() {
        let mut region = RegionFile::new();

        region
            .write_chunk(3, 3, b"data", CompressionMethod::ZLib, 100)
            .unwrap();
        assert_eq!(region.chunk_count(), 1);

        region.remove_chunk(3, 3).unwrap();
        assert!(!region.has_chunk(3, 3).unwrap());
        assert_eq!(region.chunk_count(), 0);
    }

    #[test]
    fn present_chunks_iterator() {
        let mut region = RegionFile::new();

        region
            .write_chunk(0, 0, b"a", CompressionMethod::None, 0)
            .unwrap();
        region
            .write_chunk(5, 10, b"b", CompressionMethod::None, 0)
            .unwrap();
        region
            .write_chunk(31, 31, b"c", CompressionMethod::None, 0)
            .unwrap();

        let present: Vec<(u8, u8)> = region.present_chunks().collect();
        assert_eq!(present.len(), 3);
        assert!(present.contains(&(0, 0)));
        assert!(present.contains(&(5, 10)));
        assert!(present.contains(&(31, 31)));
    }

    #[test]
    fn serialize_and_reparse() {
        let mut region = RegionFile::new();

        region
            .write_chunk(0, 0, b"chunk 0,0", CompressionMethod::ZLib, 100)
            .unwrap();
        region
            .write_chunk(15, 15, b"chunk 15,15", CompressionMethod::GZip, 200)
            .unwrap();
        region
            .write_chunk(31, 0, b"chunk 31,0", CompressionMethod::None, 300)
            .unwrap();

        let bytes = region.to_bytes();
        let reparsed = RegionFile::from_bytes(&bytes).unwrap();

        assert_eq!(reparsed.chunk_count(), 3);
        assert_eq!(reparsed.read_chunk(0, 0).unwrap().unwrap(), b"chunk 0,0");
        assert_eq!(
            reparsed.read_chunk(15, 15).unwrap().unwrap(),
            b"chunk 15,15"
        );
        assert_eq!(reparsed.read_chunk(31, 0).unwrap().unwrap(), b"chunk 31,0");
        assert_eq!(reparsed.get_timestamp(0, 0).unwrap(), 100);
        assert_eq!(reparsed.get_timestamp(15, 15).unwrap(), 200);
        assert_eq!(reparsed.get_timestamp(31, 0).unwrap(), 300);
    }

    #[test]
    fn file_too_small_error() {
        let result = RegionFile::from_bytes(&[0u8; 100]);
        assert!(result.is_err());
    }

    #[test]
    fn compression_method_roundtrip() {
        assert_eq!(
            CompressionMethod::from_id(1).unwrap(),
            CompressionMethod::GZip
        );
        assert_eq!(
            CompressionMethod::from_id(2).unwrap(),
            CompressionMethod::ZLib
        );
        assert_eq!(
            CompressionMethod::from_id(3).unwrap(),
            CompressionMethod::None
        );
        assert!(CompressionMethod::from_id(0).is_err());
        assert!(CompressionMethod::from_id(4).is_err());
    }

    #[test]
    fn large_chunk_data() {
        let mut region = RegionFile::new();
        // Write a chunk with significant data (100KB)
        let large_data = vec![42u8; 100_000];

        region
            .write_chunk(0, 0, &large_data, CompressionMethod::ZLib, 999)
            .unwrap();

        let read_data = region.read_chunk(0, 0).unwrap().unwrap();
        assert_eq!(read_data, large_data);
    }

    // --- Edge case and hardening tests ---

    #[test]
    fn header_only_file() {
        // A valid file with just the 8KB header and no chunk data
        let data = vec![0u8; DATA_OFFSET];
        let region = RegionFile::from_bytes(&data).unwrap();
        assert_eq!(region.chunk_count(), 0);
        for x in 0..32u8 {
            for z in 0..32u8 {
                assert!(!region.has_chunk(x, z).unwrap());
            }
        }
    }

    #[test]
    fn corrupted_location_pointing_past_file() {
        let mut data = vec![0u8; DATA_OFFSET];
        // Set chunk (0,0) location to sector offset 100, count 1
        // but the file only has 2 sectors (header)
        data[0] = 0;
        data[1] = 0;
        data[2] = 100; // offset = 100
        data[3] = 1; // sector_count = 1

        let region = RegionFile::from_bytes(&data).unwrap();
        assert!(region.has_chunk(0, 0).unwrap());
        let result = region.read_chunk(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn zero_data_length_in_sector() {
        let mut data = vec![0u8; DATA_OFFSET + SECTOR_BYTES];
        // Location: sector 2, count 1
        data[0] = 0;
        data[1] = 0;
        data[2] = 2;
        data[3] = 1;
        // Chunk header at sector 2: data_len = 0 (already zeroed)

        let region = RegionFile::from_bytes(&data).unwrap();
        let result = region.read_chunk(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn data_length_exceeds_sector_allocation() {
        let mut data = vec![0u8; DATA_OFFSET + SECTOR_BYTES];
        data[0] = 0;
        data[1] = 0;
        data[2] = 2;
        data[3] = 1; // 1 sector = 4096 bytes
        // Chunk header: data_len = 5000 (exceeds single sector)
        let len_bytes = 5000u32.to_be_bytes();
        let base = DATA_OFFSET;
        data[base..base + 4].copy_from_slice(&len_bytes);

        let region = RegionFile::from_bytes(&data).unwrap();
        let result = region.read_chunk(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_compression_in_chunk() {
        let mut data = vec![0u8; DATA_OFFSET + SECTOR_BYTES];
        data[0] = 0;
        data[1] = 0;
        data[2] = 2;
        data[3] = 1;
        let base = DATA_OFFSET;
        data[base + 3] = 2; // length = 2
        data[base + 4] = 99; // unknown compression method
        data[base + 5] = 0;

        let region = RegionFile::from_bytes(&data).unwrap();
        let result = region.read_chunk(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn all_1024_chunks() {
        let mut region = RegionFile::new();
        for z in 0..32u8 {
            for x in 0..32u8 {
                let data = format!("({x},{z})");
                region
                    .write_chunk(x, z, data.as_bytes(), CompressionMethod::ZLib, 0)
                    .unwrap();
            }
        }
        assert_eq!(region.chunk_count(), 1024);

        for z in 0..32u8 {
            for x in 0..32u8 {
                let expected = format!("({x},{z})");
                let read = region.read_chunk(x, z).unwrap().unwrap();
                assert_eq!(read, expected.as_bytes());
            }
        }

        let bytes = region.to_bytes();
        let reparsed = RegionFile::from_bytes(&bytes).unwrap();
        assert_eq!(reparsed.chunk_count(), 1024);

        assert_eq!(reparsed.read_chunk(0, 0).unwrap().unwrap(), b"(0,0)");
        assert_eq!(
            reparsed.read_chunk(31, 31).unwrap().unwrap(),
            b"(31,31)"
        );
    }

    #[test]
    fn write_remove_rewrite() {
        let mut region = RegionFile::new();

        region
            .write_chunk(0, 0, b"original", CompressionMethod::ZLib, 100)
            .unwrap();
        region
            .write_chunk(1, 0, b"second", CompressionMethod::ZLib, 200)
            .unwrap();
        assert_eq!(region.chunk_count(), 2);

        region.remove_chunk(0, 0).unwrap();
        assert_eq!(region.chunk_count(), 1);

        region
            .write_chunk(0, 0, b"replaced", CompressionMethod::ZLib, 300)
            .unwrap();
        assert_eq!(region.chunk_count(), 2);
        assert_eq!(region.read_chunk(0, 0).unwrap().unwrap(), b"replaced");
        assert_eq!(region.read_chunk(1, 0).unwrap().unwrap(), b"second");
    }

    #[test]
    fn timestamp_updates_correctly() {
        let mut region = RegionFile::new();

        region
            .write_chunk(5, 5, b"data1", CompressionMethod::ZLib, 1000)
            .unwrap();
        assert_eq!(region.get_timestamp(5, 5).unwrap(), 1000);

        region
            .write_chunk(5, 5, b"data2", CompressionMethod::ZLib, 2000)
            .unwrap();
        assert_eq!(region.get_timestamp(5, 5).unwrap(), 2000);

        let bytes = region.to_bytes();
        let reparsed = RegionFile::from_bytes(&bytes).unwrap();
        assert_eq!(reparsed.get_timestamp(5, 5).unwrap(), 2000);
    }

    #[test]
    fn max_sector_offset() {
        let loc = ChunkLocation {
            offset: 0xFF_FFFF,
            sector_count: 255,
        };
        let bytes = loc.to_bytes();
        let decoded = ChunkLocation::from_bytes(bytes);
        assert_eq!(decoded.offset, 0xFF_FFFF);
        assert_eq!(decoded.sector_count, 255);
    }

    #[test]
    fn write_to_writer() {
        let mut region = RegionFile::new();
        region
            .write_chunk(0, 0, b"test", CompressionMethod::None, 0)
            .unwrap();

        let mut buf = Vec::new();
        region.write_to(&mut buf).unwrap();
        assert_eq!(buf, region.to_bytes());
    }

    #[test]
    fn file_one_byte_too_small() {
        let data = vec![0u8; DATA_OFFSET - 1];
        assert!(RegionFile::from_bytes(&data).is_err());
    }

    #[test]
    fn nbt_integration() {
        // Test with actual NBT data
        use crate::Nbt;
        use crate::compound::NbtCompound;
        use crate::tag::NbtTag;

        let mut compound = NbtCompound::new();
        compound.put_int("DataVersion", 4671);
        compound.put_int("xPos", 0);
        compound.put_int("zPos", 0);
        compound.put_string("Status", "full".to_string());
        compound.put("Sections", NbtTag::List(vec![]));

        let nbt = Nbt::new("".to_string(), compound);
        let nbt_bytes = nbt.write();

        let mut region = RegionFile::new();
        region
            .write_chunk(0, 0, &nbt_bytes, CompressionMethod::ZLib, 1234)
            .unwrap();

        let read_bytes = region.read_chunk(0, 0).unwrap().unwrap();
        assert_eq!(read_bytes, nbt_bytes.as_ref());
    }
}

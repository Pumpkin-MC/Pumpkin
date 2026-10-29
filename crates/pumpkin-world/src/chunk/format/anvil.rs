use bytes::{Buf, BufMut, Bytes};
use flate2::read::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use lz4_java_wrc::Context;
use pumpkin_config::chunk::AnvilChunkConfig;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashMap;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::warn;

use crate::chunk::{
    ChunkParsingError, ChunkReadingError, ChunkSerializingError, ChunkWritingError,
    CompressionError,
    io::{Dirtiable, RegionFile, read_exact_at, write_all_at, write_file_atomically},
};

/// The side size of a region in chunks (one region is 32x32 chunks)
pub const REGION_SIZE: usize = 32;

/// The number of chunks in a region
pub const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// The number of bytes in a sector (4 KiB)
const SECTOR_BYTES: usize = 4096;

// 26.3
pub const WORLD_DATA_VERSION: i32 = 5023;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    /// `GZip` Compression
    GZip = Self::GZIP_ID,
    /// `ZLib` Compression
    ZLib = Self::ZLIB_ID,
    /// LZ4 Compression (since 24w04a)
    LZ4 = Self::LZ4_ID,
    /// Custom compression algorithm (since 24w05a)
    Custom = Self::CUSTOM_ID,
}

pub enum CompressionRead<R: Read> {
    GZip(GzDecoder<R>),
    ZLib(ZlibDecoder<R>),
    LZ4(lz4_java_wrc::Lz4BlockInput<R>),
}

impl<R: Read> Read for CompressionRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::GZip(gzip) => gzip.read(buf),
            Self::ZLib(zlib) => zlib.read(buf),
            Self::LZ4(lz4) => lz4.read(buf),
        }
    }
}

#[derive(Clone)]
pub struct AnvilChunkData {
    compression: Option<Compression>,
    // Length is always the length of this + compression byte (1) so we dont need to save a length
    compressed_data: Bytes,
}

impl Compression {
    const GZIP_ID: u8 = 1;
    const ZLIB_ID: u8 = 2;
    const NO_COMPRESSION_ID: u8 = 3;
    const LZ4_ID: u8 = 4;
    const CUSTOM_ID: u8 = 127;

    fn decompress_data(self, compressed_data: &[u8]) -> Result<Box<[u8]>, CompressionError> {
        fn decode<R: std::io::Read>(mut reader: R, capacity: usize) -> std::io::Result<Box<[u8]>> {
            let mut buf = Vec::with_capacity(capacity);
            reader.read_to_end(&mut buf)?;
            Ok(buf.into_boxed_slice())
        }

        let initial_capacity = compressed_data.len();

        match self {
            Self::GZip => decode(GzDecoder::new(compressed_data), initial_capacity)
                .map_err(CompressionError::GZipError),
            Self::ZLib => decode(ZlibDecoder::new(compressed_data), initial_capacity)
                .map_err(CompressionError::ZlibError),
            Self::LZ4 => decode(
                lz4_java_wrc::Lz4BlockInput::new(compressed_data),
                initial_capacity,
            )
            .map_err(CompressionError::LZ4Error),
            Self::Custom => Err(CompressionError::UnknownCompression),
        }
    }

    const LZ4_COMPRESSION_LEVEL_BASE: u32 = 10;
    fn compress_data(
        self,
        uncompressed_data: &[u8],
        compression_level: u32,
    ) -> Result<Vec<u8>, CompressionError> {
        match self {
            Self::GZip => {
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
            Self::ZLib => {
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
            Self::LZ4 => {
                let mut compressed_data = Vec::new();
                let block_size = 1 << (Self::LZ4_COMPRESSION_LEVEL_BASE + compression_level);
                let mut encoder = lz4_java_wrc::Lz4BlockOutput::with_context(
                    &mut compressed_data,
                    Context::default(),
                    block_size,
                )
                .map_err(CompressionError::LZ4Error)?;
                encoder
                    .write_all(uncompressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                drop(encoder);
                Ok(compressed_data)
            }
            Self::Custom => Err(CompressionError::UnknownCompression),
        }
    }

    /// Recognises the compression id of a payload by its leading bytes.
    fn id_from_payload(data: &[u8]) -> Option<u8> {
        match data {
            [0x1f, 0x8b, ..] => Some(Self::GZIP_ID),
            // A zlib header uses the deflate method and is a multiple of 31 (RFC 1950)
            [cmf, flg, ..] if cmf & 0x0f == 8 && u16::from_be_bytes([*cmf, *flg]) % 31 == 0 => {
                Some(Self::ZLIB_ID)
            }
            _ if data.starts_with(b"LZ4Block") => Some(Self::LZ4_ID),
            [pumpkin_nbt::COMPOUND_ID, ..] => Some(Self::NO_COMPRESSION_ID),
            _ => None,
        }
    }

    /// Returns Ok when a compression is found otherwise an Err
    #[expect(clippy::result_unit_err)]
    pub const fn from_byte(byte: u8) -> Result<Option<Self>, ()> {
        match byte {
            Self::GZIP_ID => Ok(Some(Self::GZip)),
            Self::ZLIB_ID => Ok(Some(Self::ZLib)),
            // Uncompressed (since a version before 1.15.1)
            Self::NO_COMPRESSION_ID => Ok(None),
            Self::LZ4_ID => Ok(Some(Self::LZ4)),
            Self::CUSTOM_ID => Ok(Some(Self::Custom)),
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
    /// Raw size of serialized chunk
    #[inline]
    const fn raw_write_size(&self) -> usize {
        // 4 bytes for the *length* and 1 byte for the *compression* method
        self.compressed_data.len() + 4 + 1
    }

    /// Size of serialized chunk with padding
    #[inline]
    const fn padded_size(&self) -> usize {
        let sector_count = self.sector_count() as usize;
        sector_count * SECTOR_BYTES
    }

    #[inline]
    const fn sector_count(&self) -> u32 {
        let total_size = self.raw_write_size();
        total_size.div_ceil(SECTOR_BYTES) as u32
    }

    fn from_bytes(bytes: Bytes) -> Result<Self, ChunkReadingError> {
        let mut bytes = bytes;
        if bytes.len() < size_of::<u32>() {
            return Err(ChunkReadingError::IoError(
                std::io::ErrorKind::UnexpectedEof.into(),
            ));
        }
        // Minus one for the compression byte, which the length covers, so
        // anything below one does not describe a chunk at all.
        let declared_length = bytes.get_u32() as usize;
        let Some(length) = declared_length.checked_sub(1) else {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(
                    "Chunk length does not cover its compression byte".to_string(),
                ),
            ));
        };

        // The compression byte has not been read yet, so the payload can use at
        // most one less than what remains.
        let Some(available) = bytes.len().checked_sub(1) else {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(
                    "Chunk record is missing its compression byte".to_string(),
                ),
            ));
        };

        if length > available {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(format!(
                    "Chunk length is greater than available bytes ({length} vs {available})"
                )),
            ));
        }

        let compression_method = bytes.get_u8();
        let compression = Compression::from_byte(compression_method)
            .map_err(|()| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        Ok(Self {
            compression,
            // If this has padding, we need to trim it
            compressed_data: bytes.slice(..length),
        })
    }

    /// The record as vanilla stores it in the region: length, compression byte, data and
    /// padding up to a whole sector.
    fn to_record(&self) -> Vec<u8> {
        let mut record = Vec::with_capacity(self.padded_size());
        record.put_u32((self.compressed_data.len() + 1) as u32);
        record.put_u8(
            self.compression
                .map_or(Compression::NO_COMPRESSION_ID, |c| c as u8),
        );
        record.extend_from_slice(&self.compressed_data);
        record.resize(self.padded_size(), 0);
        record
    }

    fn decompress(self) -> Result<Bytes, ChunkReadingError> {
        match self.compression {
            Some(compression) => compression
                .decompress_data(&self.compressed_data)
                .map(Bytes::from)
                .map_err(ChunkReadingError::Compression),
            None => Ok(self.compressed_data),
        }
    }

    fn compress(nbt: &[u8], chunk_config: &AnvilChunkConfig) -> Result<Self, ChunkWritingError> {
        let compression: Compression = chunk_config.compression.algorithm.into();
        let compressed_data = compression
            .compress_data(nbt, chunk_config.compression.level)
            .map_err(ChunkWritingError::Compression)?;

        Ok(Self {
            compression: Some(compression),
            compressed_data: compressed_data.into(),
        })
    }
}

/// Chunks taking this many sectors or more go to a separate `.mcc` file, as in vanilla.
const EXTERNAL_CHUNK_THRESHOLD: u32 = 256;
/// Set on the compression byte of a record whose data lives in a `.mcc` file.
const EXTERNAL_STREAM_FLAG: u8 = 128;
/// Location and timestamp tables.
const HEADER_BYTES: usize = SECTOR_BYTES * 2;

/// Which sectors of a region file are taken, mirroring vanilla's `RegionBitmap`.
#[derive(Clone, Default)]
pub(super) struct SectorBitmap(Vec<u64>);

impl SectorBitmap {
    fn is_used(&self, sector: u32) -> bool {
        self.0
            .get(sector as usize / 64)
            .is_some_and(|word| word & (1 << (sector % 64)) != 0)
    }

    pub(super) fn set(&mut self, start: u32, count: u32, used: bool) {
        for sector in start..start + count {
            let word = sector as usize / 64;
            if word >= self.0.len() {
                if !used {
                    return;
                }
                self.0.resize(word + 1, 0);
            }
            if used {
                self.0[word] |= 1 << (sector % 64);
            } else {
                self.0[word] &= !(1 << (sector % 64));
            }
        }
    }

    /// One past the last used sector.
    pub(super) fn end(&self) -> u32 {
        self.0
            .iter()
            .rposition(|word| *word != 0)
            .map_or(0, |index| {
                index as u32 * 64 + 64 - self.0[index].leading_zeros()
            })
    }

    /// The first sector from `sector` on whose used flag is not `used`, or `end`.
    fn run_end(&self, mut sector: u32, used: bool, end: u32) -> u32 {
        while sector < end {
            let word = self.0[sector as usize / 64];
            let differing = if used { !word } else { word } >> (sector % 64);
            if differing != 0 {
                return (sector + differing.trailing_zeros()).min(end);
            }
            sector = (sector / 64 + 1) * 64;
        }
        end
    }

    pub(super) fn free_runs(&self) -> FreeRuns {
        let end = self.end();
        let mut runs = Vec::new();
        let mut sector = 0;
        while sector < end {
            let start = sector;
            let used = self.is_used(sector);
            sector = self.run_end(sector, used, end);
            if !used {
                runs.push((sector - start, start));
            }
        }
        runs.sort_unstable();
        FreeRuns { runs, end }
    }
}

/// The free runs of a region, for placing the chunks of one flush. Old sectors are only freed
/// after the header is written, so runs only shrink while a flush is planned and the list
/// never goes stale.
pub(super) struct FreeRuns {
    /// `(len, start)` of every run, sorted, so the first one that fits is the smallest and
    /// lowest.
    runs: Vec<(u32, u32)>,
    end: u32,
}

impl FreeRuns {
    /// Takes the lowest of the smallest runs that fit `count` sectors, or appends after the last
    /// used sector. Best fit leaves about half the holes first fit does once chunks keep
    /// changing size.
    pub(super) fn allocate(&mut self, count: u32) -> u32 {
        let index = self.runs.partition_point(|(len, _)| *len < count);
        if index == self.runs.len() {
            let start = self.end;
            self.end += count;
            return start;
        }
        let (len, start) = self.runs.remove(index);
        if len > count {
            let rest = (len - count, start + count);
            let at = self.runs.partition_point(|run| *run < rest);
            self.runs.insert(at, rest);
        }
        start
    }
}

const fn sector_offset(location: u32) -> u32 {
    location >> 8
}

const fn sector_count(location: u32) -> u32 {
    location & 0xFF
}

fn epoch_seconds() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

/// An Anvil (`.mca`) region file.
///
/// Only the header is kept in memory. A flush writes each staged chunk into free sectors, then
/// the header, and frees the old sectors last, so a crash at any point leaves every chunk
/// readable in either its old or its new version.
pub struct AnvilRegion {
    path: PathBuf,
    region: Vector2<i32>,
    file: Option<File>,
    file_len: u64,
    locations: Box<[u32]>,
    timestamps: Box<[u32]>,
    used_sectors: SectorBitmap,
    staged: FxHashMap<usize, AnvilChunkData>,
    pending_external_deletes: Vec<PathBuf>,
    unsynced: bool,
}

impl AnvilRegion {
    fn flush_with_sync(
        &mut self,
        sync: bool,
        sync_file: impl Fn(&File) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        if self.staged.is_empty() {
            if sync
                && self.unsynced
                && let Some(file) = &self.file
            {
                sync_file(file)?;
                self.unsynced = false;
            }
            return Ok(());
        }

        // Plan on copies so a failed write leaves the region as it was, staged chunks included.
        let mut used_sectors = self.used_sectors.clone();
        let mut free_runs = self.used_sectors.free_runs();
        let mut locations = self.locations.clone();
        let mut timestamps = self.timestamps.clone();
        let mut freed = Vec::new();
        let mut stale_external = self.pending_external_deletes.clone();
        let mut writes = Vec::with_capacity(self.staged.len());
        let now = epoch_seconds();

        let mut staged: Vec<_> = self.staged.iter().collect();
        staged.sort_unstable_by_key(|(index, _)| **index);
        for (&index, chunk) in staged {
            let (record, count) = if chunk.sector_count() >= EXTERNAL_CHUNK_THRESHOLD {
                let path = self.external_path(index);
                write_file_atomically(&path, &chunk.compressed_data, sync)?;
                stale_external.retain(|pending| pending != &path);
                let mut stub = Vec::with_capacity(SECTOR_BYTES);
                stub.put_u32(1);
                stub.put_u8(
                    chunk
                        .compression
                        .map_or(Compression::NO_COMPRESSION_ID, |c| c as u8)
                        | EXTERNAL_STREAM_FLAG,
                );
                stub.resize(SECTOR_BYTES, 0);
                (stub, 1)
            } else {
                if self.is_external_stub(locations[index])? {
                    stale_external.push(self.external_path(index));
                }
                (chunk.to_record(), chunk.sector_count())
            };

            let offset = free_runs.allocate(count);
            used_sectors.set(offset, count, true);
            if locations[index] != 0 {
                freed.push(locations[index]);
            }
            locations[index] = (offset << 8) | count;
            timestamps[index] = now;
            writes.push((offset, record));
        }

        writes.sort_unstable_by_key(|(offset, _)| *offset);
        let file = self.file()?;
        let mut file_end = 0;
        let mut run_start = 0;
        let mut run: Vec<u8> = Vec::new();
        for (offset, record) in writes {
            let position = u64::from(offset) * SECTOR_BYTES as u64;
            // Neighbouring records go out in one write.
            if !run.is_empty() && run_start + run.len() as u64 != position {
                write_all_at(file, &run, run_start)?;
                run.clear();
            }
            if run.is_empty() {
                run_start = position;
            }
            run.extend_from_slice(&record);
            file_end = position + record.len() as u64;
        }
        if !run.is_empty() {
            write_all_at(file, &run, run_start)?;
        }

        let old_locations = std::mem::replace(&mut self.locations, locations);
        let old_timestamps = std::mem::replace(&mut self.timestamps, timestamps);
        let header = self.header_bytes();
        let file = self.file()?;
        if let Err(err) = write_all_at(file, &header, 0) {
            self.locations = old_locations;
            self.timestamps = old_timestamps;
            return Err(err);
        }
        for location in freed {
            used_sectors.set(sector_offset(location), sector_count(location), false);
        }
        self.file_len = self.file_len.max(file_end).max(HEADER_BYTES as u64);
        self.used_sectors = used_sectors;
        self.pending_external_deletes = stale_external;
        self.unsynced = true;
        if sync && let Some(file) = &self.file {
            sync_file(file)?;
            self.unsynced = false;
        }
        for path in self.pending_external_deletes.drain(..) {
            if let Err(err) = std::fs::remove_file(&path)
                && err.kind() != std::io::ErrorKind::NotFound
            {
                warn!("Failed to remove {}: {err}", path.display());
            }
        }
        self.truncate_unused_tail();
        self.staged.clear();
        Ok(())
    }

    fn external_path(&self, index: usize) -> PathBuf {
        let x = self.region.x * REGION_SIZE as i32 + (index % REGION_SIZE) as i32;
        let z = self.region.y * REGION_SIZE as i32 + (index / REGION_SIZE) as i32;
        self.path.with_file_name(format!("c.{x}.{z}.mcc"))
    }

    /// Whether the record at `location` points to a `.mcc` file. Stubs take one sector, so
    /// larger records are never read.
    fn is_external_stub(&self, location: u32) -> std::io::Result<bool> {
        let Some(file) = &self.file else {
            return Ok(false);
        };
        let start = u64::from(sector_offset(location)) * SECTOR_BYTES as u64;
        if location == 0 || sector_count(location) != 1 || start + 5 > self.file_len {
            return Ok(false);
        }
        let mut record_header = [0; 5];
        read_exact_at(file, &mut record_header, start)?;
        Ok(record_header[4] & EXTERNAL_STREAM_FLAG != 0)
    }

    pub(crate) fn external_chunk_paths(&self) -> Vec<PathBuf> {
        (0..CHUNK_COUNT)
            .filter(|&index| {
                self.is_external_stub(self.locations[index])
                    .unwrap_or(false)
            })
            .map(|index| self.external_path(index))
            .collect()
    }

    /// Gives back sectors freed at the end of the file. Only called once the header no longer
    /// points there.
    fn truncate_unused_tail(&mut self) {
        let used_len = u64::from(self.used_sectors.end()) * SECTOR_BYTES as u64;
        if used_len >= self.file_len {
            return;
        }
        let Some(file) = &self.file else {
            return;
        };
        match file.set_len(used_len) {
            Ok(()) => self.file_len = used_len,
            Err(err) => warn!("Failed to shrink {}: {err}", self.path.display()),
        }
    }

    fn header_bytes(&self) -> Vec<u8> {
        let mut header = Vec::with_capacity(HEADER_BYTES);
        for location in &self.locations {
            header.put_u32(*location);
        }
        for timestamp in &self.timestamps {
            header.put_u32(*timestamp);
        }
        header
    }

    fn file(&mut self) -> std::io::Result<&File> {
        if self.file.is_none() {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&self.path)?;
            self.file_len = file.metadata()?.len();
            self.file = Some(file);
        }
        self.file
            .as_ref()
            .ok_or_else(|| std::io::Error::other("region file is not open"))
    }
}

impl RegionFile for AnvilRegion {
    type Config = AnvilChunkConfig;
    type Encoded = AnvilChunkData;
    type Raw = AnvilChunkData;

    // Writes only touch the changed chunks, so there is nothing to gain from waiting.
    const FLUSH_DELAY: Duration = Duration::ZERO;
    const CACHE_CAPACITY: usize = 64;

    fn file_name(region: Vector2<i32>) -> String {
        format!("r.{}.{}.mca", region.x, region.y)
    }

    fn open(path: &Path, region: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        let mut this = Self {
            path: path.to_path_buf(),
            region,
            file: None,
            file_len: 0,
            locations: vec![0; CHUNK_COUNT].into_boxed_slice(),
            timestamps: vec![0; CHUNK_COUNT].into_boxed_slice(),
            used_sectors: SectorBitmap::default(),
            staged: FxHashMap::default(),
            pending_external_deletes: Vec::new(),
            unsynced: false,
        };
        this.used_sectors.set(0, 2, true);

        let file = match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(this),
            Err(err) => return Err(ChunkReadingError::IoError(err)),
        };
        let file_len = file.metadata().map_err(ChunkReadingError::IoError)?.len();
        let mut header = vec![0; HEADER_BYTES];
        let header_len = HEADER_BYTES.min(file_len as usize);
        if header_len < HEADER_BYTES && header_len > 0 {
            warn!(
                "Region file {} has a truncated header: {header_len} bytes",
                path.display()
            );
        }
        read_exact_at(&file, &mut header[..header_len], 0).map_err(ChunkReadingError::IoError)?;

        let (mut location_bytes, mut timestamp_bytes) = header.split_at(SECTOR_BYTES);
        for index in 0..CHUNK_COUNT {
            let location = location_bytes.get_u32();
            let timestamp = timestamp_bytes.get_u32();
            if location == 0 {
                continue;
            }
            let offset = sector_offset(location);
            let count = sector_count(location);
            // The same checks vanilla's RegionFile runs when it opens a file.
            if offset < 2 || count == 0 || u64::from(offset) * SECTOR_BYTES as u64 > file_len {
                warn!(
                    "Region file {} has an invalid sector at index {index}: offset {offset}, count {count}",
                    path.display()
                );
                continue;
            }
            this.locations[index] = location;
            this.timestamps[index] = timestamp;
            this.used_sectors.set(offset, count, true);
        }

        this.file = Some(file);
        this.file_len = file_len;
        Ok(this)
    }

    fn read_chunk(&self, index: usize) -> Result<Option<AnvilChunkData>, ChunkReadingError> {
        if let Some(staged) = self.staged.get(&index) {
            return Ok(Some(staged.clone()));
        }
        let location = self.locations[index];
        let Some(file) = &self.file else {
            return Ok(None);
        };
        if location == 0 {
            return Ok(None);
        }

        let start = u64::from(sector_offset(location)) * SECTOR_BYTES as u64;
        // The last record of a file may stop short of its sector count.
        let len = (u64::from(sector_count(location)) * SECTOR_BYTES as u64)
            .min(self.file_len.saturating_sub(start)) as usize;
        let mut record = vec![0; len];
        read_exact_at(file, &mut record, start).map_err(ChunkReadingError::IoError)?;

        if let Some(&compression_byte) = record.get(4)
            && compression_byte & EXTERNAL_STREAM_FLAG != 0
        {
            let marker = compression_byte & !EXTERNAL_STREAM_FLAG;
            let compressed_data =
                std::fs::read(self.external_path(index)).map_err(ChunkReadingError::IoError)?;
            // A flush replaces the .mcc before the header that points at the new stub, so after
            // a crash in between the old stub's marker can name the compression used before
            let compression_id = match Compression::id_from_payload(&compressed_data) {
                Some(detected) if detected != marker => {
                    warn!(
                        "External chunk {} has compression {detected} but its marker says {marker}",
                        self.external_path(index).display()
                    );
                    detected
                }
                _ => marker,
            };
            let compression = Compression::from_byte(compression_id).map_err(|()| {
                ChunkReadingError::Compression(CompressionError::UnknownCompression)
            })?;
            return Ok(Some(AnvilChunkData {
                compression,
                compressed_data: compressed_data.into(),
            }));
        }

        AnvilChunkData::from_bytes(record.into()).map(Some)
    }

    fn decode_chunk(raw: AnvilChunkData) -> Result<Bytes, ChunkReadingError> {
        raw.decompress()
    }

    fn encode_chunk(
        nbt: Bytes,
        config: &AnvilChunkConfig,
    ) -> Result<AnvilChunkData, ChunkWritingError> {
        AnvilChunkData::compress(&nbt, config)
    }

    fn stage_chunk(&mut self, index: usize, chunk: AnvilChunkData) {
        self.staged.insert(index, chunk);
    }

    fn has_staged(&self) -> bool {
        !self.staged.is_empty()
    }

    fn flush(&mut self, sync: bool) -> std::io::Result<()> {
        self.flush_with_sync(sync, File::sync_data)
    }
}

pub trait SingleChunkDataSerializer: Send + Sync + Sized + Dirtiable + 'static {
    fn to_bytes(&self) -> Result<Bytes, ChunkSerializingError>;
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError>;
    fn to_pump_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
        self.to_bytes()
    }
    fn position(&self) -> (i32, i32);
}

#[cfg(test)]
mod tests {
    use super::{
        AnvilChunkData, AnvilRegion, CHUNK_COUNT, Compression, CompressionError, SECTOR_BYTES,
        SectorBitmap, sector_count, sector_offset,
    };
    use crate::chunk::io::RegionFile;
    use bytes::{BufMut, Bytes, BytesMut};
    use pumpkin_config::chunk::AnvilChunkConfig;
    use pumpkin_util::math::vector2::Vector2;
    use tempfile::TempDir;

    /// A region file whose first location entry is `location`, whose other
    /// 1023 entries are absent, and whose single payload sector starts with
    /// `declared_len` followed by the "no compression" marker.
    fn region_with_first_location(location: u32, declared_len: u32) -> (TempDir, AnvilRegion) {
        let mut buf = BytesMut::with_capacity(SECTOR_BYTES * 3);
        buf.put_u32(location);
        buf.put_bytes(0, SECTOR_BYTES - 4);
        buf.put_bytes(0, SECTOR_BYTES);

        buf.put_u32(declared_len);
        buf.put_u8(3);
        buf.put_bytes(0, SECTOR_BYTES - 5);

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.mca");
        std::fs::write(&path, buf).unwrap();
        let region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        (dir, region)
    }

    /// Bytes that compress poorly, so their size in sectors is predictable.
    fn noise(seed: u64, len: usize) -> Bytes {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect()
    }

    fn assert_no_overlap(region: &AnvilRegion) {
        let mut taken = vec![false; 4096];
        for &location in &region.locations {
            if location == 0 {
                continue;
            }
            for sector in sector_offset(location)..sector_offset(location) + sector_count(location)
            {
                assert!(sector >= 2, "chunk inside the header");
                assert!(!taken[sector as usize], "sector {sector} used twice");
                taken[sector as usize] = true;
            }
        }
    }

    #[test]
    fn a_chunk_pointing_into_the_header_is_skipped() {
        // Sectors 0 and 1 hold the location and timestamp tables, so a chunk
        // cannot start before sector 2.
        let (_dir, region) = region_with_first_location((1 << 8) | 1, 1);

        assert!(region.read_chunk(0).unwrap().is_none());
    }

    #[test]
    fn a_chunk_at_the_first_free_sector_is_read() {
        // The control for the test above: sector 2 is the first legal one, and
        // a chunk there must not be skipped.
        let (_dir, region) = region_with_first_location((2 << 8) | 1, 1);

        assert!(region.read_chunk(0).unwrap().is_some());
    }

    #[test]
    fn a_chunk_declaring_no_length_is_an_error_not_a_panic() {
        // The length covers the compression byte, so the smallest legal value
        // is 1. Zero came from a file, so it has to be rejected rather than
        // subtracted from.
        let (_dir, region) = region_with_first_location((2 << 8) | 1, 0);

        assert!(region.read_chunk(0).is_err());
    }

    #[test]
    fn a_chunk_declaring_one_byte_too_many_is_an_error_not_a_panic() {
        // The length is checked while the compression byte is still unread, so
        // a chunk claiming exactly the bytes that remain used to pass the check
        // and then slice one byte past the end.
        let (_dir, region) = region_with_first_location((2 << 8) | 1, SECTOR_BYTES as u32 - 3);

        assert!(region.read_chunk(0).is_err());
    }

    #[test]
    fn truncated_record_headers_return_an_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.mca");
        for remaining in 0..5 {
            let mut bytes = vec![0; SECTOR_BYTES * 2 + remaining];
            bytes[..4].copy_from_slice(&((2u32 << 8) | 1).to_be_bytes());
            std::fs::write(&path, bytes).unwrap();
            let region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
            assert!(region.read_chunk(0).is_err());
        }
    }

    #[test]
    fn sync_failure_keeps_live_sectors_allocated_for_retry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.mca");
        let config = AnvilChunkConfig::default();
        let mut region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        region.stage_chunk(
            0,
            AnvilRegion::encode_chunk(Bytes::from(vec![1; 64]), &config).unwrap(),
        );
        region.flush(false).unwrap();
        let expected = Bytes::from(vec![2; 64]);
        region.stage_chunk(
            0,
            AnvilRegion::encode_chunk(expected.clone(), &config).unwrap(),
        );

        assert!(
            region
                .flush_with_sync(true, |_| Err(std::io::Error::other("sync failed")))
                .is_err()
        );
        assert!(region.has_staged());
        region.flush(false).unwrap();
        region.stage_chunk(
            1,
            AnvilRegion::encode_chunk(Bytes::from(vec![3; 64]), &config).unwrap(),
        );
        region.flush(false).unwrap();

        let reopened = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(
            AnvilRegion::decode_chunk(reopened.read_chunk(0).unwrap().unwrap()).unwrap(),
            expected
        );
        assert_no_overlap(&reopened);
    }

    #[test]
    fn allocation_takes_the_smallest_hole_that_fits() {
        let mut bitmap = SectorBitmap::default();
        // Used 0..2, hole of 3 at 2..5, used 5..6, hole of 1 at 6, used 7..8, then a used
        // word boundary so runs cross words.
        bitmap.set(0, 2, true);
        bitmap.set(5, 1, true);
        bitmap.set(7, 1, true);
        bitmap.set(60, 10, true);
        let mut runs = bitmap.free_runs();

        assert_eq!(runs.allocate(1), 6);
        assert_eq!(runs.allocate(2), 2);
        assert_eq!(runs.allocate(1), 4);
        assert_eq!(runs.allocate(52), 8);
        assert_eq!(runs.allocate(1), 70);
        assert_eq!(bitmap.end(), 70);
    }

    #[test]
    fn rewritten_chunks_reuse_sectors_and_survive_reopening() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.-1.2.mca");
        let config = AnvilChunkConfig::default();
        let mut expected = vec![None; CHUNK_COUNT];
        let mut region = AnvilRegion::open(&path, Vector2::new(-1, 2)).unwrap();

        // Grow and shrink chunks over several flushes so freed sectors get reused.
        for round in 0..6u64 {
            for index in (0..CHUNK_COUNT).step_by(37) {
                let len = 1000 + ((index as u64 * 7919 + round * 104_729) % 20_000) as usize;
                let nbt = noise(index as u64 ^ (round << 32), len);
                region.stage_chunk(
                    index,
                    AnvilRegion::encode_chunk(nbt.clone(), &config).unwrap(),
                );
                expected[index] = Some(nbt);
            }
            region.flush(false).unwrap();
            assert_no_overlap(&region);
        }
        let file_len = std::fs::metadata(&path).unwrap().len();

        let reopened = AnvilRegion::open(&path, Vector2::new(-1, 2)).unwrap();
        for (index, nbt) in expected.iter().enumerate() {
            let read = reopened
                .read_chunk(index)
                .unwrap()
                .map(|raw| AnvilRegion::decode_chunk(raw).unwrap());
            assert_eq!(read.as_ref(), nbt.as_ref(), "chunk {index}");
        }
        // Without reuse six rounds would need six times the live data.
        let live: u64 = expected.iter().flatten().map(|nbt| nbt.len() as u64).sum();
        assert!(
            file_len < live * 3,
            "file {file_len} bytes for {live} live bytes"
        );
    }

    #[test]
    fn oversized_chunks_go_to_an_external_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.mca");
        let config = AnvilChunkConfig::default();
        let mut region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();

        let big = noise(1, SECTOR_BYTES * 300);
        region.stage_chunk(33, AnvilRegion::encode_chunk(big.clone(), &config).unwrap());
        region.flush(false).unwrap();
        let external = dir.path().join("c.1.1.mcc");
        assert!(external.exists());

        let reopened = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        let raw = reopened.read_chunk(33).unwrap().unwrap();
        assert_eq!(AnvilRegion::decode_chunk(raw).unwrap(), big);

        let mut region = reopened;
        region.stage_chunk(
            33,
            AnvilRegion::encode_chunk(noise(2, 100), &config).unwrap(),
        );
        assert!(
            region
                .flush_with_sync(true, |_| Err(std::io::Error::other("sync failed")))
                .is_err()
        );
        assert!(external.exists());
        let replacement = noise(3, SECTOR_BYTES * 300);
        region.stage_chunk(
            33,
            AnvilRegion::encode_chunk(replacement.clone(), &config).unwrap(),
        );
        region.flush(false).unwrap();
        let reopened = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(
            AnvilRegion::decode_chunk(reopened.read_chunk(33).unwrap().unwrap()).unwrap(),
            replacement
        );
        region.stage_chunk(
            33,
            AnvilRegion::encode_chunk(noise(2, 100), &config).unwrap(),
        );
        region.flush(false).unwrap();
        assert!(!external.exists());
    }

    #[test]
    fn external_chunk_written_with_another_compression_still_reads() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.mca");
        let mut zlib = AnvilChunkConfig::default();
        zlib.compression.algorithm = pumpkin_config::chunk::Compression::ZLib;
        let mut region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        region.stage_chunk(
            33,
            AnvilRegion::encode_chunk(noise(1, SECTOR_BYTES * 300), &zlib).unwrap(),
        );
        region.flush(false).unwrap();

        // A crash after a save with LZ4 replaced the payload but before the header changed.
        let replacement = noise(2, SECTOR_BYTES * 300);
        let lz4 =
            AnvilRegion::encode_chunk(replacement.clone(), &AnvilChunkConfig::default()).unwrap();
        std::fs::write(dir.path().join("c.1.1.mcc"), &lz4.compressed_data).unwrap();

        let reopened = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(
            AnvilRegion::decode_chunk(reopened.read_chunk(33).unwrap().unwrap()).unwrap(),
            replacement
        );
    }

    #[test]
    fn custom_compression_returns_unknown_compression_error() {
        assert!(matches!(
            Compression::Custom.compress_data(b"chunk data", 6),
            Err(CompressionError::UnknownCompression)
        ));
    }

    #[test]
    fn custom_decompression_returns_unknown_compression_error() {
        assert!(matches!(
            Compression::Custom.decompress_data(b"chunk data"),
            Err(CompressionError::UnknownCompression)
        ));
    }

    #[test]
    fn records_have_the_vanilla_layout() {
        let chunk = AnvilChunkData {
            compression: Some(Compression::ZLib),
            compressed_data: Bytes::from_static(&[1, 2, 3]),
        };
        let record = chunk.to_record();
        assert_eq!(record.len(), SECTOR_BYTES);
        assert_eq!(&record[..8], &[0, 0, 0, 4, 2, 1, 2, 3]);
    }
}

use async_trait::async_trait;
use bytes::*;
use flate2::read::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use itertools::Itertools;
use lz4_java_wrc::Context;
use pumpkin_config::advanced_config;
use pumpkin_util::math::vector2::Vector2;
use std::{
    collections::HashSet,
    io::{Read, SeekFrom, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufWriter},
    sync::Mutex,
};

use crate::chunk::{
    ChunkParsingError, ChunkReadingError, ChunkSerializingError, ChunkWritingError,
    CompressionError,
    io::{ChunkSerializer, Dirtiable, LoadedData},
};

/// The side size of a region in chunks (one region is 32x32 chunks)
pub const REGION_SIZE: usize = 32;

/// The number of bits that identify two chunks in the same region
pub const SUBREGION_BITS: u8 = pumpkin_util::math::ceil_log2(REGION_SIZE as u32);

pub const SUBREGION_AND: i32 = i32::pow(2, SUBREGION_BITS as u32) - 1;

/// The number of chunks in a region
pub const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// The number of bytes in a sector (4 KiB)
const SECTOR_BYTES: usize = 4096;

// 1.21.7
pub const WORLD_DATA_VERSION: i32 = 4438;

#[derive(Clone, Default)]
pub struct AnvilChunkFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    /// GZip Compression
    GZip = Self::GZIP_ID,
    /// ZLib Compression
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

#[derive(Default, Clone)]
pub struct AnvilChunkData {
    compression: Option<Compression>,
    // Length is always the length of this + compression byte (1) so we dont need to save a length
    compressed_data: Bytes,
}

enum WriteAction {
    // Don't write anything
    Pass,
    // Write the entire file
    All,
    // Only write certain indices
    Parts(HashSet<usize>),
}

impl WriteAction {
    /// If we are currently not writing, sets to new Parts enum,
    /// If we have parts enum, add to it,
    /// If we have All enum, do nothing
    fn maybe_update_chunk_index(&mut self, index: usize) {
        match self {
            Self::Pass => *self = Self::Parts(HashSet::from_iter([index])),
            Self::Parts(parts) => {
                let _ = parts.insert(index);
            }
            Self::All => {}
        }
    }
}

struct AnvilChunkMetadata {
    serialized_data: AnvilChunkData,
    timestamp: u32,

    // NOTE: This is only valid if our WriteAction is `Parts`
    file_sector_offset: u32,
}

pub struct AnvilChunkFile<S: SingleChunkDataSerializer> {
    chunks_data: [Option<AnvilChunkMetadata>; CHUNK_COUNT],
    end_sector: u32,
    write_action: Mutex<WriteAction>,

    _dummy: PhantomData<S>,
}

impl Compression {
    const GZIP_ID: u8 = 1;
    const ZLIB_ID: u8 = 2;
    const NO_COMPRESSION_ID: u8 = 3;
    const LZ4_ID: u8 = 4;
    const CUSTOM_ID: u8 = 127;

    fn decompress_data(&self, compressed_data: &[u8]) -> Result<Box<[u8]>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut decoder = GzDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data.into_boxed_slice())
            }
            Compression::ZLib => {
                let mut decoder = ZlibDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data.into_boxed_slice())
            }
            Compression::LZ4 => {
                let mut decoder = lz4_java_wrc::Lz4BlockInput::new(compressed_data);
                let mut decompressed_data = Vec::new();
                decoder
                    .read_to_end(&mut decompressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                Ok(decompressed_data.into_boxed_slice())
            }
            Compression::Custom => todo!(),
        }
    }

    const LZ4_COMPRESSION_LEVEL_BASE: u32 = 10;
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
            Compression::Custom => todo!(),
        }
    }

    /// Returns Ok when a compression is found otherwise an Err
    #[allow(clippy::result_unit_err)]
    pub fn from_byte(byte: u8) -> Result<Option<Self>, ()> {
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
    fn raw_write_size(&self) -> usize {
        // 4 bytes for the *length* and 1 byte for the *compression* method
        self.compressed_data.len() + 4 + 1
    }

    /// Size of serialized chunk with padding
    #[inline]
    fn padded_size(&self) -> usize {
        let sector_count = self.sector_count() as usize;
        sector_count * SECTOR_BYTES
    }

    #[inline]
    fn sector_count(&self) -> u32 {
        let total_size = self.raw_write_size();
        total_size.div_ceil(SECTOR_BYTES) as u32
    }

    fn from_bytes(bytes: Bytes) -> Result<Self, ChunkReadingError> {
        let mut bytes = bytes;
        // Minus one for the compression byte
        let length = bytes.get_u32() as usize - 1;

        if length > bytes.len() {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(format!(
                    "Chunk length is greater than available bytes ({} vs {})",
                    length,
                    bytes.len()
                )),
            ));
        }

        let compression_method = bytes.get_u8();
        let compression = Compression::from_byte(compression_method)
            .map_err(|_| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        Ok(AnvilChunkData {
            compression,
            // If this has padding, we need to trim it
            compressed_data: bytes.slice(..length),
        })
    }

    async fn write(&self, w: &mut (impl AsyncWrite + Unpin + Send)) -> Result<(), std::io::Error> {
        let padded_size = self.padded_size();

        w.write_u32((self.compressed_data.remaining() + 1) as u32)
            .await?;
        w.write_u8(
            self.compression
                .map_or(Compression::NO_COMPRESSION_ID, |c| c as u8),
        )
        .await?;

        w.write_all(&self.compressed_data).await?;
        for _ in 0..(padded_size - self.raw_write_size()) {
            w.write_u8(0).await?;
        }

        Ok(())
    }

    fn to_chunk<S>(&self, pos: Vector2<i32>) -> Result<S, ChunkReadingError>
    where
        S: SingleChunkDataSerializer,
    {
        if let Some(compression) = self.compression {
            let decompress_bytes = compression
                .decompress_data(&self.compressed_data)
                .map_err(ChunkReadingError::Compression)?;

            S::from_bytes(decompress_bytes.into(), pos)
        } else {
            S::from_bytes(self.compressed_data.clone(), pos)
        }
    }

    async fn from_chunk<S>(
        chunk: &S,
        compression: Option<Compression>,
    ) -> Result<Self, ChunkWritingError>
    where
        S: SingleChunkDataSerializer,
    {
        let raw_bytes = chunk
            .to_bytes()
            .await
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        let compression = compression
            .unwrap_or_else(|| advanced_config().chunk.compression.algorithm.clone().into());

        // We need to buffer here anyway so there's no use in making an impl Write for this
        let compressed_data = compression
            .compress_data(&raw_bytes, advanced_config().chunk.compression.level)
            .map_err(ChunkWritingError::Compression)?;

        Ok(AnvilChunkData {
            compression: Some(compression),
            compressed_data: compressed_data.into(),
        })
    }
}

impl<S: SingleChunkDataSerializer> AnvilChunkFile<S> {
    pub const fn get_region_coords(at: &Vector2<i32>) -> (i32, i32) {
        // Divide by 32 for the region coordinates
        (at.x >> SUBREGION_BITS, at.y >> SUBREGION_BITS)
    }

    pub const fn get_chunk_index(pos: &Vector2<i32>) -> usize {
        let local_x = pos.x & SUBREGION_AND;
        let local_z = pos.y & SUBREGION_AND;
        let index = (local_z << SUBREGION_BITS) + local_x;
        index as usize
    }

    async fn write_indices(&self, path: &Path, indices: &[usize]) -> Result<(), std::io::Error> {
        log::trace!("Writing in place: {path:?}");

        let file = tokio::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(false)
            .append(false)
            .open(path)
            .await?;

        let mut write = BufWriter::new(file);
        // The first two sectors are reserved for the location table
        for (index, metadata) in self.chunks_data.iter().enumerate() {
            if let Some(chunk) = metadata {
                let chunk_data = &chunk.serialized_data;
                let sector_count = chunk_data.sector_count();
                log::trace!(
                    "Writing position for chunk {} - {}:{}",
                    index,
                    chunk.file_sector_offset,
                    sector_count
                );
                write
                    .write_u32((chunk.file_sector_offset << 8) | sector_count)
                    .await?;
            } else {
                // If the chunk is not present, we write 0 to the location and timestamp tables
                write.write_u32(0).await?;
            };
        }

        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                write.write_u32(chunk.timestamp).await?;
            } else {
                // If the chunk is not present, we write 0 to the location and timestamp tables
                write.write_u32(0).await?;
            }
        }

        let mut chunks = indices
            .iter()
            .map(|index| {
                (
                    index,
                    self.chunks_data[*index]
                        .as_ref()
                        .expect("We are trying to write a chunk, but it does not exist!"),
                )
            })
            .collect::<Vec<_>>();

        // Sort such that writes are in order
        chunks.sort_by_key(|chunk| chunk.1.file_sector_offset);

        #[cfg(debug_assertions)]
        {
            // Verify we are actually two sectors into the file
            let current_pos = write.stream_position().await?;
            assert!(current_pos as usize == 2 * SECTOR_BYTES);
        }

        let mut current_sector = 2;
        for (index, chunk) in chunks {
            debug_assert!(
                current_sector <= chunk.file_sector_offset,
                "Current sector is {} but we want to write to {}!",
                current_sector,
                chunk.file_sector_offset
            );

            // Seek only if we need to
            if chunk.file_sector_offset != current_sector {
                log::trace!("Seeking to sector {}", chunk.file_sector_offset);
                let _ = write
                    .seek(SeekFrom::Start(
                        chunk.file_sector_offset as u64 * SECTOR_BYTES as u64,
                    ))
                    .await?;
                current_sector = chunk.file_sector_offset;
            }
            log::trace!(
                "Writing chunk {} - {}:{}",
                index,
                current_sector,
                chunk.serialized_data.sector_count()
            );

            current_sector += chunk.serialized_data.sector_count();

            chunk.serialized_data.write(&mut write).await?;
        }

        write.flush().await
    }

    /// Write entire file, disregarding saved offsets
    async fn write_all(&self, path: &Path) -> Result<(), std::io::Error> {
        let temp_path = path.with_extension("tmp");
        log::trace!("Writing tmp file to disk: {temp_path:?}");

        let file = tokio::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .await?;

        let mut write = BufWriter::new(file);

        // The first two sectors are reserved for the location table
        let mut current_sector: u32 = 2;
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                let chunk = &chunk.serialized_data;
                let sector_count = chunk.sector_count();
                write
                    .write_u32((current_sector << 8) | sector_count)
                    .await?;
                current_sector += sector_count;
            } else {
                // If the chunk is not present, we write 0 to the location and timestamp tables
                write.write_u32(0).await?;
            };
        }

        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                write.write_u32(chunk.timestamp).await?;
            } else {
                // If the chunk is not present, we write 0 to the location and timestamp tables
                write.write_u32(0).await?;
            }
        }

        for chunk in self.chunks_data.iter().flatten() {
            chunk.serialized_data.write(&mut write).await?;
        }

        write.flush().await?;
        // The rename of the file works like an atomic operation ensuring
        // that the data is not corrupted before the rename is completed
        tokio::fs::rename(temp_path, path).await?;

        log::trace!("Wrote file to Disk: {path:?}");
        Ok(())
    }
}

impl<S: SingleChunkDataSerializer> Default for AnvilChunkFile<S> {
    fn default() -> Self {
        Self {
            chunks_data: [const { None }; CHUNK_COUNT],
            write_action: Mutex::new(WriteAction::Pass),
            // Two sectors for offset + timestamp
            end_sector: 2,
            _dummy: Default::default(),
        }
    }
}

#[async_trait]
pub trait SingleChunkDataSerializer: Send + Sync + Sized + Dirtiable {
    async fn to_bytes(&self) -> Result<Bytes, ChunkSerializingError>;
    fn from_bytes(bytes: Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError>;
    fn position(&self) -> &Vector2<i32>;
}

#[async_trait]
impl<S: SingleChunkDataSerializer> ChunkSerializer for AnvilChunkFile<S> {
    type Data = S;
    type WriteBackend = PathBuf;

    fn should_write(&self, is_watched: bool) -> bool {
        !is_watched
    }

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        let (region_x, region_z) = Self::get_region_coords(chunk);
        format!("./r.{region_x}.{region_z}.mca")
    }

    async fn write(&self, path: PathBuf) -> Result<(), std::io::Error> {
        let mut write_action = self.write_action.lock().await;
        match &*write_action {
            WriteAction::Pass => {
                log::debug!("Skipping write for {path:?} as there were no dirty chunks");
                Ok(())
            }
            WriteAction::All => self.write_all(&path).await,
            WriteAction::Parts(parts) => {
                self.write_indices(&path, Vec::from_iter(parts.iter().cloned()).as_slice())
                    .await
            }
        }?;

        // If we still are in memory after this, we don't need to write again!
        *write_action = WriteAction::Pass;
        Ok(())
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let mut raw_file_bytes = r;

        if raw_file_bytes.len() < SECTOR_BYTES * 2 {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let headers = raw_file_bytes.split_to(SECTOR_BYTES * 2);
        let (mut location_bytes, mut timestamp_bytes) = headers.split_at(SECTOR_BYTES);

        let mut chunk_file = AnvilChunkFile::default();

        let mut last_offset = 2;
        for i in 0..CHUNK_COUNT {
            let timestamp = timestamp_bytes.get_u32();
            let location = location_bytes.get_u32();

            let sector_count = (location & 0xFF) as usize;
            let sector_offset = (location >> 8) as usize;
            let end_offset = sector_offset + sector_count;

            // If the sector offset or count is 0, the chunk is not present (we should not parse empty chunks)
            if sector_offset == 0 || sector_count == 0 {
                continue;
            }

            if end_offset > last_offset {
                last_offset = end_offset;
            }

            // We always subtract 2 for the first two sectors for the timestamp and location tables
            // that we walked earlier
            let bytes_offset = (sector_offset - 2) * SECTOR_BYTES;
            let bytes_count = sector_count * SECTOR_BYTES;

            if bytes_offset + bytes_count > raw_file_bytes.len() {
                return Err(ChunkReadingError::ParsingError(
                    ChunkParsingError::ErrorDeserializingChunk(format!(
                        "Not enough bytes available for the chunk {} ({} vs {})",
                        i,
                        bytes_count,
                        raw_file_bytes.len().saturating_sub(bytes_offset)
                    )),
                ));
            }

            let serialized_data = AnvilChunkData::from_bytes(
                raw_file_bytes.slice(bytes_offset..bytes_offset + bytes_count),
            )?;

            chunk_file.chunks_data[i] = Some(AnvilChunkMetadata {
                serialized_data,
                timestamp,
                file_sector_offset: sector_offset as u32,
            });
        }

        chunk_file.end_sector = last_offset as u32;
        Ok(chunk_file)
    }

    async fn update_chunk(&mut self, chunk: &Self::Data) -> Result<(), ChunkWritingError> {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let index = AnvilChunkFile::<S>::get_chunk_index(chunk.position());
        // Default to the compression type read from the file
        let compression_type = self.chunks_data[index]
            .as_ref()
            .and_then(|chunk_data| chunk_data.serialized_data.compression);
        let new_chunk_data = AnvilChunkData::from_chunk(chunk, compression_type).await?;

        let mut write_action = self.write_action.lock().await;
        if !advanced_config().chunk.write_in_place {
            *write_action = WriteAction::All;
        }

        match &*write_action {
            WriteAction::All => {
                log::trace!("Write action is all: setting chunk in place");
                // Doesn't matter, just add the data
                self.chunks_data[index] = Some(AnvilChunkMetadata {
                    serialized_data: new_chunk_data,
                    timestamp: epoch,
                    file_sector_offset: 0,
                });
            }
            _ => {
                match self.chunks_data[index].as_ref() {
                    None => {
                        log::trace!(
                            "Chunk {} does not exist, appending to EOF: {}:{}",
                            index,
                            self.end_sector,
                            new_chunk_data.sector_count()
                        );
                        // This chunk didn't exist before; append to EOF
                        let new_eof = self.end_sector + new_chunk_data.sector_count();
                        self.chunks_data[index] = Some(AnvilChunkMetadata {
                            serialized_data: new_chunk_data,
                            timestamp: epoch,
                            file_sector_offset: self.end_sector,
                        });
                        self.end_sector = new_eof;
                        write_action.maybe_update_chunk_index(index);
                    }
                    Some(old_chunk) => {
                        if old_chunk.serialized_data.sector_count() == new_chunk_data.sector_count()
                        {
                            log::trace!(
                                "Chunk {} exists, writing in place: {}:{}",
                                index,
                                old_chunk.file_sector_offset,
                                new_chunk_data.sector_count()
                            );
                            // We can just add it
                            self.chunks_data[index] = Some(AnvilChunkMetadata {
                                serialized_data: new_chunk_data,
                                timestamp: epoch,
                                file_sector_offset: old_chunk.file_sector_offset,
                            });
                            write_action.maybe_update_chunk_index(index);
                        } else {
                            // Walk back the end of the list; seeing if there's something that can fit
                            // in our spot. Here we play a game between is it worth it to do all
                            // this swapping. I figure if we don't find it after 64 chunks, just
                            // re-write the whole file instead
                            // The number is a guestimation and no rigorious thought when into it.
                            // The more we leapfrog like this, there is a higher
                            // (abiet still small) of these chunks being corrupted if we are doing a
                            // write operation when there is an un-clean shutdown
                            //
                            // Writing all is "safer" in the sense that no chunks will corrupt,
                            // but will still roll back the entire region if
                            // there is an unclean shutdown

                            let mut chunks = self
                                .chunks_data
                                .iter()
                                .enumerate()
                                .filter_map(|(index, chunk)| {
                                    chunk.as_ref().map(|chunk| (index, chunk))
                                })
                                .collect::<Vec<_>>();
                            chunks.sort_by_key(|chunk| chunk.1.file_sector_offset);

                            let mut chunks_to_shift = chunks
                                .into_iter()
                                .rev()
                                .take(64)
                                .take_while_inclusive(|chunk| {
                                    chunk.1.serialized_data.sector_count()
                                        != old_chunk.serialized_data.sector_count()
                                })
                                .collect::<Vec<_>>();

                            if chunks_to_shift.last().is_none_or(|chunk| chunk.0 == index) {
                                log::trace!(
                                    "Unable to find a chunk to swap with; falling back to serialize all",
                                );

                                // give up...
                                *write_action = WriteAction::All;
                                self.chunks_data[index] = Some(AnvilChunkMetadata {
                                    serialized_data: new_chunk_data,
                                    timestamp: epoch,
                                    file_sector_offset: 0,
                                });
                            } else {
                                // swap last element of the chunks to shift (the first because we
                                // reversed it) and shift the rest down
                                let swap = chunks_to_shift
                                    .pop()
                                    .expect("We just checked that this exists");

                                let indices_to_shift = chunks_to_shift
                                    .iter()
                                    .map(|(index, _)| index)
                                    .copied()
                                    .collect::<Vec<_>>();
                                let swapped_sectors = swap.1.serialized_data.sector_count();
                                let new_sectors = new_chunk_data.sector_count();
                                let swapped_index = swap.0;
                                let old_offset = old_chunk.file_sector_offset;
                                self.chunks_data[index] = Some(AnvilChunkMetadata {
                                    serialized_data: new_chunk_data,
                                    timestamp: epoch,
                                    file_sector_offset: swap.1.file_sector_offset,
                                });
                                write_action.maybe_update_chunk_index(index);

                                self.chunks_data[swapped_index]
                                    .as_mut()
                                    .expect("We checked if this was none")
                                    .file_sector_offset = old_offset;
                                write_action.maybe_update_chunk_index(swapped_index);

                                // Then offset everything else

                                // If positive, now larger -> shift right, else shift left
                                let offset = new_sectors as i64 - swapped_sectors as i64;

                                log::trace!(
                                    "Swapping {index} with {swapped_index}, shifting all chunks {swapped_index} and after by {offset}"
                                );

                                for shift_index in indices_to_shift {
                                    let chunk_data = self.chunks_data[shift_index]
                                        .as_mut()
                                        .expect("We checked if this was none");
                                    let new_offset = chunk_data.file_sector_offset as i64 + offset;
                                    chunk_data.file_sector_offset = new_offset as u32;
                                    write_action.maybe_update_chunk_index(shift_index);
                                }

                                // If the shift is negative then there will be trailing data, but i
                                // think that's fine

                                let new_end = self.end_sector as i64 + offset;
                                self.end_sector = new_end as u32;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        // Don't par iter here so we can prevent backpressure with the await in the async
        // runtime
        for chunk in chunks.iter().cloned() {
            let index = AnvilChunkFile::<S>::get_chunk_index(&chunk);
            let is_ok = match &self.chunks_data[index] {
                None => stream.send(LoadedData::Missing(chunk)).await.is_ok(),
                Some(chunk_metadata) => {
                    let chunk_data = &chunk_metadata.serialized_data;
                    let result = match chunk_data.to_chunk(chunk) {
                        Ok(chunk) => LoadedData::Loaded(chunk),
                        Err(err) => LoadedData::Error((chunk, err)),
                    };

                    stream.send(result).await.is_ok()
                }
            };

            if !is_ok {
                // Stream is closed. Stop unneeded work and IO
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use pumpkin_config::{AdvancedConfiguration, advanced_config, override_config_for_testing};
    use pumpkin_data::BlockDirection;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use temp_dir::TempDir;
    use tokio::sync::RwLock;

    use crate::chunk::ChunkData;
    use crate::chunk::format::anvil::{AnvilChunkFile, SingleChunkDataSerializer};
    use crate::chunk::io::file_manager::{ChunkFileManager, PathFromLevelFolder};
    use crate::chunk::io::{FileIO, LoadedData};
    use crate::dimension::Dimension;
    use crate::generation::{Seed, get_world_gen};
    use crate::level::{Level, LevelFolder, SyncChunk};
    use crate::world::{BlockAccessor, BlockRegistryExt};

    struct BlockRegistry;

    #[async_trait]
    impl BlockRegistryExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _block_accessor: &dyn BlockAccessor,
            _block_pos: &BlockPos,
            _face: BlockDirection,
        ) -> bool {
            true
        }
    }

    async fn get_chunks<S>(
        saver: &ChunkFileManager<AnvilChunkFile<S>>,
        folder: &LevelFolder,
        chunks: &[(Vector2<i32>, SyncChunk)],
    ) -> Box<[Arc<RwLock<S>>]>
    where
        S: SingleChunkDataSerializer + PathFromLevelFolder + 'static,
    {
        let mut read_chunks = Vec::new();
        let (send, mut recv) = tokio::sync::mpsc::channel(1);

        let chunk_pos = chunks.iter().map(|(at, _)| *at).collect::<Vec<_>>();
        let spawn = saver.fetch_chunks(folder, &chunk_pos, send);
        let collect = async {
            while let Some(data) = recv.recv().await {
                read_chunks.push(data);
            }
        };

        tokio::join!(spawn, collect);

        let read_chunks = read_chunks
            .into_iter()
            .map(|chunk| match chunk {
                LoadedData::Loaded(chunk) => chunk,
                LoadedData::Missing(_) => panic!("Missing chunk"),
                LoadedData::Error((position, error)) => {
                    panic!("Error reading chunk at {position:?} | Error: {error:?}")
                }
            })
            .collect::<Vec<_>>();

        read_chunks.into_boxed_slice()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();

        let mut chunks = Vec::new();
        let (send, mut recv) = tokio::sync::mpsc::channel(1);

        chunk_saver
            .fetch_chunks(
                &LevelFolder {
                    root_folder: PathBuf::from(""),
                    region_folder: region_path,
                    entities_folder: PathBuf::from(""),
                },
                &[Vector2::new(0, 0)],
                send,
            )
            .await;

        while let Some(data) = recv.recv().await {
            chunks.push(data);
        }

        assert!(chunks.len() == 1 && matches!(chunks[0], LoadedData::Missing(_)));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_in_place() {
        let mut config = AdvancedConfiguration::default();
        config.chunk.write_in_place = true;
        override_config_for_testing(config);
        assert!(advanced_config().chunk.write_in_place);

        let _ = env_logger::try_init();

        let generator = get_world_gen(Seed(0), Dimension::Overworld);

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
            entities_folder: PathBuf::from("entities"),
        };
        fs::create_dir(&level_folder.region_folder).expect("couldn't create region folder");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let block_registry = Arc::new(BlockRegistry);

        // Generate chunks
        let mut chunks = vec![];
        let level = Arc::new(Level::from_root_folder(
            temp_dir.path().to_path_buf(),
            block_registry.clone(),
            0,
            Dimension::Overworld,
        ));
        for x in -5..5 {
            for y in -5..5 {
                let position = Vector2::new(x, y);
                let chunk = generator.generate_chunk(&level, block_registry.as_ref(), &position);
                chunks.push((position, Arc::new(RwLock::new(chunk))));
            }
        }

        // TEST APPEND TO END

        chunk_saver
            .save_chunks(&level_folder, chunks.clone())
            .await
            .expect("Failed to write chunk");

        // Create a new manager to ensure nothing is cached
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let read_chunks = get_chunks(&chunk_saver, &level_folder, &chunks).await;

        for (_, chunk) in &chunks {
            let chunk = chunk.read().await;
            for read_chunk in read_chunks.iter() {
                let read_chunk = read_chunk.read().await;
                if read_chunk.position == chunk.position {
                    let original = chunk.section.dump_blocks();
                    let read = read_chunk.section.dump_blocks();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    let original = chunk.section.dump_biomes();
                    let read = read_chunk.section.dump_biomes();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });
                    break;
                }
            }
        }

        // TEST WRITE IN PLACE

        // Idk what blocks these are, they just have to be different
        let mut chunk = chunks.first().unwrap().1.write().await;
        chunk.section.set_relative_block(0, 0, 0, 1000);
        // Mark dirty so we actually write it
        chunk.dirty = true;
        drop(chunk);
        let mut chunk = chunks.last().unwrap().1.write().await;
        chunk.section.set_relative_block(0, 0, 0, 1000);
        // Mark dirty so we actually write it
        chunk.dirty = true;
        drop(chunk);

        chunk_saver
            .save_chunks(&level_folder, chunks.clone())
            .await
            .expect("Failed to write chunk");

        // Create a new manager to ensure nothing is cached
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let read_chunks = get_chunks(&chunk_saver, &level_folder, &chunks).await;

        for (_, chunk) in &chunks {
            let chunk = chunk.read().await;
            for read_chunk in read_chunks.iter() {
                let read_chunk = read_chunk.read().await;
                if read_chunk.position == chunk.position {
                    let original = chunk.section.dump_blocks();
                    let read = read_chunk.section.dump_blocks();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    let original = chunk.section.dump_biomes();
                    let read = read_chunk.section.dump_biomes();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    break;
                }
            }
        }

        // TEST SWAP SHIFT

        // Make a big chunk
        let mut chunk = chunks.first().unwrap().1.write().await;
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..4 {
                    let block_id = 16 * 16 * y + 16 * z + x;
                    chunk.section.set_relative_block(x, y, z, block_id as u16);
                }
            }
        }
        // Mark dirty so we actually write it
        chunk.dirty = true;
        drop(chunk);
        let mut chunk = chunks[2].1.write().await;
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..4 {
                    let block_id = 16 * 16 * y + 16 * z + x;
                    chunk.section.set_relative_block(x, y, z, block_id as u16);
                }
            }
        }
        // Mark dirty so we actually write it
        chunk.dirty = true;
        drop(chunk);

        chunk_saver
            .save_chunks(&level_folder, chunks.clone())
            .await
            .expect("Failed to write chunk");

        // Create a new manager to ensure nothing is cached
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let read_chunks = get_chunks(&chunk_saver, &level_folder, &chunks).await;

        for (_, chunk) in &chunks {
            let chunk = chunk.read().await;
            for read_chunk in read_chunks.iter() {
                let read_chunk = read_chunk.read().await;
                if read_chunk.position == chunk.position {
                    let original = chunk.section.dump_blocks();
                    let read = read_chunk.section.dump_blocks();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    let original = chunk.section.dump_biomes();
                    let read = read_chunk.section.dump_biomes();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    break;
                }
            }
        }

        // TEST DEFAULT TO WRITE ALL

        // Make an even bigger chunk
        let mut chunk = chunks.last().unwrap().1.write().await;
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..16 {
                    let block_id = 16 * 16 * y + 16 * z + x;
                    chunk.section.set_relative_block(x, y, z, block_id as u16);
                }
            }
        }
        // Mark dirty so we actually write it
        chunk.dirty = true;
        drop(chunk);

        chunk_saver
            .save_chunks(&level_folder, chunks.clone())
            .await
            .expect("Failed to write chunk");

        // Create a new manager to ensure nothing is cached
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let read_chunks = get_chunks(&chunk_saver, &level_folder, &chunks).await;

        for (_, chunk) in &chunks {
            let chunk = chunk.read().await;
            for read_chunk in read_chunks.iter() {
                let read_chunk = read_chunk.read().await;
                if read_chunk.position == chunk.position {
                    let original = chunk.section.dump_blocks();
                    let read = read_chunk.section.dump_blocks();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });

                    let original = chunk.section.dump_biomes();
                    let read = read_chunk.section.dump_biomes();

                    original
                        .into_iter()
                        .zip(read)
                        .enumerate()
                        .for_each(|(i, (o, r))| {
                            if o != r {
                                panic!("Data miss-match expected {o}, got {r} ({i})");
                            }
                        });
                    break;
                }
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_bulk() {
        let mut config = AdvancedConfiguration::default();
        config.chunk.write_in_place = false;
        override_config_for_testing(config);
        assert!(!advanced_config().chunk.write_in_place);

        let _ = env_logger::try_init();

        let generator = get_world_gen(Seed(0), Dimension::Overworld);

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
            entities_folder: PathBuf::from("entities"),
        };
        fs::create_dir(&level_folder.region_folder).expect("couldn't create region folder");
        let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
        let block_registry = Arc::new(BlockRegistry);

        // Generate chunks
        let mut chunks = vec![];
        let level = Arc::new(Level::from_root_folder(
            temp_dir.path().to_path_buf(),
            block_registry.clone(),
            0,
            Dimension::Overworld,
        ));
        for x in -5..5 {
            for y in -5..5 {
                let position = Vector2::new(x, y);
                let chunk = generator.generate_chunk(&level, block_registry.as_ref(), &position);
                chunks.push((position, Arc::new(RwLock::new(chunk))));
            }
        }

        for _ in 0..5 {
            // Mark the chunks as dirty so we save them again
            for (_, chunk) in &chunks {
                let mut chunk = chunk.write().await;
                chunk.dirty = true;
            }

            chunk_saver
                .save_chunks(&level_folder, chunks.clone())
                .await
                .expect("Failed to write chunk");

            // Create a new manager to ensure nothing is cached
            let chunk_saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::default();
            let read_chunks = get_chunks(&chunk_saver, &level_folder, &chunks).await;

            for (_, chunk) in &chunks {
                let chunk = chunk.read().await;
                for read_chunk in read_chunks.iter() {
                    let read_chunk = read_chunk.read().await;
                    if read_chunk.position == chunk.position {
                        let original = chunk.section.dump_blocks();
                        let read = read_chunk.section.dump_blocks();

                        original
                            .into_iter()
                            .zip(read)
                            .enumerate()
                            .for_each(|(i, (o, r))| {
                                if o != r {
                                    panic!("Data miss-match expected {o}, got {r} ({i})");
                                }
                            });

                        let original = chunk.section.dump_biomes();
                        let read = read_chunk.section.dump_biomes();

                        original
                            .into_iter()
                            .zip(read)
                            .enumerate()
                            .for_each(|(i, (o, r))| {
                                if o != r {
                                    panic!("Data miss-match expected {o}, got {r} ({i})");
                                }
                            });
                        break;
                    }
                }
            }
        }
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

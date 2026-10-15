use bytes::{Buf, BufMut, Bytes};
use flate2::read::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use itertools::Itertools;
use lz4_java_wrc::Context;
use pumpkin_config::chunk::AnvilChunkConfig;
use pumpkin_util::math::vector2::Vector2;
use std::{
    io::{Read, SeekFrom, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufWriter},
    sync::Mutex,
};
use tracing::{debug, trace};

use crate::chunk::{
    ChunkParsingError, ChunkReadingError, ChunkSerializingError, ChunkWritingError,
    CompressionError,
    io::{ChunkSerializer, Dirtiable, LoadedData, run_blocking},
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

// 26.2
pub const WORLD_DATA_VERSION: i32 = 4903;

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

enum WriteAction {
    // Don't write anything
    Pass,
    // Write the entire file
    All,
    // Only write certain indices
    Parts(Vec<usize>),
}

impl WriteAction {
    /// If we are currently not writing, sets to new Parts enum,
    /// If we have parts enum, add to it,
    /// If we have All enum, do nothing
    fn maybe_update_chunk_index(&mut self, index: usize) {
        match self {
            Self::Pass => *self = Self::Parts(vec![index]),
            Self::Parts(parts) => {
                if !parts.contains(&index) {
                    parts.push(index);
                }
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

        let padding_len = padded_size - self.raw_write_size();
        if padding_len > 0 {
            static PADDING: [u8; SECTOR_BYTES] = [0; SECTOR_BYTES];
            w.write_all(&PADDING[..padding_len]).await?;
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

            S::from_bytes(&decompress_bytes.into(), pos)
        } else {
            S::from_bytes(&self.compressed_data, pos)
        }
    }

    fn from_chunk<S>(
        chunk: &S,
        compression: Option<Compression>,
        chunk_config: &AnvilChunkConfig,
    ) -> Result<Self, ChunkWritingError>
    where
        S: SingleChunkDataSerializer,
    {
        let raw_bytes = chunk
            .to_bytes()
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        let compression = compression.unwrap_or_else(|| chunk_config.compression.algorithm.into());
        let level = chunk_config.compression.level;
        let compressed_data = compression
            .compress_data(&raw_bytes, level)
            .map_err(ChunkWritingError::Compression)?;

        Ok(Self {
            compression: Some(compression),
            compressed_data: compressed_data.into(),
        })
    }
}

impl<S: SingleChunkDataSerializer> AnvilChunkFile<S> {
    #[must_use]
    pub const fn get_region_coords(at: &Vector2<i32>) -> (i32, i32) {
        // Divide by 32 for the region coordinates
        (at.x >> SUBREGION_BITS, at.y >> SUBREGION_BITS)
    }

    #[must_use]
    pub const fn get_chunk_index(x: i32, z: i32) -> usize {
        let local_x = x & SUBREGION_AND;
        let local_z = z & SUBREGION_AND;
        let index = (local_z << SUBREGION_BITS) + local_x;
        index as usize
    }

    async fn write_indices<I>(&self, path: &Path, indices: I) -> Result<(), std::io::Error>
    where
        I: IntoIterator<Item = usize>,
    {
        trace!("Writing in place: {}", path.display());

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .append(false)
            .open(path)
            .await?;

        let mut write = BufWriter::new(file);
        // The first two sectors are reserved for the location table
        let mut header = Vec::with_capacity(SECTOR_BYTES * 2);

        // Location Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                let sector_count = chunk.serialized_data.sector_count();
                header.put_u32((chunk.file_sector_offset << 8) | sector_count);
            } else {
                header.put_u32(0);
            }
        }

        // Timestamp Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                header.put_u32(chunk.timestamp);
            } else {
                header.put_u32(0);
            }
        }

        // Write all 8 KiB in a single async call
        write.write_all(&header).await?;

        let mut chunks = indices
            .into_iter()
            .filter_map(|index| self.chunks_data[index].as_ref().map(|c| (index, c)))
            .collect::<Vec<_>>();

        // Sort such that writes are in order
        chunks.sort_by_key(|chunk| chunk.1.file_sector_offset);

        #[cfg(debug_assertions)]
        {
            // Verify we are actually two sectors into the file
            let current_pos = write.stream_position().await?;
            assert_eq!(current_pos as usize, 2 * SECTOR_BYTES);
        };

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
                trace!("Seeking to sector {}", chunk.file_sector_offset);
                let _ = write
                    .seek(SeekFrom::Start(
                        chunk.file_sector_offset as u64 * SECTOR_BYTES as u64,
                    ))
                    .await?;
                current_sector = chunk.file_sector_offset;
            }
            trace!(
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
        trace!("Writing tmp file to disk: {temp_path:?}");

        let file = tokio::fs::File::create(&temp_path).await?;
        let mut write = BufWriter::new(file);

        // Build the 8 KiB header in memory
        let mut header = Vec::with_capacity(SECTOR_BYTES * 2);
        let mut current_sector: u32 = 2;

        // Location Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                let sector_count = chunk.serialized_data.sector_count();
                header.put_u32((current_sector << 8) | sector_count);
                current_sector += sector_count;
            } else {
                header.put_u32(0);
            }
        }

        // Timestamp Table
        for metadata in &self.chunks_data {
            if let Some(chunk) = metadata {
                header.put_u32(chunk.timestamp);
            } else {
                header.put_u32(0);
            }
        }

        // Write all 8 KiB in a single async call
        write.write_all(&header).await?;

        // Write chunk data
        for chunk in self.chunks_data.iter().flatten() {
            chunk.serialized_data.write(&mut write).await?;
        }

        write.flush().await?;
        tokio::fs::rename(temp_path, path).await?;
        Ok(())
    }
}

#[expect(clippy::large_stack_arrays)]
impl<S: SingleChunkDataSerializer> Default for AnvilChunkFile<S> {
    fn default() -> Self {
        Self {
            chunks_data: [const { None }; CHUNK_COUNT],
            write_action: Mutex::new(WriteAction::Pass),
            // Two sectors for offset + timestamp
            end_sector: 2,
            _dummy: PhantomData,
        }
    }
}

pub trait SingleChunkDataSerializer: Send + Sync + Sized + Dirtiable + 'static {
    fn to_bytes(&self) -> Result<Bytes, ChunkSerializingError>;
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError>;
    fn position(&self) -> (i32, i32);
}

impl<S: SingleChunkDataSerializer + 'static> ChunkSerializer for AnvilChunkFile<S> {
    type Data = S;
    type WriteBackend = PathBuf;

    type ChunkConfig = AnvilChunkConfig;

    fn should_write(&self, is_watched: bool) -> bool {
        !is_watched
    }

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        let (region_x, region_z) = Self::get_region_coords(chunk);
        format!("./r.{region_x}.{region_z}.mca")
    }

    async fn write(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let mut write_action = self.write_action.lock().await;
        match &*write_action {
            WriteAction::Pass => {
                debug!(
                    "Skipping write for {}, as there were no dirty chunks",
                    path.display()
                );
                Ok(())
            }
            WriteAction::All => self.write_all(path).await,
            WriteAction::Parts(parts) => self.write_indices(path, parts.iter().copied()).await,
        }?;

        // If we still are in memory after this, we don't need to write again!
        *write_action = WriteAction::Pass;
        Ok(())
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let mut raw_file_bytes = r;

        if raw_file_bytes.is_empty() {
            return Ok(Self::default());
        }

        if raw_file_bytes.len() < SECTOR_BYTES * 2 {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let headers = raw_file_bytes.split_to(SECTOR_BYTES * 2);
        let (mut location_bytes, mut timestamp_bytes) = headers.split_at(SECTOR_BYTES);

        let mut chunk_file = Self::default();

        let mut last_offset = 2;
        for i in 0..CHUNK_COUNT {
            let timestamp = timestamp_bytes.get_u32();
            let location = location_bytes.get_u32();

            let sector_count = (location & 0xFF) as usize;
            let sector_offset = (location >> 8) as usize;
            let end_offset = sector_offset + sector_count;

            // If the sector offset or count is 0, the chunk is not present (we should not parse empty chunks).
            // Sector 1 is the timestamp table, so a chunk cannot start there either.
            if sector_offset < 2 || sector_count == 0 {
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

    #[expect(clippy::too_many_lines)]
    async fn update_chunk(
        &mut self,
        chunk: Arc<Self::Data>,
        chunk_config: &Self::ChunkConfig,
    ) -> Result<(), ChunkWritingError> {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        let index = Self::get_chunk_index(chunk.position().0, chunk.position().1);
        // Default to the compression type read from the file
        let compression_type = self.chunks_data[index]
            .as_ref()
            .and_then(|chunk_data| chunk_data.serialized_data.compression);
        let chunk_config_snapshot = chunk_config.clone();
        let new_chunk_data = run_blocking(move || {
            AnvilChunkData::from_chunk(&*chunk, compression_type, &chunk_config_snapshot)
        })
        .await
        .map_err(|_| {
            ChunkWritingError::IoError(std::io::Error::other("chunk serialization task failed"))
        })??;

        let mut write_action = self.write_action.lock().await;
        if !chunk_config.write_in_place {
            *write_action = WriteAction::All;
        }

        match &*write_action {
            WriteAction::All => {
                trace!("Write action is all: setting chunk in place");
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
                        trace!(
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
                            trace!(
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

                            // Only a chunk of exactly the old slot's size may take it.
                            if chunks_to_shift.last().is_none_or(|chunk| {
                                chunk.0 == index
                                    || chunk.1.serialized_data.sector_count()
                                        != old_chunk.serialized_data.sector_count()
                            }) {
                                trace!(
                                    "Unable to find a chunk to swap with; falling back to serialize all",
                                );

                                // give up...
                                *write_action = WriteAction::All;
                                self.chunks_data[index] = Some(AnvilChunkMetadata {
                                    serialized_data: new_chunk_data,
                                    timestamp: epoch,
                                    file_sector_offset: 0,
                                });
                            } else if let Some(swap) = chunks_to_shift.pop() {
                                // swap last element of the chunks to shift (the first because we
                                // reversed it) and shift the rest down
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

                                if let Some(swapped) = self.chunks_data[swapped_index].as_mut() {
                                    swapped.file_sector_offset = old_offset;
                                }
                                write_action.maybe_update_chunk_index(swapped_index);

                                // Then offset everything else

                                // If positive, now larger -> shift right, else shift left
                                let offset = new_sectors as i64 - swapped_sectors as i64;

                                trace!(
                                    "Swapping {index} with {swapped_index}, shifting all chunks {swapped_index} and after by {offset}"
                                );

                                for shift_index in indices_to_shift {
                                    if let Some(chunk_data) = self.chunks_data[shift_index].as_mut()
                                    {
                                        let new_offset =
                                            chunk_data.file_sector_offset as i64 + offset;
                                        chunk_data.file_sector_offset = new_offset as u32;
                                        write_action.maybe_update_chunk_index(shift_index);
                                    }
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
        chunks: Vec<Vector2<i32>>,
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        let chunk_items: Vec<(Vector2<i32>, Option<AnvilChunkData>)> = chunks
            .into_iter()
            .map(|chunk| {
                let index = Self::get_chunk_index(chunk.x, chunk.y);
                let data = self.chunks_data[index]
                    .as_ref()
                    .map(|chunk_metadata| chunk_metadata.serialized_data.clone());
                (chunk, data)
            })
            .collect();

        let (tx, mut rx) = tokio::sync::mpsc::channel(chunk_items.len().max(1));

        rayon::spawn(move || {
            use rayon::prelude::*;
            chunk_items
                .into_par_iter()
                .for_each(|(chunk, serialized_data)| {
                    let result = serialized_data.map_or_else(
                        || LoadedData::Missing(chunk),
                        |data| match data.to_chunk(chunk) {
                            Ok(chunk_res) => LoadedData::Loaded(chunk_res),
                            Err(err) => LoadedData::Error((chunk, err)),
                        },
                    );
                    let _ = tx.blocking_send(result);
                });
        });

        while let Some(item) = rx.recv().await {
            if stream.send(item).await.is_err() {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnvilChunkFile, Compression, CompressionError, SECTOR_BYTES};
    use crate::chunk::ChunkData;
    use crate::chunk::format::Dirtiable as _;
    use crate::chunk::io::{ChunkSerializer, LoadedData};
    use bytes::{BufMut, Bytes, BytesMut};

    use crate::chunk::io::FileIO;
    use crate::chunk::io::file_manager::ChunkFileManager;
    use crate::level::LevelFolder;
    use pumpkin_config::chunk::AnvilChunkConfig;
    use pumpkin_data::BlockStateId;
    use pumpkin_util::math::vector2::Vector2;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn level_folder(root: &std::path::Path) -> LevelFolder {
        let region_folder = root.join("region");
        std::fs::create_dir_all(&region_folder).expect("region folder");
        LevelFolder {
            root_folder: root.to_path_buf(),
            dim_folder: root.to_path_buf(),
            region_folder,
            entities_folder: PathBuf::from("entities"),
            poi_folder: PathBuf::from("poi"),
        }
    }

    /// `fill` is how many layers are non-air, which sets how many sectors the chunk takes.
    fn chunk_at(position: Vector2<i32>, fill: usize) -> Arc<ChunkData> {
        let chunk = ChunkData::empty(position.x, position.y);
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..fill {
                    let raw = 1 + u16::try_from((x + z + y) % 60).expect("small");
                    let state = BlockStateId::new(raw).expect("a valid block state");
                    chunk.section.set_block_no_heightmap_update(x, y, z, state);
                }
            }
        }
        // Only a dirty chunk is handed to the serializer.
        chunk.mark_dirty(true);
        Arc::new(chunk)
    }

    async fn save_and_reload(
        folder: &LevelFolder,
        chunks: &[(Vector2<i32>, Arc<ChunkData>)],
        config: AnvilChunkConfig,
    ) -> Vec<Arc<ChunkData>> {
        let saver = ChunkFileManager::<AnvilChunkFile<ChunkData>>::new(config.clone());
        saver
            .save_chunks(folder, chunks.to_vec())
            .await
            .expect("chunks are saved");

        // A fresh manager, so nothing is served out of the cache.
        let reader = ChunkFileManager::<AnvilChunkFile<ChunkData>>::new(config);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(chunks.len().max(1));
        let positions: Vec<Vector2<i32>> = chunks.iter().map(|(position, _)| *position).collect();
        reader.fetch_chunks(folder, &positions, sender).await;

        let mut out = Vec::new();
        while let Some(data) = receiver.recv().await {
            match data {
                LoadedData::Loaded(chunk) => out.push(chunk),
                LoadedData::Missing(position) => panic!("chunk {position:?} went missing"),
                LoadedData::Error((position, error)) => {
                    panic!("chunk {position:?} failed to load: {error}")
                }
            }
        }
        out
    }

    fn assert_round_trip(chunks: &[(Vector2<i32>, Arc<ChunkData>)], read_back: &[Arc<ChunkData>]) {
        assert_eq!(read_back.len(), chunks.len(), "every chunk comes back");
        for (position, written) in chunks {
            let mut found = false;
            for read in read_back {
                if (read.x, read.z) != (written.x, written.z) {
                    continue;
                }
                found = true;
                assert_eq!(
                    written.section.dump_blocks(),
                    read.section.dump_blocks(),
                    "blocks differ at {position:?}"
                );
                assert_eq!(
                    written.section.dump_biomes(),
                    read.section.dump_biomes(),
                    "biomes differ at {position:?}"
                );
            }
            assert!(found, "chunk {position:?} is in the region file");
        }
    }

    fn both_write_modes() -> [AnvilChunkConfig; 2] {
        [
            AnvilChunkConfig {
                write_in_place: false,
                ..Default::default()
            },
            AnvilChunkConfig {
                write_in_place: true,
                ..Default::default()
            },
        ]
    }

    /// Chunks rewritten over their own sectors read back unchanged.
    #[tokio::test(flavor = "multi_thread")]
    async fn chunks_read_back_unchanged() {
        for config in both_write_modes() {
            let temp = tempfile::tempdir().expect("temp dir");
            let folder = level_folder(temp.path());

            let chunks: Vec<_> = (0..8)
                .map(|i| {
                    let position = Vector2::new(i % 4, i / 4);
                    (position, chunk_at(position, 4))
                })
                .collect();

            let read_back = save_and_reload(&folder, &chunks, config.clone()).await;
            assert_round_trip(&chunks, &read_back);

            // Saving clears the dirty flag; set it so the second save rewrites them.
            for (_, chunk) in &chunks {
                chunk.mark_dirty(true);
            }
            let read_back = save_and_reload(&folder, &chunks, config).await;
            assert_round_trip(&chunks, &read_back);
        }
    }

    /// A grown chunk swaps with one of its old size, and the chunks after that one shift.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_chunk_that_outgrows_its_sectors_keeps_its_neighbours_intact() {
        for config in both_write_modes() {
            let temp = tempfile::tempdir().expect("temp dir");
            let folder = level_folder(temp.path());

            // 0 and 3 are small: growing 0 swaps it with 3 and shifts 4 and 5.
            let mut chunks: Vec<_> = (0..6)
                .map(|i| {
                    let position = Vector2::new(i, 0);
                    (
                        position,
                        chunk_at(position, if i == 0 || i == 3 { 2 } else { 120 }),
                    )
                })
                .collect();
            let read_back = save_and_reload(&folder, &chunks, config.clone()).await;
            assert_round_trip(&chunks, &read_back);

            chunks[0] = (chunks[0].0, chunk_at(chunks[0].0, 120));
            let read_back = save_and_reload(&folder, &chunks, config).await;
            assert_round_trip(&chunks, &read_back);
        }
    }

    /// With no chunk of the old slot's size to swap with, the region is written whole.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_chunk_with_nothing_to_swap_with_keeps_its_neighbours_intact() {
        for config in both_write_modes() {
            let temp = tempfile::tempdir().expect("temp dir");
            let folder = level_folder(temp.path());

            // None of the last 64 is the first chunk's size.
            let mut chunks: Vec<_> = (0..71)
                .map(|i| {
                    let position = Vector2::new(i % 32, i / 32);
                    (position, chunk_at(position, if i == 0 { 2 } else { 120 }))
                })
                .collect();
            let read_back = save_and_reload(&folder, &chunks, config.clone()).await;
            assert_round_trip(&chunks, &read_back);

            chunks[0] = (chunks[0].0, chunk_at(chunks[0].0, 120));
            let read_back = save_and_reload(&folder, &chunks, config).await;
            assert_round_trip(&chunks, &read_back);
        }
    }

    /// A region file whose first location entry is `location`, whose other
    /// 1023 entries are absent, and whose single payload sector starts with
    /// `declared_len` followed by the "no compression" marker.
    fn region_with_first_location(location: u32, declared_len: u32) -> Bytes {
        let mut buf = BytesMut::with_capacity(SECTOR_BYTES * 3);
        buf.put_u32(location);
        buf.put_bytes(0, SECTOR_BYTES - 4);
        buf.put_bytes(0, SECTOR_BYTES);

        buf.put_u32(declared_len);
        buf.put_u8(3);
        buf.put_bytes(0, SECTOR_BYTES - 5);

        buf.freeze()
    }

    #[test]
    fn a_chunk_pointing_into_the_header_is_skipped() {
        // Sectors 0 and 1 hold the location and timestamp tables, so a chunk
        // cannot start before sector 2. Offset 0 is already skipped; offset 1
        // is just as impossible and used to be subtracted from anyway.
        let file = AnvilChunkFile::<ChunkData>::read(region_with_first_location((1 << 8) | 1, 1))
            .expect("one bad location entry should not fail the whole region");

        assert!(file.chunks_data[0].is_none());
    }

    #[test]
    fn a_chunk_at_the_first_free_sector_is_read() {
        // The control for the test above: sector 2 is the first legal one, and
        // a chunk there must not be skipped.
        let file = AnvilChunkFile::<ChunkData>::read(region_with_first_location((2 << 8) | 1, 1))
            .expect("a chunk at the first free sector is well formed");

        assert!(file.chunks_data[0].is_some());
    }

    #[test]
    fn a_chunk_declaring_no_length_is_an_error_not_a_panic() {
        // The length covers the compression byte, so the smallest legal value
        // is 1 and the byte count is that minus one. Zero came from a file, so
        // it has to be rejected rather than subtracted from.
        let file = AnvilChunkFile::<ChunkData>::read(region_with_first_location((2 << 8) | 1, 0));

        assert!(file.is_err());
    }

    #[test]
    fn a_chunk_declaring_one_byte_too_many_is_an_error_not_a_panic() {
        // The length is checked while the compression byte is still unread, so
        // a chunk claiming exactly the bytes that remain used to pass the check
        // and then slice one byte past the end.
        let file = AnvilChunkFile::<ChunkData>::read(region_with_first_location(
            (2 << 8) | 1,
            SECTOR_BYTES as u32 - 3,
        ));

        assert!(file.is_err());
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
}

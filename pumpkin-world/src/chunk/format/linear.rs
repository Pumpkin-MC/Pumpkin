use std::io::ErrorKind;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chunk::format::anvil::AnvilChunkFile;
use crate::chunk::io::{ChunkSerializer, LoadedData};
use crate::chunk::{ChunkData, ChunkReadingError, ChunkWritingError};
use async_trait::async_trait;
use bytes::{Buf, BufMut};
use log::error;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zstd::zstd_safe::WriteBuf;

use super::anvil::{CHUNK_COUNT, chunk_to_bytes};

/// The signature of the linear file format
/// used as a header and footer described in https://gist.github.com/Aaron2550/5701519671253d4c6190bde6706f9f98
const SIGNATURE: [u8; 8] = u64::to_be_bytes(0xc3ff13183cca9d9a);

#[derive(Default, Clone, Copy)]
struct LinearChunkHeader {
    size: u32,
    timestamp: u32,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum LinearVersion {
    #[default]
    /// Represents an invalid or uninitialized version.
    None = 0x00,
    /// Version 1 of the Linear Region File Format. (Default)
    ///
    /// Described in: https://github.com/xymb-endcrystalme/LinearRegionFileFormatTools/blob/linearv2/LINEAR.md
    V1 = 0x01,
    /// Version 2 of the Linear Region File Format (currently unsupported).
    ///
    /// Described in: https://github.com/xymb-endcrystalme/LinearRegionFileFormatTools/blob/linearv2/LINEARv2.md
    V2 = 0x02,
}
struct LinearFileHeader {
    /// ( 0.. 1 Bytes) The version of the Linear Region File format.
    version: LinearVersion,
    /// ( 1.. 9 Bytes) The timestamp of the newest chunk in the region file.
    newest_timestamp: u64,
    /// ( 9..10 Bytes) The zstd compression level used for chunk data.
    compression_level: u8,
    /// (10..12 Bytes) The number of non-zero-size chunks in the region file.
    chunks_count: u16,
    /// (12..16 Bytes) The total size in bytes of the compressed chunk headers and chunk data.
    chunks_bytes: usize,
    /// (16..24 Bytes) A hash of the region file (unused).
    region_hash: u64,
}
pub struct LinearFile {
    chunks_headers: [LinearChunkHeader; CHUNK_COUNT],
    chunks_data: [Option<Box<[u8]>>; CHUNK_COUNT],
}

impl LinearChunkHeader {
    const CHUNK_HEADER_SIZE: usize = 8;
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut bytes = bytes;
        LinearChunkHeader {
            size: bytes.get_u32(),
            timestamp: bytes.get_u32(),
        }
    }

    fn to_bytes(self) -> Box<[u8]> {
        let mut bytes = Vec::with_capacity(LinearChunkHeader::CHUNK_HEADER_SIZE);

        bytes.put_u32(self.size);
        bytes.put_u32(self.timestamp);

        // This should be a clear code error if the size of the header is not the expected
        // so we can unwrap the conversion safely or panic the entire program if not
        bytes.into_boxed_slice()
    }
}

impl From<u8> for LinearVersion {
    fn from(value: u8) -> Self {
        match value {
            0x01 => LinearVersion::V1,
            0x02 => LinearVersion::V2,
            _ => LinearVersion::None,
        }
    }
}

impl LinearFileHeader {
    const FILE_HEADER_SIZE: usize = 24;

    fn check_version(&self) -> Result<(), ChunkReadingError> {
        match self.version {
            LinearVersion::None => {
                error!("Invalid version in the file header");
                Err(ChunkReadingError::InvalidHeader)
            }
            LinearVersion::V2 => {
                error!("LinearFormat Version 2 for Chunks is not supported yet");
                Err(ChunkReadingError::InvalidHeader)
            }
            _ => Ok(()),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = bytes;

        LinearFileHeader {
            version: buf.get_u8().into(),
            newest_timestamp: buf.get_u64(),
            compression_level: buf.get_u8(),
            chunks_count: buf.get_u16(),
            chunks_bytes: buf.get_u32() as usize,
            region_hash: buf.get_u64(),
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes: Vec<u8> = Vec::with_capacity(LinearFileHeader::FILE_HEADER_SIZE);

        bytes.put_u8(self.version as u8);
        bytes.put_u64(self.newest_timestamp);
        bytes.put_u8(self.compression_level);
        bytes.put_u16(self.chunks_count);
        bytes.put_u32(self.chunks_bytes as u32);
        bytes.put_u64(self.region_hash);

        // This should be a clear code error if the size of the header is not the expected
        // so we can unwrap the conversion safely or panic the entire program if not
        bytes.into_boxed_slice()
    }
}

impl LinearFile {
    const fn get_chunk_index(at: &Vector2<i32>) -> usize {
        AnvilChunkFile::get_chunk_index(at)
    }

    fn check_signature(bytes: &[u8]) -> Result<(), ChunkReadingError> {
        if bytes != SIGNATURE {
            error!("Linear signature is invalid!");
            Err(ChunkReadingError::InvalidHeader)
        } else {
            Ok(())
        }
    }
}

impl Default for LinearFile {
    fn default() -> Self {
        LinearFile {
            chunks_headers: [LinearChunkHeader::default(); CHUNK_COUNT],
            chunks_data: [const { None }; CHUNK_COUNT],
        }
    }
}

#[async_trait]
impl ChunkSerializer for LinearFile {
    type Data = ChunkData;

    fn get_chunk_key(chunk: Vector2<i32>) -> String {
        let (region_x, region_z) = AnvilChunkFile::get_region_coords(chunk);
        format!("./r.{}.{}.linear", region_x, region_z)
    }

    async fn write(&self, w: impl AsyncWrite + Unpin + Send) -> Result<(), std::io::Error> {
        let mut writer = w;

        // Parse the headers to a buffer
        let mut data_buffer: Vec<u8> = self
            .chunks_headers
            .iter()
            .flat_map(|header| header.to_bytes())
            .collect();

        for chunk in self.chunks_data.iter().flatten() {
            data_buffer.extend_from_slice(chunk);
        }

        // TODO: maybe zstd lib has memory leaks
        let compressed_buffer = zstd::bulk::compress(
            data_buffer.as_slice(),
            ADVANCED_CONFIG.chunk.compression.level as i32,
        )
        .expect("Failed to compress the data buffer")
        .into_boxed_slice();

        let file_header = LinearFileHeader {
            chunks_bytes: compressed_buffer.len(),
            compression_level: ADVANCED_CONFIG.chunk.compression.level as u8,
            chunks_count: self
                .chunks_headers
                .iter()
                .filter(|&header| header.size != 0)
                .count() as u16,
            newest_timestamp: self
                .chunks_headers
                .iter()
                .map(|header| header.timestamp)
                .max()
                .unwrap_or(0) as u64,
            version: LinearVersion::V1,
            region_hash: 0,
        }
        .to_bytes();

        // TODO: Can we stream this?
        writer.write_all(&SIGNATURE).await?;
        writer.write_all(file_header.as_slice()).await?;
        writer.write_all(compressed_buffer.as_slice()).await?;
        writer.write_all(&SIGNATURE).await?;

        Ok(())
    }

    async fn read(r: impl AsyncRead + Unpin + Send) -> Result<Self, ChunkReadingError> {
        let mut file_reader = r;

        let mut raw_file = Vec::new();
        file_reader
            .read_to_end(&mut raw_file)
            .await
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let Some((signature, raw_file_bytes)) = raw_file.split_at_checked(SIGNATURE.len()) else {
            return Err(ChunkReadingError::IoError(ErrorKind::UnexpectedEof));
        };

        Self::check_signature(signature)?;

        let Some((header_bytes, raw_file_bytes)) =
            raw_file_bytes.split_at_checked(LinearFileHeader::FILE_HEADER_SIZE)
        else {
            return Err(ChunkReadingError::IoError(ErrorKind::UnexpectedEof));
        };

        // Parse the header
        let file_header = LinearFileHeader::from_bytes(header_bytes);
        file_header.check_version()?;

        let Some((raw_file_bytes, signature)) =
            raw_file_bytes.split_at_checked(file_header.chunks_bytes)
        else {
            return Err(ChunkReadingError::IoError(ErrorKind::UnexpectedEof));
        };

        Self::check_signature(signature)?;

        // TODO: Review the buffer size limit or find ways to improve performance (maybe zstd lib has memory leaks)
        let buffer = zstd::bulk::decompress(raw_file_bytes, 200 * 1024 * 1024) // 200MB limit for the decompression buffer size
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let (headers_buffer, buffer) =
            buffer.split_at(LinearChunkHeader::CHUNK_HEADER_SIZE * CHUNK_COUNT);

        // Parse the chunk headers
        let chunk_headers: [LinearChunkHeader; CHUNK_COUNT] = headers_buffer
            .chunks_exact(8)
            .map(LinearChunkHeader::from_bytes)
            .collect::<Vec<LinearChunkHeader>>()
            .try_into()
            .map_err(|_| ChunkReadingError::InvalidHeader)?;

        // Check if the total bytes of the chunks match the header
        let total_bytes = chunk_headers.iter().map(|header| header.size).sum::<u32>() as usize;
        if buffer.len() != total_bytes {
            error!(
                "Invalid total bytes of the chunks {} != {}",
                total_bytes,
                buffer.len(),
            );
            return Err(ChunkReadingError::InvalidHeader);
        }

        let mut chunks = [const { None }; CHUNK_COUNT];
        let mut bytes_offset = 0;
        for (i, header) in chunk_headers.iter().enumerate() {
            if header.size != 0 {
                let last_index = bytes_offset;
                bytes_offset += header.size as usize;
                chunks[i] = Some(buffer[last_index..bytes_offset].into());
            }
        }

        Ok(LinearFile {
            chunks_headers: chunk_headers,
            chunks_data: chunks,
        })
    }

    fn update_chunks(&mut self, chunks_data: &[&Self::Data]) -> Result<(), ChunkWritingError> {
        for chunk_data in chunks_data {
            let index = LinearFile::get_chunk_index(&chunk_data.position);
            let chunk_raw = chunk_to_bytes(chunk_data)
                .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?
                .into_boxed_slice();

            let header = &mut self.chunks_headers[index];
            header.size = chunk_raw.len() as u32;
            header.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;

            // We update the data buffer
            self.chunks_data[index] = Some(chunk_raw);
        }

        Ok(())
    }

    fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
    ) -> Vec<LoadedData<Self::Data, ChunkReadingError>> {
        chunks
            .par_iter()
            .map(|chunk| {
                let index = LinearFile::get_chunk_index(chunk);
                if let Some(data) = &self.chunks_data[index] {
                    match ChunkData::from_bytes(data.as_slice(), *chunk) {
                        Ok(chunk) => LoadedData::Loaded(chunk),
                        Err(err) => {
                            LoadedData::Error((*chunk, ChunkReadingError::ParsingError(err)))
                        }
                    }
                } else {
                    LoadedData::Missing(*chunk)
                }
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use temp_dir::TempDir;
    use tokio::sync::RwLock;

    use crate::chunk::format::linear::LinearFile;
    use crate::chunk::io::chunk_file_manager::ChunkFileManager;
    use crate::chunk::io::{ChunkIO, LoadedData};
    use crate::generation::{Seed, get_world_gen};
    use crate::level::LevelFolder;

    #[tokio::test(flavor = "multi_thread")]
    async fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<LinearFile>::default();

        let chunks = chunk_saver
            .fetch_chunks(
                &LevelFolder {
                    root_folder: PathBuf::from(""),
                    region_folder: region_path,
                },
                &[Vector2::new(0, 0)],
            )
            .await;

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
        let chunk_saver = ChunkFileManager::<LinearFile>::default();

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

            let read_chunks = chunk_saver
                .fetch_chunks(
                    &level_folder,
                    &chunks.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
                )
                .await
                .into_iter()
                .map(|chunk| match chunk {
                    LoadedData::Loaded(chunk) => chunk,
                    LoadedData::Missing(_) => panic!("Missing chunk"),
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

        println!("Checked chunks successfully");
    }
}

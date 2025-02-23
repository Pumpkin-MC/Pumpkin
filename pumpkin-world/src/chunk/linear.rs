use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chunk::anvil::AnvilChunkFile;
use crate::chunks_io::{ChunkSerializer, LoadedData};
use async_trait::async_trait;
use bytes::{Buf, BufMut};
use log::error;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::sync::Notify;

use super::anvil::{CHUNK_COUNT, SUBREGION_BITS, chunk_to_bytes};
use super::{ChunkData, ChunkReadingError, ChunkWritingError};

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
    chunks_bytes: u32,
    /// (16..24 Bytes) A hash of the region file (unused).
    region_hash: u64,
}
pub struct LinearFile {
    chunks_headers: [LinearChunkHeader; CHUNK_COUNT],
    chunks_data: [Option<Arc<[u8]>>; CHUNK_COUNT],
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

    fn to_bytes(self) -> [u8; 8] {
        let mut bytes = Vec::with_capacity(LinearChunkHeader::CHUNK_HEADER_SIZE);

        bytes.put_u32(self.size);
        bytes.put_u32(self.timestamp);

        // This should be a clear code error if the size of the header is not the expected
        // so we can unwrap the conversion safely or panic the entire program if not
        bytes
            .try_into()
            .unwrap_or_else(|_| panic!("ChunkHeader Struct/Size Mismatch"))
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
            chunks_bytes: buf.get_u32(),
            region_hash: buf.get_u64(),
        }
    }

    fn to_bytes(&self) -> [u8; Self::FILE_HEADER_SIZE] {
        let mut bytes: Vec<u8> = Vec::with_capacity(LinearFileHeader::FILE_HEADER_SIZE);

        bytes.put_u8(self.version as u8);
        bytes.put_u64(self.newest_timestamp);
        bytes.put_u8(self.compression_level);
        bytes.put_u16(self.chunks_count);
        bytes.put_u32(self.chunks_bytes);
        bytes.put_u64(self.region_hash);

        // This should be a clear code error if the size of the header is not the expected
        // so we can unwrap the conversion safely or panic the entire program if not
        bytes
            .try_into()
            .unwrap_or_else(|_| panic!("Header Struct/Size Mismatch"))
    }
}

impl LinearFile {
    const fn get_chunk_index(at: &Vector2<i32>) -> usize {
        // we need only the 5 last bits of the x and z coordinates
        let decode_x = at.x - ((at.x >> SUBREGION_BITS) << SUBREGION_BITS);
        let decode_z = at.z - ((at.z >> SUBREGION_BITS) << SUBREGION_BITS);

        // we calculate the index of the chunk in the region file
        ((decode_z << SUBREGION_BITS) + decode_x) as usize
    }
    fn check_signature(bytes: &[u8]) -> Result<(), ChunkReadingError> {
        if bytes[0..8] != SIGNATURE {
            error!("Signature at the start of the file is invalid");
            return Err(ChunkReadingError::InvalidHeader);
        }

        if bytes[bytes.len() - 8..] != SIGNATURE {
            error!("Signature at the end of the file is invalid");
            return Err(ChunkReadingError::InvalidHeader);
        }

        Ok(())
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

    fn to_bytes(&self) -> Box<[u8]> {
        // Parse the headers to a buffer
        let headers_buffer: Vec<u8> = self
            .chunks_headers
            .iter()
            .flat_map(|header| header.to_bytes())
            .collect();

        let uncompressed_size = self
            .chunks_headers
            .iter()
            .map(|header| header.size)
            .sum::<u32>();

        let mut chunks_bytes = Vec::with_capacity(uncompressed_size as usize);
        for chunk in self.chunks_data.iter().flatten() {
            chunks_bytes.put_slice(chunk);
        }

        // Compress the data buffer
        let compressed_buffer = zstd::encode_all(
            [headers_buffer.as_slice(), chunks_bytes.as_slice()]
                .concat()
                .as_slice(),
            ADVANCED_CONFIG.chunk.compression.level as i32,
        )
        .unwrap();

        let file_header = LinearFileHeader {
            chunks_bytes: compressed_buffer.len() as u32,
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

        [
            SIGNATURE.as_slice(),
            file_header.as_slice(),
            compressed_buffer.as_slice(),
            SIGNATURE.as_slice(),
        ]
        .concat()
        .into_boxed_slice()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError> {
        Self::check_signature(bytes)?;

        let mut file = Cursor::new(bytes);

        // Skip the signature and read the header
        let mut header_bytes = [0; LinearFileHeader::FILE_HEADER_SIZE];
        file.seek(SeekFrom::Start(8))
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        file.read_exact(&mut header_bytes)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        // Parse the header
        let file_header = LinearFileHeader::from_bytes(&header_bytes);
        file_header.check_version()?;

        // Read the compressed data
        let mut compressed_data = vec![0; file_header.chunks_bytes as usize];
        file.read_exact(compressed_data.as_mut_slice())
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        if compressed_data.len() != file_header.chunks_bytes as usize {
            error!(
                "Invalid compressed data size {} != {}",
                compressed_data.len(),
                file_header.chunks_bytes
            );
            return Err(ChunkReadingError::InvalidHeader);
        }

        // Uncompress the data (header + chunks)
        let mut buffer = zstd::decode_all(compressed_data.as_slice())
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let headers_buffer: Vec<u8> = buffer
            .drain(..LinearChunkHeader::CHUNK_HEADER_SIZE * CHUNK_COUNT)
            .collect();

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
        for (i, header) in chunk_headers.iter().enumerate() {
            if header.size != 0 {
                chunks[i] = Some(buffer.drain(..header.size as usize).collect());
            }
        }

        Ok(LinearFile {
            chunks_headers: chunk_headers,
            chunks_data: chunks,
        })
    }

    fn add_chunk_data(&mut self, chunks_data: &[Self::Data]) -> Result<(), ChunkWritingError> {
        for chunk_data in chunks_data {
            let index = LinearFile::get_chunk_index(&chunk_data.position);
            let chunk_raw = chunk_to_bytes(chunk_data)
                .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

            let header = &mut self.chunks_headers[index];
            header.size = chunk_raw.len() as u32;
            header.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;

            // We update the data buffer
            self.chunks_data[index] = Some(chunk_raw.into());
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
            let index = LinearFile::get_chunk_index(chunk);
            if let Some(data) = &self.chunks_data[index] {
                chunk_data.push((*chunk, data.clone(), channel.clone()))
            } else {
                channel
                    .send(LoadedData::Missing(*chunk))
                    .await
                    .expect("Failed to stream missing message from linear chunk serializer!");
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
                .for_each(|(at, chunk_bytes, channel)| {
                    let result = match ChunkData::from_bytes(&chunk_bytes, at) {
                        Ok(chunk) => LoadedData::Loaded(chunk),
                        Err(err) => LoadedData::Error((at, ChunkReadingError::ParsingError(err))),
                    };

                    channel
                        .blocking_send(result)
                        .expect("Failed to stream data from linear chunk serializer!");
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

#[cfg(test)]
mod tests {
    use core::panic;
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use temp_dir::TempDir;
    use tokio::sync::{RwLock, mpsc};

    use crate::generation::{Seed, get_world_gen};
    use crate::{
        chunk::linear::LinearFile,
        chunks_io::{ChunkFileManager, ChunkIO, LoadedData},
        level::LevelFolder,
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<LinearFile>::default();
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
}

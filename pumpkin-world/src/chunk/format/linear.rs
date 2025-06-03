use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chunk::format::anvil::AnvilChunkFile;
use crate::chunk::io::{ChunkSerializer, LoadedData};
use crate::chunk::{ChunkData, ChunkReadingError, ChunkWritingError};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes};
use log::error;
use pumpkin_config::advanced_config;
use pumpkin_util::math::vector2::Vector2;
use tokio::io::{AsyncWriteExt, BufWriter};

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
    chunks_data: [Option<Bytes>; CHUNK_COUNT],
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
    type WriteBackend = PathBuf;

    fn should_write(&self, is_watched: bool) -> bool {
        !is_watched
    }

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        let (region_x, region_z) = AnvilChunkFile::get_region_coords(chunk);
        format!("./r.{region_x}.{region_z}.linear")
    }

    async fn write(&self, path: PathBuf) -> Result<(), std::io::Error> {
        let temp_path = path.with_extension("tmp");
        log::trace!("Writing tmp file to disk: {:?}", temp_path);

        let file = tokio::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .await?;

        let mut write = BufWriter::new(file);

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
            advanced_config().chunk.compression.level as i32,
        )
        .expect("Failed to compress the data buffer")
        .into_boxed_slice();

        let file_header = LinearFileHeader {
            chunks_bytes: compressed_buffer.len(),
            compression_level: advanced_config().chunk.compression.level as u8,
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

        write.write_all(&SIGNATURE).await?;
        write.write_all(&file_header).await?;
        write.write_all(&compressed_buffer).await?;
        write.write_all(&SIGNATURE).await?;

        write.flush().await?;

        // The rename of the file works like an atomic operation ensuring
        // that the data is not corrupted before the rename is completed
        tokio::fs::rename(temp_path, &path).await?;

        log::trace!("Wrote file to Disk: {:?}", path);
        Ok(())
    }

    fn read(raw_file: Bytes) -> Result<Self, ChunkReadingError> {
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
        let mut buffer: Bytes = zstd::bulk::decompress(raw_file_bytes, 200 * 1024 * 1024) // 200MB limit for the decompression buffer size
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?
            .into();

        let headers_buffer = buffer.split_to(LinearChunkHeader::CHUNK_HEADER_SIZE * CHUNK_COUNT);

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
                if bytes_offset > buffer.len() {
                    log::warn!(
                        "Not enough bytes are available for chunk {} ({} vs {})",
                        i,
                        header.size,
                        buffer.len() - last_index
                    );
                } else {
                    chunks[i] = Some(buffer.slice(last_index..bytes_offset));
                }
            }
        }

        Ok(LinearFile {
            chunks_headers: chunk_headers,
            chunks_data: chunks,
        })
    }

    async fn update_chunk(&mut self, chunk: &ChunkData) -> Result<(), ChunkWritingError> {
        let index = LinearFile::get_chunk_index(&chunk.position);
        let chunk_raw: Bytes = chunk_to_bytes(chunk)
            .await
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?
            .into();

        let header = &mut self.chunks_headers[index];
        header.size = chunk_raw.len() as u32;
        header.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // We update the data buffer
        self.chunks_data[index] = Some(chunk_raw);

        Ok(())
    }

    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<ChunkData, ChunkReadingError>>,
    ) {
        // Don't par iter here so we can prevent backpressure with the await in the async
        // runtime
        for chunk in chunks.iter().cloned() {
            let index = LinearFile::get_chunk_index(&chunk);
            let linear_chunk_data = &self.chunks_data[index];

            let result = if let Some(data) = linear_chunk_data {
                match ChunkData::from_bytes(data, chunk).map_err(ChunkReadingError::ParsingError) {
                    Ok(chunk) => LoadedData::Loaded(chunk),
                    Err(err) => LoadedData::Error((chunk, err)),
                }
            } else {
                LoadedData::Missing(chunk)
            };

            if stream.send(result).await.is_err() {
                // The stream is closed. Return early to prevent unneeded work and IO
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use core::panic;
    use pumpkin_data::BlockDirection;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use temp_dir::TempDir;
    use tokio::sync::RwLock;

    use crate::chunk::format::linear::LinearFile;
    use crate::chunk::io::chunk_file_manager::ChunkFileManager;
    use crate::chunk::io::{ChunkIO, LoadedData};
    use crate::dimension::Dimension;
    use crate::generation::{Seed, get_world_gen};
    use crate::level::{Level, LevelFolder};
    use crate::world::{BlockAccessor, BlockRegistryExt};

    struct BlockRegistry;

    #[async_trait]
    impl BlockRegistryExt for BlockRegistry {
        async fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _block_accessor: &dyn BlockAccessor,
            _block_pos: &BlockPos,
            _face: BlockDirection,
        ) -> bool {
            true
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let chunk_saver = ChunkFileManager::<LinearFile>::default();

        let mut chunks = Vec::new();
        let (send, mut recv) = tokio::sync::mpsc::channel(1);

        chunk_saver
            .fetch_chunks(
                &LevelFolder {
                    root_folder: PathBuf::from(""),
                    region_folder: region_path,
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
    async fn test_writing() {
        let _ = env_logger::try_init();

        let generator = get_world_gen(Seed(0), Dimension::Overworld);

        let temp_dir = TempDir::new().unwrap();
        let level_folder = LevelFolder {
            root_folder: temp_dir.path().to_path_buf(),
            region_folder: temp_dir.path().join("region"),
        };
        fs::create_dir(&level_folder.region_folder).expect("couldn't create region folder");
        let chunk_saver = ChunkFileManager::<LinearFile>::default();
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
                let chunk = generator
                    .generate_chunk(&level, block_registry.as_ref(), &position)
                    .await;
                chunks.push((position, Arc::new(RwLock::new(chunk))));
            }
        }

        for i in 0..5 {
            println!("Iteration {}", i + 1);
            // Mark the chunks as dirty so we save them again
            for (_, chunk) in &chunks {
                let mut chunk = chunk.write().await;
                chunk.dirty = true;
            }

            chunk_saver
                .save_chunks(
                    &level_folder,
                    chunks.clone().into_iter().collect::<Vec<_>>(),
                )
                .await
                .expect("Failed to write chunk");

            let mut read_chunks = Vec::new();
            let (send, mut recv) = tokio::sync::mpsc::channel(1);

            let chunk_pos = chunks.iter().map(|(at, _)| *at).collect::<Vec<_>>();
            let spawn = chunk_saver.fetch_chunks(&level_folder, &chunk_pos, send);

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
                        panic!("Error reading chunk at {:?} | Error: {:?}", position, error)
                    }
                })
                .collect::<Vec<_>>();

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
                                    panic!("Data miss-match expected {}, got {} ({})", o, r, i);
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
                                    panic!("Data miss-match expected {}, got {} ({})", o, r, i);
                                }
                            });
                        break;
                    }
                }
            }
        }

        println!("Checked chunks successfully");
    }
}

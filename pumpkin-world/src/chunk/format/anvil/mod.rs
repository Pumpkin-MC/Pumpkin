use bytes::Bytes;
use flate2::read::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};
use lz4_java_wrc::Context;
use pumpkin_util::math::vector2::Vector2;
use std::{
    io::{Read, Write},
    marker::PhantomData,
    pin::Pin,
};
use tokio::sync::Mutex;

use crate::chunk::{ChunkReadingError, ChunkSerializingError, CompressionError, io::Dirtiable};

mod data;
mod file;

/// The side size of a region in chunks (one region is 32x32 chunks)
pub const REGION_SIZE: usize = 32;

/// The number of bits that identify two chunks in the same region
pub const SUBREGION_BITS: u8 = pumpkin_util::math::ceil_log2(REGION_SIZE as u32);

pub const SUBREGION_AND: i32 = i32::pow(2, SUBREGION_BITS as u32) - 1;

/// The number of chunks in a region
pub const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;

/// The number of bytes in a sector (4 KiB)
const SECTOR_BYTES: usize = 4096;

// 26.1.2
pub const WORLD_DATA_VERSION: i32 = 4790;

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

pub trait SingleChunkDataSerializer: Send + Sync + Sized + Dirtiable {
    fn to_bytes(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>>;
    fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError>;
    fn position(&self) -> (i32, i32);
}

/*
#[cfg(test)]
mod tests {

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
    async fn write_in_place() {
        let mut config = AdvancedConfiguration::default();
        config.chunk.write_in_place = true;
        override_config_for_testing(config);
        assert!(advanced_config().chunk.write_in_place);

        let _ = env_logger::try_init();

        let generator = get_world_gen(Seed(0), Dimension::Overworld, false, Vec::new(), String::new());

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
    async fn write_bulk() {
        let mut config = AdvancedConfiguration::default();
        config.chunk.write_in_place = false;
        override_config_for_testing(config);
        assert!(!advanced_config().chunk.write_in_place);

        let _ = env_logger::try_init();

        let generator = get_world_gen(Seed(0), Dimension::Overworld, false, Vec::new(), String::new());

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
    fn load_java_chunk() {
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
 */
#[cfg(test)]
mod tests {
    use super::{Compression, CompressionError};

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

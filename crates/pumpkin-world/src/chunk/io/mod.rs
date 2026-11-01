use std::{error, fs::File, path::Path, time::Duration};

use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;

use super::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

pub mod converter;
pub mod file_manager;

pub(crate) async fn run_blocking<T, F>(task: F) -> Result<T, tokio::task::JoinError>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(task).await
}

/// The result of loading a chunk data.
///
/// It can be the data loaded successfully, the data not found or an error
/// with the chunk coordinates and the error that occurred.
pub enum LoadedData<D: Send, Err: error::Error> {
    /// The chunk data was loaded successfully
    Loaded(D),

    /// The chunk data was not found
    Missing(Vector2<i32>),

    /// An error occurred while loading the chunk data
    Error((Vector2<i32>, Err)),
}

impl<D: Send, E: error::Error> LoadedData<D, E> {
    pub fn map_loaded<D2: Send>(self, map: impl FnOnce(D) -> D2) -> LoadedData<D2, E> {
        match self {
            Self::Loaded(data) => LoadedData::Loaded(map(data)),
            Self::Missing(pos) => LoadedData::Missing(pos),
            Self::Error(err) => LoadedData::Error(err),
        }
    }
}

pub trait Dirtiable {
    fn is_dirty(&self) -> bool;
    fn mark_dirty(&self, flag: bool);
}

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like `ChunkData` or `EntityData`
pub trait FileIO
where
    Self: Send + Sync,
{
    type Data: Send + Sync + Sized;

    /// Load the chunks data
    fn fetch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) -> impl Future<Output = ()> + Send + 'a;

    /// Persist the chunks data
    fn save_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> impl Future<Output = Result<(), ChunkWritingError>> + Send + 'a;

    /// Writes regions whose staged chunks have waited long enough and drops idle regions
    /// from the cache.
    fn flush_due(&self) -> impl Future<Output = ()> + Send + '_;

    /// Writes every staged chunk and syncs the region files.
    fn flush_all(&self) -> impl Future<Output = ()> + Send + '_;
}

/// One open region file (32x32 chunks) of a storage format.
///
/// Every method except `file_name` does blocking I/O or compression, so callers run them off the
/// async workers.
pub trait RegionFile: Send + Sync + Sized + 'static {
    type Config: Clone + Send + Sync + 'static;
    /// A chunk prepared for `stage_chunk`, built in parallel by `encode_chunk`.
    type Encoded: Send + 'static;
    /// A chunk as `read_chunk` found it, turned into NBT in parallel by `decode_chunk`.
    type Raw: Send + 'static;

    /// How long staged chunks may wait before the region is written out. Formats that rewrite
    /// the whole file on every write wait so several saves share one write.
    const FLUSH_DELAY: Duration;
    /// The most regions kept open per storage.
    const CACHE_CAPACITY: usize;

    fn file_name(region: Vector2<i32>) -> String;

    /// Opens the region at `path`. A missing file is an empty region; it is created on the
    /// first flush.
    fn open(path: &Path, region: Vector2<i32>) -> Result<Self, ChunkReadingError>;

    /// Reads the chunk at `index`, or returns `None` when it was never saved.
    fn read_chunk(&self, index: usize) -> Result<Option<Self::Raw>, ChunkReadingError>;

    /// Returns the uncompressed NBT of a chunk from `read_chunk`.
    fn decode_chunk(raw: Self::Raw) -> Result<Bytes, ChunkReadingError>;

    fn serialize_chunk<D: crate::chunk::format::anvil::SingleChunkDataSerializer>(
        chunk: &D,
    ) -> Result<Bytes, crate::chunk::ChunkSerializingError> {
        chunk.to_bytes()
    }

    fn encode_chunk(nbt: Bytes, config: &Self::Config) -> Result<Self::Encoded, ChunkWritingError>;

    /// Replaces the chunk at `index`. It is readable right away and reaches the disk on the
    /// next `flush`.
    fn stage_chunk(&mut self, index: usize, chunk: Self::Encoded);

    fn has_staged(&self) -> bool;

    /// Writes the staged chunks to disk, and syncs the file when `sync` is set.
    fn flush(&mut self, sync: bool) -> std::io::Result<()>;
}

/// Index of a chunk inside its 32x32 region.
#[must_use]
pub const fn region_chunk_index(chunk: Vector2<i32>) -> usize {
    ((chunk.y & 31) * 32 + (chunk.x & 31)) as usize
}

#[must_use]
pub const fn region_of(chunk: Vector2<i32>) -> Vector2<i32> {
    Vector2::new(chunk.x >> 5, chunk.y >> 5)
}

pub(crate) fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
    }
    #[cfg(windows)]
    {
        let mut done = 0;
        while done < buf.len() {
            let read = std::os::windows::fs::FileExt::seek_read(
                file,
                &mut buf[done..],
                offset + done as u64,
            )?;
            if read == 0 {
                return Err(std::io::ErrorKind::UnexpectedEof.into());
            }
            done += read;
        }
        Ok(())
    }
}

pub(crate) fn write_all_at(file: &File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::write_all_at(file, buf, offset)
    }
    #[cfg(windows)]
    {
        let mut done = 0;
        while done < buf.len() {
            let written = std::os::windows::fs::FileExt::seek_write(
                file,
                &buf[done..],
                offset + done as u64,
            )?;
            if written == 0 {
                return Err(std::io::ErrorKind::WriteZero.into());
            }
            done += written;
        }
        Ok(())
    }
}

/// Writes `bytes` next to `path` and renames it over `path`, so a crash leaves either the old or
/// the new file.
pub(crate) fn write_file_atomically(path: &Path, bytes: &[u8], sync: bool) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    std::io::Write::write_all(&mut file, bytes)?;
    if sync {
        file.sync_data()?;
    }
    drop(file);
    std::fs::rename(temp_path, path)
}

/// Syncs a file that an earlier `write_file_atomically` wrote without syncing.
pub(crate) fn sync_file(path: &Path) -> std::io::Result<()> {
    // Windows needs write access to flush a file's buffers.
    File::options().write(true).open(path)?.sync_data()
}

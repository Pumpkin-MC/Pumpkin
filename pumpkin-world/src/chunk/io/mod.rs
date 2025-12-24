use std::{error, pin::Pin};

use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;

use super::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

pub mod file_manager;

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
    fn mark_dirty(&mut self, flag: bool);
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like ChunkData or EntityData
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
    ) -> BoxFuture<'a, ()>; // Returns BoxFuture<()>

    /// Persist the chunks data
    fn save_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxFuture<'a, Result<(), ChunkWritingError>>; // Returns BoxFuture<Result>

    /// Tells the `ChunkIO` that these chunks are currently loaded in memory
    fn watch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks: &'a [Vector2<i32>],
    ) -> BoxFuture<'a, ()>;

    /// Tells the `ChunkIO` that these chunks are no longer loaded in memory
    fn unwatch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks: &'a [Vector2<i32>],
    ) -> BoxFuture<'a, ()>;

    /// Tells the `ChunkIO` that no more chunks are loaded in memory
    fn clear_watched_chunks(&self) -> BoxFuture<'_, ()>;

    fn clean_up_log(&self) -> BoxFuture<'_, ()>;

    /// Ensure that all ongoing operations are finished
    fn block_and_await_ongoing_tasks(&self) -> BoxFuture<'_, ()>;
}

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData
pub trait ChunkSerializer: Send + Sync + Default {
    type Data: Send + Sync + Sized + Dirtiable;
    type WriteBackend;

    type ChunkConfig;

    /// Get the key for the chunk (like the file name)
    fn get_chunk_key(chunk: &Vector2<i32>) -> String;

    fn should_write(&self, is_watched: bool) -> bool;

    /// Serialize the data to bytes.
    fn write(
        &self,
        backend: &Self::WriteBackend,
    ) -> impl Future<Output = Result<(), std::io::Error>> + Send;

    /// Create a new instance from bytes
    fn read(r: Bytes) -> Result<Self, ChunkReadingError>;

    /// Add the chunk data to the serializer
    fn update_chunk(
        &mut self,
        chunk_data: &Self::Data,
        chunk_config: &Self::ChunkConfig,
    ) -> impl Future<Output = Result<(), ChunkWritingError>> + Send;

    /// Get the chunks data from the serializer
    fn get_chunks(
        &self,
        chunks: Vec<Vector2<i32>>,
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) -> impl Future<Output = ()> + Send;
}

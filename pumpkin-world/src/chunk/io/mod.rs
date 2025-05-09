use std::error;

use async_trait::async_trait;
use pumpkin_util::math::vector2::Vector2;

use super::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

pub mod file_manager;

/// The result of loading a chunk data.
///
/// It can be the data loaded successfully, the data not found or an error
/// with the chunk coordinates and the error that occurred.
pub enum LoadedData<D, Err: error::Error>
where
    D: Send,
{
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

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like ChunkData or EntityData
#[async_trait]
pub trait FileIO
where
    Self: Send + Sync,
{
    type Data: Send + Sync + Sized;

    /// Load the chunks data
    async fn fetch_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    );

    /// Persist the chunks data
    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> Result<(), ChunkWritingError>;

    /// Tells the `ChunkIO` that these chunks are currently loaded in memory
    async fn watch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]);

    /// Tells the `ChunkIO` that these chunks are no longer loaded in memory
    async fn unwatch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]);

    /// Tells the `ChunkIO` that no more chunks are loaded in memory
    async fn clear_watched_chunks(&self);

    async fn clean_up_log(&self);

    /// Ensure that all ongoing operations are finished
    async fn block_and_await_ongoing_tasks(&self);
}

pub trait Dirtiable {
    fn is_dirty(&self) -> bool;
    fn mark_dirty(&mut self, flag: bool);
}

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData
#[async_trait]
pub trait ChunkSerializer: Send + Sync + Default {
    type Data: Send + Sync + Sized + Dirtiable;
    type WriteBackend;

    /// Get the key for the chunk (like the file name)
    fn get_chunk_key(chunk: &Vector2<i32>) -> String;

    fn should_write(&self, is_watched: bool) -> bool;

    /// Serialize the data to bytes.
    async fn write(&self, backend: Self::WriteBackend) -> Result<(), std::io::Error>;

    /// Create a new instance from the backend
    async fn read(data: Self::WriteBackend) -> Result<Self, ChunkReadingError>;

    /// Add the chunk data to the serializer
    async fn update_chunk(&mut self, chunk_data: &Self::Data) -> Result<(), ChunkWritingError>;

    /// Get the chunks data from the serializer
    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    );
}

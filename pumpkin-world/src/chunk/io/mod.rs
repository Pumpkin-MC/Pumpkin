use std::error;

use async_trait::async_trait;
use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;
use tokio::io::AsyncWrite;

use super::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

pub mod chunk_file_manager;

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

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like ChunkData or EntityData
#[async_trait]
pub trait ChunkIO<D>
where
    Self: Send + Sync,
    D: Send + Sized,
{
    /// Load the chunks data
    async fn fetch_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<D, ChunkReadingError>>,
    );

    /// Persist the chunks data
    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, D)>,
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

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData
#[async_trait]
pub trait ChunkSerializer: Send + Sync + Default {
    type Data: Send + Sync + Sized;

    /// Get the key for the chunk (like the file name)
    fn get_chunk_key(chunk: &Vector2<i32>) -> String;

    /// Serialize the data to bytes.
    async fn write(&self, w: &mut (impl AsyncWrite + Unpin + Send)) -> Result<(), std::io::Error>;

    /// Create a new instance from bytes
    fn read(r: Bytes) -> Result<Self, ChunkReadingError>;

    /// Add the chunks data to the serializer
    async fn update_chunks(&mut self, chunk_data: &[Self::Data]) -> Result<(), ChunkWritingError>;

    /// Get the chunks data from the serializer
    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    );
}

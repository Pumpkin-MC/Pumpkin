use std::{error, sync::Arc};

use async_trait::async_trait;
use pumpkin_util::math::vector2::Vector2;
use tokio::sync::{RwLock, mpsc};

use super::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

pub mod chunk_file_manager;

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
        channel: mpsc::Sender<LoadedData<D, ChunkReadingError>>,
    );

    /// Persist the chunks data
    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Arc<RwLock<D>>)>,
    ) -> Result<(), ChunkWritingError>;

    async fn clean_up_log(&self);

    async fn await_tasks(&self);
}

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData
pub trait ChunkSerializer: Send + Sync + Sized + Default {
    type Data: Send;

    fn get_chunk_key(chunk: Vector2<i32>) -> String;

    fn to_bytes(&self) -> Box<[u8]>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError>;

    fn add_chunks_data(&mut self, chunk_data: &[&Self::Data]) -> Result<(), ChunkWritingError>;

    fn get_chunks_data(
        &self,
        chunks: &[Vector2<i32>],
    ) -> Vec<LoadedData<Self::Data, ChunkReadingError>>;
}

//! Adapter bridging pumpkin-world's legacy [`FileIO`] chunk I/O into
//! pumpkin-storage's [`ChunkStorage<T>`] trait.
//!
//! The legacy trait takes a `&LevelFolder` on every call; the new one assumes
//! the storage knows its own folder. This adapter holds the `LevelFolder` so
//! call sites can work exclusively through `ChunkStorage<T>`.

use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_storage::StorageError;
use pumpkin_storage::chunk::{ChunkStorage, LoadedData};
use pumpkin_util::math::vector2::Vector2;
use tokio::sync::mpsc;

use crate::chunk::io::{FileIO, LoadedData as OldLoadedData};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use crate::level::LevelFolder;

fn read_to_storage(err: ChunkReadingError) -> StorageError {
    match err {
        ChunkReadingError::ChunkNotExist => StorageError::NotFound {
            message: "chunk not found".to_string(),
        },
        ChunkReadingError::IoError(kind) => StorageError::io(std::io::Error::from(kind)),
        other => StorageError::Deserialize(other.to_string()),
    }
}

fn write_to_storage(err: ChunkWritingError) -> StorageError {
    match err {
        ChunkWritingError::IoError(kind) => StorageError::io(std::io::Error::from(kind)),
        other => StorageError::Serialize(other.to_string()),
    }
}

/// Wraps an `Arc<dyn FileIO<Data = T>>` plus a `LevelFolder` and exposes it
/// as an `Arc<dyn ChunkStorage<T>>`.
pub struct FolderBoundFileIO<T: Send + Sync + 'static> {
    inner: Arc<dyn FileIO<Data = T>>,
    folder: LevelFolder,
}

impl<T: Send + Sync + 'static> FolderBoundFileIO<T> {
    pub fn new(inner: Arc<dyn FileIO<Data = T>>, folder: LevelFolder) -> Self {
        Self { inner, folder }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> ChunkStorage<T> for FolderBoundFileIO<T> {
    async fn fetch_chunks(&self, chunk_coords: &[Vector2<i32>], out: mpsc::Sender<LoadedData<T>>) {
        let (old_tx, mut old_rx) = mpsc::channel(chunk_coords.len().max(1));

        let fetch = self.inner.fetch_chunks(&self.folder, chunk_coords, old_tx);
        let forward = async {
            while let Some(msg) = old_rx.recv().await {
                let translated = match msg {
                    OldLoadedData::Loaded(v) => LoadedData::Loaded(v),
                    OldLoadedData::Missing(p) => LoadedData::Missing(p),
                    OldLoadedData::Error((pos, err)) => LoadedData::Error {
                        pos,
                        error: read_to_storage(err),
                    },
                };
                if out.send(translated).await.is_err() {
                    break;
                }
            }
        };
        tokio::join!(fetch, forward);
    }

    async fn save_chunks(&self, chunks: Vec<(Vector2<i32>, T)>) -> Result<(), StorageError> {
        self.inner
            .save_chunks(&self.folder, chunks)
            .await
            .map_err(write_to_storage)
    }

    async fn watch_chunks(&self, chunks: &[Vector2<i32>]) {
        self.inner.watch_chunks(&self.folder, chunks).await;
    }

    async fn unwatch_chunks(&self, chunks: &[Vector2<i32>]) {
        self.inner.unwatch_chunks(&self.folder, chunks).await;
    }

    async fn clear_watched_chunks(&self) {
        self.inner.clear_watched_chunks().await;
    }

    async fn block_and_await_ongoing_tasks(&self) {
        self.inner.block_and_await_ongoing_tasks().await;
    }
}

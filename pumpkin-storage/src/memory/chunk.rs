//! Format-agnostic, in-memory [`ChunkStorage`] implementation.
//!
//! Backed by a single `HashMap<ChunkPos, T>`. `save_chunks` inserts/overwrites;
//! `fetch_chunks` streams cloned values. Watch tracking is a no-op since
//! nothing is paged to disk.

use std::collections::HashMap;

use async_trait::async_trait;
use pumpkin_util::math::vector2::Vector2;
use tokio::sync::{RwLock, mpsc};

use crate::chunk::{ChunkStorage, LoadedData};
use crate::error::StorageError;

#[derive(Debug, Default)]
pub struct MemoryChunkStorage<T> {
    data: RwLock<HashMap<Vector2<i32>, T>>,
}

impl<T> MemoryChunkStorage<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> ChunkStorage<T> for MemoryChunkStorage<T> {
    async fn fetch_chunks(
        &self,
        chunk_coords: &[Vector2<i32>],
        stream: mpsc::Sender<LoadedData<T>>,
    ) {
        let guard = self.data.read().await;
        for coord in chunk_coords {
            let msg = match guard.get(coord).cloned() {
                Some(v) => LoadedData::Loaded(v),
                None => LoadedData::Missing(*coord),
            };
            if stream.send(msg).await.is_err() {
                // Receiver dropped; stop pushing.
                break;
            }
        }
    }

    async fn save_chunks(&self, chunks: Vec<(Vector2<i32>, T)>) -> Result<(), StorageError> {
        let mut guard = self.data.write().await;
        for (pos, data) in chunks {
            guard.insert(pos, data);
        }
        Ok(())
    }

    async fn watch_chunks(&self, _chunks: &[Vector2<i32>]) {}
    async fn unwatch_chunks(&self, _chunks: &[Vector2<i32>]) {}
    async fn clear_watched_chunks(&self) {}
    async fn block_and_await_ongoing_tasks(&self) {}
}

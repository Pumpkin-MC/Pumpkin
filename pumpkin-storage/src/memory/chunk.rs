//! Format-agnostic, in-memory [`ChunkStorage`] implementation.
//!
//! Backed by a single `HashMap<ChunkPos, T>`. `save_chunks` inserts/overwrites;
//! `fetch_chunks` streams cloned values. Watch tracking is a no-op since
//! nothing is paged to disk.

use std::collections::HashMap;

use pumpkin_util::math::vector2::Vector2;
use tokio::sync::{RwLock, mpsc};

use crate::BoxFuture;
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

impl<T: Clone + Send + Sync + 'static> ChunkStorage<T> for MemoryChunkStorage<T> {
    fn fetch_chunks<'a>(
        &'a self,
        chunk_coords: &'a [Vector2<i32>],
        stream: mpsc::Sender<LoadedData<T>>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let guard = self.data.read().await;
            for coord in chunk_coords {
                let msg = guard
                    .get(coord)
                    .cloned()
                    .map_or(LoadedData::Missing(*coord), LoadedData::Loaded);
                if stream.send(msg).await.is_err() {
                    // Receiver dropped; stop pushing.
                    break;
                }
            }
        })
    }

    fn save_chunks(
        &self,
        chunks: Vec<(Vector2<i32>, T)>,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.data.write().await;
            for (pos, data) in chunks {
                guard.insert(pos, data);
            }
            Ok(())
        })
    }

    fn watch_chunks<'a>(&'a self, _chunks: &'a [Vector2<i32>]) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn unwatch_chunks<'a>(&'a self, _chunks: &'a [Vector2<i32>]) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn clear_watched_chunks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {})
    }
    fn block_and_await_ongoing_tasks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async {})
    }
}

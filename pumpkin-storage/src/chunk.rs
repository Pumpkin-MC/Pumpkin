//! Chunk I/O abstraction.
//!
//! The trait is generic over `T` (the chunk payload — e.g. `Arc<ChunkData>`
//! or `Arc<ChunkEntityData>` in `pumpkin-world`) so the same trait can be
//! implemented for blocks-per-chunk and entity-per-chunk storage, and so
//! callers can plug in alternative payload types in tests or custom backends.

use async_trait::async_trait;
use pumpkin_util::math::vector2::Vector2;
use tokio::sync::mpsc;

use crate::error::StorageError;

/// Outcome of a single-chunk read.
#[derive(Debug)]
pub enum LoadedData<T> {
    Loaded(T),
    /// No entry exists yet. Callers generate or default the chunk.
    Missing(Vector2<i32>),
    Error {
        pos: Vector2<i32>,
        error: StorageError,
    },
}

/// Storage for per-chunk data (blocks, entities) keyed by chunk coordinates.
///
/// The payload type `T` is typically an `Arc<ChunkData>` /
/// `Arc<ChunkEntityData>` so callers can share chunk state cheaply. `T` is
/// left unconstrained by this crate — callers choose the shape they need.
#[async_trait]
pub trait ChunkStorage<T: Send + Sync + 'static>: Send + Sync {
    /// Reads the requested chunks, emitting one [`LoadedData`] per requested
    /// coord through `stream`. Order of delivery is implementation-defined.
    async fn fetch_chunks(
        &self,
        chunk_coords: &[Vector2<i32>],
        stream: mpsc::Sender<LoadedData<T>>,
    );

    /// Persists chunks. Batched so region-based backends can group writes.
    async fn save_chunks(
        &self,
        chunks: Vec<(Vector2<i32>, T)>,
    ) -> Result<(), StorageError>;

    /// Marks chunks as resident in memory — region-file backends use this to
    /// keep serializer caches alive while any chunk in a region is watched.
    async fn watch_chunks(&self, chunks: &[Vector2<i32>]);

    async fn unwatch_chunks(&self, chunks: &[Vector2<i32>]);

    async fn clear_watched_chunks(&self);

    /// Waits until any in-flight saves/fetches have completed.
    async fn block_and_await_ongoing_tasks(&self);
}

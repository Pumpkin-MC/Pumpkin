//! [`FileIO`] backends that serve a SlimeWorld file straight from memory.
//!
//! Unlike the region-based formats (Anvil / Linear / Pump), a `.slime` file
//! holds an entire world in one blob that is meant to be loaded wholesale into
//! RAM. These backends parse and convert the whole world once (at construction)
//! and then answer chunk/entity requests from the resulting in-memory maps,
//! which keeps them off the region-oriented hot path entirely.
//!
//! SlimeWorld worlds are currently loaded **read-only**: [`save_chunks`] is a
//! no-op, so runtime modifications are not written back to the `.slime` file
//! (this matches the common minigame / lobby use case). Use Anvil, Linear or
//! Pump for worlds that must persist changes.
//!
//! [`save_chunks`]: FileIO::save_chunks

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashMap;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::chunk::io::{FileIO, LoadedData};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use crate::level::{LevelFolder, SyncChunk, SyncEntityChunk};

use super::{SlimeWorld, chunk_to_chunk_data, chunk_to_entity_data, read_slime_world};

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Read and parse the first `*.slime` file found directly in `root`.
fn read_slime_file(root: &Path) -> Option<SlimeWorld> {
    let entry = std::fs::read_dir(root).ok()?.flatten().find(|entry| {
        entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("slime"))
    })?;

    let bytes = std::fs::read(entry.path()).ok()?;
    match read_slime_world(Bytes::from(bytes)) {
        Ok(world) => Some(world),
        Err(error) => {
            error!("Failed to read slime world at {:?}: {error}", entry.path());
            None
        }
    }
}

/// [`FileIO`] serving block data from an in-memory SlimeWorld.
pub(crate) struct SlimeChunkIo {
    chunks: FxHashMap<Vector2<i32>, SyncChunk>,
}

impl SlimeChunkIo {
    /// Load and convert every chunk in the world below `root`.
    ///
    /// `min_section_y` is the bottom-most section index of the target dimension
    /// (SlimeWorld stores sections bottom-up without an explicit Y).
    pub(crate) fn new(root: &Path, min_section_y: i32) -> Self {
        let mut chunks = FxHashMap::default();
        if let Some(world) = read_slime_file(root) {
            for chunk in &world.chunks {
                match chunk_to_chunk_data(chunk, min_section_y) {
                    Ok(data) => {
                        chunks.insert(Vector2::new(chunk.x, chunk.z), Arc::new(data));
                    }
                    Err(error) => {
                        error!(
                            "Failed to convert slime chunk {},{}: {error}",
                            chunk.x, chunk.z
                        );
                    }
                }
            }
        }
        Self { chunks }
    }
}

impl FileIO for SlimeChunkIo {
    type Data = SyncChunk;

    fn fetch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            for &coord in chunk_coords {
                let data = self.chunks.get(&coord).map_or_else(
                    || LoadedData::Missing(coord),
                    |chunk| LoadedData::Loaded(chunk.clone()),
                );
                if stream.send(data).await.is_err() {
                    break;
                }
            }
        })
    }

    fn save_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxedFuture<'a, Result<(), ChunkWritingError>> {
        // SlimeWorld worlds are loaded read-only; nothing is written back.
        Box::pin(async { Ok(()) })
    }

    fn watch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks: &'a [Vector2<i32>],
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async {})
    }

    fn unwatch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks: &'a [Vector2<i32>],
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async {})
    }

    fn clear_watched_chunks(&self) -> BoxedFuture<'_, ()> {
        Box::pin(async {})
    }

    fn block_and_await_ongoing_tasks(&self) -> BoxedFuture<'_, ()> {
        Box::pin(async {})
    }
}

/// [`FileIO`] serving entity data from an in-memory SlimeWorld.
pub(crate) struct SlimeEntityIo {
    entities: FxHashMap<Vector2<i32>, SyncEntityChunk>,
}

impl SlimeEntityIo {
    /// Load and convert every chunk's entities in the world below `root`.
    pub(crate) fn new(root: &Path) -> Self {
        let mut entities = FxHashMap::default();
        if let Some(world) = read_slime_file(root) {
            for chunk in &world.chunks {
                match chunk_to_entity_data(chunk) {
                    Ok(data) => {
                        entities.insert(Vector2::new(chunk.x, chunk.z), Arc::new(data));
                    }
                    Err(error) => {
                        error!(
                            "Failed to convert slime entities for chunk {},{}: {error}",
                            chunk.x, chunk.z
                        );
                    }
                }
            }
        }
        Self { entities }
    }
}

impl FileIO for SlimeEntityIo {
    type Data = SyncEntityChunk;

    fn fetch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async move {
            for &coord in chunk_coords {
                let data = self.entities.get(&coord).map_or_else(
                    || LoadedData::Missing(coord),
                    |chunk| LoadedData::Loaded(chunk.clone()),
                );
                if stream.send(data).await.is_err() {
                    break;
                }
            }
        })
    }

    fn save_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxedFuture<'a, Result<(), ChunkWritingError>> {
        Box::pin(async { Ok(()) })
    }

    fn watch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks: &'a [Vector2<i32>],
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async {})
    }

    fn unwatch_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        _chunks: &'a [Vector2<i32>],
    ) -> BoxedFuture<'a, ()> {
        Box::pin(async {})
    }

    fn clear_watched_chunks(&self) -> BoxedFuture<'_, ()> {
        Box::pin(async {})
    }

    fn block_and_await_ongoing_tasks(&self) -> BoxedFuture<'_, ()> {
        Box::pin(async {})
    }
}

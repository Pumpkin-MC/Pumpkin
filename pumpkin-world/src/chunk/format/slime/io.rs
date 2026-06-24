//! [`FileIO`] backends that serve a SlimeWorld from a shared in-memory store.
//!
//! Unlike the region-based formats (Anvil / Linear / Pump), a `.slime` file
//! holds an entire world in one blob that is loaded wholesale into RAM. Both
//! backends share a single [`SlimeWorldStore`]: reads are answered from the
//! in-memory maps, and a save from either side re-serializes the whole world
//! back to the one file.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use pumpkin_util::math::vector2::Vector2;
use tokio::sync::mpsc::Sender;

use crate::chunk::io::{FileIO, LoadedData};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use crate::level::{LevelFolder, SyncChunk, SyncEntityChunk};

use super::store::SlimeWorldStore;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// [`FileIO`] serving block data from a shared [`SlimeWorldStore`].
pub(crate) struct SlimeChunkIo {
    store: Arc<SlimeWorldStore>,
}

impl SlimeChunkIo {
    pub(crate) fn new(store: Arc<SlimeWorldStore>) -> Self {
        Self { store }
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
                let data = self
                    .store
                    .get_chunk(&coord)
                    .map_or_else(|| LoadedData::Missing(coord), LoadedData::Loaded);
                if stream.send(data).await.is_err() {
                    break;
                }
            }
        })
    }

    fn save_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxedFuture<'a, Result<(), ChunkWritingError>> {
        Box::pin(async move {
            self.store.store_chunks(chunks_data);
            self.store
                .persist()
                .await
                .map_err(ChunkWritingError::IoError)
        })
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

/// [`FileIO`] serving entity data from a shared [`SlimeWorldStore`].
pub(crate) struct SlimeEntityIo {
    store: Arc<SlimeWorldStore>,
}

impl SlimeEntityIo {
    pub(crate) fn new(store: Arc<SlimeWorldStore>) -> Self {
        Self { store }
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
                let data = self
                    .store
                    .get_entity(&coord)
                    .map_or_else(|| LoadedData::Missing(coord), LoadedData::Loaded);
                if stream.send(data).await.is_err() {
                    break;
                }
            }
        })
    }

    fn save_chunks<'a>(
        &'a self,
        _folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxedFuture<'a, Result<(), ChunkWritingError>> {
        Box::pin(async move {
            self.store.store_entities(chunks_data);
            self.store
                .persist()
                .await
                .map_err(ChunkWritingError::IoError)
        })
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

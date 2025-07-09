use std::{
    collections::BTreeMap,
    io::ErrorKind,
    ops::{AddAssign, SubAssign},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use futures::future::join_all;
use log::{error, trace};
use num_traits::Zero;
use pumpkin_util::math::vector2::Vector2;
use tokio::{
    io::AsyncReadExt,
    join,
    sync::{OnceCell, RwLock, mpsc},
};

use crate::{
    chunk::{ChunkReadingError, ChunkWritingError, io::Dirtiable},
    level::LevelFolder,
};

use super::{ChunkSerializer, FileIO, LoadedData};

/// A simple implementation of the ChunkSerializer trait
/// that load and save the data from a file in the disk
/// using parallelism and a cache for the files with ongoing IO operations.
///
/// It also avoid IO operations that could produce dataraces thanks to the
/// custom *DashMap*-like implementation.
pub struct ChunkFileManager<S: ChunkSerializer<WriteBackend = PathBuf>> {
    // Dashmap has rw-locks on shards, but we want per-serializer.
    //
    // Lock lookups need to be:
    // - Relatively quick insertion / lookup as to not extraneously block other concurrent hashmap
    // ops
    // - Guarantee that there is only one serializer per file at a time
    // - Lazily load files as to support point 1
    // - Allow for ease of usage (able to return the serializer from a function)
    file_locks: RwLock<BTreeMap<PathBuf, ChunkSerializerLazyLoader<S>>>,
    watchers: RwLock<BTreeMap<PathBuf, usize>>,
}

pub(crate) trait PathFromLevelFolder {
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf;
}

struct ChunkSerializerLazyLoader<S: ChunkSerializer<WriteBackend = PathBuf>> {
    path: PathBuf,
    internal: OnceCell<Arc<RwLock<S>>>,
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkSerializerLazyLoader<S> {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            internal: OnceCell::new(),
        }
    }

    /// We can only remove this entry from the map if we are the only ones with a reference to it
    /// IMPORTANT: This must be called within the write lock of the parent map
    async fn can_remove(&self) -> bool {
        match self.internal.get() {
            Some(arc) => {
                let _write_lock = arc.write().await;
                Arc::strong_count(arc) == 1
            }
            None => true,
        }
    }

    async fn get(&self) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        self.internal
            .get_or_try_init(|| async {
                let serializer = self.read_from_disk().await?;
                Ok(Arc::new(RwLock::new(serializer)))
            })
            .await
            .cloned()
    }

    async fn read_from_disk(&self) -> Result<S, ChunkReadingError> {
        trace!("Opening file from Disk: {:?}", self.path);
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .truncate(false)
            .open(&self.path)
            .await
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            });

        let value = match file {
            Ok(mut file) => {
                let capacity = match file.metadata().await {
                    Ok(metadata) => metadata.len() as usize,
                    Err(_) => 4096, // A sane default
                };

                // TODO: Memmap?
                let mut file_bytes = Vec::with_capacity(capacity);
                file.read_to_end(&mut file_bytes)
                    .await
                    .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
                S::read(file_bytes.into())?
            }
            Err(ChunkReadingError::ChunkNotExist) => S::default(),
            Err(err) => return Err(err),
        };

        trace!("Successfully read file from Disk: {:?}", self.path);
        Ok(value)
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            watchers: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    async fn get_serializer(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        // We get the entry from the DashMap and try to insert a new lock if it doesn't exist
        // using dead-lock safe methods like `or_try_insert_with`

        // We use a lazy loader here to quickly make an insertion into the map without holding the
        // lock for too long starving other threads
        if let Some(serializer_loader) = self.file_locks.read().await.get(path) {
            serializer_loader.get().await
        } else {
            self.file_locks
                .write()
                .await
                .entry(path.into())
                .or_insert_with(|| ChunkSerializerLazyLoader::new(path.into()))
                .get()
                .await
        }
    }
}

#[async_trait]
impl<P, S> FileIO for ChunkFileManager<S>
where
    P: PathFromLevelFolder + Send + Sync + Sized + Dirtiable + 'static,
    S: ChunkSerializer<Data = P, WriteBackend = PathBuf>,
{
    type Data = Arc<RwLock<S::Data>>;

    async fn watch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]) {
        // It is intentional that regions are watched multiple times (once per chunk)
        let mut watchers = self.watchers.write().await;
        for chunk in chunks {
            let key = S::get_chunk_key(chunk);
            let map_key = P::file_path(folder, &key);
            match watchers.entry(map_key) {
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    let _ = vacant.insert(1);
                }
                std::collections::btree_map::Entry::Occupied(mut occupied) => {
                    occupied.get_mut().add_assign(1);
                }
            }
        }
    }

    async fn unwatch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]) {
        let mut watchers = self.watchers.write().await;
        for chunk in chunks {
            let key = S::get_chunk_key(chunk);
            let map_key = P::file_path(folder, &key);
            match watchers.entry(map_key) {
                std::collections::btree_map::Entry::Vacant(_vacant) => {}
                std::collections::btree_map::Entry::Occupied(mut occupied) => {
                    occupied.get_mut().sub_assign(1);
                    if occupied.get().is_zero() {
                        occupied.remove_entry();
                    }
                }
            }
        }
    }

    async fn clear_watched_chunks(&self) {
        self.watchers.write().await.clear();
    }

    async fn fetch_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        let mut regions_chunks: BTreeMap<String, Vec<Vector2<i32>>> = BTreeMap::new();

        for at in chunk_coords {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(*at))
                .or_insert(vec![*at]);
        }

        // we use a Sync Closure with an Async Block to execute the tasks concurrently
        // Also improves File Cache utilizations.
        let region_read_tasks = regions_chunks.into_iter().map(async |(file_name, chunks)| {
            let path = P::file_path(folder, &file_name);
            let chunk_serializer = match self.get_serializer(&path).await {
                Ok(chunk_serializer) => chunk_serializer,
                Err(ChunkReadingError::ChunkNotExist) => {
                    unreachable!("Default Serializer must be created")
                }
                Err(err) => {
                    let _ = stream.send(LoadedData::Error((chunks[0], err))).await;
                    return;
                }
            };

            // Intermediate channel for wrapping the data with the Arc<RwLock>
            let (send, mut recv) = mpsc::channel::<LoadedData<S::Data, ChunkReadingError>>(1);

            let intermediary = async {
                while let Some(data) = recv.recv().await {
                    let wrapped_data = data.map_loaded(|data| Arc::new(RwLock::new(data)));
                    if stream.send(wrapped_data).await.is_err() {
                        // Stream is closed, so stop unneeded computation and io
                        return;
                    }
                }
            };

            // We need to block the read to avoid other threads to write/modify the data
            let serializer = chunk_serializer.read().await;
            let reader = serializer.get_chunks(&chunks, send);

            join!(intermediary, reader);
        });

        let _ = join_all(region_read_tasks).await;
    }

    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: BTreeMap<String, Vec<Self::Data>> = BTreeMap::new();

        for (at, chunk) in chunks_data {
            let key = S::get_chunk_key(&at);

            match regions_chunks.entry(key) {
                std::collections::btree_map::Entry::Occupied(mut occupied) => {
                    occupied.get_mut().push(chunk);
                }
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    vacant.insert(vec![chunk]);
                }
            }
        }

        // we use a Sync Closure with an Async Block to execute the tasks in parallel
        // with out waiting the future. Also it improve we File Cache utilizations.
        let tasks = regions_chunks
            .into_iter()
            .map(async |(file_name, chunk_locks)| {
                let path = P::file_path(folder, &file_name);
                log::trace!("Updating data for file {path:?}");

                let chunk_serializer = match self.get_serializer(&path).await {
                    Ok(file) => Ok(file),
                    Err(ChunkReadingError::ChunkNotExist) => {
                        unreachable!("Must be managed by the cache")
                    }
                    Err(ChunkReadingError::IoError(err)) => {
                        error!("Error reading the data before write: {err}");
                        Err(ChunkWritingError::IoError(err))
                    }
                    Err(_) => {
                        Err(ChunkWritingError::IoError(std::io::ErrorKind::Other))
                    }
                }?;

                // Create a task for each chunk_lock
                let update_tasks = chunk_locks.into_iter().map(|chunk_lock| {
                    let chunk_serializer = chunk_serializer.clone();
                    async move {
                        let mut chunk = chunk_lock.write().await;
                        let chunk_is_dirty = chunk.is_dirty();
                        // Edge case: this chunk is loaded while we were saving, mark it as cleaned since we are
                        // updating what we will write here
                        chunk.mark_dirty(false);
                        // It is important that we keep the lock after we mark the chunk as clean so no one else
                        // can modify it
                        let chunk = chunk.downgrade();

                        // We only need to update the chunk if it is dirty
                        if chunk_is_dirty {
                            chunk_serializer.write().await.update_chunk(&*chunk).await?;
                        }
                        Ok::<(), ChunkWritingError>(())
                    }
                });
                // Run all update tasks concurrently and propagate any error
                futures::future::try_join_all(update_tasks).await?;
                log::trace!("Updated data for file {path:?}");

                let is_watched = self
                    .watchers
                    .read()
                    .await
                    .get(&path)
                    .is_some_and(|count| !count.is_zero());

                if !is_watched {
                    // With the modification done, we can drop the write lock but keep the read lock
                    // to avoid other threads to write/modify the data, but allow other threads to read it
                    let serializer = chunk_serializer.read().await;

                    log::debug!("Writing file for {path:?}");
                    serializer
                        .write(path.clone())
                        .await
                        .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

                    // Remove lock
                    drop(serializer);
                    // Decrement strong count
                    drop(chunk_serializer);

                    // If there are still no watchers, drop from the locks
                    let mut locks = self.file_locks.write().await;

                    if self
                        .watchers
                        .read()
                        .await
                        .get(&path)
                        .is_none_or(|count| count.is_zero())
                    {
                        let can_remove = if let Some(loader) = locks.get(&path) {
                            loader.can_remove().await
                        } else {
                            true
                        };

                        if can_remove {
                            locks.remove(&path);
                            log::trace!("Removed lockfile cache {path:?}");
                        } else {
                            log::trace!("Wanted to remove lockfile cache {path:?} but someone still holds a reference to it!");
                        }
                    }
                }

                Ok(())
            });

        //TODO: we need to handle the errors and return the result
        // files to save
        let _test: Vec<Result<(), ChunkWritingError>> = join_all(tasks).await;

        Ok(())
    }

    async fn clean_up_log(&self) {
        let locks = self.file_locks.read().await;
        log::debug!("{} File locks remain in cache", locks.len());
    }

    async fn block_and_await_ongoing_tasks(&self) {
        //we need to block any other operation
        let serializer_cache = self.file_locks.write().await;

        // Acquire a write lock on all entries to verify they are complete
        let tasks = serializer_cache
            .values()
            .map(async |serializer| serializer.can_remove());

        // We need to wait to ensure that all the locks are acquired
        // so there is no **operation** ongoing
        let _ = join_all(tasks).await;
    }
}

use std::{
    collections::BTreeMap,
    io::ErrorKind,
    ops::{AddAssign, Deref, SubAssign},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use futures::future::join_all;
use log::{error, trace};
use num_traits::Zero;
use pumpkin_util::math::vector2::Vector2;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    sync::{OnceCell, RwLock},
};

use crate::{
    chunk::{ChunkReadingError, ChunkWritingError},
    level::LevelFolder,
};

use super::{ChunkIO, ChunkSerializer, LoadedData};

/// A simple implementation of the ChunkSerializer trait
/// that load and save the data from a file in the disk
/// using parallelism and a cache for the files with ongoing IO operations.
///
/// It also avoid IO operations that could produce dataraces thanks to the
/// custom *DashMap* like implementation.
pub struct ChunkFileManager<S: ChunkSerializer> {
    // Dashmap has rw-locks on shards, but we want per-serializer
    file_locks: RwLock<BTreeMap<PathBuf, SerializerCacheEntry<S>>>,
    watchers: RwLock<BTreeMap<PathBuf, usize>>,
}
//to avoid clippy warnings we extract the type alias
type SerializerCacheEntry<S> = OnceCell<Arc<RwLock<S>>>;

impl<S: ChunkSerializer> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            watchers: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<S: ChunkSerializer> ChunkFileManager<S> {
    fn map_key(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.region_folder.join(file_name)
    }

    pub async fn read_file(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        // We get the entry from the DashMap and try to insert a new lock if it doesn't exist
        // using dead-lock safe methods like `or_try_insert_with`

        async fn read_from_disk<S: ChunkSerializer>(
            path: &Path,
        ) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
            trace!("Opening file from Disk: {:?}", path);
            let file = tokio::fs::OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .truncate(false)
                .open(path)
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

                    let mut file_bytes = Vec::with_capacity(capacity);
                    file.read_to_end(&mut file_bytes)
                        .await
                        .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
                    S::read(file_bytes.into())?
                }
                Err(ChunkReadingError::ChunkNotExist) => S::default(),
                Err(err) => return Err(err),
            };

            trace!("Successfully read file from Disk: {:?}", path);
            Ok(Arc::new(RwLock::new(value)))
        }

        // We use a once lock here to quickly make an insertion into the map without holding the
        // lock for too long starving other threads
        let serializer = if let Some(once_cell) = self.file_locks.read().await.get(path) {
            log::trace!("Loading file lock from cache: {:?}", path);
            once_cell
                .get_or_try_init(|| read_from_disk(path))
                .await?
                .clone()
        } else {
            log::trace!("Cache miss loading file lock from cache: {:?}", path);
            let mut file_locks = self.file_locks.write().await;
            file_locks
                .entry(path.to_path_buf())
                .or_insert_with(OnceCell::new);
            let file_locks = file_locks.downgrade();
            let once_cell = file_locks.get(path).expect("We just inserted this!");
            once_cell
                .get_or_try_init(|| read_from_disk(path))
                .await?
                .clone()
        };

        Ok(serializer)
    }

    pub async fn write_file(path: &Path, serializer: &S) -> Result<(), ChunkWritingError> {
        // We use tmp files to avoid corruption of the data if the process is abruptly interrupted.
        let tmp_path = &path.with_extension("tmp");
        trace!("Writing tmp file to disk: {:?}", tmp_path);

        let file = tokio::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(tmp_path)
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
        let mut buf_writer = BufWriter::new(file);

        serializer
            .write(&mut buf_writer)
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        buf_writer
            .flush()
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        // The rename of the file works like an atomic operation ensuring
        // that the data is not corrupted before the rename is completed
        tokio::fs::rename(tmp_path, path)
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        trace!("Wrote file to Disk: {:?}", path);
        Ok(())
    }
}

#[async_trait]
impl<S, D> ChunkIO<D> for ChunkFileManager<S>
where
    D: 'static + Send + Sync + Sized,
    S: ChunkSerializer<Data = D>,
{
    async fn watch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]) {
        // It is intentional that regions are watched multiple times (once per chunk)
        let mut watchers = self.watchers.write().await;
        for chunk in chunks {
            let key = S::get_chunk_key(chunk);
            let map_key = Self::map_key(folder, &key);
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
            let map_key = Self::map_key(folder, &key);
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
        stream: tokio::sync::mpsc::Sender<LoadedData<D, ChunkReadingError>>,
    ) {
        let mut regions_chunks: BTreeMap<String, Vec<Vector2<i32>>> = BTreeMap::new();

        for at in chunk_coords {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(*at))
                .or_insert(vec![*at]);
        }

        // we use a Sync Closure with an Async Block to execute the tasks in parallel
        // with out waiting the future. Also it improve we File Cache utilizations.
        let tasks = regions_chunks.into_iter().map(async |(file_name, chunks)| {
            let path = Self::map_key(folder, &file_name);
            let chunk_serializer = match self.read_file(&path).await {
                Ok(chunk_serializer) => chunk_serializer,
                Err(ChunkReadingError::ChunkNotExist) => {
                    unreachable!("Default Serializer must be created")
                }
                Err(err) => {
                    if let Err(err) = stream.send(LoadedData::Error((chunks[0], err))).await {
                        log::warn!("Failed to send data to the chunk stream: {:?}", err);
                    };
                    return;
                }
            };

            // We need to block the read to avoid other threads to write/modify the data
            let serializer = chunk_serializer.read().await;
            serializer.get_chunks(&chunks, stream.clone()).await;
        });

        let _ = join_all(tasks).await;
    }

    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, D)>,
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: BTreeMap<String, Vec<D>> = BTreeMap::new();

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
                let path = Self::map_key(folder, &file_name);
                log::trace!("Updating data for file {:?}", path);

                let chunk_serializer = match self.read_file(&path).await {
                    Ok(file) => Ok(file),
                    Err(ChunkReadingError::ChunkNotExist) => {
                        unreachable!("Must be managed by the cache")
                    }
                    Err(ChunkReadingError::IoError(err)) => {
                        error!("Error reading the data before write: {}", err);
                        Err(ChunkWritingError::IoError(err))
                    }
                    Err(err) => {
                        error!("Error reading the data before write: {:?}", err);
                        Err(ChunkWritingError::IoError(std::io::ErrorKind::Other))
                    }
                }?;

                let mut serializer = chunk_serializer.write().await;
                serializer.update_chunks(&chunk_locks).await?;
                log::trace!("Updated data for file {:?}", path);

                // Only write the file if no chunks are being used
                if self
                    .watchers
                    .read()
                    .await
                    .get(&path)
                    .is_none_or(|count| count.is_zero())
                {
                    // With the modification done, we can drop the write lock but keep the read lock
                    // to avoid other threads to write/modify the data, but allow other threads to read it
                    let serializer = serializer.downgrade();
                    let serializer_ref = serializer.deref();
                    Self::write_file(&path, serializer_ref).await?;
                    log::trace!("Saved file {:?}", path);
                    drop(serializer);

                    // If there are still no watchers, drop from the locks
                    let mut locks = self.file_locks.write().await;

                    if self
                        .watchers
                        .read()
                        .await
                        .get(&path)
                        .is_none_or(|count| count.is_zero())
                    {
                        locks.remove(&path);
                        log::trace!("Removed lockfile cache {:?}", path);
                    }
                }

                Ok(())
            });

        //TODO: we need to handle the errors and return the result
        // files to save
        let _: Vec<Result<(), ChunkWritingError>> = join_all(tasks).await;

        Ok(())
    }

    async fn clean_up_log(&self) {
        let locks = self.file_locks.read().await;
        log::debug!("{} File locks remain in cache", locks.len());
    }

    async fn block_and_await_ongoing_tasks(&self) {
        //we need to block any other operation
        let serializer_cache = self.file_locks.write().await;

        let locks: Vec<_> = serializer_cache
            .iter()
            .map(|(pos, value)| (pos, value.clone()))
            .collect();

        // Acquire a write lock on all entries to verify they are complete
        let tasks = locks.iter().map(async |(pos, serializer)| {
            if let Some(lock) = serializer.get() {
                Some(lock.write().await)
            } else {
                log::warn!(
                    "Closing FileManager while the File {} is being loaded",
                    pos.display()
                );
                None
            }
        });

        // We need to wait to ensure that all the locks are acquired
        // so there is no **operation** ongoing
        let _ = join_all(tasks).await;
    }
}

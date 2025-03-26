use std::{
    collections::BTreeMap,
    io::ErrorKind,
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
    chunk::{ChunkData, ChunkReadingError, ChunkWritingError},
    level::{LevelFolder, SyncChunk},
};

use super::{ChunkIO, ChunkSerializer, LoadedData};

/// A simple implementation of the ChunkSerializer trait
/// that load and save the data from a file in the disk
/// using parallelism and a cache for the files with ongoing IO operations.
///
/// It also avoid IO operations that could produce dataraces thanks to the
/// custom *DashMap* like implementation.
pub struct ChunkFileManager<S: ChunkSerializer<WriteBackend = PathBuf>> {
    // Dashmap has rw-locks on shards, but we want per-serializer
    file_locks: RwLock<BTreeMap<PathBuf, SerializerCacheEntry<S>>>,
    watchers: RwLock<BTreeMap<PathBuf, usize>>,
}
//to avoid clippy warnings we extract the type alias
type SerializerCacheEntry<S> = OnceCell<Arc<RwLock<S>>>;

impl<S: ChunkSerializer<WriteBackend = PathBuf>> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            watchers: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    fn map_key(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.region_folder.join(file_name)
    }

    async fn read_file(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
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
}

#[async_trait]
impl<S> ChunkIO for ChunkFileManager<S>
where
    S: ChunkSerializer<Data = ChunkData, WriteBackend = PathBuf>,
{
    type Data = SyncChunk;

    async fn watch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]) {
        let mut watchers = self.watchers.write().await;
        for chunk in chunks {
            let key = S::get_chunk_key(chunk);
            let map_key = Self::map_key(folder, &key);
            let counter = watchers.entry(map_key).or_insert(0);
            *counter += 1; // Increment the counter directly
        }
    }

    async fn unwatch_chunks(&self, folder: &LevelFolder, chunks: &[Vector2<i32>]) {
        let mut watchers = self.watchers.write().await;
        for chunk in chunks {
            let key = S::get_chunk_key(chunk);
            let map_key = Self::map_key(folder, &key);
            if let Some(counter) = watchers.get_mut(&map_key) {
                if *counter > 1 {
                    *counter -= 1; // Decrement the counter if more than 1
                } else {
                    watchers.remove(&map_key); // Remove the entry if the counter reaches 0
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
        stream: tokio::sync::mpsc::Sender<LoadedData<SyncChunk, ChunkReadingError>>,
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

            // Intermediate channel for wrapping the data with the Arc<RwLock>
            let (send, mut recv) = mpsc::channel::<LoadedData<ChunkData, ChunkReadingError>>(1);

            let intermediary = async {
                while let Some(data) = recv.recv().await {
                    let wrapped_data = data.map_loaded(|data| Arc::new(RwLock::new(data)));
                    stream
                        .send(wrapped_data)
                        .await
                        .expect("Failed chunk wrapper intermediary");
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
        chunks_data: Vec<(Vector2<i32>, SyncChunk)>,
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: BTreeMap<String, Vec<SyncChunk>> = BTreeMap::new();
    
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
    
        // Execute the tasks in parallel while improving file cache utilization
        let tasks = regions_chunks.into_iter().map(async |(file_name, chunk_locks)| {
            let path = Self::map_key(folder, &file_name);
            log::trace!("Updating data for file {:?}", path);
    
            // Attempt to read the file and handle errors
            let chunk_serializer = match self.read_file(&path).await {
                Ok(file) => Ok(file),
                Err(ChunkReadingError::ChunkNotExist) => {
                    unreachable!("This should be managed by the cache")
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
            for chunk_lock in chunk_locks {
                let mut chunk = chunk_lock.write().await;
                let chunk_is_dirty = chunk.dirty;
                chunk.dirty = false; // Mark the chunk as cleaned
    
                // Only update if the chunk is dirty
                if chunk_is_dirty {
                    if let Err(e) = serializer.update_chunk(&*chunk).await {
                        error!("Failed to update chunk for {:?}: {:?}", path, e);
                        return Err(ChunkWritingError::IoError(std::io::ErrorKind::Other));
                    }
                }
            }
            log::trace!("Updated data for file {:?}", path);
    
            // Check if the file is being watched
            let is_watched = self
                .watchers
                .read()
                .await
                .get(&path)
                .is_some_and(|count| !count.is_zero());
    
            // Write the file if required
            if serializer.should_write(is_watched) {
                let serializer = serializer.downgrade();
    
                log::debug!("Writing file for {:?}", path);
                if let Err(err) = serializer.write(path.clone()).await {
                    error!("Error writing the file {:?}: {}", path, err);
                    return Err(ChunkWritingError::IoError(err.kind()));
                }
    
                // After writing, check if the file can be removed from cache if it's no longer watched
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
    
        // Collect and handle errors from all tasks
        let results: Vec<Result<(), ChunkWritingError>> = join_all(tasks).await;
    
        // Check if any task resulted in an error
        if results.iter().any(|result| result.is_err()) {
            return Err(ChunkWritingError::IoError(std::io::ErrorKind::Other));
        }
    
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

use std::{
    collections::BTreeMap,
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use log::{error, trace};
use pumpkin_util::math::vector2::Vector2;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{OnceCell, RwLock, mpsc},
    task::JoinSet,
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
    file_locks: RwLock<BTreeMap<PathBuf, Arc<SerializerCacheEntry<S>>>>,
}
//to avoid clippy warnings we extract the type alias
type SerializerCacheEntry<S> = OnceCell<Arc<RwLock<S>>>;

impl<S: ChunkSerializer> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<S: ChunkSerializer> ChunkFileManager<S> {
    async fn clean_cache(&self) {
        log::trace!("Cleaning cache");

        let paths_to_remove = self
            .file_locks
            .read()
            .await
            .iter()
            .filter_map(|(path, lock)| {
                if let Some(lock) = lock.get() {
                    if Arc::strong_count(lock) <= 1 {
                        return Some(path.clone());
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        if paths_to_remove.is_empty() {
            return;
        }

        let mut locks = self.file_locks.write().await;
        for path in paths_to_remove {
            if let Some(lock) = locks.get(&path) {
                if let Some(lock) = lock.get() {
                    // If we have only two strong references, it means that the lock is only being used by the cache, so we can remove it from the cache to avoid memory leaks.
                    if Arc::strong_count(lock) <= 1 {
                        locks.remove(&path);
                        log::trace!("Removed lock for file: {:?}", path);
                    }
                }
            }
        }
        log::trace!("Cleaned cache");
    }

    pub async fn read_file(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        // We get the entry from the DashMap and try to insert a new lock if it doesn't exist
        // using dead-lock safe methods like `or_try_insert_with`

        async fn read_from_disk<S: ChunkSerializer>(
            path: &Path,
        ) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
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
                    let mut file_bytes = Vec::new();
                    file.read_to_end(&mut file_bytes)
                        .await
                        .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

                    //We need to make this operation blocking to avoid context switching
                    // and improve compute heavy operations performance (like decompression)
                    tokio::task::block_in_place(|| S::from_bytes(&file_bytes))?
                }
                Err(ChunkReadingError::ChunkNotExist) => S::default(),
                Err(err) => return Err(err),
            };

            Ok(Arc::new(RwLock::new(value)))
        }

        // We use a once lock here to quickly make an insertion into the map without holding the
        // lock for too long starving other threads

        let once_cell = if let Some(once_cell) = self.file_locks.read().await.get(path) {
            once_cell.clone()
        } else {
            let mut map_guard = self.file_locks.write().await;
            if !map_guard.contains_key(path) {
                map_guard.insert(path.to_path_buf(), Arc::new(OnceCell::new()));
            }
            map_guard
                .get(path)
                .expect("We just inserted this within a lock")
                .clone()
        };

        let serializer = once_cell
            .get_or_try_init(|| read_from_disk(path))
            .await?
            .clone();

        Ok(serializer)
    }

    pub async fn write_file(path: &Path, serializer: &S) -> Result<(), ChunkWritingError> {
        trace!("Writing file to Disk: {:?}", path);

        // We use tmp files to avoid corruption of the data if the process is abruptly interrupted.
        let tmp_path = &path.with_extension("tmp");

        let mut file = tokio::fs::OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(tmp_path)
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        //We need to make this operation blocking to avoid context switching
        // and improve compute heavy operations performance (like compression)
        file.write_all(&tokio::task::block_in_place(|| serializer.to_bytes()))
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        file.flush()
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
    async fn fetch_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
        channel: mpsc::Sender<LoadedData<D, ChunkReadingError>>,
    ) {
        let mut regions_chunks: BTreeMap<String, Vec<Vector2<i32>>> = BTreeMap::new();

        for &at in chunk_coords {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(at))
                .or_insert(vec![at]);
        }

        // we use a Sync Closure with an Async Block to execute the tasks in parallel
        // with out waiting the future. Also it improve we File Cache utilizations.
        let tasks = regions_chunks
            .into_iter()
            .map(async |(file_name, chunks)| {
                let path = folder.region_folder.join(file_name);
                let chunk_serializer = match self.read_file(&path).await {
                    Ok(chunk_serializer) => chunk_serializer,
                    Err(ChunkReadingError::ChunkNotExist) => {
                        unreachable!("Default Serializer must be created")
                    }
                    Err(err) => {
                        channel
                            .send(LoadedData::<D, ChunkReadingError>::Error((chunks[0], err)))
                            .await
                            .expect("Failed to send error from stream_chunks!");

                        return;
                    }
                };

                let mut stream_tasks = JoinSet::new();
                // We need to block the read to avoid other threads to write/modify the data
                let chunk_guard = chunk_serializer.read().await;

                //We need to make this operation blocking to avoid context switching
                // and improve compute heavy operations performance
                for chunk in tokio::task::block_in_place(|| chunk_guard.get_chunks(&chunks)) {
                    let channel = channel.clone();
                    stream_tasks.spawn(async move {
                        channel
                            .send(chunk)
                            .await
                            .expect("Failed to send chunk from stream_chunks!");
                    });
                }
                let _ = stream_tasks.join_all().await;
            })
            .collect::<Vec<_>>();

        for task in tasks {
            task.await;
        }

        self.clean_cache().await;
    }

    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Arc<RwLock<D>>)>,
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: BTreeMap<String, Vec<Arc<RwLock<D>>>> = BTreeMap::new();

        for (at, chunk) in chunks_data {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(chunk.clone()))
                .or_insert(vec![chunk.clone()]);
        }

        // we use a Sync Closure with an Async Block to execute the tasks in parallel
        // with out waiting the future. Also it improve we File Cache utilizations.
        let tasks = regions_chunks
            .into_iter()
            .map(async |(file_name, chunk_locks)| {
                let path = folder.region_folder.join(file_name);
                log::trace!("Saving file {}", path.display());

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

                let chunk_tasks = chunk_locks
                    .iter()
                    .map(async |c| c.read().await)
                    .collect::<Vec<_>>();

                let mut chunks = Vec::with_capacity(chunk_tasks.len());
                for chunk in chunk_tasks {
                    chunks.push(chunk.await);
                }
                let mut chunk_guard = chunk_serializer.write().await;

                //We need to make this operation blocking to avoid context switching
                // and improve compute heavy operations performance
                tokio::task::block_in_place(|| {
                    chunk_guard.update_chunks(&chunks.iter().map(|c| c.deref()).collect::<Vec<_>>())
                })?;

                // With the modification done, we can drop the write lock but keep the read lock
                // to avoid other threads to write/modify the data, but allow other threads to read it
                let chunk_guard = chunk_guard.downgrade();
                Self::write_file(&path, &chunk_guard).await?;

                Ok(())
            })
            .collect::<Vec<_>>();

        // files to save
        for task in tasks {
            task.await?;
        }

        self.clean_cache().await;
        Ok(())
    }

    async fn clean_up_log(&self) {
        let locks = self.file_locks.read().await;
        log::debug!("{} File locks remain in cache", locks.len());
    }

    async fn close(&self) {
        let locks: Vec<_> = self
            .file_locks
            .read()
            .await
            .iter()
            .map(|(_, value)| value.clone())
            .collect();

        // Acquire a write lock on all entries to verify they are complete
        for lock in locks {
            let _lock = lock
                .get()
                .expect("We initialize the once cells immediately")
                .write()
                .await;
        }
    }
}

use std::{
    collections::BTreeMap,
    error,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use log::{error, trace};
use pumpkin_util::math::vector2::Vector2;
use tokio::{
    fs::OpenOptions,
    io::AsyncReadExt,
    sync::{Mutex, OnceCell, mpsc},
    task::JoinSet,
};
use tokio::{io::AsyncWriteExt, sync::RwLock};

use crate::{
    chunk::{ChunkReadingError, ChunkWritingError},
    level::LevelFolder,
};

pub enum LoadedData<D, Err: error::Error>
where
    D: Send,
{
    /// The chunk data was loaded successfully
    Loaded(D),
    /// The chunk data was not found
    Missing(Vector2<i32>),

    /// An error occurred while loading the chunk data
    Error((Vector2<i32>, Err)),
}

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like ChunkData or EntityData
#[async_trait]
pub trait ChunkIO<D>
where
    Self: Send + Sync,
    D: Send + Sized,
{
    /// Load the chunks data
    async fn stream_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
        channel: mpsc::Sender<LoadedData<D, ChunkReadingError>>,
    );

    /// Persist the chunks data
    async fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Arc<RwLock<D>>)>,
    ) -> Result<(), ChunkWritingError>;

    async fn clean_up_log(&self);

    async fn await_tasks(&self);
}

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData

#[async_trait]
pub trait ChunkSerializer: Send + Sync + Sized + Default {
    type Data: Send;

    fn get_chunk_key(chunk: Vector2<i32>) -> String;

    fn to_bytes(&self) -> Box<[u8]>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError>;

    fn add_chunk_data(&mut self, chunk_data: &[Self::Data]) -> Result<(), ChunkWritingError>;

    async fn stream_chunk_data(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    );
}

/// A simple implementation of the ChunkSerializer trait
/// that load and save the data from a file in the disk
/// using parallelism and a cache for the files with ongoing IO operations.
///
/// It also avoid IO operations that could produce dataraces thanks to the
/// DashMap that manages the locks for the files.
pub struct ChunkFileManager<S: ChunkSerializer> {
    // Dashmap has rw-locks on shards, but we want per-serializer
    file_locks: RwLock<BTreeMap<PathBuf, SerializerCacheEntry<S>>>,
    paths_to_check: Mutex<Vec<PathBuf>>,
}
//to avoid clippy warnings we extract the type alias
type SerializerCacheEntry<S> = OnceCell<Arc<RwLock<S>>>;

impl<S: ChunkSerializer> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            paths_to_check: Mutex::new(Vec::new()),
        }
    }
}

impl<S: ChunkSerializer> ChunkFileManager<S> {
    async fn clean_cache(&self) {
        log::trace!("Cleaning cache");
        let mut locks = self.file_locks.write().await;
        let mut paths = self.paths_to_check.lock().await;
        for path in paths.iter() {
            if let Some(lock) = locks.get(path) {
                let lock = lock.get().expect("Serializer should be populated");
                // If we have only two strong references, it means that the lock is only being used by the cache, so we can remove it from the cache to avoid memory leaks.
                if Arc::strong_count(lock) <= 1 {
                    locks.remove(path);
                    trace!("Removed lock for file: {:?}", path);
                }
            }
        }
        paths.clear();
        log::trace!("Cleaned cache");
    }

    pub async fn read_file(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        // We get the entry from the DashMap and try to insert a new lock if it doesn't exist
        // using dead-lock safe methods like `or_try_insert_with`

        async fn read_from_disk<S: ChunkSerializer>(path: &Path) -> Result<S, ChunkReadingError> {
            let file = OpenOptions::new()
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

                    // This should be performant enough that we don't need to block
                    S::from_bytes(&file_bytes)?
                }
                Err(ChunkReadingError::ChunkNotExist) => S::default(),
                Err(err) => return Err(err),
            };

            Ok(value)
        }

        // We use a once lock here to quickly make an insertion into the map without holding the
        // lock for too long starving other threads

        let map_read_guard = self.file_locks.read().await;
        let lock = match map_read_guard.get(path) {
            Some(once_cell) => once_cell
                .get_or_try_init(|| async {
                    let serializer = read_from_disk(path).await?;
                    Ok(Arc::new(RwLock::new(serializer)))
                })
                .await?
                .clone(),
            None => {
                drop(map_read_guard);
                let mut map_write_guard = self.file_locks.write().await;
                if !map_write_guard.contains_key(path) {
                    map_write_guard.insert(path.to_path_buf(), OnceCell::new());
                }
                let map_read_guard = map_write_guard.downgrade();
                let once_cell = map_read_guard
                    .get(path)
                    .expect("We just inserted this within a lock");

                // Had to duplicate this because of lifetime stuff, is there a way to consolidate?
                once_cell
                    .get_or_try_init(|| async {
                        let serializer = read_from_disk(path).await?;
                        Ok(Arc::new(RwLock::new(serializer)))
                    })
                    .await?
                    .clone()
            }
        };

        let mut paths_to_check = self.paths_to_check.lock().await;
        paths_to_check.push(path.to_path_buf());
        Ok(lock)
    }

    pub async fn write_file(path: &Path, serializer: &S) -> Result<(), ChunkWritingError> {
        trace!("Writing file to Disk: {:?}", path);

        // We use tmp files to avoid corruption of the data if the process is abruptly interrupted.
        let tmp_path = &path.with_extension("tmp");

        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(tmp_path)
            .await
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        file.write_all(&serializer.to_bytes())
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
impl<D, S> ChunkIO<D> for ChunkFileManager<S>
where
    D: 'static + Send + Sized + Sync + Clone,
    S: 'static + ChunkSerializer<Data = D>,
{
    async fn stream_chunks(
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

        let mut stream_tasks = JoinSet::new();
        for (file_name, chunks) in regions_chunks {
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

            let channel = channel.clone();
            stream_tasks.spawn(async move {
                log::trace!("Starting stream chunks for {:?}", path);
                // We need to block the read to avoid other threads to write/modify the data
                let chunk_guard = chunk_serializer.read().await;
                chunk_guard.stream_chunk_data(&chunks, channel).await;
                log::trace!("Completed stream chunks for {:?}", path);
            });
        }

        let _ = stream_tasks.join_all().await;
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

        let mut write_tasks: JoinSet<Result<(), ChunkWritingError>> = JoinSet::new();

        for (file_name, chunk_locks) in regions_chunks.into_iter() {
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

            let chunks = {
                let mut chunks = Vec::with_capacity(chunk_locks.len());
                for chunk in chunk_locks {
                    let chunk = match Arc::try_unwrap(chunk) {
                        Ok(lock) => lock.into_inner(),
                        Err(arc) => {
                            // This is the less common case by far
                            let chunk = arc.read().await;
                            chunk.clone()
                        }
                    };
                    chunks.push(chunk);
                }
                chunks
            };

            write_tasks.spawn(async move {
                log::trace!("Starting save chunks for {:?}", path);
                let mut chunk_guard = chunk_serializer.write().await;
                chunk_guard.add_chunk_data(&chunks)?;

                // With the modification done, we can drop the write lock but keep the read lock
                // to avoid other threads to write/modify the data, but allow other threads to read it
                let chunk_guard = &chunk_guard.downgrade();
                Self::write_file(&path, chunk_guard).await?;

                log::trace!("Completed save chunks for {:?}", path);
                Ok(())
            });
        }

        // TODO: How do we want to handle errors? Currently this stops all of the rest of the
        // files to save
        let _ = write_tasks.join_all().await;

        self.clean_cache().await;
        Ok(())
    }

    async fn clean_up_log(&self) {
        let locks = self.file_locks.read().await;
        log::debug!("{} File locks remain in cache", locks.len());
    }

    async fn await_tasks(&self) {
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

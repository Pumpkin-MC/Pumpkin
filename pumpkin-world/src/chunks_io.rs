use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use log::{error, trace};
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use tokio::sync::RwLock;

use crate::{
    chunk::{ChunkReadingError, ChunkWritingError},
    level::LevelFolder,
};

pub enum LoadedData<R>
where
    R: Send,
{
    /// The chunk data was loaded successfully
    Loaded(R),
    /// The chunk data was not found
    Missing(Vector2<i32>),
}

/// Trait to handle the IO of chunks
/// for loading and saving chunks data
/// can be implemented for different types of IO
/// or with different optimizations
///
/// The `R` type is the type of the data that will be loaded/saved
/// like ChunkData or EntityData
pub trait ChunkIO<R>
where
    Self: Send + Sync,
    R: Send + Sized,
{
    /// Load the chunks data
    fn load_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
    ) -> Result<Vec<LoadedData<R>>, ChunkReadingError>;

    /// Persist the chunks data
    fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: &[(Vector2<i32>, &R)],
    ) -> Result<(), ChunkWritingError>;
}

/// Trait to serialize and deserialize the chunk data to and from bytes.
///
/// The `Data` type is the type of the data that will be updated or serialized/deserialized
/// like ChunkData or EntityData
pub trait ChunkSerializer: Send + Sync + Sized + Default {
    type Data: Send;

    fn get_chunk_key(chunk: Vector2<i32>) -> String;

    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, ChunkReadingError>;

    fn add_chunk_data(&mut self, chunk_data: &Self::Data) -> Result<(), ChunkWritingError>;
    fn get_chunk_data(
        &self,
        chunk: Vector2<i32>,
    ) -> Result<LoadedData<Self::Data>, ChunkReadingError>;
}

/// A simple implementation of the ChunkSerializer trait
/// that load and save the data from a file in the disk
/// using parallelism and a cache for the files with ongoing IO operations.
///
/// It also avoid IO operations that could produce dataraces thanks to the
/// DashMap that manages the locks for the files.
pub struct ChunkFileManager<S: ChunkSerializer> {
    file_locks: Arc<DashMap<PathBuf, Arc<RwLock<S>>>>,
    _serializer: std::marker::PhantomData<S>,
}

impl<S: ChunkSerializer> Default for ChunkFileManager<S> {
    fn default() -> Self {
        Self {
            file_locks: Arc::default(),
            _serializer: std::marker::PhantomData,
        }
    }
}

impl<S: ChunkSerializer> ChunkFileManager<S> {
    pub fn read_file(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        // We get the entry from the DashMap and try to insert a new lock if it doesn't exist
        // using dead-lock save methods like `or_try_insert_with`
        let serializer = self
            .file_locks
            .entry(path.to_path_buf())
            .or_try_insert_with(|| {
                let file = OpenOptions::new()
                    .write(false)
                    .read(true)
                    .create(false)
                    .truncate(false)
                    .open(path)
                    .map_err(|err| match err.kind() {
                        ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                        kind => ChunkReadingError::IoError(kind),
                    });

                match file {
                    Ok(file) => {
                        let file_bytes = file
                            .bytes()
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

                        Ok(Arc::new(RwLock::new(S::from_bytes(&file_bytes)?)))
                    }
                    Err(ChunkReadingError::ChunkNotExist) => {
                        Ok(Arc::new(RwLock::new(S::default())))
                    }
                    Err(err) => Err(err),
                }
            })?
            .downgrade()
            .clone();

        Ok(serializer)
    }

    pub fn write_file(&self, path: &Path, serializer: &S) -> Result<(), ChunkWritingError> {
        // We need to lock the dashmap entry to avoid removing the lock while writing
        // and also to avoid other threads to write/read the file at the same time
        let _guard = self.file_locks.entry(path.to_path_buf());
        let mut file = OpenOptions::new()
            .write(true)
            .read(false)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        file.write_all(serializer.to_bytes().as_slice())
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        file.flush()
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        Ok(())
    }

    fn clean_cache(&self) {
        // We need to collect the paths to check, because we can't remove from the DashMap while iterating
        let to_check = self
            .file_locks
            .iter()
            .filter_map(|entry| {
                let path = entry.key();
                let lock = entry.value();

                if Arc::strong_count(lock) <= 1 {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Remove the locks that are not being used,
        // there will be always at least one strong reference, the one in the DashMap
        to_check.par_iter().for_each(|path| {
            self.file_locks
                .remove_if(path, |_, lock| Arc::strong_count(lock) <= 1);
        });

        trace!(
            "ChunkFile Cache cleaning-cycle, remaining: {}",
            self.file_locks.len()
        );
    }
}

impl<R, S> ChunkIO<R> for ChunkFileManager<S>
where
    R: Send + Sized,
    for<'a> &'a R: Send,
    S: ChunkSerializer<Data = R>,
{
    fn load_chunks(
        &self,
        folder: &LevelFolder,
        chunk_coords: &[Vector2<i32>],
    ) -> Result<Vec<LoadedData<R>>, ChunkReadingError> {
        let mut regions_chunks: BTreeMap<String, Vec<Vector2<i32>>> = BTreeMap::new();

        for &at in chunk_coords {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(at))
                .or_insert(vec![at]);
        }

        let chunks_by_region: Vec<Result<Vec<LoadedData<R>>, _>> = regions_chunks
            .into_par_iter()
            .map(|(file_name, chunks)| {
                let path = folder.region_folder.join(file_name);

                let chunks_data = tokio::task::block_in_place(|| {
                    let chunk_serializer = match self.read_file(&path) {
                        Ok(chunk_serializer) => Ok(chunk_serializer),
                        Err(ChunkReadingError::ChunkNotExist) => {
                            unreachable!("Must be managed by the cache")
                        }

                        Err(err) => Err(err),
                    }?;

                    // We need to block the read to avoid other threads to write/modify the data
                    let chunk_guard = chunk_serializer.blocking_read();
                    let mut chunks_data = Vec::with_capacity(chunks.len());

                    chunks.into_iter().try_for_each(|at| {
                        chunks_data.push(chunk_guard.get_chunk_data(at)?);
                        Ok(())
                    })?;

                    Ok(chunks_data)
                })?;

                Ok(chunks_data)
            })
            .collect();

        let mut final_chunks: Vec<_> = Vec::with_capacity(chunk_coords.len());
        for chunks in chunks_by_region {
            final_chunks.extend(chunks?)
        }

        self.clean_cache();

        Ok(final_chunks)
    }

    fn save_chunks(
        &self,
        folder: &LevelFolder,
        chunks_data: &[(Vector2<i32>, &R)],
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: BTreeMap<String, Vec<&R>> = BTreeMap::new();

        for &(at, chunk) in chunks_data {
            let key = S::get_chunk_key(at);

            regions_chunks
                .entry(key)
                .and_modify(|chunks| chunks.push(chunk))
                .or_insert(vec![chunk]);
        }

        regions_chunks
            .into_par_iter()
            .try_for_each(|(file_name, chunks)| {
                let path = folder.region_folder.join(file_name);

                tokio::task::block_in_place(|| {
                    let chunk_serializer = match self.read_file(&path) {
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

                    // We need to block the read to avoid other threads to write/modify/read the data
                    let mut chunk_guard = chunk_serializer.blocking_write();
                    chunks
                        .iter()
                        .try_for_each(|&chunk| chunk_guard.add_chunk_data(chunk))?;

                    // With the modification done, we can drop the write lock but keep the read lock
                    // to avoid other threads to write/modify the data, but allow other threads to read it
                    let chunk_guard = chunk_guard.downgrade();
                    self.write_file(&path, &chunk_guard)
                })?;

                Ok(())
            })?;

        self.clean_cache();

        Ok(())
    }
}

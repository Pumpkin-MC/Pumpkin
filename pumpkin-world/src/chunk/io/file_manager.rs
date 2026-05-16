use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::future::join_all;
use pumpkin_util::math::vector2::Vector2;
use tokio::{
    join,
    sync::{OnceCell, RwLock, mpsc},
};
use tracing::{debug, error, trace};

use crate::{
    chunk::{
        ChunkReadingError, ChunkWritingError,
        io::{BoxFuture, Dirtiable},
    },
    level::LevelFolder,
};

use super::{ChunkSerializer, FileIO, LoadedData};

/// A simple implementation of the `ChunkSerializer` trait that loads and saves data
/// to disk using parallelism and a lazy-loading cache keyed by file path.
///
/// ### Concurrency model
///
/// * `file_locks` — one `Arc<RwLock<S>>` per on-disk file, created lazily.
///   All readers/writers for the same region file share this lock, so there
///   are never two concurrent writers for the same file.
/// * `watchers` — a ref-count per path.  While a path has active watchers the
///   serializer is **not** evicted from the cache and the file is **not**
///   flushed to disk (the caller owns the flush lifecycle).
///
/// ### Lock ordering (must never be violated to avoid deadlocks)
///
/// 1. `file_locks`  (outer)
/// 2. individual `RwLock<S>` inside each loader  (inner)
/// 3. `watchers`  (independent — never held at the same time as either above)
///
/// `watchers` is always acquired in its own critical section, after all
/// serializer locks are released, which keeps it strictly independent.
pub struct ChunkFileManager<S: ChunkSerializer<WriteBackend = PathBuf>> {
    file_locks: RwLock<BTreeMap<PathBuf, Arc<ChunkSerializerLazyLoader<S>>>>,
    watchers: RwLock<BTreeMap<PathBuf, usize>>,
    chunk_config: S::ChunkConfig,
}

pub(crate) trait PathFromLevelFolder {
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf;
}

struct ChunkSerializerLazyLoader<S: ChunkSerializer<WriteBackend = PathBuf>> {
    path: PathBuf,
    /// Initialised at most once; subsequent calls reuse the same Arc.
    internal: OnceCell<Arc<RwLock<S>>>,
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkSerializerLazyLoader<S> {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            internal: OnceCell::new(),
        }
    }

    /// Returns `true` only when no outside caller still holds a clone of this
    /// loader *or* the inner serializer.
    ///
    /// # Safety requirement
    /// **Must be called while the write-lock on the parent `file_locks` map is
    /// held.**  That guarantees no new `Arc` clones can be issued while we
    /// inspect the strong counts.
    fn can_remove(loader: &Arc<Self>) -> bool {
        // The map itself holds 1 strong count; anything above that means an
        // active caller still has a handle.
        if Arc::strong_count(loader) > 1 {
            return false;
        }
        match loader.internal.get() {
            // The loader holds 1 strong count; anything above that means
            // a read/write guard is still live somewhere.
            Some(arc) => Arc::strong_count(arc) == 1,
            None => true,
        }
    }

    /// Returns the serializer, initialising it from disk on the first call.
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
        trace!("Opening file from disk: {}", self.path.display());

        match tokio::fs::read(&self.path).await {
            Ok(bytes) => {
                let value = S::read(bytes.into())?;
                trace!("Successfully read file from disk: {}", self.path.display());
                Ok(value)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                trace!("File not found, using default for: {}", self.path.display());
                Ok(S::default())
            }
            Err(err) => Err(ChunkReadingError::IoError(err)),
        }
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    pub fn new(chunk_config: S::ChunkConfig) -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            watchers: RwLock::new(BTreeMap::new()),
            chunk_config,
        }
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    /// Returns the serializer for `path`, inserting a lazy-loader if absent.
    ///
    /// Uses an optimistic read-first pattern: in the common case (cache hit)
    /// we never need a write-lock on the map.
    async fn get_serializer(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        {
            let locks = self.file_locks.read().await;
            if let Some(loader) = locks.get(path) {
                // Clone the Arc *before* releasing the lock so it stays alive.
                let loader = loader.clone();
                drop(locks);
                return loader.get().await;
            }
        }

        let loader = {
            let mut locks = self.file_locks.write().await;
            locks
                .entry(path.into())
                .or_insert_with(|| Arc::new(ChunkSerializerLazyLoader::new(path.into())))
                .clone()
            // Write-lock dropped here — `loader.get()` may block on I/O and
            // must not hold the map lock.
        };

        loader.get().await
    }

    /// Attempt to evict the cached serializer for `path`.
    ///
    /// The entry is only removed when *both* conditions hold:
    /// 1. No watcher still references the path.
    /// 2. No other `Arc` clone is live (ensured via `can_remove`).
    async fn maybe_evict(&self, path: &PathBuf) {
        // Check watchers independently of file_locks to honour lock ordering.
        let still_watched = {
            let watchers = self.watchers.read().await;
            watchers.get(path).is_some_and(|&c| c > 0)
        };

        if still_watched {
            return;
        }

        let mut locks = self.file_locks.write().await;
        let removable = locks
            .get(path)
            .is_some_and(ChunkSerializerLazyLoader::can_remove);

        if removable {
            locks.remove(path);
            trace!("Evicted serializer cache for {}", path.display());
        } else {
            trace!(
                "Skipping eviction for {} — references still live",
                path.display()
            );
        }
    }
}

impl<P, S> FileIO for ChunkFileManager<S>
where
    P: PathFromLevelFolder + Send + Sync + Sized + Dirtiable + 'static,
    S: ChunkSerializer<Data = P, WriteBackend = PathBuf>,
    S::ChunkConfig: Send + Sync,
{
    type Data = Arc<S::Data>;

    fn watch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks: &'a [Vector2<i32>],
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let paths: Vec<_> = chunks
                .iter()
                .map(|c| P::file_path(folder, &S::get_chunk_key(c)))
                .collect();

            let mut watchers = self.watchers.write().await;
            for path in paths {
                *watchers.entry(path).or_insert(0) += 1;
            }
        })
    }

    fn unwatch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks: &'a [Vector2<i32>],
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let paths: Vec<_> = chunks
                .iter()
                .map(|c| P::file_path(folder, &S::get_chunk_key(c)))
                .collect();

            let mut watchers = self.watchers.write().await;
            for path in paths {
                if let std::collections::btree_map::Entry::Occupied(mut e) = watchers.entry(path) {
                    let count = e.get_mut();
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        e.remove();
                    }
                }
            }
        })
    }

    fn clear_watched_chunks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.watchers.write().await.clear();
        })
    }

    fn fetch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // Group requested chunk coords by their region file.
            let mut regions_chunks: BTreeMap<String, Vec<Vector2<i32>>> = BTreeMap::new();
            for at in chunk_coords {
                regions_chunks
                    .entry(S::get_chunk_key(at))
                    .or_default()
                    .push(*at);
            }

            let region_tasks = regions_chunks.into_iter().map(|(file_name, chunks)| {
                let task_stream = stream.clone();
                async move {
                    let path = P::file_path(folder, &file_name);

                    let chunk_serializer = match self.get_serializer(&path).await {
                        Ok(s) => s,
                        Err(ChunkReadingError::ChunkNotExist) => {
                            unreachable!("get_serializer always returns a default serializer")
                        }
                        Err(err) => {
                            // Best-effort: report the error for the first coord in the batch.
                            let _ = task_stream.send(LoadedData::Error((chunks[0], err))).await;
                            return;
                        }
                    };

                    // A bounded channel of 1 keeps backpressure between the
                    // serializer and the caller without unbounded buffering.
                    let (send, mut recv) =
                        mpsc::channel::<LoadedData<S::Data, ChunkReadingError>>(1);

                    // Forward received chunks, wrapping them in `Arc`.
                    // Captured move is intentional — `task_stream` is consumed here.
                    let forward = async move {
                        while let Some(data) = recv.recv().await {
                            let wrapped = data.map_loaded(Arc::new);
                            if task_stream.send(wrapped).await.is_err() {
                                // Receiver dropped; abort early to avoid wasted work.
                                return;
                            }
                        }
                    };

                    // Hold the read lock only for the duration of `get_chunks`.
                    let read = async move {
                        let serializer = chunk_serializer.read().await;
                        serializer.get_chunks(chunks, send).await;
                    };

                    join!(forward, read);
                }
            });

            join_all(region_tasks).await;
        })
    }

    fn save_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> BoxFuture<'a, Result<(), ChunkWritingError>> {
        Box::pin(async move {
            // Group chunks by region file.
            let mut regions_chunks: BTreeMap<String, Vec<Self::Data>> = BTreeMap::new();
            for (at, chunk) in chunks_data {
                regions_chunks
                    .entry(S::get_chunk_key(&at))
                    .or_default()
                    .push(chunk);
            }

            let tasks = regions_chunks
                .into_iter()
                .map(|(file_name, chunk_locks)| async move {
                    let path = P::file_path(folder, &file_name);
                    trace!("Saving chunks into {}", path.display());

                    let chunk_serializer = match self.get_serializer(&path).await {
                        Ok(s) => s,
                        Err(ChunkReadingError::ChunkNotExist) => {
                            unreachable!("get_serializer always returns a default serializer")
                        }
                        Err(ChunkReadingError::IoError(err)) => {
                            error!("I/O error reading region before write: {err}");
                            return Err(ChunkWritingError::IoError(err));
                        }
                        Err(err) => {
                            return Err(ChunkWritingError::IoError(std::io::Error::other(
                                err.to_string(),
                            )));
                        }
                    };

                    {
                        let mut writer = chunk_serializer.write().await;
                        for chunk in &chunk_locks {
                            // Atomically snapshot and clear the dirty flag before we
                            // write so that any mutation that races in *during* this
                            // serialisation round will mark dirty again correctly.
                            let was_dirty = chunk.is_dirty();
                            chunk.mark_dirty(false);

                            if was_dirty {
                                writer.update_chunk(&**chunk, &self.chunk_config).await?;
                            }
                        }
                        // Write-lock released here — flush can proceed under a read-lock.
                    }

                    trace!("Chunk data updated for {}", path.display());

                    // We check watchers *after* releasing the write-lock to honour
                    // lock ordering (serializer lock → watchers, never the reverse).
                    let is_watched = {
                        let watchers = self.watchers.read().await;
                        watchers.get(&path).is_some_and(|&c| c > 0)
                    };

                    if !is_watched {
                        // A read-lock suffices for `write()` since we have already
                        // applied all mutations above.
                        {
                            let serializer = chunk_serializer.read().await;
                            debug!("Flushing {} to disk", path.display());
                            serializer
                                .write(&path)
                                .await
                                .map_err(ChunkWritingError::IoError)?
                            // Read-lock released here.
                        };

                        // Drop our handle so `can_remove` may succeed.
                        drop(chunk_serializer);

                        // Evict the cache entry when no longer needed.
                        self.maybe_evict(&path).await;
                    }

                    Ok(())
                });

            // Collect all region results; surface the first error encountered.
            let results: Vec<Result<(), ChunkWritingError>> = join_all(tasks).await;
            results.into_iter().find(Result::is_err).unwrap_or(Ok(()))
        })
    }

    /// Blocks until all in-flight serialiser operations have completed by
    /// acquiring (and immediately releasing) a write-lock on every cached
    /// serialiser.
    ///
    /// This is a linearisation point: after this future resolves no mutation
    /// started before the call is still running.
    fn block_and_await_ongoing_tasks(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            // Snapshot the current set of loaders under a read-lock so we do
            // not block new insertions longer than necessary.
            let loaders: Vec<Arc<ChunkSerializerLazyLoader<S>>> =
                { self.file_locks.read().await.values().cloned().collect() };

            // For each loader that has been initialised, acquire a write-lock
            // and release it immediately.  This guarantees that any concurrent
            // read or write operation that was in progress has finished.
            let drain_tasks = loaders.into_iter().map(|loader| async move {
                if let Some(serializer_arc) = loader.internal.get() {
                    // Acquiring + immediately dropping the write-lock acts as a
                    // barrier: it can only succeed once all current lock holders
                    // have released their guards.
                    let _guard = serializer_arc.write().await;
                }
            });

            join_all(drain_tasks).await;
        })
    }
}

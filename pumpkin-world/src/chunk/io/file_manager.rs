use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    },
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

/// Upper bound on how many region files stay resident in the serializer cache.
///
/// Vanilla's `RegionFileStorage` keeps at most this many open region files and
/// closes the least recently used one when the bound is reached
/// (`/root/Vanilla/src/net/minecraft/world/level/chunk/storage/RegionFileStorage.java:30`
/// `MAX_CACHE_SIZE = 256`, with the LRU order maintained by the
/// `Long2ObjectLinkedOpenHashMap` at `:31` and the eviction at `:48-50`).
///
/// A cached entry here is much heavier than vanilla's: vanilla keeps only the
/// 8 KiB region header in memory and streams payloads from the file channel,
/// while Pumpkin's serializers hold every compressed chunk payload of the
/// region. The count bound is still vanilla's, so worlds behave the same way
/// with respect to *which* regions stay hot.
const MAX_CACHE_SIZE: usize = 256;

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
/// ### Cache bound
///
/// `file_locks` is capped at [`MAX_CACHE_SIZE`] entries with least-recently-used
/// eviction, mirroring vanilla's `RegionFileStorage` region cache. Eviction runs
/// on the read path as well as the write path, so a region that is only ever
/// read still leaves the cache.
///
/// The bound is *soft*: an entry that is watched, holds un-flushed updates, or is
/// still referenced by an in-flight operation is never evicted, so the map may
/// sit above the bound while such entries dominate. Every subsequent insertion
/// retries the trim.
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
    /// Monotonic source for the LRU stamps stored on each loader.
    access_clock: AtomicU64,
    chunk_config: S::ChunkConfig,
}

pub(crate) trait PathFromLevelFolder {
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf;
}

struct ChunkSerializerLazyLoader<S: ChunkSerializer<WriteBackend = PathBuf>> {
    path: PathBuf,
    /// Initialised at most once; subsequent calls reuse the same Arc.
    internal: OnceCell<Arc<RwLock<S>>>,
    /// Last value taken from `ChunkFileManager::access_clock`; higher is newer.
    last_used: AtomicU64,
    /// Set while the in-memory serializer holds chunk updates that have not
    /// been flushed to disk yet. Such an entry must never be evicted — dropping
    /// it would silently discard those updates.
    ///
    /// Cleared only by a successful flush. An entry updated while the path was
    /// watched therefore stays pinned until the caller's flush finally happens
    /// (watched paths are skipped by both eviction paths anyway); the bias is
    /// deliberately towards keeping memory over losing a write.
    unflushed: AtomicBool,
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkSerializerLazyLoader<S> {
    fn new(path: PathBuf, stamp: u64) -> Self {
        Self {
            path,
            internal: OnceCell::new(),
            last_used: AtomicU64::new(stamp),
            unflushed: AtomicBool::new(false),
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
        if loader.unflushed.load(Relaxed) {
            return false;
        }
        loader
            .internal
            .get()
            .is_none_or(|arc| Arc::strong_count(arc) == 1)
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

/// Orders `candidates` least-recently-used first and keeps at most `excess`.
///
/// `candidates` are `(stamp, path)` pairs of entries that are *not* known to be
/// unsafe to evict; the caller still re-verifies each survivor under the map
/// write-lock. Ties are broken by path so the choice is deterministic.
fn pick_lru_victims(mut candidates: Vec<(u64, PathBuf)>, excess: usize) -> Vec<PathBuf> {
    if excess == 0 {
        return Vec::new();
    }
    candidates.sort_unstable();
    candidates.truncate(excess);
    candidates.into_iter().map(|(_, path)| path).collect()
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    pub fn new(chunk_config: S::ChunkConfig) -> Self {
        Self {
            file_locks: RwLock::new(BTreeMap::new()),
            watchers: RwLock::new(BTreeMap::new()),
            access_clock: AtomicU64::new(0),
            chunk_config,
        }
    }
}

impl<S: ChunkSerializer<WriteBackend = PathBuf>> ChunkFileManager<S> {
    fn next_stamp(&self) -> u64 {
        self.access_clock.fetch_add(1, Relaxed)
    }

    /// Returns the cache entry for `path`, inserting a lazy-loader if absent.
    ///
    /// Uses an optimistic read-first pattern: in the common case (cache hit)
    /// we never need a write-lock on the map. Every call refreshes the entry's
    /// LRU stamp, vanilla's `getAndMoveToFirst` / `putAndMoveToFirst`
    /// (`RegionFileStorage.java:44` and `:54`).
    async fn get_loader(&self, path: &Path) -> Arc<ChunkSerializerLazyLoader<S>> {
        {
            let locks = self.file_locks.read().await;
            if let Some(loader) = locks.get(path) {
                loader.last_used.store(self.next_stamp(), Relaxed);
                // Clone the Arc *before* releasing the lock so it stays alive.
                return loader.clone();
            }
        }

        let (loader, inserted) = {
            let mut locks = self.file_locks.write().await;
            let stamp = self.next_stamp();
            let mut inserted = false;
            let loader = locks
                .entry(path.into())
                .or_insert_with(|| {
                    inserted = true;
                    Arc::new(ChunkSerializerLazyLoader::new(path.into(), stamp))
                })
                .clone();
            loader.last_used.store(stamp, Relaxed);
            (loader, inserted)
            // Write-lock dropped here — trimming and `loader.get()` may block
            // on I/O and must not hold the map lock.
        };

        if inserted {
            // Vanilla trims before inserting (`RegionFileStorage.java:48-50`);
            // we insert first and trim after so the fast path stays lock-free
            // for cache hits. The steady-state bound is the same.
            self.trim_cache().await;
        }

        loader
    }

    /// Returns the serializer for `path`, loading it from disk if needed.
    async fn get_serializer(&self, path: &Path) -> Result<Arc<RwLock<S>>, ChunkReadingError> {
        self.get_loader(path).await.get().await
    }

    /// Drops least-recently-used entries until the cache is back at
    /// [`MAX_CACHE_SIZE`].
    ///
    /// Entries that are watched, hold un-flushed updates, or are still
    /// referenced elsewhere are skipped — evicting one of those would lose
    /// writes. No lock is held across an `await`, and `watchers` is acquired in
    /// its own critical section to preserve the documented lock ordering.
    async fn trim_cache(&self) {
        // Phase 1: snapshot LRU candidates under a read-lock.
        let victims = {
            let locks = self.file_locks.read().await;
            if locks.len() <= MAX_CACHE_SIZE {
                return;
            }
            let excess = locks.len() - MAX_CACHE_SIZE;
            // `unflushed` is the only pin we can test cheaply and reliably from
            // a read-lock; strong counts are re-checked in phase 3.
            let candidates = locks
                .iter()
                .filter(|(_, loader)| !loader.unflushed.load(Relaxed))
                .map(|(path, loader)| (loader.last_used.load(Relaxed), path.clone()))
                .collect();
            pick_lru_victims(candidates, excess)
        };

        if victims.is_empty() {
            return;
        }

        // Phase 2: filter out watched paths in a `watchers`-only critical section.
        let victims: Vec<PathBuf> = {
            let watchers = self.watchers.read().await;
            victims
                .into_iter()
                .filter(|path| watchers.get(path).is_none_or(|&count| count == 0))
                .collect()
        };

        if victims.is_empty() {
            return;
        }

        // Phase 3: re-verify and remove under the map write-lock. No await here.
        let mut locks = self.file_locks.write().await;
        for path in victims {
            if locks.len() <= MAX_CACHE_SIZE {
                break;
            }
            let removable = locks.get(&path).is_some_and(ChunkSerializerLazyLoader::can_remove);
            if removable {
                locks.remove(&path);
                trace!("Evicted LRU serializer cache entry {}", path.display());
            } else {
                trace!(
                    "Skipping LRU eviction for {} — references still live",
                    path.display()
                );
            }
        }
    }

    /// Attempt to evict the cached serializer for `path`.
    ///
    /// The entry is only removed when *both* conditions hold:
    /// 1. No watcher still references the path.
    /// 2. No other `Arc` clone is live and nothing is un-flushed (`can_remove`).
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
                            return;
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

                    let loader = self.get_loader(&path).await;
                    let chunk_serializer = match loader.get().await {
                        Ok(s) => s,
                        Err(ChunkReadingError::ChunkNotExist) => {
                            return Err(ChunkWritingError::IoError(std::io::Error::other(
                                "get_serializer returned ChunkNotExist",
                            )));
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
                                // Pin the cache entry *before* mutating it: from
                                // here on the in-memory serializer is ahead of
                                // the file and evicting it would lose the write.
                                loader.unflushed.store(true, Relaxed);
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
                                .map_err(ChunkWritingError::IoError)?;
                            // Read-lock released here.
                        };

                        // On disk and in memory now agree, so the entry may be
                        // evicted again. A failed flush leaves the pin in place
                        // on purpose: the data only exists in memory.
                        loader.unflushed.store(false, Relaxed);

                        // Drop our handles so `can_remove` may succeed.
                        drop(chunk_serializer);
                        drop(loader);

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

#[cfg(test)]
mod tests {
    use super::{MAX_CACHE_SIZE, PathBuf, pick_lru_victims};

    fn candidates(entries: &[(u64, &str)]) -> Vec<(u64, PathBuf)> {
        entries
            .iter()
            .map(|(stamp, path)| (*stamp, PathBuf::from(*path)))
            .collect()
    }

    #[test]
    fn lru_victims_are_the_oldest_entries() {
        let victims = pick_lru_victims(
            candidates(&[(30, "r.0.0"), (10, "r.1.0"), (20, "r.2.0"), (40, "r.3.0")]),
            2,
        );
        assert_eq!(
            victims,
            vec![PathBuf::from("r.1.0"), PathBuf::from("r.2.0")]
        );
    }

    #[test]
    fn lru_victims_respect_the_requested_count() {
        let all = candidates(&[(1, "a"), (2, "b"), (3, "c")]);
        assert!(pick_lru_victims(all.clone(), 0).is_empty());
        assert_eq!(pick_lru_victims(all.clone(), 1).len(), 1);
        assert_eq!(pick_lru_victims(all, 9).len(), 3);
    }

    #[test]
    fn lru_victims_skip_entries_the_caller_filtered_out() {
        // The oldest entry (stamp 1) is pinned (un-flushed) so the caller never
        // offers it; the next oldest must be chosen instead.
        let victims = pick_lru_victims(candidates(&[(2, "b"), (3, "c")]), 1);
        assert_eq!(victims, vec![PathBuf::from("b")]);
    }

    #[test]
    fn lru_victims_break_ties_deterministically() {
        let victims = pick_lru_victims(candidates(&[(5, "z"), (5, "a"), (5, "m")]), 2);
        assert_eq!(victims, vec![PathBuf::from("a"), PathBuf::from("m")]);
    }

    #[test]
    fn cache_bound_matches_vanilla() {
        // RegionFileStorage.java:30 `MAX_CACHE_SIZE = 256`
        assert_eq!(MAX_CACHE_SIZE, 256);
    }
}

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use futures::future::join_all;
use pumpkin_util::math::vector2::Vector2;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tokio::sync::{OnceCell, mpsc};
use tracing::error;
use xxhash_rust::xxh64::xxh64;

use crate::{
    chunk::{
        ChunkReadingError, ChunkWritingError,
        format::{
            anvil::{AnvilRegion, CHUNK_COUNT, SingleChunkDataSerializer},
            linear::LinearV2File,
            pump::PumpFile,
        },
    },
    level::LevelFolder,
};

use super::{FileIO, LoadedData, RegionFile, region_chunk_index, region_of, run_blocking};

/// A region nobody touched for this long is closed, once its staged chunks are written.
const REGION_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

type RegionBatch<D> = Vec<(Vector2<i32>, Arc<D>)>;

pub(crate) trait PathFromLevelFolder {
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf;
}

struct RegionState<R> {
    file: R,
    staged_since: Option<Instant>,
}

struct RegionEntry<R> {
    state: OnceCell<RwLock<RegionState<R>>>,
    /// xxh64 of the NBT last read or staged for each chunk, 0 when unknown. Saving a chunk whose
    /// NBT hashes the same is skipped, which catches chunks marked dirty without a real change.
    hashes: Box<[AtomicU64]>,
    /// Milliseconds since the manager was created.
    last_used: AtomicU64,
}

impl<R: RegionFile> RegionEntry<R> {
    fn new(now: u64) -> Self {
        Self {
            state: OnceCell::new(),
            hashes: (0..CHUNK_COUNT).map(|_| AtomicU64::new(0)).collect(),
            last_used: AtomicU64::new(now),
        }
    }

    async fn open(&self, path: &Path, region: Vector2<i32>) -> Result<(), ChunkReadingError> {
        self.state
            .get_or_try_init(|| async {
                let path = path.to_path_buf();
                let file = run_blocking(move || R::open(&path, region))
                    .await
                    .map_err(|_| {
                        ChunkReadingError::IoError(std::io::Error::other("region open task failed"))
                    })??;
                Ok(RwLock::new(RegionState {
                    file,
                    staged_since: None,
                }))
            })
            .await
            .map(|_| ())
    }

    /// Only valid after `open` succeeded.
    fn state(&self) -> Option<&RwLock<RegionState<R>>> {
        self.state.get()
    }

    fn has_staged(&self) -> bool {
        self.state().is_some_and(|state| {
            state
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .file
                .has_staged()
        })
    }

    fn flush(&self, sync: bool, only_due: bool) -> std::io::Result<()> {
        let Some(state) = self.state() else {
            return Ok(());
        };
        let mut state = state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if only_due
            && state
                .staged_since
                .is_none_or(|since| since.elapsed() < R::FLUSH_DELAY)
        {
            return Ok(());
        }
        state.file.flush(sync)?;
        state.staged_since = None;
        Ok(())
    }
}

/// Loads and saves chunks of one storage (block chunks or entity chunks) through a cache of open
/// region files.
///
/// Saving only touches the chunks that changed: the region stays open between saves, chunks
/// whose NBT did not change are skipped, and the region format decides how much of the file a
/// flush rewrites. All file access and compression runs on blocking or Rayon threads.
pub struct ChunkFileManager<R: RegionFile, D> {
    regions: Mutex<FxHashMap<Vector2<i32>, Arc<RegionEntry<R>>>>,
    config: R::Config,
    created: Instant,
    _data: PhantomData<fn() -> D>,
}

impl<R: RegionFile, D> ChunkFileManager<R, D> {
    pub fn new(config: R::Config) -> Self {
        Self {
            regions: Mutex::new(FxHashMap::default()),
            config,
            created: Instant::now(),
            _data: PhantomData,
        }
    }

    fn now(&self) -> u64 {
        self.created.elapsed().as_millis() as u64
    }

    fn entry(&self, region: Vector2<i32>) -> Arc<RegionEntry<R>> {
        let now = self.now();
        let mut regions = self
            .regions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = regions
            .entry(region)
            .or_insert_with(|| Arc::new(RegionEntry::new(now)));
        entry.last_used.store(now, Ordering::Relaxed);
        entry.clone()
    }

    fn entries(&self) -> Vec<Arc<RegionEntry<R>>> {
        self.regions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect()
    }

    /// Closes regions that are idle or over the cache capacity. Regions with staged chunks or
    /// an operation in flight stay open.
    fn evict(&self) {
        let now = self.now();
        let idle_ms = REGION_IDLE_TIMEOUT.as_millis() as u64;
        let mut regions = self
            .regions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // The map holds the only reference while nothing uses the entry, and new references are
        // only handed out under this lock.
        let mut closable: Vec<(Vector2<i32>, u64)> = regions
            .iter()
            .filter(|(_, entry)| Arc::strong_count(entry) == 1 && !entry.has_staged())
            .map(|(pos, entry)| (*pos, entry.last_used.load(Ordering::Relaxed)))
            .collect();
        closable.sort_unstable_by_key(|(_, last_used)| *last_used);

        let mut over_capacity = regions.len().saturating_sub(R::CACHE_CAPACITY);
        for (pos, last_used) in closable {
            if over_capacity > 0 {
                over_capacity -= 1;
            } else if now.saturating_sub(last_used) < idle_ms {
                break;
            }
            regions.remove(&pos);
        }
    }

    async fn flush_entries(&self, sync: bool, only_due: bool) {
        let entries = self.entries();
        let result = run_blocking(move || {
            for entry in entries {
                if let Err(err) = entry.flush(sync, only_due) {
                    error!("Failed writing region file: {err}");
                }
            }
        })
        .await;
        if result.is_err() {
            error!("Region flush task failed");
        }
    }
}

impl<R, D> FileIO for ChunkFileManager<R, D>
where
    R: RegionFile,
    D: SingleChunkDataSerializer + PathFromLevelFolder,
{
    type Data = Arc<D>;

    async fn fetch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        let mut regions_chunks: FxHashMap<Vector2<i32>, Vec<Vector2<i32>>> = FxHashMap::default();
        for at in chunk_coords {
            regions_chunks.entry(region_of(*at)).or_default().push(*at);
        }

        let region_tasks = regions_chunks.into_iter().map(|(region, chunks)| {
            let stream = stream.clone();
            async move {
                let path = D::file_path(folder, &R::file_name(region));
                let entry = self.entry(region);
                if let Err(err) = entry.open(&path, region).await {
                    error!("Failed to open region file {}: {err}", path.display());
                    for pos in chunks {
                        let err = ChunkReadingError::IoError(std::io::Error::other(
                            "region file could not be opened",
                        ));
                        if stream.send(LoadedData::Error((pos, err))).await.is_err() {
                            return;
                        }
                    }
                    return;
                }

                let read_entry = entry.clone();
                let Ok(raw_chunks) = run_blocking(move || {
                    let Some(state) = read_entry.state() else {
                        return Vec::new();
                    };
                    let state = state
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    chunks
                        .into_iter()
                        .map(|pos| (pos, state.file.read_chunk(region_chunk_index(pos))))
                        .collect::<Vec<_>>()
                })
                .await
                else {
                    error!("Region read task failed for {}", path.display());
                    return;
                };

                let (tx, mut rx) = mpsc::channel(raw_chunks.len().max(1));
                rayon::spawn(move || {
                    raw_chunks.into_par_iter().for_each(|(pos, nbt)| {
                        let loaded = match nbt {
                            Ok(None) => LoadedData::Missing(pos),
                            Ok(Some(raw)) => match R::decode_chunk(raw) {
                                Err(err) => LoadedData::Error((pos, err)),
                                Ok(nbt) => {
                                    // Keep a hash a concurrent save already recorded.
                                    let _ = entry.hashes[region_chunk_index(pos)].compare_exchange(
                                        0,
                                        xxh64(&nbt, 0),
                                        Ordering::Relaxed,
                                        Ordering::Relaxed,
                                    );
                                    match D::from_bytes(&nbt, pos) {
                                        Ok(chunk) => LoadedData::Loaded(Arc::new(chunk)),
                                        Err(err) => LoadedData::Error((pos, err)),
                                    }
                                }
                            },
                            Err(err) => LoadedData::Error((pos, err)),
                        };
                        let _ = tx.blocking_send(loaded);
                    });
                });

                while let Some(item) = rx.recv().await {
                    if stream.send(item).await.is_err() {
                        return;
                    }
                }
            }
        });

        join_all(region_tasks).await;
    }

    async fn save_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> Result<(), ChunkWritingError> {
        let mut regions_chunks: FxHashMap<Vector2<i32>, RegionBatch<D>> = FxHashMap::default();
        for (at, chunk) in chunks_data {
            // Clear the flag before serializing, so a change made meanwhile marks it again.
            if chunk.is_dirty() {
                chunk.mark_dirty(false);
                regions_chunks
                    .entry(region_of(at))
                    .or_default()
                    .push((at, chunk));
            }
        }

        let tasks = regions_chunks
            .into_iter()
            .map(|(region, chunks)| async move {
                let path = D::file_path(folder, &R::file_name(region));
                let entry = self.entry(region);
                entry.open(&path, region).await.map_err(|err| match err {
                    ChunkReadingError::IoError(err) => ChunkWritingError::IoError(err),
                    err => ChunkWritingError::IoError(std::io::Error::other(err.to_string())),
                })?;

                let config = self.config.clone();
                run_blocking(move || {
                    let encoded = chunks
                        .into_par_iter()
                        .filter_map(|(pos, chunk)| {
                            let nbt = match chunk.to_bytes() {
                                Ok(nbt) => nbt,
                                Err(err) => {
                                    return Some(Err(ChunkWritingError::ChunkSerializingError(
                                        err.to_string(),
                                    )));
                                }
                            };
                            let index = region_chunk_index(pos);
                            let hash = xxh64(&nbt, 0);
                            if entry.hashes[index].load(Ordering::Relaxed) == hash {
                                return None;
                            }
                            Some(R::encode_chunk(nbt, &config).map(|chunk| (index, hash, chunk)))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if encoded.is_empty() {
                        return Ok(());
                    }

                    let Some(state) = entry.state() else {
                        return Ok(());
                    };
                    let mut state = state
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    for (index, hash, chunk) in encoded {
                        state.file.stage_chunk(index, chunk);
                        entry.hashes[index].store(hash, Ordering::Relaxed);
                    }
                    state.staged_since.get_or_insert_with(Instant::now);
                    if R::FLUSH_DELAY.is_zero() {
                        state
                            .file
                            .flush(false)
                            .map_err(ChunkWritingError::IoError)?;
                        state.staged_since = None;
                    }
                    Ok(())
                })
                .await
                .map_err(|_| {
                    ChunkWritingError::IoError(std::io::Error::other("chunk save task failed"))
                })?
            });

        let results: Vec<Result<(), ChunkWritingError>> = join_all(tasks).await;
        results.into_iter().find(Result::is_err).unwrap_or(Ok(()))
    }

    async fn flush_due(&self) {
        self.flush_entries(false, true).await;
        self.evict();
    }

    async fn flush_all(&self) {
        self.flush_entries(true, false).await;
        self.evict();
    }
}

pub enum LevelFileIO<D> {
    Linear(ChunkFileManager<LinearV2File, D>),
    Anvil(ChunkFileManager<AnvilRegion, D>),
    Pump(ChunkFileManager<PumpFile, D>),
}

impl<D> FileIO for LevelFileIO<D>
where
    D: SingleChunkDataSerializer + PathFromLevelFolder,
{
    type Data = Arc<D>;

    async fn fetch_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunk_coords: &'a [Vector2<i32>],
        stream: mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        match self {
            Self::Linear(io) => io.fetch_chunks(folder, chunk_coords, stream).await,
            Self::Anvil(io) => io.fetch_chunks(folder, chunk_coords, stream).await,
            Self::Pump(io) => io.fetch_chunks(folder, chunk_coords, stream).await,
        }
    }

    async fn save_chunks<'a>(
        &'a self,
        folder: &'a LevelFolder,
        chunks_data: Vec<(Vector2<i32>, Self::Data)>,
    ) -> Result<(), ChunkWritingError> {
        match self {
            Self::Linear(io) => io.save_chunks(folder, chunks_data).await,
            Self::Anvil(io) => io.save_chunks(folder, chunks_data).await,
            Self::Pump(io) => io.save_chunks(folder, chunks_data).await,
        }
    }

    async fn flush_due(&self) {
        match self {
            Self::Linear(io) => io.flush_due().await,
            Self::Anvil(io) => io.flush_due().await,
            Self::Pump(io) => io.flush_due().await,
        }
    }

    async fn flush_all(&self) {
        match self {
            Self::Linear(io) => io.flush_all().await,
            Self::Anvil(io) => io.flush_all().await,
            Self::Pump(io) => io.flush_all().await,
        }
    }
}

use std::{
    path::PathBuf,
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

pub(crate) trait PathFromLevelFolder {
    fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf;
}

struct RegionState<R> {
    file: R,
    staged_since: Option<Instant>,
}

struct RegionEntry<R, D> {
    path: PathBuf,
    region: Vector2<i32>,
    pending: Mutex<FxHashMap<usize, Arc<D>>>,
    save_lock: tokio::sync::Mutex<()>,
    state: OnceCell<RwLock<RegionState<R>>>,
    /// xxh64 of the NBT last read or staged for each chunk, 0 when unknown. Saving a chunk whose
    /// NBT hashes the same is skipped, which catches chunks marked dirty without a real change.
    hashes: Box<[AtomicU64]>,
    /// Milliseconds since the manager was created.
    last_used: AtomicU64,
}

impl<R: RegionFile, D> RegionEntry<R, D> {
    fn new(now: u64, path: PathBuf, region: Vector2<i32>) -> Self {
        Self {
            path,
            region,
            pending: Mutex::new(FxHashMap::default()),
            save_lock: tokio::sync::Mutex::new(()),
            state: OnceCell::new(),
            hashes: (0..CHUNK_COUNT).map(|_| AtomicU64::new(0)).collect(),
            last_used: AtomicU64::new(now),
        }
    }

    async fn open(&self) -> Result<(), ChunkReadingError> {
        self.state
            .get_or_try_init(|| async {
                let path = self.path.clone();
                let region = self.region;
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
        let pending = !self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty();
        pending
            || self.state().is_some_and(|state| {
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

impl<R: RegionFile, D: SingleChunkDataSerializer> RegionEntry<R, D> {
    async fn read_pending(
        &self,
        chunks: &mut Vec<Vector2<i32>>,
    ) -> Vec<LoadedData<Arc<D>, ChunkReadingError>> {
        let pending_chunks = {
            let pending = self
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut pending_chunks = Vec::new();
            chunks.retain(|pos| {
                pending.get(&region_chunk_index(*pos)).is_none_or(|chunk| {
                    pending_chunks.push((*pos, chunk.clone()));
                    false
                })
            });
            pending_chunks
        };
        if pending_chunks.is_empty() {
            return Vec::new();
        }
        let positions: Vec<_> = pending_chunks.iter().map(|(pos, _)| *pos).collect();
        run_blocking(move || {
            pending_chunks
                .into_par_iter()
                .map(|(pos, chunk)| {
                    // Loading consumes entity and block-entity NBT; keep the retry source intact.
                    let loaded = R::serialize_chunk(&*chunk)
                        .map_err(|err| {
                            ChunkReadingError::IoError(std::io::Error::other(err.to_string()))
                        })
                        .and_then(|nbt| D::from_bytes(&nbt, pos));
                    match loaded {
                        Ok(chunk) => LoadedData::Loaded(Arc::new(chunk)),
                        Err(err) => LoadedData::Error((pos, err)),
                    }
                })
                .collect()
        })
        .await
        .unwrap_or_else(|err| {
            positions
                .into_iter()
                .map(|pos| {
                    LoadedData::Error((
                        pos,
                        ChunkReadingError::IoError(std::io::Error::other(err.to_string())),
                    ))
                })
                .collect()
        })
    }

    async fn save_pending(self: &Arc<Self>, config: R::Config) -> Result<(), ChunkWritingError> {
        let _save = self.save_lock.lock().await;
        if self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
        {
            return Ok(());
        }
        self.open().await.map_err(|err| match err {
            ChunkReadingError::IoError(err) => ChunkWritingError::IoError(err),
            err => ChunkWritingError::IoError(std::io::Error::other(err.to_string())),
        })?;
        let chunks = std::mem::take(
            &mut *self
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let retry = chunks.clone();
        let entry = self.clone();
        let result = run_blocking(move || {
            let encoded: Vec<_> = chunks
                .into_par_iter()
                .map(|(index, chunk)| {
                    // A mutation during serialization must mark the chunk dirty again.
                    chunk.mark_dirty(false);
                    let nbt = R::serialize_chunk(&*chunk)
                        .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;
                    let hash = xxh64(&nbt, 0);
                    if entry.hashes[index].load(Ordering::Relaxed) == hash {
                        return Ok(None);
                    }
                    R::encode_chunk(nbt, &config).map(|chunk| Some((index, hash, chunk)))
                })
                .collect();
            let Some(state) = entry.state() else {
                return Ok(());
            };
            let mut state = state
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut error = None;
            let mut staged = false;
            for chunk in encoded {
                match chunk {
                    Ok(Some((index, hash, chunk))) => {
                        state.file.stage_chunk(index, chunk);
                        entry.hashes[index].store(hash, Ordering::Relaxed);
                        staged = true;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        error.get_or_insert(err);
                    }
                }
            }
            if staged {
                state.staged_since.get_or_insert_with(Instant::now);
                if R::FLUSH_DELAY.is_zero() {
                    state
                        .file
                        .flush(false)
                        .map_err(ChunkWritingError::IoError)?;
                    state.staged_since = None;
                }
            }
            error.map_or(Ok(()), Err)
        })
        .await
        .unwrap_or_else(|_| {
            Err(ChunkWritingError::IoError(std::io::Error::other(
                "chunk save task failed",
            )))
        });
        if result.is_err() {
            let mut pending = self
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (index, chunk) in retry {
                chunk.mark_dirty(true);
                // A later save may already have queued a newer chunk at this position.
                pending.entry(index).or_insert(chunk);
            }
        }
        result
    }
}

/// Loads and saves chunks of one storage (block chunks or entity chunks) through a cache of open
/// region files.
///
/// Saving only touches the chunks that changed: the region stays open between saves, chunks
/// whose NBT did not change are skipped, and the region format decides how much of the file a
/// flush rewrites. All file access and compression runs on blocking or Rayon threads.
pub struct ChunkFileManager<R: RegionFile, D> {
    regions: Mutex<RegionCache<R, D>>,
    config: R::Config,
    created: Instant,
}

type RegionCache<R, D> = FxHashMap<Vector2<i32>, Arc<RegionEntry<R, D>>>;

impl<R: RegionFile, D: SingleChunkDataSerializer> ChunkFileManager<R, D> {
    pub fn new(config: R::Config) -> Self {
        Self {
            regions: Mutex::new(FxHashMap::default()),
            config,
            created: Instant::now(),
        }
    }

    fn now(&self) -> u64 {
        self.created.elapsed().as_millis() as u64
    }

    fn entry(&self, region: Vector2<i32>, path: PathBuf) -> Arc<RegionEntry<R, D>> {
        let now = self.now();
        let mut regions = self
            .regions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = regions
            .entry(region)
            .or_insert_with(|| Arc::new(RegionEntry::new(now, path, region)));
        entry.last_used.store(now, Ordering::Relaxed);
        entry.clone()
    }

    fn entries(&self) -> Vec<Arc<RegionEntry<R, D>>> {
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
        join_all(entries.iter().map(|entry| async {
            if let Err(err) = entry.save_pending(self.config.clone()).await {
                error!("Failed saving pending chunks: {err}");
            }
        }))
        .await;
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
                let entry = self.entry(region, path.clone());
                let save = entry.save_lock.lock().await;
                let mut disk_chunks = chunks;
                for loaded in entry.read_pending(&mut disk_chunks).await {
                    if stream.send(loaded).await.is_err() {
                        return;
                    }
                }
                if disk_chunks.is_empty() {
                    return;
                }
                if let Err(err) = entry.open().await {
                    error!("Failed to open region file {}: {err}", path.display());
                    for pos in disk_chunks {
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
                    disk_chunks
                        .into_iter()
                        .map(|pos| (pos, state.file.read_chunk(region_chunk_index(pos))))
                        .collect::<Vec<_>>()
                })
                .await
                else {
                    error!("Region read task failed for {}", path.display());
                    return;
                };
                drop(save);

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
        let mut entries = FxHashMap::default();
        for (pos, chunk) in chunks_data {
            if chunk.is_dirty() {
                let region = region_of(pos);
                let entry = entries.entry(region).or_insert_with(|| {
                    self.entry(region, D::file_path(folder, &R::file_name(region)))
                });
                entry
                    .pending
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(region_chunk_index(pos), chunk);
            }
        }

        let results = join_all(
            entries
                .values()
                .map(|entry| entry.save_pending(self.config.clone())),
        )
        .await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{ChunkEntityData, ChunkSerializingError, io::Dirtiable};
    use bytes::Bytes;
    use pumpkin_config::chunk::AnvilChunkConfig;
    use std::{path::Path, sync::atomic::AtomicBool};

    fn folder(root: &Path) -> LevelFolder {
        LevelFolder {
            root_folder: root.into(),
            dim_folder: root.into(),
            region_folder: root.join("region"),
            entities_folder: root.join("entities"),
            poi_folder: root.join("poi"),
        }
    }

    fn entity_chunk(x: i32) -> ChunkEntityData {
        ChunkEntityData {
            x,
            z: 0,
            data: Mutex::new(Vec::new()),
            live: AtomicBool::new(false),
            dirty: AtomicBool::new(true),
        }
    }

    struct FallibleChunk {
        chunk: ChunkEntityData,
        fail: AtomicBool,
    }

    impl Dirtiable for FallibleChunk {
        fn is_dirty(&self) -> bool {
            self.chunk.is_dirty()
        }

        fn mark_dirty(&self, flag: bool) {
            self.chunk.mark_dirty(flag);
        }
    }

    impl PathFromLevelFolder for FallibleChunk {
        fn file_path(folder: &LevelFolder, file_name: &str) -> PathBuf {
            ChunkEntityData::file_path(folder, file_name)
        }
    }

    impl SingleChunkDataSerializer for FallibleChunk {
        fn to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
            if self.fail.load(Ordering::Relaxed) {
                return Err(ChunkSerializingError::ErrorSerializingChunk(
                    pumpkin_nbt::Error::SerdeError("serialization failed".into()),
                ));
            }
            self.chunk.to_bytes()
        }

        fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
            Ok(Self {
                chunk: ChunkEntityData::from_bytes(bytes, pos)?,
                fail: AtomicBool::new(false),
            })
        }

        fn position(&self) -> (i32, i32) {
            self.chunk.position()
        }
    }

    #[tokio::test]
    async fn failed_region_open_retries_unloaded_chunks() {
        let dir = tempfile::TempDir::new().unwrap();
        let folder = folder(dir.path());
        let entities = &folder.entities_folder;
        std::fs::write(entities, b"not a directory").unwrap();
        let manager =
            ChunkFileManager::<AnvilRegion, ChunkEntityData>::new(AnvilChunkConfig::default());
        let chunk = Arc::new(entity_chunk(0));
        let mut entity = pumpkin_nbt::compound::NbtCompound::new();
        entity.put_string("id", "minecraft:item".into());
        entity.put_int("Age", 123);
        chunk.data.lock().unwrap().push(entity.clone());
        assert!(
            manager
                .save_chunks(&folder, vec![(Vector2::new(0, 0), chunk.clone())])
                .await
                .is_err()
        );
        assert!(chunk.is_dirty());
        drop(chunk);
        std::fs::remove_file(entities).unwrap();
        std::fs::create_dir(entities).unwrap();
        let (tx, mut rx) = mpsc::channel(1);
        manager
            .fetch_chunks(&folder, &[Vector2::new(0, 0)], tx)
            .await;
        let Some(LoadedData::Loaded(chunk)) = rx.recv().await else {
            panic!("pending chunk was not returned");
        };
        assert_eq!(
            std::mem::take(&mut *chunk.data.lock().unwrap()),
            vec![entity.clone()]
        );
        drop(chunk);
        manager.flush_all().await;

        let reopened = AnvilRegion::open(&entities.join("r.0.0.mca"), Vector2::new(0, 0)).unwrap();
        let nbt = AnvilRegion::decode_chunk(reopened.read_chunk(0).unwrap().unwrap()).unwrap();
        let chunk = ChunkEntityData::from_bytes(&nbt, Vector2::new(0, 0)).unwrap();
        assert_eq!(*chunk.data.lock().unwrap(), vec![entity]);
    }

    #[tokio::test]
    async fn serialization_failure_preserves_the_rest_of_the_batch() {
        let dir = tempfile::TempDir::new().unwrap();
        let folder = folder(dir.path());
        let manager =
            ChunkFileManager::<AnvilRegion, FallibleChunk>::new(AnvilChunkConfig::default());
        let chunks = [false, true].map(|fail| {
            Arc::new(FallibleChunk {
                chunk: entity_chunk(i32::from(fail)),
                fail: AtomicBool::new(fail),
            })
        });
        assert!(
            manager
                .save_chunks(
                    &folder,
                    chunks
                        .iter()
                        .enumerate()
                        .map(|(index, chunk)| (Vector2::new(index as i32, 0), chunk.clone()))
                        .collect(),
                )
                .await
                .is_err()
        );
        let path = folder.entities_folder.join("r.0.0.mca");
        let region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        assert!(region.read_chunk(0).unwrap().is_some());
        assert!(chunks[1].is_dirty());
        chunks[1].fail.store(false, Ordering::Relaxed);
        drop(chunks);
        manager.flush_all().await;
        let region = AnvilRegion::open(&path, Vector2::new(0, 0)).unwrap();
        assert!(region.read_chunk(1).unwrap().is_some());
    }
}

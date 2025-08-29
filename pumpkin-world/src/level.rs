use crate::chunk_system::{ChunkListener, ChunkLoading, GenerationSchedule, LevelChannel};
use crate::generation::generator::VanillaGenerator;
use crate::{
    BlockStateId,
    block::{RawBlockState, entities::BlockEntity},
    chunk::{
        ChunkData, ChunkEntityData, ChunkReadingError,
        format::{anvil::AnvilChunkFile, linear::LinearFile},
        io::{Dirtiable, FileIO, LoadedData, file_manager::ChunkFileManager},
    },
    dimension::Dimension,
    generation::{Seed, generator::WorldGenerator, get_world_gen},
    tick::{OrderedTick, ScheduledTick, TickPriority},
    world::BlockRegistryExt,
};
use crossbeam::channel::Sender;
use dashmap::{DashMap, Entry};
use log::trace;
use num_traits::Zero;
use pumpkin_config::{advanced_config, chunk::ChunkFormat};
use pumpkin_data::biome::Biome;
use pumpkin_data::{Block, block_properties::has_random_ticks, fluid::Fluid};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::sync::Mutex;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tokio::{
    select,
    sync::{
        Notify, RwLock,
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::task::TaskTracker;

pub type SyncChunk = Arc<RwLock<ChunkData>>;
pub type SyncEntityChunk = Arc<RwLock<ChunkEntityData>>;
pub struct ChunkRequest {
    pub pos: Vector2<i32>,
    pub response: oneshot::Sender<(SyncChunk, bool)>, // bool = is_new
}

/// The `Level` module provides functionality for working with chunks within or outside a Minecraft world.
///
/// Key features include:
///
/// - **Chunk Loading:** Efficiently loads chunks from disk.
/// - **Chunk Caching:** Stores accessed chunks in memory for faster access.
/// - **Chunk Generation:** Generates new chunks on-demand using a specified `WorldGenerator`.
///
/// For more details on world generation, refer to the `WorldGenerator` module.
pub struct Level {
    pub seed: Seed,
    pub block_registry: Arc<dyn BlockRegistryExt>,
    pub level_folder: LevelFolder,

    /// Counts the number of ticks that have been scheduled for this world
    schedule_tick_counts: AtomicU64,

    // Chunks that are paired with chunk watchers. When a chunk is no longer watched, it is removed
    // from the loaded chunks map and sent to the underlying ChunkIO
    pub loaded_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    loaded_entity_chunks: Arc<DashMap<Vector2<i32>, SyncEntityChunk>>,
    pub chunk_loading: Mutex<ChunkLoading>,

    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,

    pub chunk_saver: Arc<dyn FileIO<Data = SyncChunk>>,
    entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>>,

    pub world_gen: Arc<VanillaGenerator>,

    /// Tracks tasks associated with this world instance
    tasks: TaskTracker,
    /// Notification that interrupts tasks for shutdown
    pub shutdown_notifier: Notify,
    pub is_shutting_down: AtomicBool,

    gen_entity_request_tx: Sender<Vector2<i32>>,
    pending_entity_generations: Arc<DashMap<Vector2<i32>, Vec<oneshot::Sender<SyncEntityChunk>>>>,

    pub level_channel: Arc<LevelChannel>,
    pub thread_tracker: Mutex<Vec<std::thread::JoinHandle<()>>>,
    pub chunk_listener: Arc<ChunkListener>,
}

pub struct TickData {
    pub block_ticks: Vec<OrderedTick<&'static Block>>,
    pub fluid_ticks: Vec<OrderedTick<&'static Fluid>>,
    pub random_ticks: Vec<ScheduledTick<()>>,
    pub block_entities: Vec<Arc<dyn BlockEntity>>,
}

#[derive(Clone)]
pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
    pub entities_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(
        root_folder: PathBuf,
        block_registry: Arc<dyn BlockRegistryExt>,
        seed: i64,
        dimension: Dimension,
    ) -> Arc<Self> {
        // If we are using an already existing world we want to read the seed from the level.dat, If not we want to check if there is a seed in the config, if not lets create a random one
        let region_folder = root_folder.join("region");
        if !region_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        }
        let entities_folder = root_folder.join("entities");
        if !entities_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Entities folder");
        }
        let level_folder = LevelFolder {
            root_folder,
            region_folder,
            entities_folder,
        };

        // TODO: Load info correctly based on world format type
        let seed = Seed(seed as u64);
        let world_gen = get_world_gen(seed, dimension).into();

        let chunk_saver: Arc<dyn FileIO<Data = SyncChunk>> = match advanced_config().chunk.format {
            ChunkFormat::Linear => Arc::new(ChunkFileManager::<LinearFile<ChunkData>>::default()),
            ChunkFormat::Anvil => {
                Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkData>>::default())
            }
        };
        let entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>> =
            match advanced_config().chunk.format {
                ChunkFormat::Linear => {
                    Arc::new(ChunkFileManager::<LinearFile<ChunkEntityData>>::default())
                }
                ChunkFormat::Anvil => {
                    Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkEntityData>>::default())
                }
            };

        let (gen_entity_request_tx, gen_entity_request_rx) = crossbeam::channel::unbounded();
        let pending_entity_generations = Arc::new(DashMap::new());

        let level_channel = Arc::new(LevelChannel::new());
        let thread_tracker = Mutex::new(Vec::new());
        let listener = Arc::new(ChunkListener::new());

        let level_ref = Arc::new(Self {
            seed,
            block_registry,
            world_gen,
            level_folder,
            chunk_saver,
            entity_saver,
            schedule_tick_counts: AtomicU64::new(0),
            loaded_chunks: Arc::new(DashMap::new()),
            loaded_entity_chunks: Arc::new(DashMap::new()),
            chunk_loading: Mutex::new(ChunkLoading::new(level_channel.clone())),
            chunk_watchers: Arc::new(DashMap::new()),
            tasks: TaskTracker::new(),
            shutdown_notifier: Notify::new(),
            is_shutting_down: AtomicBool::new(false),
            gen_entity_request_tx,
            pending_entity_generations: pending_entity_generations.clone(),
            level_channel: level_channel.clone(),
            thread_tracker,
            chunk_listener: listener.clone(),
        });

        let num_threads = num_cpus::get().saturating_sub(1).max(1);

        GenerationSchedule::create(
            &level_ref.tasks,
            4,
            num_threads,
            level_ref.clone(),
            level_channel,
            listener,
            level_ref.thread_tracker.lock().unwrap().as_mut(),
        );

        // Entity Chunks
        for thread_id in 0..num_threads {
            let level_clone = level_ref.clone();
            let pending_clone = pending_entity_generations.clone();
            let rx = gen_entity_request_rx.clone();

            std::thread::spawn(move || {
                while let Ok(pos) = rx.recv() {
                    if level_clone.is_shutting_down.load(Ordering::Relaxed) {
                        break;
                    }

                    log::debug!(
                        "Generating entity chunk {pos:?}, worker thread {thread_id:?}, queue length {}",
                        rx.len()
                    );

                    let chunk = ChunkEntityData {
                        chunk_position: pos,
                        data: HashMap::new(),
                        dirty: true,
                    };
                    let arc_chunk = Arc::new(RwLock::new(chunk));

                    level_clone
                        .loaded_entity_chunks
                        .insert(pos, arc_chunk.clone());

                    if let Some(waiters) = pending_clone.remove(&pos) {
                        for tx in waiters.1 {
                            let _ = tx.send(arc_chunk.clone());
                        }
                    }
                }
            });
        }
        level_ref
            .chunk_loading
            .lock()
            .unwrap()
            .add_ticket(Vector2::<i32>::new(0, 0), 17);
        // .add_ticket(Vector2::<i32>::new(0, 0), ChunkLoading::FULL_CHUNK_LEVEL);
        level_ref
    }

    async fn load_single_chunk(
        &self,
        pos: Vector2<i32>,
    ) -> Result<(SyncChunk, bool), ChunkReadingError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // Call the existing fetch_chunks with a single chunk
        self.chunk_saver
            .fetch_chunks(&self.level_folder, &[pos], tx)
            .await;

        // Wait for the result
        match rx.recv().await {
            Some(LoadedData::Loaded(chunk)) => Ok((chunk, false)),
            Some(LoadedData::Missing(_)) => Err(ChunkReadingError::ChunkNotExist),
            Some(LoadedData::Error((_, err))) => Err(err),
            None => Err(ChunkReadingError::ChunkNotExist),
        }
    }

    /// Spawns a task associated with this world. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn(task)
    }

    pub async fn shutdown(&self) {
        log::info!("Saving level...");

        self.is_shutting_down.store(true, Ordering::Relaxed);
        self.shutdown_notifier.notify_waiters();
        self.level_channel.notify();

        let mut lock = self.thread_tracker.lock().unwrap();
        log::info!("Wait {} jobs stop", lock.len());
        while !lock.is_empty() {
            let i = lock.pop().unwrap();
            log::info!(
                "Waiting Thread {:?} {} stop",
                i.thread().id(),
                i.thread().name().unwrap_or("unknown")
            );
        }
        log::info!("All Thread stop");

        self.tasks.close();
        log::debug!("Awaiting level tasks");
        self.tasks.wait().await;
        log::debug!("Done awaiting level chunk tasks");

        // wait for chunks currently saving in other threads
        self.chunk_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        // let chunks_to_write = self
        //     .loaded_chunks
        //     .iter()
        //     .map(|chunk| (*chunk.key(), chunk.value().clone()))
        //     .collect::<Vec<_>>();
        // self.loaded_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        // self.chunk_saver.clear_watched_chunks().await;
        // self.write_chunks(chunks_to_write).await;

        log::debug!("Done awaiting level entity tasks");

        // wait for chunks currently saving in other threads
        self.entity_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_entity_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_entity_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        self.entity_saver.clear_watched_chunks().await;
        self.write_entity_chunks(chunks_to_write).await;
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub async fn clean_up_log(&self) {
        self.chunk_saver.clean_up_log().await;
        self.entity_saver.clean_up_log().await;
    }

    pub fn list_cached(&self) {
        for entry in self.loaded_chunks.iter() {
            log::debug!("In map: {:?}", entry.key());
        }
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub async fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        for chunk in chunks {
            log::trace!("{chunk:?} marked as newly watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    if let Some(new_value) = value.checked_add(1) {
                        *value = new_value;
                        //log::debug!("Watch value for {:?}: {}", chunk, value);
                    } else {
                        log::error!("Watching overflow on chunk {chunk:?}");
                    }
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(1);
                }
            }
        }

        // self.chunk_saver
        //     .watch_chunks(&self.level_folder, chunks)
        //     .await;
        self.entity_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
    }

    #[inline]
    pub async fn mark_chunk_as_newly_watched(&self, chunk: Vector2<i32>) {
        self.mark_chunks_as_newly_watched(&[chunk]).await;
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub async fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        let mut chunks_to_clean = Vec::new();

        for chunk in chunks {
            log::trace!("{chunk:?} marked as no longer watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    *value = value.saturating_sub(1);

                    if *value == 0 {
                        occupied.remove_entry();
                        chunks_to_clean.push(*chunk);
                    }
                }
                Entry::Vacant(_) => {
                    // This can be:
                    // - Player disconnecting before all packets have been sent
                    // - Player moving so fast that the chunk leaves the render distance before it
                    // is loaded into memory
                }
            }
        }
        self.entity_saver
            .unwatch_chunks(&self.level_folder, chunks)
            .await;
        chunks_to_clean
    }

    /// Returns whether the chunk should be removed from memory
    #[inline]
    pub async fn mark_chunk_as_not_watched(&self, chunk: Vector2<i32>) -> bool {
        !self.mark_chunks_as_not_watched(&[chunk]).await.is_empty()
    }

    pub async fn clean_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        panic!();
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_chunks.remove(pos).map(|chunk| (*pos, chunk.1))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();

            level.write_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, chunk) in chunks_to_remove {
                // Add them back if they have watchers
                if level.chunk_watchers.get(&pos).is_some() {
                    let entry = level.loaded_chunks.entry(pos);
                    if let Entry::Vacant(vacant) = entry {
                        vacant.insert(chunk);
                    }
                }
            }
        });
    }

    pub async fn clean_entity_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_entity_chunks
                        .get(pos)
                        .map(|chunk| (*pos, chunk.value().clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();
            level.write_entity_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, _) in chunks_to_remove {
                let _ = level.loaded_entity_chunks.remove_if(&pos, |_, _| {
                    // Recheck that there is no one watching
                    level
                        .chunk_watchers
                        .get(&pos)
                        .is_none_or(|count| count.is_zero())
                });
            }
        });
    }

    // Gets random ticks, block ticks and fluid ticks
    pub async fn get_tick_data(&self) -> TickData {
        let mut ticks = TickData {
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            random_ticks: Vec::with_capacity(self.loaded_chunks.len() * 3 * 16 * 16),
            block_entities: Vec::new(),
        };

        let mut rng = SmallRng::from_os_rng();
        for chunk in self.loaded_chunks.iter() {
            let mut chunk = chunk.write().await;
            ticks.block_ticks.append(&mut chunk.block_ticks.step_tick());
            ticks.fluid_ticks.append(&mut chunk.fluid_ticks.step_tick());

            let chunk = chunk.downgrade();

            let chunk_x_base = chunk.position.x * 16;
            let chunk_z_base = chunk.position.y * 16;

            let mut section_blocks = Vec::new();
            for i in 0..chunk.section.sections.len() {
                let mut section_block_data = Vec::new();

                //TODO use game rules to determine how many random ticks to perform
                for _ in 0..3 {
                    let r = rng.random::<u32>();
                    let x_offset = (r & 0xF) as i32;
                    let y_offset = ((r >> 4) & 0xF) as i32 - 32;
                    let z_offset = (r >> 8 & 0xF) as i32;

                    let random_pos = BlockPos::new(
                        chunk_x_base + x_offset,
                        i as i32 * 16 + y_offset,
                        chunk_z_base + z_offset,
                    );

                    let block_id = chunk
                        .section
                        .get_block_absolute_y(x_offset as usize, random_pos.0.y, z_offset as usize)
                        .unwrap_or(Block::AIR.default_state.id);

                    section_block_data.push((random_pos, block_id));
                }
                section_blocks.push(section_block_data);
            }

            for section_data in section_blocks {
                for (random_pos, block_id) in section_data {
                    if has_random_ticks(block_id) {
                        ticks.random_ticks.push(ScheduledTick {
                            position: random_pos,
                            delay: 0,
                            priority: TickPriority::Normal,
                            value: (),
                        });
                    }
                }
            }

            ticks
                .block_entities
                .extend(chunk.block_entities.values().cloned());
        }

        ticks.block_ticks.sort_unstable();
        ticks.fluid_ticks.sort_unstable();

        ticks
    }

    pub async fn clean_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        // self.clean_chunks(&[*chunk]).await;
    }

    pub async fn clean_entity_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_entity_chunks(&[*chunk]).await;
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self) {
        self.chunk_watchers.retain(|_, watcher| !watcher.is_zero());
        // self.loaded_chunks
        //     .retain(|at, _| self.chunk_watchers.get(at).is_some());
        self.loaded_entity_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.chunk_watchers.capacity() - self.chunk_watchers.len() >= 4096 {
            self.chunk_watchers.shrink_to_fit();
        }

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.loaded_chunks.capacity() - self.loaded_chunks.len() >= 4096 {
            self.loaded_chunks.shrink_to_fit();
        }

        if self.loaded_entity_chunks.capacity() - self.loaded_entity_chunks.len() >= 4096 {
            self.loaded_entity_chunks.shrink_to_fit();
        }
    }

    pub async fn get_chunk(self: &Arc<Self>, pos: Vector2<i32>) -> SyncChunk {
        // Already loaded?
        if let Some(chunk) = self.loaded_chunks.get(&pos) {
            chunk.clone()
        } else {
            log::info!("Missing Chunk {pos:?}. Fetching.");
            let recv = self.chunk_listener.add_single_chunk_listener(pos);

            {
                let mut lock = self.chunk_loading.lock().unwrap();
                lock.add_ticket(pos, ChunkLoading::FULL_CHUNK_LEVEL);
                lock.send_change();
            }

            let ret = if let Some(chunk) = self.loaded_chunks.get(&pos) {
                // try again here. otherwise deadlock
                chunk.clone()
            } else {
                recv.await.unwrap()
            };

            let mut lock = self.chunk_loading.lock().unwrap();
            lock = self.chunk_loading.lock().unwrap();
            lock.remove_ticket(pos, ChunkLoading::FULL_CHUNK_LEVEL);
            ret
        }
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    /// Spawns a tokio task to stream chunks.
    /// Important: must be called from an async function (or changed to accept a tokio runtime
    /// handle)
    pub fn receive_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncChunk, bool)> {
        panic!("");
        let (sender, receiver) = mpsc::unbounded_channel();
        let level = self.clone();

        log::trace!("Receiving chunks: {}", chunks.len());

        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();

            let fetch_task = async {
                // Separate already-loaded chunks from ones we need to fetch
                let mut to_fetch = Vec::new();
                for pos in &chunks {
                    if let Some(chunk) = level.loaded_chunks.get(pos) {
                        let _ = sender.send((chunk.clone(), false));
                    } else {
                        to_fetch.push(*pos);
                        panic!("not chunk found at {pos:?}");
                    }
                }

                if !to_fetch.is_empty() {
                    // Channel for fetch_chunks to send results
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<
                        LoadedData<SyncChunk, ChunkReadingError>,
                    >(to_fetch.len());

                    // Fetch all missing chunks from disk in one go
                    level
                        .chunk_saver
                        .fetch_chunks(&level.level_folder, &to_fetch, tx)
                        .await;

                    // Process loaded/missing/error results
                    while let Some(data) = rx.recv().await {
                        match data {
                            LoadedData::Loaded(chunk) => {
                                let pos = chunk.read().await.position;
                                level.loaded_chunks.insert(pos, chunk.clone());
                                let _ = sender.send((chunk, false));
                            }
                            LoadedData::Missing(pos) | LoadedData::Error((pos, _)) => {
                                panic!("chunk found at {pos:?}");
                                let sender_clone = sender.clone();

                                // Need to generate — but don't block here
                                let level_clone = level.clone();

                                // tokio::spawn(async move {
                                //     let (tx, rx) = oneshot::channel();
                                //
                                //     match level_clone.pending_generations.entry(pos) {
                                //         dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                                //             entry.get_mut().push(tx);
                                //         }
                                //         dashmap::mapref::entry::Entry::Vacant(entry) => {
                                //             entry.insert(vec![tx]);
                                //             let _ = level_clone.gen_request_tx.send(pos);
                                //         }
                                //     }
                                //
                                //     if let Ok(chunk) = rx.await {
                                //         let _ = sender_clone.send((chunk, true));
                                //     }
                                // });
                            }
                        }
                    }
                }
            };

            // Stop early if shutting down
            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            };
        });

        receiver
    }

    async fn load_single_entity_chunk(
        &self,
        pos: Vector2<i32>,
    ) -> Result<(SyncEntityChunk, bool), ChunkReadingError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        self.entity_saver
            .fetch_chunks(&self.level_folder, &[pos], tx)
            .await;

        match rx.recv().await {
            Some(LoadedData::Loaded(chunk)) => Ok((chunk, false)),
            Some(LoadedData::Missing(_)) => Err(ChunkReadingError::ChunkNotExist),
            Some(LoadedData::Error((_, err))) => Err(err),
            None => Err(ChunkReadingError::ChunkNotExist),
        }
    }

    pub fn receive_entity_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncEntityChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let level = self.clone();

        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();

            let fetch_task = async {
                let mut to_fetch = Vec::new();
                for pos in &chunks {
                    if let Some(chunk) = level.loaded_entity_chunks.get(pos) {
                        let _ = sender.send((chunk.clone(), false));
                    } else {
                        to_fetch.push(*pos);
                    }
                }

                if !to_fetch.is_empty() {
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<
                        LoadedData<SyncEntityChunk, ChunkReadingError>,
                    >(to_fetch.len());

                    level
                        .entity_saver
                        .fetch_chunks(&level.level_folder, &to_fetch, tx)
                        .await;

                    while let Some(data) = rx.recv().await {
                        match data {
                            LoadedData::Loaded(chunk) => {
                                let pos = chunk.read().await.chunk_position;
                                level.loaded_entity_chunks.insert(pos, chunk.clone());
                                let _ = sender.send((chunk, false));
                            }
                            LoadedData::Missing(pos) | LoadedData::Error((pos, _)) => {
                                let sender_clone = sender.clone();
                                let level_clone = level.clone();

                                tokio::spawn(async move {
                                    let (tx, rx) = oneshot::channel();
                                    match level_clone.pending_entity_generations.entry(pos) {
                                        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                                            entry.get_mut().push(tx);
                                        }
                                        dashmap::mapref::entry::Entry::Vacant(entry) => {
                                            entry.insert(vec![tx]);
                                            let _ = level_clone.gen_entity_request_tx.send(pos);
                                        }
                                    }
                                    if let Ok(chunk) = rx.await {
                                        let _ = sender_clone.send((chunk, true));
                                    }
                                });
                            }
                        }
                    }
                }
            };

            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            }
        });

        receiver
    }

    pub async fn get_entity_chunk(self: &Arc<Self>, pos: Vector2<i32>) -> SyncEntityChunk {
        if let Some(chunk) = self.loaded_entity_chunks.get(&pos) {
            return chunk.clone();
        }

        match self.load_single_entity_chunk(pos).await {
            Ok((chunk, _)) => {
                self.loaded_entity_chunks.insert(pos, chunk.clone());
                chunk
            }
            Err(_) => {
                let (tx, rx) = oneshot::channel();
                match self.pending_entity_generations.entry(pos) {
                    dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(tx);
                    }
                    dashmap::mapref::entry::Entry::Vacant(entry) => {
                        entry.insert(vec![tx]);
                        let _ = self.gen_entity_request_tx.send(pos);
                    }
                }
                rx.await.expect("Entity generation worker dropped")
            }
        }
    }

    pub async fn get_block_state(self: &Arc<Self>, position: &BlockPos) -> RawBlockState {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;

        let Some(id) = chunk.read().await.section.get_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
        ) else {
            return RawBlockState(Block::VOID_AIR.default_state.id);
        };

        RawBlockState(id)
    }
    pub async fn get_rough_biome(self: &Arc<Self>, position: &BlockPos) -> &'static Biome {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;

        let Some(id) = chunk.read().await.section.get_rough_biome_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
        ) else {
            return &Biome::THE_VOID;
        };

        Biome::from_id(id).unwrap()
    }

    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;
        let mut chunk = chunk.write().await;

        let replaced_block_state_id = chunk.section.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            block_state_id,
        );
        if replaced_block_state_id != block_state_id {
            chunk.mark_dirty(true);
        }
        replaced_block_state_id
    }

    pub async fn write_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    pub async fn write_entity_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncEntityChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.entity_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    pub fn try_get_chunk(
        &self,
        coordinates: &Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkData>>>> {
        self.loaded_chunks.try_get(coordinates).try_unwrap()
    }

    pub fn try_get_entity_chunk(
        &self,
        coordinates: Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkEntityData>>>> {
        self.loaded_entity_chunks.try_get(&coordinates).try_unwrap()
    }

    pub async fn schedule_block_tick(
        self: &Arc<Self>,
        block: &Block,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk = chunk.write().await;
        chunk.block_ticks.schedule_tick(
            ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*(block as *const Block) },
            },
            self.schedule_tick_counts.load(Ordering::Relaxed),
        );
        self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn schedule_fluid_tick(
        self: &Arc<Self>,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk = chunk.write().await;
        chunk.fluid_ticks.schedule_tick(
            ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*(fluid as *const Fluid) },
            },
            self.schedule_tick_counts.load(Ordering::Relaxed),
        );
        self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn is_block_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        block: &Block,
    ) -> bool {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let chunk = chunk.read().await;
        chunk.block_ticks.is_scheduled(*block_pos, block)
    }

    pub async fn is_fluid_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        fluid: &Fluid,
    ) -> bool {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let chunk = chunk.read().await;
        chunk.fluid_ticks.is_scheduled(*block_pos, fluid)
    }
}

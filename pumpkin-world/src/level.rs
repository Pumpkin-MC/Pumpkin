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
    generation::get_world_gen,
    tick::{OrderedTick, ScheduledTick, TickPriority},
    world::BlockRegistryExt,
};
use crossbeam::channel::Sender;
use dashmap::DashMap;
use log::trace;
use num_traits::Zero;
use pumpkin_config::{chunk::ChunkConfig, lighting::LightingEngineConfig, world::LevelConfig};
use pumpkin_data::biome::Biome;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, block_properties::has_random_ticks, fluid::Fluid};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_util::world_seed::Seed;
use rustc_hash::FxHashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
// use std::time::Duration;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};
// use tokio::runtime::Handle;
use tokio::{
    select,
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::task::TaskTracker;

pub type SyncChunk = Arc<RwLock<ChunkData>>;
pub type SyncEntityChunk = Arc<RwLock<ChunkEntityData>>;

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
    pub lighting_config: LightingEngineConfig,

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
    pub chunk_system_tasks: TaskTracker,
    /// Notification that interrupts tasks for shutdown
    pub cancel_token: CancellationToken,

    pub shut_down_chunk_system: AtomicBool,
    pub should_save: AtomicBool,
    pub should_unload: AtomicBool,

    gen_entity_request_tx: Sender<Vector2<i32>>,
    pending_entity_generations: Arc<DashMap<Vector2<i32>, Vec<oneshot::Sender<SyncEntityChunk>>>>,

    pub level_channel: Arc<LevelChannel>,
    pub thread_tracker: Mutex<Vec<thread::JoinHandle<()>>>,
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

#[ignore]
#[cfg(feature = "tokio_taskdump")]
pub async fn dump() {
    // let handle = Handle::current();
    // if let Ok(dump) = timeout(Duration::from_secs(100), handle.dump()).await {
    //     for (i, task) in dump.tasks().iter().enumerate() {
    //         let trace = task.trace();
    //         log::error!("TASK {i}:");
    //         log::error!("{trace}\n");
    //     }
    // }
}

impl Level {
    pub fn from_root_folder(
        level_config: &LevelConfig,
        root_folder: PathBuf,
        block_registry: Arc<dyn BlockRegistryExt>,
        seed: i64,
        dimension: Dimension,
    ) -> Arc<Self> {
        let region_folder = root_folder.join("region");
        let entities_folder = root_folder.join("entities");

        std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        std::fs::create_dir_all(&entities_folder).expect("Failed to create Entities folder");

        let level_folder = LevelFolder {
            root_folder,
            region_folder,
            entities_folder,
        };

        let seed = Seed(seed as u64);
        let world_gen = get_world_gen(seed, dimension).into();

        let chunk_saver: Arc<dyn FileIO<Data = SyncChunk>> = match &level_config.chunk {
            ChunkConfig::Linear(config) => Arc::new(
                ChunkFileManager::<LinearFile<ChunkData>>::new(config.clone()),
            ),
            ChunkConfig::Anvil(config) => Arc::new(
                ChunkFileManager::<AnvilChunkFile<ChunkData>>::new(config.clone()),
            ),
        };
        let entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>> = match &level_config.chunk {
            ChunkConfig::Linear(config) => Arc::new(
                ChunkFileManager::<LinearFile<ChunkEntityData>>::new(config.clone()),
            ),
            ChunkConfig::Anvil(config) => Arc::new(ChunkFileManager::<
                AnvilChunkFile<ChunkEntityData>,
            >::new(config.clone())),
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
            lighting_config: level_config.lighting,
            chunk_saver,
            entity_saver,
            schedule_tick_counts: AtomicU64::new(0),
            loaded_chunks: Arc::new(DashMap::new()),
            loaded_entity_chunks: Arc::new(DashMap::new()),
            chunk_loading: Mutex::new(ChunkLoading::new(level_channel.clone())),
            chunk_watchers: Arc::new(DashMap::new()),
            tasks: TaskTracker::new(),
            chunk_system_tasks: TaskTracker::new(),
            cancel_token: CancellationToken::new(),
            shut_down_chunk_system: AtomicBool::new(false),
            should_save: AtomicBool::new(false),
            should_unload: AtomicBool::new(false),
            gen_entity_request_tx,
            pending_entity_generations: pending_entity_generations.clone(),
            level_channel: level_channel.clone(),
            thread_tracker,
            chunk_listener: listener.clone(),
        });

        // TODO
        let total_cores = num_cpus::get().saturating_sub(2).max(1);
        let threads_per_dimension = (total_cores / 2).max(1);
        let entity_threads = (threads_per_dimension / 2).max(1);

        GenerationSchedule::create(
            2,
            threads_per_dimension,
            level_ref.clone(),
            level_channel,
            listener,
            level_ref.thread_tracker.lock().unwrap().as_mut(),
        );

        let mut tracker_lock = level_ref.thread_tracker.lock().unwrap();

        for thread_id in 0..entity_threads {
            let level_clone = level_ref.clone();
            let pending_clone = pending_entity_generations.clone();
            let rx = gen_entity_request_rx.clone();

            let handle = thread::Builder::new()
                .name(format!("Entity Chunk Generation Thread {thread_id}"))
                .spawn(move || {
                    loop {
                        if level_clone.cancel_token.is_cancelled() {
                            break;
                        }

                        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
                            Ok(pos) => {
                                let arc_chunk = Arc::new(RwLock::new(ChunkEntityData {
                                    x: pos.x,
                                    z: pos.y,
                                    data: FxHashMap::default(),
                                    dirty: true,
                                }));

                                level_clone
                                    .loaded_entity_chunks
                                    .insert(pos, arc_chunk.clone());

                                if let Some((_, waiters)) = pending_clone.remove(&pos) {
                                    for tx in waiters {
                                        let _ = tx.send(arc_chunk.clone());
                                    }
                                }
                            }
                            Err(crossbeam::channel::RecvTimeoutError::Timeout) => continue,
                            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                })
                .expect("Failed to spawn Entity Generation Thread");

            tracker_lock.push(handle);
        }

        drop(tracker_lock);
        level_ref
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
        let world_id = self.level_folder.root_folder.display();
        log::info!("Saving level ({})...", world_id);
        self.cancel_token.cancel();
        self.shut_down_chunk_system.store(true, Ordering::Relaxed);
        self.level_channel.notify();

        self.tasks.close();
        self.chunk_system_tasks.close();

        let handles = {
            let mut lock = self.thread_tracker.lock().unwrap();
            lock.drain(..).collect::<Vec<_>>()
        };

        let handle_count = handles.len();
        log::info!("Joining {} threads for {}...", handle_count, world_id);
        let join_task = tokio::task::spawn_blocking(move || {
            let mut failed_count = 0;
            for handle in handles.into_iter() {
                if handle.join().is_err() {
                    failed_count += 1;
                }
            }
            failed_count
        });

        match timeout(Duration::from_secs(3), join_task).await {
            Ok(Ok(failed_count)) => {
                if failed_count > 0 {
                    log::warn!("{} threads failed to join properly for {}.", failed_count, world_id);
                }
            }
            Ok(Err(_)) => {
                log::warn!("Thread join task panicked for {}.", world_id);
            }
            Err(_) => {
                log::warn!("Timed out waiting for threads to join for {}.", world_id);
            }
        }

        self.tasks.wait().await;
        self.chunk_system_tasks.wait().await;

        log::info!("Flushing data to disk for {}...", world_id);
        self.chunk_saver.block_and_await_ongoing_tasks().await;
        self.entity_saver.block_and_await_ongoing_tasks().await;

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
            self.chunk_watchers
                .entry(*chunk)
                .and_modify(|count| *count = count.saturating_add(1))
                .or_insert(1);
        }

        // self.chunk_saver
        //     .watch_chunks(&self.level_folder, chunks)
        //     .await;
        self.entity_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub async fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        let mut chunks_to_clean = Vec::new();

        for chunk in chunks {
            let mut should_remove = false;

            if let Some(mut count) = self.chunk_watchers.get_mut(chunk) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    should_remove = true;
                }
            }

            if should_remove {
                self.chunk_watchers.remove(chunk);
                chunks_to_clean.push(*chunk);
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

    // In Level::clean_entity_chunks()
    pub fn clean_entity_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        let chunks_to_process: Vec<_> = chunks
            .iter()
            .filter_map(|pos| {
                // Only include chunks with no watchers
                let has_watchers = self
                    .chunk_watchers
                    .get(pos)
                    .is_some_and(|count| !count.is_zero());

                if has_watchers {
                    return None;
                }

                // Remove immediately to prevent race conditions
                self.loaded_entity_chunks.remove(pos)
            })
            .collect();

        if chunks_to_process.is_empty() {
            return;
        }

        let level = self.clone();
        self.spawn_task(async move {
            log::debug!("Writing {} entity chunks to disk", chunks_to_process.len());
            level.write_entity_chunks(chunks_to_process).await;
        });
    }

    pub async fn get_tick_data(&self) -> TickData {
        let mut ticks = TickData {
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            random_ticks: Vec::with_capacity(self.loaded_chunks.len() * 3),
            block_entities: Vec::new(),
        };

        let r = rand::random::<u32>();

        for chunk_sync in self.loaded_chunks.iter() {
            let mut chunk = chunk_sync.write().await;

            let chunk_x_base = chunk.x * 16;
            let chunk_z_base = chunk.z * 16;
            let section_count = chunk.section.sections.len();

            ticks
                .block_entities
                .extend(chunk.block_entities.values().cloned());

            for i in 0..section_count {
                let y_base = i as i32 * 16;
                for _ in 0..3 {
                    let x_offset = (r & 0xF) as usize;
                    let z_offset = (r >> 8 & 0xF) as usize;
                    let y_in_section = ((r >> 4) & 0xF) as i32;
                    let absolute_y = y_base + y_in_section;

                    if let Some(block_state_id) = chunk
                        .section
                        .get_block_absolute_y(x_offset, absolute_y, z_offset)
                        && has_random_ticks(block_state_id)
                    {
                        ticks.random_ticks.push(ScheduledTick {
                            position: BlockPos::new(
                                chunk_x_base + x_offset as i32,
                                absolute_y,
                                chunk_z_base + z_offset as i32,
                            ),
                            delay: 0,
                            priority: TickPriority::Normal,
                            value: (),
                        });
                    }
                }
            }

            ticks.block_ticks.append(&mut chunk.block_ticks.step_tick());
            ticks.fluid_ticks.append(&mut chunk.fluid_ticks.step_tick());
        }

        ticks.block_ticks.sort_unstable();
        ticks.fluid_ticks.sort_unstable();
        ticks
    }

    pub fn clean_entity_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_entity_chunks(&[*chunk]);
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self) {
        self.chunk_watchers.retain(|_, watcher| !watcher.is_zero());
        self.loaded_entity_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.chunk_watchers.capacity() - self.chunk_watchers.len() >= 4096 {
            self.chunk_watchers.shrink_to_fit();
        }

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        // if self.loaded_chunks.capacity() - self.loaded_chunks.len() >= 4096 {
        //     self.loaded_chunks.shrink_to_fit();
        // }

        if self.loaded_entity_chunks.capacity() - self.loaded_entity_chunks.len() >= 4096 {
            self.loaded_entity_chunks.shrink_to_fit();
        }
    }

    pub async fn get_chunk(self: &Arc<Self>, pos: Vector2<i32>) -> SyncChunk {
        // Check if already in memory
        if let Some(chunk) = self.loaded_chunks.get(&pos) {
            return chunk.clone();
        }

        let recv = self.chunk_listener.add_single_chunk_listener(pos);

        {
            let mut lock = self.chunk_loading.lock().unwrap();
            lock.add_ticket(pos, 31);
            lock.send_change();
        };

        let chunk = if let Some(chunk) = self.loaded_chunks.get(&pos) {
            chunk.clone()
        } else {
            recv.await
                .expect("Chunk listener dropped without sending chunk")
        };

        {
            let mut lock = self.chunk_loading.lock().unwrap();
            lock.remove_ticket(pos, 31);
            lock.send_change();
        }

        chunk
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
            Some(LoadedData::Error((_, err))) => Err(err),
            _ => Err(ChunkReadingError::ChunkNotExist),
        }
    }

    pub fn receive_entity_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncEntityChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let level = self.clone();

        self.spawn_task(async move {
            let cancel_notifier = level.cancel_token.cancelled();

            let fetch_task = async {
                let to_fetch: Vec<_> = chunks
                    .iter()
                    .filter(|pos| {
                        level.loaded_entity_chunks.get(pos).is_none_or(|chunk| {
                            let _ = sender.send((chunk.clone(), false));
                            false // Don't fetch
                        })
                    })
                    .copied()
                    .collect();

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
                                let tmp_chunk = chunk.read().await;
                                let pos = Vector2::new(tmp_chunk.x, tmp_chunk.z);
                                drop(tmp_chunk);
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

        if let Ok((chunk, _)) = self.load_single_entity_chunk(pos).await {
            self.loaded_entity_chunks.insert(pos, chunk.clone());
            chunk
        } else {
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

    pub fn try_get_chunk(&self, coordinates: &Vector2<i32>) -> Option<Arc<RwLock<ChunkData>>> {
        self.loaded_chunks
            .get(coordinates)
            .map(|x| x.value().clone())
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
        let chunk = self.get_chunk(block_pos.chunk_position()).await;
        let tick_order = self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
        let mut chunk = chunk.write().await;
        chunk.block_ticks.schedule_tick(
            &ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*std::ptr::from_ref::<Block>(block) },
            },
            tick_order,
        );
    }

    pub async fn schedule_fluid_tick(
        self: &Arc<Self>,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        let chunk = self.get_chunk(block_pos.chunk_position()).await;
        let tick_order = self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
        let mut chunk = chunk.write().await;
        chunk.fluid_ticks.schedule_tick(
            &ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*std::ptr::from_ref::<Fluid>(fluid) },
            },
            tick_order,
        );
    }

    pub async fn is_block_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        block: &Block,
    ) -> bool {
        let chunk = self.get_chunk(block_pos.chunk_position()).await;
        let chunk = chunk.read().await;
        chunk.block_ticks.is_scheduled(*block_pos, block)
    }

    pub async fn is_fluid_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        fluid: &Fluid,
    ) -> bool {
        let chunk = self.get_chunk(block_pos.chunk_position()).await;
        let chunk = chunk.read().await;
        chunk.fluid_ticks.is_scheduled(*block_pos, fluid)
    }
}
